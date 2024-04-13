use super::model::Image;
use crate::utils::{verify_apikey, ApiTags, JsonError, JsonSuccess, ResponseObject};
use base64::{engine::general_purpose, Engine as _};
use poem::{Request, Result};
use poem_openapi::{param::Query, OpenApi};
use std::sync::Arc;
use thirtyfour::prelude::*;
use tokio::sync::{Mutex, MutexGuard};
use tracing::error;

#[derive(Clone)]
pub struct Selenium {
    driver: Arc<Mutex<WebDriver>>,
}

#[OpenApi(
    prefix_path = "/browser/",
    request_header(name = "API-Key", ty = "String", description = "Private API Key",),
    tag = "ApiTags::Selenium"
)]
impl Selenium {
    // create new instance
    pub fn new(driver: Arc<Mutex<WebDriver>>) -> Self {
        Selenium { driver }
    }

    /// get rendered html
    #[oai(path = "/html/", method = "get", operation_id = "browser::get_html")]
    async fn get_html(
        &self,
        req: &Request,
        /// url of the page to render
        url: Query<String>,
        /// delay in milliseconds wait for the page to be fully loaded
        delay: Query<u64>,
        /// whether to try to bypass paywall
        bypass_paywall: Query<bool>,
    ) -> Result<JsonSuccess<String>, JsonError<String>> {
        let driver = self.driver.lock().await;

        match verify_apikey(req).await {
            Ok(_) => (),
            Err(e) => {
                return Err(ResponseObject::unauthorized(e));
            }
        }

        let mut url = url.0;
        if bypass_paywall.0 {
            url = format!("https://12ft.io/{}", url);
        }

        let driver = match self.setup_driver(&driver, url.as_str()).await {
            Ok(d) => d,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to setup driver");
                return Err(ResponseObject::internal_server_error(
                    "Failed to setup driver",
                ));
            }
        };
        tokio::time::sleep(tokio::time::Duration::from_millis(delay.0)).await;

