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
        .unwrap();
        // .expect("build runtime");

    // rt.block_on(health_listen("health", 7979, &liveness));

    // let local = tokio::task::LocalSet::new();
    // local.block_on(&rt, health_listen("health", 7979, &liveness));


    // This creates the async functions from a non-awsync function
    rt.block_on(async {
        println!("hello");

        let local = tokio::task::LocalSet::new();
        local.run_until(async move {
            println!("GOT HRERE");
            health_listen("health", 7979, &liveness).await;
            println!("DONE");
        }).await;
    });



    println!("uService {}: Stop", config.name);

}
