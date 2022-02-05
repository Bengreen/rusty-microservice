//! Sample microservice demonstrating lifecycle hooks and small runtime loop with health probe included.

use crate::uservice::HandleChannel;
use tokio::sync::mpsc;
use warp::Filter;
use log::{info};

mod filters {
    use super::handlers;
    use warp::Filter;

    pub fn sample(
        basepath: &'static str,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path(basepath).and(sample_1())
    }

    pub fn sample_1() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone
    {
        warp::get()
            .and(warp::path!("sample1"))
            .and_then(handlers::sample_h)
    }
}

mod handlers {
    use std::convert::Infallible;
    use log::{info};
    use tokio::time::{sleep, Duration};

    pub async fn sample_h() -> Result<impl warp::Reply, Infallible> {
        info!("Sample called");
        let wait: u8 = rand::random();
        sleep(Duration::from_millis(u64::from(100+wait/5))).await; // simulate some random work
        Ok("Sample")
    }
}

pub async fn sample_listen<'a>(basepath: &'static str, port: u16) -> HandleChannel {
    info!("Starting sample service http on {}", port);

    let api = filters::sample(basepath);

    let routes = api.with(warp::log("sample"));
    let (channel, mut rx) = mpsc::channel(1);

    let (_addr, server) =
        warp::serve(routes).bind_with_graceful_shutdown(([0, 0, 0, 0], port), async move {
            rx.recv().await;
        });

    let handle = tokio::task::spawn(server);

    HandleChannel { handle, channel }
}
