use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use warp::Filter;
use atomic::Atomic;

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
        HealthProbe{
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
        println!("Creating HealthCheck");

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
            .probe_list.lock().unwrap()
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
) {
    println!("Starting health http on {}", port);

    let api = filters::health(basepath, liveness.clone(), readyness.clone());

    let routes = api.with(warp::log("health"));

    println!("Starting health service");
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}


mod filters {
    use warp::Filter;
    use crate::k8slifecycle::HealthCheck;
    use super::handlers;

    pub fn health(
        basepath: &'static str,
        liveness: HealthCheck,
        readyness: HealthCheck,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path(basepath)
            .and(
                liveness_check(liveness.clone())
                .or(readyness_check(readyness.clone()))
            )
    }

    pub fn liveness_check(
        liveness: HealthCheck,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::get()
            .and(warp::path!("alive"))
            .and(with_heathcheck(liveness))
            .and_then(handlers::liveness)
    }
    pub fn readyness_check(
        readyness: HealthCheck,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::get()
            .and(warp::path!("ready"))
            .and(with_heathcheck(readyness))
            .and_then(handlers::readyness)
    }

    fn with_heathcheck(hc: HealthCheck) -> impl Filter<Extract = (HealthCheck,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || hc.clone())
    }
}

mod handlers {
    use std::convert::Infallible;
    use warp::http::StatusCode;
    use crate::k8slifecycle::HealthCheck;

    pub async fn liveness(liveness: HealthCheck) -> Result<impl warp::Reply, Infallible> {
        let (happy, detail) = liveness.status();
        println!("Liveness: {}", if happy {"OK"} else {"Fail"});
        Ok(warp::reply::with_status(warp::reply::json(&detail), if happy {StatusCode::OK} else {StatusCode::REQUEST_TIMEOUT}))
    }
    pub async fn readyness(readyness: HealthCheck) -> Result<impl warp::Reply, Infallible> {
        let (happy, detail) = readyness.status();
        println!("Readyness: {}", if happy {"OK"} else {"Fail"});
        Ok(warp::reply::with_status(warp::reply::json(&detail), if happy {StatusCode::OK} else {StatusCode::REQUEST_TIMEOUT}))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

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
