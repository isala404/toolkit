use super::model::Image;
use crate::utils::{ApiTags, MyResponse, ResponseObject};
use base64::{engine::general_purpose, Engine as _};
use poem::Request;
use poem_openapi::{param::Query, OpenApi};
use thirtyfour::prelude::*;
use tokio::sync::{Mutex, MutexGuard};
use tracing::error;

pub struct Selenium {
    driver: Mutex<WebDriver>,
    api_key: String,
}

#[OpenApi(
    prefix_path = "/browser/",
    request_header(name = "API-Key", ty = "String", description = "Private API Key",),
    tag = "ApiTags::Selenium"
)]
impl Selenium {
    // create new instance
    pub fn new(driver: Mutex<WebDriver>, api_key: String) -> Self {
        Selenium { driver, api_key }
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
    ) -> MyResponse<String> {
        let driver = self.driver.lock().await;

        match self.verify_apikey(req).await {
            Ok(_) => (),
            Err(e) => {
                return ResponseObject::unauthorized(e);
            }
        }

        let driver = match self.setup_driver(&driver, url.as_str()).await {
            Ok(d) => d,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to setup driver");
                return ResponseObject::internal_server_error("Failed to setup driver");
            }
        };
        tokio::time::sleep(tokio::time::Duration::from_millis(delay.0)).await;

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

    /// get body text of the rendered page
    #[oai(path = "/text/", method = "get", operation_id = "browser::get_text")]
    async fn get_text(
        &self,
        req: &Request,
        /// url of the page to render
        url: Query<String>,
        /// delay in milliseconds wait for the page to be fully loaded
        delay: Query<u64>,
    ) -> MyResponse<String> {
        let driver = self.driver.lock().await;

        match self.verify_apikey(req).await {
            Ok(_) => (),
            Err(e) => {
                return ResponseObject::unauthorized(e);
            }
        }

        let driver = match self.setup_driver(&driver, url.as_str()).await {
            Ok(d) => d,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to setup driver");
                return ResponseObject::internal_server_error("Failed to setup driver");
            }
        };
        tokio::time::sleep(tokio::time::Duration::from_millis(delay.0)).await;

        let body = match driver.find(By::Tag("body")).await {
            Ok(h) => h,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to get the body of the page");
                return ResponseObject::internal_server_error("Failed to get the body of the page");
            }
        };

        let text = match body.text().await {
            Ok(t) => t,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to get the text of the body");
                return ResponseObject::internal_server_error("Failed to get the text of the body");
            }
        };

        match self.cleanup_driver(driver, url.as_str()).await {
            Ok(_) => (),
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to cleanup driver");
                return ResponseObject::internal_server_error("Failed to cleanup driver");
            }
        }

        return ResponseObject::ok(text);
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
    ) -> MyResponse<String> {
        let driver = self.driver.lock().await;

        match self.verify_apikey(req).await {
            Ok(_) => (),
            Err(e) => {
                return ResponseObject::unauthorized(e);
            }
        }

        let driver = match self.setup_driver(&driver, url.as_str()).await {
            Ok(d) => d,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to setup driver");
                return ResponseObject::internal_server_error("Failed to setup driver");
            }
        };
        tokio::time::sleep(tokio::time::Duration::from_millis(delay.0)).await;

        let screenshot = match driver.screenshot_as_png().await {
            Ok(h) => h,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to get the screenshot of the page");
                return ResponseObject::internal_server_error(
                    "Failed to get the screenshot of the page",
                );
            }
        };
        let b64 = general_purpose::STANDARD.encode(screenshot);

        match self.cleanup_driver(driver, url.as_str()).await {
            Ok(_) => (),
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to cleanup driver");
                return ResponseObject::internal_server_error("Failed to cleanup driver");
            }
        }

        return ResponseObject::ok(format!("data:image/png;base64,{}", b64));
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
    ) -> MyResponse<Vec<Image>> {
        let driver = self.driver.lock().await;

        match self.verify_apikey(req).await {
            Ok(_) => (),
            Err(e) => {
                return ResponseObject::unauthorized(e);
            }
        }

        let driver = match self.setup_driver(&driver, url.as_str()).await {
            Ok(d) => d,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to setup driver");
                return ResponseObject::internal_server_error("Failed to setup driver");
            }
        };
        tokio::time::sleep(tokio::time::Duration::from_millis(delay.0)).await;

        let images = match driver.find_all(By::Tag("img")).await {
            Ok(h) => h,
            Err(e) => {
                error!(url=?*url, error=?e, "Failed to get the images of the page");
                return ResponseObject::internal_server_error(
                    "Failed to get the images of the page",
                );
            }
        };

        let mut images_vec = Vec::new();

        for image in images {
            let src = match image.attr("src").await {
                Ok(s) => s,
                Err(e) => {
                    error!(url=?*url, error=?e, "Failed to get the src of the image");
                    return ResponseObject::internal_server_error(
                        "Failed to get the src of the image",
                    );
                }
            };
            let alt = match image.attr("alt").await {
                Ok(s) => s,
                Err(e) => {
                    error!(url=?*url, error=?e, "Failed to get the alt of the image");
                    return ResponseObject::internal_server_error(
                        "Failed to get the alt of the image",
                    );
                }
            };

            if src.is_none() {
                continue;
            }

            let width = match image.rect().await {
                Ok(r) => r.width,
                Err(e) => {
                    error!(url=?*url, error=?e, "Failed to get the width of the image");
                    return ResponseObject::internal_server_error(
                        "Failed to get the width of the image",
                    );
                }
            };

            let height = match image.rect().await {
                Ok(r) => r.height,
                Err(e) => {
                    error!(url=?*url, error=?e, "Failed to get the height of the image");
                    return ResponseObject::internal_server_error(
                        "Failed to get the height of the image",
                    );
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
                return ResponseObject::internal_server_error("Failed to cleanup driver");
            }
        }

        // sort images by size
        images_vec.sort_by(|a, b| b.size.partial_cmp(&a.size).unwrap());

        return ResponseObject::ok(images_vec);
    }

    async fn verify_apikey(&self, req: &Request) -> Result<(), String> {
        // extract user id from token
        let api_key = match req.header("API-Key") {
            Some(key) => key,
            None => {
                return Err("API-Key header is missing".to_string());
            }
        };

        if api_key != self.api_key {
            return Err("Invalid API-Key".to_string());
        }

        return Ok(());
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
        match driver.find(By::Tag("img")).await {
            Ok(_) => (),
            Err(_) => (),
        }

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
