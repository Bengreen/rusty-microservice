//! Create a micro service

use crate::ffi_service::SoService;
use crate::k8slifecycle::health_listen;
use crate::k8slifecycle::HealthCheck;
use crate::picoservice::PicoService;
use futures::future;
use log::info;
use warp::Filter;

use std::collections::HashMap;
use std::mem;
use std::sync::{Arc, Mutex};
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct UServiceConfig {
    pub name: String,
}


/// A UService allowsing multiple pServices to be assembled within it.
/// The UService provides the basic scaffolding for the web service and a loader capabilty to load and service picoservices.
/// The basic UService will reply with status information on the picoservcies provided
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
    version: String,
    port: u16,
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
            version: "v1".to_string(),
            port: 8080,
        }
    }


    /// Initialise the async service and initialise services within it
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


    /// Start async processes for the UService
    /// Start the health service
    /// Start the services themselves and insert appropriate web services
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

        let (channel_svc, kill_recv_svc) = mpsc::channel(1);
        self.add(
            channel_svc,
            self.service_listen(
                kill_recv_svc,
            )
            .await,

        );


        //let kill_send_health = self.kill.as_ref().unwrap().lock().unwrap().clone();

        let (channel_health, kill_recv_health) = mpsc::channel(1);

        self.add(
            channel_health,
            health_listen(
                "health",
                7979,
                &self.liveness,
                &self.readyness,
                kill_recv_health,
                self.kill.as_ref().unwrap().lock().unwrap().clone(),
            )
            .await,
        );

        let channels_register = self.channels.clone();
        tokio::spawn(async move {
            // TODO: Check if this future should be waited via the join

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

            // Once signal handlers have triggered shutdowns then send the kill signal to each registered shutdown
            UService::shutdown(channels_register).await;
        });

        self.join().await;
    }

    /// Create a task that runs in async parallel. Each task must provide a channel that allow sthe task to receive a message and exit
    /// The task must also return a handle which can be awaited to confirm the function has completed successfully.
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

    pub async fn service_listen(
        &self,
        mut kill_recv: Receiver<()>,
    ) -> tokio::task::JoinHandle<()> {

        let api = self.service();

        let routes = api.with(warp::log("uservice"));


        let (_addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], self.port), async move {
            kill_recv.recv().await;
        });

        info!("Serving service ({}) on port {}", self.name, self.port);
        tokio::task::spawn(server)
    }

    fn  with_name(
        &self,
    ) -> impl Filter<Extract = (String,), Error = std::convert::Infallible> + Clone
    {
        let myname = self.name.clone();
        warp::any().map(move || myname.clone())
    }


    pub fn service(&self, ) -> impl Filter<Extract = impl warp::Reply, Error=warp::Rejection> + Clone{
        // let with_services = warp::any().map(move || self.so_services.clone());

        let so_services: Arc<Mutex<HashMap<String, Box<SoService>>>> = self.so_services.clone();

        let myname = self.name.clone();
        let with_so_services = warp::any().map(move || so_services.clone());

        warp::path(self.name.clone())
            .and(warp::path(self.version.clone()))
            .and(warp::path("pservice"))
            .and(warp::get())
            .and(self.with_name())
            // .and(with_so_services)
            .map(|name| {
                // let pservicecount=so_services.lock().unwrap().iter().count();
                format!("Hello {}, whose agent is {}", "self.name", name)
            })
    }

}
