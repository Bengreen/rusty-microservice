use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::{k8slifecycle::{HealthCheck}};


#[async_trait]
pub trait PicoService {
    /** Start the service passing in health checks and kill signals
     * Return a future to allow waiting for the completion
     */
    async fn start(&self, alive_check: &HealthCheck, ready_check: &HealthCheck, mut kill: mpsc::Receiver<()>) -> tokio::task::JoinHandle<()>;
    fn status(&self) -> &str;
}
