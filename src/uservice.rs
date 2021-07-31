
use crate::k8slifecycle::{HealthProbe, HealthCheck};
use std::time::Duration;
use std::cell::RefCell;
use std::rc::Rc;

pub fn start() {
    println!("uService: Start");
    let mut liveness = HealthCheck::new("liveness");

    let mut probe0 = Rc::new(RefCell::new(HealthProbe::new(
        "HealthCheck",
        Duration::from_secs(30),
    )));
    liveness.add(&probe0);




    println!("uService: Stop");
}