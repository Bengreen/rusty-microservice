//! supporting functions for a microservice

use crate::uservice::HandleChannel;
use atomic::Atomic;
use lazy_static::lazy_static;
use prometheus::{HistogramOpts, HistogramVec, IntCounter, IntCounterVec, Opts, Registry};
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use warp::Filter;

lazy_static! {
    pub static ref INCOMING_REQUESTS: IntCounter =
        IntCounter::new("incoming_requests", "Incoming Requests").expect("metric can be created");
    pub static ref RESPONSE_CODE_COLLECTOR: IntCounterVec = IntCounterVec::new(
        Opts::new("response_code", "Response Codes"),
        &["env", "statuscode", "type"]
    )
    .expect("metric can be created");
    pub static ref RESPONSE_TIME_COLLECTOR: HistogramVec = HistogramVec::new(
        HistogramOpts::new("response_time", "Response Times"),
        &["env"]
    )
    .expect("metric can be created");
    pub static ref REGISTRY: Registry = Registry::new();
}

fn register_custom_metrics() {
    REGISTRY
        .register(Box::new(INCOMING_REQUESTS.clone()))
        .expect("collector can be registered");

    REGISTRY
        .register(Box::new(RESPONSE_CODE_COLLECTOR.clone()))
        .expect("collector can be registered");

    REGISTRY
        .register(Box::new(RESPONSE_TIME_COLLECTOR.clone()))
        .expect("collector can be registered");
}

#[derive(Debug)]
pub struct HealthProbe {
    name: String,
    margin: Duration,
    time: Arc<Atomic<Instant>>,
}
impl HealthProbe {
    pub fn new(name: &str, margin: Duration) -> HealthProbe {
        HealthProbe {
            name: name.to_string(),
            margin,
            time: Arc::new(Atomic::new(Instant::now())),
        }
    }

    pub fn tick(&mut self) {
        self.time.store(Instant::now(), Ordering::SeqCst);
    }
    fn valid(&self) -> bool {
        self.time.load(Ordering::SeqCst).elapsed() <= self.margin
    }
}
impl Clone for HealthProbe {
    fn clone(&self) -> HealthProbe {
        HealthProbe {
            name: self.name.clone(),
            margin: self.margin,
            time: self.time.clone(),
        }
    }
}

#[derive(Clone)]
pub struct HealthCheck {
    name: String,
    probe_list: Arc<Mutex<Vec<HealthProbe>>>,
}

impl HealthCheck {
    pub fn new(name: &str) -> HealthCheck {
        println!("Creating HealthCheck: {}", name);

        HealthCheck {
            name: name.to_string(),
            probe_list: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add(&mut self, probe: &HealthProbe) {
        self.probe_list.lock().unwrap().push(probe.clone());
    }

    pub fn status(&self) -> (bool, HashMap<String, bool>) {
        let mut happy = true;

        let detail: HashMap<_, _> = self
            .probe_list
            .lock()
            .unwrap()
            .iter()
            .map(|x| {
                if !x.valid() {
                    happy = false;
                }
                (x.name.clone(), x.valid())
            })
            .collect();
        (happy, detail)
    }
}

pub async fn health_listen<'a>(
    basepath: &'static str,
    port: u16,
    liveness: &'a HealthCheck,
    readyness: &'a HealthCheck,
    channel_http_kill: tokio::sync::mpsc::Sender<()>,
) -> HandleChannel {
    println!("Starting health http on {}", port);

    register_custom_metrics();

    let api = filters::health(basepath, liveness.clone(), readyness.clone(), channel_http_kill);

    let routes = api.with(warp::log("health"));

    println!("Starting health service");

    let (channel, mut rx) = mpsc::channel(1);

    let (_addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], port), async move {
            rx.recv().await;
        });

    let handle = tokio::task::spawn(server);

    HandleChannel { handle, channel }
}

mod filters {
    use super::handlers;
    use crate::k8slifecycle::HealthCheck;
    use warp::Filter;

    pub fn health(
        basepath: &'static str,
        liveness: HealthCheck,
        readyness: HealthCheck,
        channel_http_kill: tokio::sync::mpsc::Sender<()>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path(basepath).and(
            liveness_check(liveness.clone())
                .or(readyness_check(readyness.clone()))
                .or(kill_signal(channel_http_kill))
                .or(prometheus_metrics()),
        )
    }
    pub fn kill_signal(
        channel_http_kill: tokio::sync::mpsc::Sender<()>,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::get()
            .and(warp::path!("kill"))
            .and(with_channel(channel_http_kill))
            .and_then(handlers::kill)
    }
    pub fn liveness_check(
        liveness: HealthCheck,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::get()
            .and(warp::path!("alive"))
            .and(with_heathcheck(liveness))
            .and_then(handlers::liveness)
    }
    pub fn readyness_check(
        readyness: HealthCheck,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::get()
            .and(warp::path!("ready"))
            .and(with_heathcheck(readyness))
            .and_then(handlers::readyness)
    }
    pub fn prometheus_metrics(
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::get()
            .and(warp::path("metrics"))
            // .and(with_metrics())
            .and_then(handlers::metrics)
    }

