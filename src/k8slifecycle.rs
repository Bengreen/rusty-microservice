use std::time::SystemTime;
use std::time::Duration;
use std::{thread, time};


#[derive(Debug)]
pub struct Health {
    name: String,
    margin: Duration,
    time: SystemTime,
}
impl Health {
    fn new(name: &str, margin: Duration) -> Health {
        Health{
            name: name.to_string(),
            margin,
            time: SystemTime::now(),
        }
    }

    fn tick(&mut self) {
        self.time=SystemTime::now();
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn valid(&self) -> bool {
        self.time+self.margin >= SystemTime::now()
    }
}

pub struct Abc {
    name: String,
    probelist: Vec<Health>,
}

impl Abc {
    pub fn new(name: &str) -> Abc {
        println!("Creating Abc");

        Abc {
            name: name.to_string(),
            probelist: Vec::new(),
        }
    }

    fn add(&mut self, probe: Health ) -> &mut Health {
        self.probelist.push(probe);
        self.probelist.last_mut().unwrap()
    }

    fn status(&self) -> bool {
        let mut happy = true;
        for val in self.probelist.iter() {
            if !val.valid() {
                happy = false;
            }
        }
        happy
    }
    // fn add<'a>(&'a mut self, probe: impl HealthProbe + 'a) {
    //     self.probelist.push(Box::new(probe));
    // }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn abc_generation() {
        println!("ready to go");

        let mut abc = Abc::new("def");


        let health0 = abc.add(Health::new("abc", time::Duration::from_millis(15)));
        health0.tick();
        let health1 = abc.add(Health::new("def", time::Duration::from_millis(25)));

        health1.tick();

        println!("health probe = {:?}", health1);
        // println!("health probe = {:?}", health0);

        assert!(abc.status());

    }


    #[test]
    fn health_ticking() {
        println!("ready to go");

        let mut health = Health::new("abc", time::Duration::from_millis(15));

        health.tick();

        let oldtick = health.time;
        thread::sleep(time::Duration::from_millis(10));
        health.tick();
        assert!(oldtick < health.time);

        assert!(health.valid());
        thread::sleep(time::Duration::from_millis(20));
        assert!(!health.valid());

        health.tick();
        assert!(health.valid());
    }






    #[test]
    fn monkey() {
        println!("monkey");
    }
}


