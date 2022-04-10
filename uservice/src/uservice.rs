//! Create a micro service

use crate::ffi_service::SoService;
use crate::k8slifecycle::health_listen;
use crate::k8slifecycle::HealthCheck;
use crate::picoservice::PicoService;
use futures::future;
use log::info;

use std::collections::HashMap;
use std::mem;
use std::sync::{Arc, Mutex};
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct UServiceConfig {
    pub name: String,
}

#[derive(Debug)]
pub struct UService<'a> {
    pub name: String,
    // pub rt: tokio::runtime::Runtime,
    channels: Arc<Mutex<Vec<mpsc::Sender<()>>>>,
    handles: Arc<Mutex<Vec<tokio::task::JoinHandle<()>>>>,
    so_services: Arc<Mutex<HashMap<String, Box<SoService<'a>>>>>,
    liveness: HealthCheck,
    readyness: HealthCheck,
    kill: Option<Mutex<Sender<()>>>,
}

impl<'a> UService<'a> {
    pub fn new(name: &str) -> UService {
        UService {
            name: name.to_string(),

            channels: Arc::new(Mutex::new(vec![])),
            handles: Arc::new(Mutex::new(vec![])),
            so_services: Arc::new(Mutex::new(HashMap::new())),
            liveness: HealthCheck::new("liveness"),
            readyness: HealthCheck::new("readyness"),
            kill: None,
        }
    }
    pub fn start(&mut self) {
        info!("Starting uservice here");

        let (channel_kill, rx_kill) = mpsc::channel::<()>(1);
        self.kill = Some(Mutex::new(channel_kill));

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Runtime created in current thread");
        let _guard = rt.enter();

        rt.block_on(self.start_async(rx_kill));

        info!("uService {}: Stopped", self.name);
    }

    pub fn stop(&mut self) {
        self.kill
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .blocking_send(())
            .expect("Send close to async");
    }

    pub fn add_soservice<'b: 'a>(&'b mut self, name: &str, so_service: Box<SoService<'a>>) {
        self.so_services
            .lock()
            .unwrap()
            .insert(name.to_string(), so_service);
        // todo!("add_soservice")
    }

    pub fn remove_soservice(&mut self, name: &str) -> Box<SoService> {
        self.so_services
            .lock()
            .unwrap()
            .remove(name)
            .expect("remove soservice from map")
    }

    pub fn add_picoservice(&mut self, pico: &mut dyn PicoService) {
        info!("Status = {}", pico.status());
        todo!("pico service add")
    }

    pub async fn start_async(&mut self, mut kill_signal: Receiver<()>) {
        info!("Starting ASYNC");

        // let time_loop = HealthProbe::new("Timer", Duration::from_secs(60));
        // liveness.add(&time_loop);

        // uservice.add(init_service().await);

        // Init any service loops at this point
        // Anything created should return a HandleChannel to provide a kill option for the loop AND async handle to allow it to be joined and awaited.

        self.so_services.lock().unwrap().iter().for_each(
            |(name, service)| {
            info!("Dispatching SoService: {}", name);
            (&service.init)(12);
            info!("Called init for {}", name);
            }
        );

        let kill_send = self.kill.as_ref().unwrap().lock().unwrap().clone();

        let (channel, kill_recv) = mpsc::channel(1);

        self.add(
            channel,
            health_listen(
                "health",
                7979,
                &self.liveness,
                &self.readyness,
                kill_recv,
                kill_send,
            )
            .await,
        );

        let channels_register = self.channels.clone();
        tokio::spawn(async move {
            let mut sig_terminate =
                signal(SignalKind::terminate()).expect("Register terminate signal handler");
            let mut sig_quit = signal(SignalKind::quit()).expect("Register quit signal handler");
            let mut sig_hup = signal(SignalKind::hangup()).expect("Register hangup signal handler");

            info!("registered signal handlers");

            tokio::select! {
                _ = tokio::signal::ctrl_c() => info!("Received ctrl-c signal"),
                _ = kill_signal.recv() => info!("Received kill from library"),
                // _ = rx_http_kill.recv() => info!("Received HTTP kill signal"),
                _ = sig_terminate.recv() => info!("Received TERM signal"),
                _ = sig_quit.recv() => info!("Received QUIT signal"),
                _ = sig_hup.recv() => info!("Received HUP signal"),
            };
            info!("Signal handler triggered to start Shutdown");

            UService::shutdown(channels_register).await;
        });

        self.join().await;
    }

    pub fn add(&self, channel: Sender<()>, handle: tokio::task::JoinHandle<()>) {
        self.handles.lock().unwrap().push(handle);
        self.channels.lock().unwrap().push(channel);
    }

    pub async fn shutdown(channels: Arc<Mutex<Vec<mpsc::Sender<()>>>>) {
        let channels = channels.lock().unwrap().clone();

        for channel in channels.iter() {
            let channel_rx = channel.send(()).await;
            match channel_rx {
                Ok(_v) => info!("Shutdown signal sent"),
                Err(e) => info!("Error sending close signal: {:?}", e),
            }
        }
    }

    pub async fn join(&self) {
        let mut handles = self
            .handles
            .lock()
            .expect("Could not lock mutex for handles");
        info!("Waiting for services: {:?}", handles);
        future::join_all(mem::take(&mut *handles)).await;
        info!("Services completed");
    }
}
