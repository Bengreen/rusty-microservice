
//! Create a micro service


use crate::k8slifecycle::{HealthCheck, HealthProbe};
use std::time::Duration;
use tokio::time::sleep;
use crate::k8slifecycle::health_listen;
use crate::sampleservice::sample_listen;
use std::sync::{Arc, Mutex};
use futures::future;
use std::mem;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct UServiceConfig {
        pub name: String,
    }

#[derive(Debug)]
pub struct HandleChannel {
    pub handle: tokio::task::JoinHandle<()>,
    pub channel: std::sync::mpsc::Sender<()>,
}

pub struct UService {
    pub name: String,
    rt: tokio::runtime::Runtime,
    channels: Arc<Mutex<Vec<std::sync::mpsc::Sender<()>>>>,
    handles: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
}

impl UService {
    pub fn new(name: &str) -> UService {
        UService {
            name: name.to_string(),
            rt: tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
            channels: Arc::new(Mutex::new(vec!())),
            handles: Arc::new(Mutex::new(vec!())),
        }
    }

    pub fn add(&self, hc: HandleChannel) {
        self.handles.lock().unwrap().push(hc.handle);
        self.channels.lock().unwrap().push(hc.channel);
    }

    pub fn shutdown(channels: &Arc<Mutex<Vec<std::sync::mpsc::Sender<()>>>>) {
        for channel in channels.lock().unwrap().iter() {
            match channel.send(()) {
                Ok(_v) => println!("Shutdown signal sent"),
                Err(e) => println!("Error sending close signal: {:?}", e),
            }
        }
    }

    pub async fn join(&self) {
        let mut handles = self.handles.lock().expect("Could not lock mutex for handles");
        future::join_all(mem::take(&mut *handles)).await;
    }
}

async fn simple_loop(probe: &HealthProbe) -> HandleChannel {
    let mut probe = probe.clone();
    let loop_sleep = Duration::from_secs(5);

    let (channel, rx) = std::sync::mpsc::channel();
    let alive = Arc::new(AtomicBool::new(true));

    let handle = tokio::task::spawn(async move {

        let alive_recv= alive.clone();
        tokio::spawn(async move {
            // Speawn a receive channel to close the loop when signal received
            let _reci = rx.recv().expect("Receive close signal");
            alive_recv.store(false, Ordering::Relaxed);
            println!("Setting Loop close stop");
        });

        while alive.load(Ordering::Relaxed) {
            println!("in loop");

            probe.tick();
            sleep(loop_sleep).await;
        }
        println!("Simple loop closed");
    });

    HandleChannel{handle, channel}
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

    let uservice = UService::new(&config.name);


    let register_signal = |signal| {
        let channels_register = uservice.channels.clone();
        unsafe {
            signal_hook::low_level::register(signal, move || {
                println!("Received {} signal", signal);
                UService::shutdown(&channels_register);
            })
        }.expect("Register signal")
    };

    let registered_signals = vec!(
        register_signal(signal_hook::consts::SIGINT),
        register_signal(signal_hook::consts::SIGTERM),
    );


    // This creates the async functions from a non-awsync function
    uservice.rt.block_on(async {
        // ToDo: Look at this for clue on how to run on LocalSet : https://docs.rs/tokio/1.9.0/tokio/task/struct.LocalSet.html
        uservice.add(simple_loop(&time_loop).await);
        uservice.add(health_listen("health", 7979, &liveness, &readyness).await);
        uservice.add(sample_listen("sample", 8080).await);
        uservice.join().await;

        println!("Stopping service");
    });

    for signal in registered_signals {
        signal_hook::low_level::unregister(signal);
    }

    println!("uService {}: Stop", config.name);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use warp::hyper::Client;
    use crate::uservice;

    #[tokio::test]
    async fn service_loading() {
        println!("Loading uService");

        let my_config = UServiceConfig{name: String::from("test0")};

        let ben = thread::spawn(move || {

            uservice::start(&my_config);
            // for i in 1..10 {
            //     println!("hi number {} from the spawned thread! {}", i, my_config.name);
            //     thread::sleep(Duration::from_millis(1));

            // }
        });

        std::thread::sleep(Duration::from_secs(5));

        let client = Client::new();
        let uri = "http://localhost:7979/health/alive".parse().unwrap();
        let resp = client.get(uri).await.unwrap();
        println!("Response: {}", resp.status());
        ben.join().unwrap();
        unreachable!();
    }

}