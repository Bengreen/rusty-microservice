use async_trait::async_trait;

use crate::{uservice::HandleChannel, k8slifecycle::{HealthCheck}};


#[async_trait]
pub trait PicoService {
    async fn start(&self, alive_check: &HealthCheck,ready_check: &HealthCheck) -> HandleChannel;
    fn status(&self) -> &str;
}