        let html = match driver.source().await {
            Ok(h) => h,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to get page source");
                return Err(ResponseObject::internal_server_error(
                    "Failed to get page source",
                ));
            }
        };

        match self.cleanup_driver(driver, url.as_str()).await {
            Ok(_) => (),
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to cleanup driver");
                return Err(ResponseObject::internal_server_error(
                    "Failed to cleanup driver",
                ));
            }
        }

        Ok(ResponseObject::ok(html))
    }

    /// get body text of the rendered page
    #[oai(path = "/text/", method = "get", operation_id = "browser::get_text")]
    async fn get_text(
        &self,
        req: &Request,
        /// url of the page to render
        url: Query<String>,
        /// delay in milliseconds wait for the page to be fully loaded
        delay: Query<u64>,
        /// whether to try to bypass paywall
        bypass_paywall: Query<bool>,
    ) -> Result<JsonSuccess<String>, JsonError<String>> {
        let driver = self.driver.lock().await;

        match verify_apikey(req).await {
            Ok(_) => (),
            Err(e) => {
                return Err(ResponseObject::unauthorized(e));
            }
        }

        let mut url = url.0;
        if bypass_paywall.0 {
            url = format!("https://12ft.io/api/proxy?ref=&q={}", url);
        }

        let driver = match self.setup_driver(&driver, url.as_str()).await {
            Ok(d) => d,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to setup driver");
                return Err(ResponseObject::internal_server_error(
                    "Failed to setup driver",
                ));
            }
        };
        tokio::time::sleep(tokio::time::Duration::from_millis(delay.0)).await;

        let body = match driver.find(By::Tag("body")).await {
            Ok(h) => h,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to get the body of the page");
                return Err(ResponseObject::internal_server_error(
                    "Failed to get the body of the page",
                ));
            }
        };

        let text = match body.text().await {
            Ok(t) => t,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to get the text of the body");
                return Err(ResponseObject::internal_server_error(
                    "Failed to get the text of the body",
                ));
            }
        };

        match self.cleanup_driver(driver, url.as_str()).await {
            Ok(_) => (),
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to cleanup driver");
                return Err(ResponseObject::internal_server_error(
                    "Failed to cleanup driver",
                ));
            }
        }

        Ok(ResponseObject::ok(text))
    }

    /// get screenshot of the rendered page
    #[oai(
        path = "/screenshot/",
        method = "get",
        operation_id = "browser::get_screenshot"
    )]
    async fn get_screenshot(
        &self,
        req: &Request,
        /// url of the page to render
        url: Query<String>,
        /// delay in milliseconds wait for the page to be fully loaded
        delay: Query<u64>,
        /// whether to try to bypass paywall
        bypass_paywall: Query<bool>,
    ) -> Result<JsonSuccess<String>, JsonError<Option<String>>> {
        let driver = self.driver.lock().await;

        match verify_apikey(req).await {
            Ok(_) => (),
            Err(e) => {
                return Err(ResponseObject::unauthorized(e));
            }
        }

        let mut url = url.0;
        if bypass_paywall.0 {
            url = format!("https://12ft.io/api/proxy?ref=&q={}", url);
        }

        let driver = match self.setup_driver(&driver, url.as_str()).await {
            Ok(d) => d,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to setup driver");
                return Err(ResponseObject::internal_server_error(
                    "Failed to setup driver",
                ));
            }
        };
        tokio::time::sleep(tokio::time::Duration::from_millis(delay.0)).await;

        let screenshot = match driver.screenshot_as_png().await {
            Ok(h) => h,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to get the screenshot of the page");
                return Err(ResponseObject::internal_server_error(
                    "Failed to get the screenshot of the page",
                ));
            }
        };
        let b64 = general_purpose::STANDARD.encode(screenshot);

        match self.cleanup_driver(driver, url.as_str()).await {
            Ok(_) => (),
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to cleanup driver");
                return Err(ResponseObject::internal_server_error(
                    "Failed to cleanup driver",
                ));
            }
        }

        Ok(ResponseObject::ok(format!("data:image/png;base64,{}", b64)))
    }

    /// get list of images in the rendered page
    #[oai(
        path = "/images/",
        method = "get",
        operation_id = "browser::get_images"
    )]
    async fn get_images(
        &self,
        req: &Request,
        /// url of the page to render
        url: Query<String>,
        /// delay in milliseconds wait for the page to be fully loaded
        delay: Query<u64>,
        /// whether to try to bypass paywall
        bypass_paywall: Query<bool>,
    ) -> Result<JsonSuccess<Vec<Image>>, JsonError<String>> {
        let driver = self.driver.lock().await;

        match verify_apikey(req).await {
            Ok(_) => (),
            Err(e) => {
                return Err(ResponseObject::unauthorized(e));
            }
        }

        let mut url = url.0;
        if bypass_paywall.0 {
            url = format!("https://12ft.io/api/proxy?ref=&q={}", url);
        }

        let driver = match self.setup_driver(&driver, url.as_str()).await {
            Ok(d) => d,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to setup driver");
                return Err(ResponseObject::internal_server_error(
                    "Failed to setup driver",
                ));
            }
        };
        tokio::time::sleep(tokio::time::Duration::from_millis(delay.0)).await;

        let images = match driver.find_all(By::Tag("img")).await {
            Ok(h) => h,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to get the images of the page");
                return Err(ResponseObject::internal_server_error(
                    "Failed to get the images of the page",
                ));
            }
        };

        let mut images_vec = Vec::new();

        for image in images {
            let src = match image.attr("src").await {
                Ok(s) => s,
                Err(e) => {
                    error!(url=?*url, error=?e, "Failed to get the src of the image");
                    continue;
                }
            };
            let alt = match image.attr("alt").await {
                Ok(s) => s,
                Err(e) => {
                    error!(url=?*url, error=?e, "Failed to get the alt of the image");
                    None
                }
            };

            if src.is_none() {
                continue;
            }

            let width = match image.rect().await {
                Ok(r) => r.width,
                Err(e) => {
                    error!(url=?*url, error=?e, "Failed to get the width of the image");
                    continue;
                }
            };

            let height = match image.rect().await {
                Ok(r) => r.height,
                Err(e) => {
                    error!(url=?*url, error=?e, "Failed to get the height of the image");
                    continue;
                }
            };
            images_vec.push(Image {
                url: src.unwrap(),
                alt,
                width,
                height,
                size: width * height,
            });
        }

        match self.cleanup_driver(driver, url.as_str()).await {
            Ok(_) => (),
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to cleanup driver");
                return Err(ResponseObject::internal_server_error(
                    "Failed to cleanup driver",
                ));
            }
        }

        // sort images by size
        images_vec.sort_by(|a, b| b.size.partial_cmp(&a.size).unwrap());

        Ok(ResponseObject::ok(images_vec))
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
                self.cleanup_driver(driver, url).await?;
                return Err("Failed to switch to new tab".to_string());
            }
        }

        match driver.goto(url.to_string()).await {
            Ok(_) => (),
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to navigate to URL");
                self.cleanup_driver(driver, url).await?;
                return Err("Failed to navigate to URL".to_string());
            }
        }

        // Wait for page to be loaded.
        let _ = driver.find(By::Tag("img")).await;

        Ok(driver)
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
                Ok(())
            }
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to switch to window");
                Err("Failed to switch to window".to_string())
            }
        }
    }

    pub async fn health(&self) -> anyhow::Result<(), anyhow::Error> {
        let driver = self.driver.lock().await;

        let driver = match self.setup_driver(&driver, "https://example.com").await {
            Ok(d) => d,
            Err(e) => {
                error!(error=?e, "Failed to setup driver");
                return Err(anyhow::anyhow!("Failed to setup driver: {}", e));
            }
        };

        match driver.source().await {
            Ok(h) => h,
            Err(e) => {
                error!(error=?e, "Failed to get page source");
                return Err(anyhow::anyhow!("Failed to get page source: {}", e));
            }
        };

        match self.cleanup_driver(driver, "https://example.com").await {
            Ok(_) => (),
            Err(e) => {
                error!(error=?e, "Failed to cleanup driver");
                return Err(anyhow::anyhow!("Failed to cleanup driver: {}", e));
            }
        }

        Ok(())
    }
}
