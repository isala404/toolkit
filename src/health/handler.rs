use crate::utils::ApiTags;
use poem_openapi::{payload::PlainText, OpenApi};

#[derive(Default)]
pub struct HealthCheck;

#[OpenApi(prefix_path = "/health/", tag = "ApiTags::HealthCheck")]
impl HealthCheck {
    #[oai(path = "/liveness", method = "get")]
    async fn liveness(&self) -> PlainText<String> {
        PlainText("OK".to_string())
    }
    #[oai(path = "/readiness", method = "get")]
    async fn readiness(&self) -> PlainText<String> {
        PlainText("OK".to_string())
    }
}
