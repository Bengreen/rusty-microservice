use std::sync::{Arc, Mutex};
use std::sync::atomic::Ordering;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use warp::Filter;
use atomic::Atomic;
use std::thread;

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
    // fn add<'a>(&'a mut self, probe: impl HealthProbeProbe + 'a) {
    //     self.probelist.push(Box::new(probe));
    // }
}


pub async fn health_listen(basepath: &'static str , port: u16, liveness: &HealthCheck) {
    println!("Starting Health http on {}", port);

    let api = filters::health(liveness.clone());

    let routes = api.with(warp::log("health"));

    println!("Starting health service");
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}



mod filters {
    use warp::Filter;
    use crate::k8slifecycle::HealthCheck;
    use super::handlers;

    pub fn health(liveness: HealthCheck) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        liveness_check(liveness.clone())
    }

    pub fn liveness_check(
        liveness: HealthCheck
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("alive")
            .and(warp::get())
            .and(with_liveness(liveness))
            .and_then(handlers::liveness)
    }

    fn with_liveness(liveness: HealthCheck) -> impl Filter<Extract = (HealthCheck,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || liveness.clone())
    }
}

mod handlers {
    use std::convert::Infallible;
    use warp::http::StatusCode;
    use crate::k8slifecycle::HealthCheck;

    pub async fn liveness(liveness: HealthCheck) -> Result<impl warp::Reply, Infallible> {
        let (happy, detail) = liveness.status();
        Ok(warp::reply::with_status(warp::reply::json(&detail), if happy {StatusCode::OK} else {StatusCode::REQUEST_TIMEOUT}))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter_map() {
        println!("start map test");

        let myvec = vec![1, 2, 3];

        println!("my vector is {:?}", myvec);

        let newvec: Vec<_> = myvec.iter().map(|x| format!("ABC-{}", *x)).collect();
        println!("NEW vector is {:?}", newvec);

        let newmap: HashMap<_, _> = myvec
            .iter()
            .enumerate()
            .map(|(pos, x)| (pos, format!("ABC-{}", *x)))
            .collect();
        println!("NEW hashmap is {:?}", newmap);

        println!("object 1 = {:?}", newmap[&2]);
    }

    #[test]
    fn health_check_generation() {
        println!("ready to go");

        let mut health_check = HealthCheck::new("def");

        let health_probe0 = Rc::new(RefCell::new(HealthProbe::new(
            "HealthCheck",
            Duration::from_millis(15),
        )));
        health_check.add(&health_probe0);

        health_probe0.borrow_mut().tick();

        let health_probe1 = Rc::new(RefCell::new(HealthProbe::new(
            "def",
            Duration::from_millis(25),
        )));
        health_check.add(&health_probe1);

        println!("health_probe0 RC = {}", Rc::strong_count(&health_probe0));

        health_probe1.borrow_mut().tick();

        println!("HealthProbe probe = {:?}", health_probe1);

        println!("health probe status = {:?}", health_check.status());
        assert!(health_check.status().0);
    }

    #[test]
    fn health_probe_ticking() {
        println!("ready to go");

        let mut health_probe = HealthProbe::new("HealthCheck", Duration::from_millis(15));

        health_probe.tick();

        let oldtick = health_probe.time;
        thread::sleep(Duration::from_millis(10));
        health_probe.tick();
        assert!(oldtick < health_probe.time);

        assert!(health_probe.valid());
        thread::sleep(Duration::from_millis(20));
        assert!(!health_probe.valid());

        health_probe.tick();
        assert!(health_probe.valid());
    }

    #[test]
    fn tryout() {
        println!("try this out");

        {
            let ben = Rc::new(HealthProbe::new(
                "HealthCheck",
                Duration::from_millis(15),
            ));

            let a = Rc::clone(&ben);
            println!("TIME = {:?}", a.time);
            println!("RC = {}", Rc::strong_count(&ben));
            {
                let _b = Rc::clone(&ben);
                println!("RC = {}", Rc::strong_count(&ben));
                // b.tick(); cannot borrow as mutable
            }
            println!("RC = {}", Rc::strong_count(&ben));
            println!("TIME = {:?}", a.time);
        }

        {
            let ben = Rc::new(RefCell::new(HealthProbe::new(
                "HealthCheck",
                Duration::from_millis(15),
            )));

            let a = Rc::clone(&ben);
            println!("TIME = {:?}", a.borrow().time);
            println!("RC = {}", Rc::strong_count(&ben));
            {
                let _b = Rc::clone(&ben);
                println!("RC = {}", Rc::strong_count(&ben));
            }
            ben.borrow_mut().tick();

            println!("RC = {}", Rc::strong_count(&ben));
            println!("TIME = {:?}", a.borrow().time);
        }

        if false {
            // This breaks the mut and immute at same time rules
            let ben = RefCell::new(HealthProbe::new(
                "HealthCheck",
                Duration::from_millis(15),
            ));

            let a = ben.borrow();
            println!("TIME = {:?}", a);
            // println!("RC = {}", Rc::strong_count(&ben));
            {
                let b = ben.borrow();
                println!("RC = {:?}", b);
            }
            ben.borrow_mut().tick();

            println!("TIME = {:?}", a);
        }

        println!("DONE");
    }

    #[test]
    fn monkey() {
        println!("monkey");
    }
}
