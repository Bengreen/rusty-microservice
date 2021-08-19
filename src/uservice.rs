use crate::k8slifecycle::{HealthCheck, HealthProbe};
use std::time::Duration;
use crate::k8slifecycle::health_listen;
use tokio::time::sleep;

pub struct UServiceConfig {
        pub name: String,
    }


async fn simple_loop(probe: &HealthProbe) {
    let mut probe = probe.clone();
    loop {
        probe.tick();
        println!("in loop ");
        sleep(Duration::from_secs(30)).await
    }
}

pub fn start(config: &UServiceConfig) {
    println!("uService: Start");
    let mut liveness = HealthCheck::new("liveness");
    let readyness = HealthCheck::new("readyness");

    let time_loop = HealthProbe::new(
        "Timer",
        Duration::from_secs(60),
    );
    liveness.add(&time_loop);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    // This creates the async functions from a non-awsync function
    rt.block_on(async {
        // ToDo: Look at this for clue on how to run on LocalSet : https://docs.rs/tokio/1.9.0/tokio/task/struct.LocalSet.html
        tokio::join!(
            health_listen("health", 7979, &liveness, &readyness),
            simple_loop(&time_loop),
        );
    });

    // probe0.tick();

    println!("uService {}: Stop", config.name);

}
