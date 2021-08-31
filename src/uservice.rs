//! Create a micro service

use crate::k8slifecycle::health_listen;
use crate::k8slifecycle::{HealthCheck, HealthProbe};
use crate::sampleservice::sample_listen;
use futures::future;
use std::mem;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc;
use tokio::time::sleep;

pub struct UServiceConfig {
    pub name: String,
}

#[derive(Debug)]
pub struct HandleChannel {
    pub handle: tokio::task::JoinHandle<()>,
    pub channel: mpsc::Sender<()>,
}

pub struct UService {
    pub name: String,
    rt: tokio::runtime::Runtime,
    channels: Arc<Mutex<Vec<mpsc::Sender<()>>>>,
    handles: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
}

impl UService {
    pub fn new(name: &str) -> UService {
        UService {
            name: name.to_string(),
            rt: tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Runtime created in current thread"),
            channels: Arc::new(Mutex::new(vec![])),
            handles: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn add(&self, hc: HandleChannel) {
        self.handles.lock().unwrap().push(hc.handle);
        self.channels.lock().unwrap().push(hc.channel);
    }

    pub async fn shutdown(channels: Arc<Mutex<Vec<mpsc::Sender<()>>>>) {
        let ben = channels.lock().unwrap().clone();

        for channel in ben.iter() {
            let channel_rx = channel.send(()).await;
            match channel_rx {
                Ok(_v) => println!("Shutdown signal sent"),
                Err(e) => println!("Error sending close signal: {:?}", e),
            }
        }
    }

    pub async fn join(&self) {
        let mut handles = self
            .handles
            .lock()
            .expect("Could not lock mutex for handles");
        println!("Waiting for services: {:?}", handles);
        future::join_all(mem::take(&mut *handles)).await;
        println!("Services completed");
    }
}

async fn simple_loop(probe: &HealthProbe) -> HandleChannel {
    let mut probe = probe.clone();
    let loop_sleep = Duration::from_secs(5);

    let (channel, mut rx) = mpsc::channel(1);
    let alive = Arc::new(AtomicBool::new(true));

    let handle = tokio::spawn(async move {
        let alive_recv = alive.clone();
        tokio::spawn(async move {
            // Speawn a receive channel to close the loop when signal received
            let _reci = rx.recv().await;
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

    HandleChannel { handle, channel }
}

pub fn start(config: &UServiceConfig) {
    println!("uService: Start");
    let mut liveness = HealthCheck::new("liveness");
    let readyness = HealthCheck::new("readyness");

    let time_loop = HealthProbe::new("Timer", Duration::from_secs(60));
    liveness.add(&time_loop);

    let uservice = UService::new(&config.name);

    let _guard = uservice.rt.enter();
    // This creates the async functions from a non-async function and uses .enter to ensure context
    uservice.rt.block_on(async {
        // ToDo: Look at this for clue on how to run on LocalSet : https://docs.rs/tokio/1.9.0/tokio/task/struct.LocalSet.html
        let (channel_http_kill, mut rx_http_kill) = mpsc::channel::<()>(1);

        uservice.add(simple_loop(&time_loop).await);
        uservice.add(health_listen("health", 7979, &liveness, &readyness, channel_http_kill).await);
        uservice.add(sample_listen("sample", 8080).await);

        let channels_register = uservice.channels.clone();
        tokio::spawn(async move {
            let mut sig_terminate =
                signal(SignalKind::terminate()).expect("Register terminate signal handler");
            let mut sig_quit = signal(SignalKind::quit()).expect("Register quit signal handler");
            let mut sig_hup = signal(SignalKind::hangup()).expect("Register hangup signal handler");

            println!("registered signal handlers");
            tokio::select! {
                _ = rx_http_kill.recv() => println!("Received HTTP kill signal"),
                _ = sig_terminate.recv() => println!("Received TERM signal"),
                _ = sig_quit.recv() => println!("Received QUIT signal"),
                _ = sig_hup.recv() => println!("Received HUP signal"),
            };
            println!("Signal handler triggered to start Shutdown");

            UService::shutdown(channels_register).await;
        });

        uservice.join().await;
    });

    println!("uService {}: Stop", config.name);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uservice;
    use std::thread;
    use warp::hyper::Client;

    async fn send_kill() {
        let client = Client::new();
        let uri = "http://localhost:7979/health/kill".parse().unwrap();
        let resp = client.get(uri).await.unwrap();
        println!("Response: {}", resp.status());
    }

    #[tokio::test]
    async fn service_loading() {
        println!("Loading uService");

        let my_config = UServiceConfig {
            name: String::from("test0"),
        };

        let ben = thread::spawn(move || {
            uservice::start(&my_config);
            // for i in 1..10 {
            //     println!("hi number {} from the spawned thread! {}", i, my_config.name);
            //     thread::sleep(Duration::from_millis(1));
            // }
        });
        println!("Waiting for the 5 secs");
        std::thread::sleep(Duration::from_secs(5));

        let client = Client::new();
        let uri = "http://localhost:7979/health/alive".parse().unwrap();
        let resp = client.get(uri).await.unwrap();
        println!("Response: {}", resp.status());

        send_kill().await;

        ben.join().unwrap();
        // unreachable!();
    }
}
