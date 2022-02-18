use std::{sync::{atomic::AtomicBool, Arc}, time::Duration};

use async_trait::async_trait;
use atomic::Ordering;
use log::info;
use tokio::{time::sleep, sync::mpsc};

use crate::{uservice::HandleChannel, k8slifecycle::{HealthProbe, HealthCheck}};
use crate::{picoservice::PicoService};


struct SimplePicoLoop {
    pub name: String,
    loop_sleep: Duration,
}

impl SimplePicoLoop {
    fn new(name: &str, loop_sleep: Duration) -> SimplePicoLoop {
        SimplePicoLoop {
            name: name.to_string(),
            loop_sleep,
        }
    }
}
impl Drop for SimplePicoLoop {
    fn drop(&mut self) {
        println!("Dropping SimplePico! {}", self.name);
    }
}

#[async_trait]
impl PicoService for SimplePicoLoop {
    fn status(&self) -> &str {
        return &self.name
    }

    async fn start(&self, alive_check: &HealthCheck, _ready_check: &HealthCheck) -> HandleChannel {
        let alive_check = alive_check.clone();

        let mut alive_probe = HealthProbe::new(&self.name, Duration::from_secs(30));

        alive_check.add(&alive_probe);

        let my_sleep = self.loop_sleep; // Copy so we dont have to Send the self into the future

        let (channel, mut rx) = mpsc::channel(1);
        let alive = Arc::new(AtomicBool::new(true));

        let handle = tokio::spawn(async move {
            let alive_recv = alive.clone();
            tokio::spawn(async move {
                // Spawn a receive channel to close the loop when signal received
                let _reci = rx.recv().await;
                alive_recv.store(false, Ordering::Relaxed);
                info!("Setting Loop close stop");
            });

            while alive.load(Ordering::Relaxed) {
                info!("in loop");

                alive_probe.tick();
                sleep(my_sleep).await;
            }
            alive_check.remove(&alive_probe);

            info!("Simple loop closed");
        });

        HandleChannel { handle, channel }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn simple_test_exec() {

        let my_pico = SimplePicoLoop::new("apple", Duration::from_secs(10));
        let alive_check = HealthCheck::new("alive");
        let ready_check = HealthCheck::new("ready");

        println!("mypico status = {}", my_pico.status());

        let my_handle = my_pico.start(&alive_check,&ready_check).await;

        println!("status = {:?}", alive_check.status());
        sleep(Duration::from_secs(5)).await;


        my_handle.channel.send(()).await.expect("Send completed");

        my_handle.handle.await.expect("Task should not panic");

        println!("have joined");

        println!("status = {:?}", alive_check.status());


        assert!(true);
    }
}