    fn with_channel(
        channel: tokio::sync::mpsc::Sender<()>,
    ) -> impl Filter<Extract = (tokio::sync::mpsc::Sender<()>,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || channel.clone())
    }

    fn with_heathcheck(
        hc: HealthCheck,
    ) -> impl Filter<Extract = (HealthCheck,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || hc.clone())
    }
}

mod handlers {
    use crate::k8slifecycle::HealthCheck;
    use crate::k8slifecycle::REGISTRY;
    use std::convert::Infallible;
    use warp::http::StatusCode;

    pub async fn kill(channel: tokio::sync::mpsc::Sender<()>) -> Result<impl warp::Reply, Infallible> {
        println!("Kill signal received");
        channel.send(()).await.expect("Kill signal should be sent");
        Ok("OK")
    }

    pub async fn liveness(liveness: HealthCheck) -> Result<impl warp::Reply, Infallible> {
        let (happy, detail) = liveness.status();
        println!("Liveness: {}", if happy { "OK" } else { "Fail" });
        Ok(warp::reply::with_status(
            warp::reply::json(&detail),
            if happy {
                StatusCode::OK
            } else {
                StatusCode::REQUEST_TIMEOUT
            },
        ))
    }
    pub async fn readyness(readyness: HealthCheck) -> Result<impl warp::Reply, Infallible> {
        let (happy, detail) = readyness.status();
        println!("Readyness: {}", if happy { "OK" } else { "Fail" });
        Ok(warp::reply::with_status(
            warp::reply::json(&detail),
            if happy {
                StatusCode::OK
            } else {
                StatusCode::REQUEST_TIMEOUT
            },
        ))
    }
    pub async fn metrics() -> Result<impl warp::Reply, Infallible> {
        println!("returning metrics");
        use prometheus::Encoder;
        let encoder = prometheus::TextEncoder::new();
        let mut buffer = Vec::new();
        if let Err(e) = encoder.encode(&REGISTRY.gather(), &mut buffer) {
            eprintln!("could not encode custom metrics: {}", e);
        };
        let mut res = match String::from_utf8(buffer.clone()) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("custom metrics could not be from_utf8'd: {}", e);
                String::default()
            }
        };
        buffer.clear();

        let mut buffer = Vec::new();
        if let Err(e) = encoder.encode(&prometheus::gather(), &mut buffer) {
            eprintln!("could not encode prometheus metrics: {}", e);
        };
        let res_custom = match String::from_utf8(buffer.clone()) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("prometheus metrics could not be from_utf8'd: {}", e);
                String::default()
            }
        };
        buffer.clear();

        res.push_str(&res_custom);
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn health_probe_ticking() {
        //! Test that a HalthProbe provides valid and clears valid when tick'ed

        let mut health_probe = HealthProbe::new("HealthCheck", Duration::from_millis(15));

        health_probe.tick();

        let oldtick = health_probe.time.load(Ordering::SeqCst);
        thread::sleep(Duration::from_millis(10));
        health_probe.tick();
        assert!(oldtick < health_probe.time.load(Ordering::SeqCst));

        assert!(health_probe.valid());
        thread::sleep(Duration::from_millis(20));
        assert!(!health_probe.valid());
        thread::sleep(Duration::from_millis(20));

        // health_probe.tick();
        assert!(!health_probe.valid());
    }

    #[test]
    fn health_check_ticking() {
        //! Test that a HealthCheck provides works correctly returning both happy and detail
        let mut hp0 = HealthProbe::new("HealthCheck0", Duration::from_millis(15));
        let mut hp1 = HealthProbe::new("HealthCheck1", Duration::from_millis(15));

        let mut hc0 = HealthCheck::new("simple");

        hc0.add(&hp0);
        hc0.add(&hp1);

        let (happy, detail) = hc0.status();
        println!("detail = {:?}", detail);
        assert!(happy);
        assert!(detail.len() == 2);
        hp0.tick();

        thread::sleep(Duration::from_millis(20));
        let (happy, _detail) = hc0.status();
        assert!(!happy);
        hp0.tick();
        let (happy, detail) = hc0.status();
        assert!(!happy);
        assert!(detail[&hp0.name]);
        assert!(!detail[&hp1.name]);

        hp1.tick();

        let (happy, detail) = hc0.status();
        assert!(happy);
        assert!(detail[&hp0.name]);
        assert!(detail[&hp1.name]);
    }
}
