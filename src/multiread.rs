use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Instant, Duration};
use std::collections::HashMap;
use atomic::Atomic;

pub fn multiread() -> String {
    "blamonge".to_string()
}

#[derive(Debug)]
struct HealthProbe {
    name: String,
    margin: Duration,
    time: Arc<Atomic<Instant>>
}

impl HealthProbe{
    fn new(name: &str, margin: Duration) ->  HealthProbe {
        // let vec = vec![init_value; size];
        HealthProbe{
            name: name.to_string(),
            margin,
            time: Arc::new(Atomic::new(Instant::now())),
        }
    }
    fn valid(&self) -> bool {
        self.time.load(Ordering::SeqCst).elapsed() <= self.margin
    }
    fn tick(&mut self) {
        println!("ben = {:?}", self);
        println!("sc = {}", Arc::strong_count(&self.time));
        self.time.store(Instant::now(), Ordering::SeqCst);
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

struct HealthCheck {
    name: String,
    probe_list: Vec<HealthProbe>,
}

impl HealthCheck{
    pub fn new(name: &str) -> HealthCheck {
        println!("creating HealthCheck({})", name);
        HealthCheck{
            name: name.to_string(),
            probe_list: Vec::new(),
        }
    }
    pub fn add(&mut self, probe: &HealthProbe) {
        println!("Adding probe {:?}", probe);
        self.probe_list.push(probe.clone());
    }

    pub fn status(&self) -> (bool, HashMap<String, bool>) {
        let mut happy = true;

        let detail: HashMap<_, _> = self
            .probe_list
            .iter()
            .map(|x| {
                let tempme = x;
                if !tempme.valid() {
                    happy = false;
                }
                (tempme.name.clone(), tempme.valid())
            })
            .collect();
        (happy, detail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn probe_cloning() {

        let mut probe0 = HealthProbe::new("probe0", Duration::from_secs(180));
        probe0.tick();
        {
            let mut probe1 = probe0.clone();
            println!("probe0 = {:?}", probe0);
            println!("probe1 = {:?}", probe1);
            probe0.tick();
            probe1.tick();
            println!("probe0 = {:?}", probe0);
            println!("probe1 = {:?}", probe1);
        }
        probe0.tick();
        println!("probe0 = {:?}", probe0);
    }
    #[test]
    fn simple_test() {
        println!("simple test {}", multiread());

        let mut hp = HealthCheck::new("roy");

        println!("Healthcheck = {}", hp.name);
        let (happy, detail) = hp.status();
        println!("happy = {:?}, detail = {:?}", happy, detail);


        let mut probe0 = HealthProbe::new("probe0", Duration::from_secs(180));

        hp.add(&probe0);

        let (happy, detail) = hp.status();
        println!("happy = {:?}, detail = {:?}", happy, detail);

        println!("main test done");








        let mut ben = vec![2;5];

        for item in &ben {
            println!("ben []= {}", item);
        }

        ben[3]=7;
        ben.push(19);

        for item in &ben {
            println!("ben2 []= {}", item);
        }


    }
}
