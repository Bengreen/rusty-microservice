//! supporting functions for a microservice

use crate::uservice::HandleChannel;
use atomic::Atomic;
use lazy_static::lazy_static;
use log::info;
use prometheus::{HistogramOpts, HistogramVec, IntCounter, IntCounterVec, Opts, Registry};
use std::collections::HashMap;
use std::ptr;
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

/// A structure to create kubernetes [HealthProbe]s
///
/// [HealthProbe]s provide the low level mechanism to instrument lifecycle checks within code. These are added to [HealthCheck]s to create a k8s health check.
/// The [HealthCheck] is used to generate the alive or ready calls for kubernetes services.
#[derive(Debug)]
pub struct HealthProbe {
    /// name of the [HealthProbe] used to make named responses to [HealthCheck]
    name: String,
    /// Time by which the [HealthProbe] can remain un[tick](HealthProbe::tick)ed before it reports failed
    margin: Duration,
    /// Time of last checkin
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

    /// Trigger an update of the [HealthProbe] keeping it wthin the time [HealthProbe::margin]
    pub fn tick(&mut self) {
        self.time.store(Instant::now(), Ordering::SeqCst);
    }

    /// Check and reply if the probe is valid
    ///
    /// Valid means the probe has been [ticked](HealthCheck::tick) within the [HealthProbe::margin]
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

/// A structure to create kubernetes health checks.
///
/// [HealthProbe]s can be added to it and these then must be updated at regular intervals or will result in failing the [HealthCheck].
/// Summary information is provided via the [HealthCheck::status] fucntion to return the current state of the HealthCheck
///
/// The design is such that multiple [HealthProbe]s can be created and added to [HealthCheck]s each [HealthProbe] has its on time based time based configurations to enable it to be calculcated if the probe is still valid or not.
///
/// During operation the [HealthProbe] is updated by the service. The service does not need any direct relationship with the [HealthCheck]
///
#[derive(Clone, Debug)]
pub struct HealthCheck {
    /// A name for the [HealthCheck]
    name: String,
    /// an internal list of the [HealthProbe]s attached to the [HealthCheck]
    probe_list: Arc<Mutex<Vec<HealthProbe>>>,
}

impl HealthCheck {
    /// Create new [HealthCheck]
    pub fn new(name: &str) -> HealthCheck {
        info!("Creating HealthCheck: {}", name);

        HealthCheck {
            name: name.to_string(),
            probe_list: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add [HealthProbe] to [HealthCheck]
    pub fn add(&self, probe: &HealthProbe) {
        self.probe_list.lock().unwrap().push(probe.clone());
    }

    pub fn remove(&self, probe: &HealthProbe) {
        self.probe_list.lock().unwrap().retain(|x| ptr::eq(x, probe));
    }

    /// get status which is a json'able object providing detail info on [HealthProbe] and a bool to summarise
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
    info!("Starting health http on {}", port);

    register_custom_metrics();

    let api = filters::health(
        basepath,
        liveness.clone(),
        readyness.clone(),
        channel_http_kill,
    );

    let routes = api.with(warp::log("health"));

    info!("Starting health service");

    let (channel, mut rx) = mpsc::channel(1);

    let (_addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], port), async move {
            rx.recv().await;
        });

    let handle = tokio::task::spawn(server);

    HandleChannel { handle, channel }
}

/// The filters through used to build up the http route for the k8s health system
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
            liveness_check(liveness)
                .or(readyness_check(readyness))
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
    ) -> impl Filter<Extract = (tokio::sync::mpsc::Sender<()>,), Error = std::convert::Infallible> + Clone
    {
        warp::any().map(move || channel.clone())
    }

    fn with_heathcheck(
        hc: HealthCheck,
    ) -> impl Filter<Extract = (HealthCheck,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || hc.clone())
    }
}

/// Handlers to reply to k8s health requests
///
/// All health k8s health handlers are provided here. These reply to k8s alive, ready and prometheus metrics.
mod handlers {
    use crate::k8slifecycle::HealthCheck;
    use crate::k8slifecycle::REGISTRY;
    use log::{debug, info};
    use std::convert::Infallible;
    use warp::http::StatusCode;

    /// Creates a signal to close the uservice cleanly
    pub async fn kill(
        channel: tokio::sync::mpsc::Sender<()>,
    ) -> Result<impl warp::Reply, Infallible> {
        info!("Kill signal received");
        channel.send(()).await.expect("Kill signal should be sent");
        Ok("OK")
    }

    /// response for k8s alive check
    pub async fn liveness(liveness: HealthCheck) -> Result<impl warp::Reply, Infallible> {
        let (happy, detail) = liveness.status();
        debug!("Liveness: {}", if happy { "OK" } else { "Fail" });
        Ok(warp::reply::with_status(
            warp::reply::json(&detail),
            if happy {
                StatusCode::OK
            } else {
                StatusCode::REQUEST_TIMEOUT
            },
        ))
    }

    /// response for k8s readyness check
    pub async fn readyness(readyness: HealthCheck) -> Result<impl warp::Reply, Infallible> {
        let (happy, detail) = readyness.status();
        debug!("Readyness: {}", if happy { "OK" } else { "Fail" });
        Ok(warp::reply::with_status(
            warp::reply::json(&detail),
            if happy {
                StatusCode::OK
            } else {
                StatusCode::REQUEST_TIMEOUT
            },
        ))
    }

    /// provide [Prometheus](https://prometheus.io) metrics
    pub async fn metrics() -> Result<impl warp::Reply, Infallible> {
        debug!("Returning metrics");
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
        //! Test that a HealthProbe provides valid and clears valid when tick'ed

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

        let hc0 = HealthCheck::new("simple");

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
