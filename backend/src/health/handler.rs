use poem_openapi::{OpenApi, payload::PlainText};
use crate::utils::ApiTags;


#[derive(Default)]
pub struct HealthCheck;

#[OpenApi(
    prefix_path = "/healthz/",
    tag = "ApiTags::HealthCheck",
)]
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
