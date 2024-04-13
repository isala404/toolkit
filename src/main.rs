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
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use utils::get_db_pool;

mod browser;
mod fcm;
mod health;
mod utils;
mod yt_dlp;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok(); // This line loads the environment variables from the ".env" file.
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_env("RUST_LOG"))
        .init();

    let pool = get_db_pool().await;
    let hostname = utils::get_host();
    let port = utils::get_port();

    let fcm_api = fcm_api(pool.clone()).await;

    let (browser_api, driver) = browser::selenium().await;
    let yt_dlp_api = yt_dlp::yt_dlp().await;

    let health_api = health::health_checks(pool.clone(), browser_api.clone()).await;

    let api_service = OpenApiService::new(
        (fcm_api, browser_api, health_api, yt_dlp_api),
        "ToolKit",
        "1.0",
    )
    .server(format!("{}/api/v1", hostname));
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

    Server::new(TcpListener::bind(format!("0.0.0.0:{}", port)))
        .run_with_graceful_shutdown(
            route,
            async move {
                let _ = tokio::signal::ctrl_c().await;
                pool.close().await;
                _ = driver.quit().await;
            },
            Some(Duration::from_secs(5)),
        )
        .await
}
