use crate::k8slifecycle::{HealthCheck, HealthProbe};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use crate::k8slifecycle::health_listen;

pub struct UServiceConfig {
        pub name: String,
    }

pub fn start(config: &UServiceConfig) {
    println!("uService: Start");
    let mut liveness = HealthCheck::new("liveness");

    let mut probe0 = Rc::new(RefCell::new(HealthProbe::new(
        "HealthCheck",
        Duration::from_secs(30),
    )));
    liveness.add(&probe0);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build runtime");

    let local = tokio::task::LocalSet::new();
    local.block_on(&rt, health_listen("health", 7979, &liveness));

    println!("uService {}: Stop", config.name);

}
