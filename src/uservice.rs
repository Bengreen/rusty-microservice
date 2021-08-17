use crate::k8slifecycle::{HealthCheck, HealthProbe};
use std::time::Duration;
use crate::k8slifecycle::health_listen;

pub struct UServiceConfig {
        pub name: String,
    }

pub fn start(config: &UServiceConfig) {
    println!("uService: Start");
    let mut liveness = HealthCheck::new("liveness");

    let mut probe0 = HealthProbe::new(
        "HP0",
        Duration::from_secs(60),
    );
    liveness.add(&probe0);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // This creates the async functions from a non-awsync function
    rt.block_on(async {
        // ToDo: Look at this for clue on how to run on LocalSet : https://docs.rs/tokio/1.9.0/tokio/task/struct.LocalSet.html
        health_listen("health", 7979, &liveness).await;
    });

    probe0.tick();

    println!("uService {}: Stop", config.name);

}
