use crate::utils::{ApiTags, MyResponse, ResponseObject};
use poem::Request;
use poem_openapi::{param::Query, OpenApi};
use thirtyfour::prelude::*;
use tokio::sync::{Mutex, MutexGuard};
use tracing::error;

pub struct Selenium {
    driver: Mutex<WebDriver>,
}

#[OpenApi(
    prefix_path = "/browser/",
    request_header(name = "API-Key", ty = "String", description = "Private API Key",),
    tag = "ApiTags::Selenium"
)]
impl Selenium {
    // create new instance
    pub fn new(driver: Mutex<WebDriver>) -> Self {
        Self { driver }
    }

    /// get rendered html
    #[oai(path = "/html/", method = "get", operation_id = "browser::get_html")]
    async fn get_html(
        &self,
        req: &Request,
        /// url of the page to render
        url: Query<String>,
    ) -> MyResponse<String> {
        let driver = self.driver.lock().await;

        // extract user id from token
        let api_key = match req.header("API-Key") {
            Some(key) => key,
            None => {
                return ResponseObject::unauthorized("API-Key header is missing");
            }
        };

        if api_key != "123456" {
            return ResponseObject::unauthorized("Invalid API-Key");
        }

        let driver = match self.setup_driver(&driver, url.as_str()).await {
            Ok(d) => d,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to setup driver");
                return ResponseObject::internal_server_error("Failed to setup driver");
            }
        };

        let html = match driver.source().await {
            Ok(h) => h,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to get page source");
                return ResponseObject::internal_server_error("Failed to get page source");
            }
        };

        match self.cleanup_driver(driver, url.as_str()).await {
            Ok(_) => (),
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to cleanup driver");
                return ResponseObject::internal_server_error("Failed to cleanup driver");
            }
        }

        return ResponseObject::ok(html);
    }

    async fn setup_driver<'a>(
        &'a self,
        driver: &'a MutexGuard<'_, WebDriver>,
        url: &str,
    ) -> Result<&MutexGuard<'_, WebDriver>, String> {
        let tab = match driver.new_tab().await {
            Ok(t) => t,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to create new tab");
                return Err("Failed to create new tab".to_string());
            }
        };

        match driver.switch_to_window(tab).await {
            Ok(_) => (),
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to switch to new tab");
                return Err("Failed to switch to new tab".to_string());
            }
        }

        match driver.goto(url.to_string()).await {
            Ok(_) => (),
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to navigate to URL");
                return Err("Failed to navigate to URL".to_string());
            }
        }

        // Wait for page to be loaded.
        match driver.find(By::Tag("img")).await {
            Ok(_) => (),
            Err(_) => (),
        }

        // sleep for 100ms for the page to be fully loaded
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        return Ok(driver);
    }

    async fn cleanup_driver<'a>(
        &'a self,
        driver: &'a MutexGuard<'_, WebDriver>,
        url: &str,
    ) -> Result<(), String> {
        match driver.close_window().await {
            Ok(_) => (),
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to close tab");
                return Err("Failed to close tab".to_string());
            }
        }

        let windows = match driver.windows().await {
            Ok(w) => w,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to get windows");
                return Err("Failed to get windows".to_string());
            }
        };

        let handle = match windows.first() {
            Some(w) => w,
            None => {
                error!(url=?*url, "Failed to get window handle");
                return Err("Failed to get window handle".to_string());
            }
        };

        let handle = handle.clone();

        match driver.switch_to_window(handle).await {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to switch to window");
                return Err("Failed to switch to window".to_string());
            }
        }
    }
}
