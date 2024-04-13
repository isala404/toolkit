use crate::utils;
use std::sync::Arc;
use thirtyfour::{ChromiumLikeCapabilities, DesiredCapabilities, WebDriver};
use tokio::sync::Mutex;

pub mod handler;
mod model;

pub async fn selenium() -> (handler::Selenium, WebDriver) {
    let mut caps = DesiredCapabilities::chrome();
    caps.set_headless().unwrap();
    caps.set_ignore_certificate_errors().unwrap();
    caps.set_no_sandbox().unwrap();
    caps.set_disable_gpu().unwrap();
    caps.set_disable_dev_shm_usage().unwrap();

    let web_driver = match WebDriver::new(utils::CHROME_DRIVER_ENDPOINT.as_str(), caps).await {
        Ok(d) => d,
        Err(e) => {
            panic!("Failed to create session: {:?}", e)
        }
    };

    let driver = Arc::new(Mutex::new(web_driver.clone()));
    let selenium_api = handler::Selenium::new(driver);

    (selenium_api, web_driver)
}
