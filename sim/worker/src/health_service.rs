use crate::protos::cao_common::{self, Empty};

#[derive(Clone, Debug)]
pub struct HealthService {}

#[tonic::async_trait]
impl cao_common::health_server::Health for HealthService {
    async fn ping(
        &self,
        _r: tonic::Request<Empty>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        Ok(tonic::Response::new(Empty {}))
    }
}
