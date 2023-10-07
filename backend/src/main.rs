#[deny(clippy::all)]
use dotenv::dotenv;
use fcm::fcm_api;
use poem::{
    listener::TcpListener,
    middleware::{Cors, Tracing},
    EndpointExt, Route, Server,
};
use poem_openapi::OpenApiService;
use tokio::time::Duration;
use tracing::Level;
use utils::get_db_pool;

mod fcm;
mod utils;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();
    dotenv().ok(); // This line loads the environment variables from the ".env" file.
    let pool = get_db_pool().await;
    let hostname = utils::get_host();

    let fcm_api = fcm_api(pool.clone()).await;

    let api_service = OpenApiService::new(fcm_api, "ToolKit", "1.0")
        .server(format!("http://{}/api/v1", hostname));
    let ui = api_service.swagger_ui().with(utils::BasicAuth::default());
    let spec = api_service
        .spec_endpoint_yaml()
        .with(utils::BasicAuth::default());

    let route = Route::new()
        .nest("/api/v1", api_service)
        .nest("/swagger", ui)
        .nest("/swagger/spec", spec)
        .with(Cors::new())
        .with(Tracing)
        .data(pool.clone());

    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run_with_graceful_shutdown(
            route,
            async move {
                let _ = tokio::signal::ctrl_c().await;
                pool.close().await;
            },
            Some(Duration::from_secs(5)),
        )
        .await
}
