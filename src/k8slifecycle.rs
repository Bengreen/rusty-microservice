use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use std::time::SystemTime;
use std::{thread, time};

#[derive(Debug)]
pub struct HealthProbe {
    name: String,
    margin: Duration,
    time: SystemTime,
}
impl HealthProbe {
    fn new(name: &str, margin: Duration) -> HealthProbe {
        HealthProbe {
            name: name.to_string(),
            margin,
            time: SystemTime::now(),
        }
    }

    fn tick(&mut self) {
        self.time = SystemTime::now();
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn valid(&self) -> bool {
        self.time + self.margin >= SystemTime::now()
    }
}

pub struct HealthCheck {
    name: String,
    probelist: Vec<Rc<RefCell<HealthProbe>>>,
}

impl HealthCheck {
    pub fn new(name: &str) -> HealthCheck {
        println!("Creating HealthCheck");

        HealthCheck {
            name: name.to_string(),
            probelist: Vec::new(),
        }
    }

    fn add(&mut self, probe: &Rc<RefCell<HealthProbe>>) {
        self.probelist.push(Rc::clone(probe));
        // self.probelist.last_mut().unwrap()
    }

    fn status(&self) -> bool {
        let mut happy = true;
        for val in self.probelist.iter() {
            if !val.borrow().valid() {
                happy = false;
            }
        }
        happy
    }
    // fn add<'a>(&'a mut self, probe: impl HealthProbeProbe + 'a) {
    //     self.probelist.push(Box::new(probe));
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_check_generation() {
        println!("ready to go");

        let mut health_check = HealthCheck::new("def");

        let health_probe0 = Rc::new(RefCell::new(HealthProbe::new(
            "HealthCheck",
            time::Duration::from_millis(15),
        )));
        health_check.add(&health_probe0);

        health_probe0.borrow_mut().tick();

        let health_probe1 = Rc::new(RefCell::new(HealthProbe::new(
            "def",
            time::Duration::from_millis(25),
        )));
        health_check.add(&health_probe1);

        println!("health_probe0 RC = {}", Rc::strong_count(&health_probe0));

        health_probe1.borrow_mut().tick();

        println!("HealthProbe probe = {:?}", health_probe1);

        assert!(health_check.status());
    }

    #[test]
    fn health_probe_ticking() {
        println!("ready to go");

        let mut health_probe = HealthProbe::new("HealthCheck", time::Duration::from_millis(15));

        health_probe.tick();

        let oldtick = health_probe.time;
        thread::sleep(time::Duration::from_millis(10));
        health_probe.tick();
        assert!(oldtick < health_probe.time);

        assert!(health_probe.valid());
        thread::sleep(time::Duration::from_millis(20));
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
                time::Duration::from_millis(15),
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
                time::Duration::from_millis(15),
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
                time::Duration::from_millis(15),
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
