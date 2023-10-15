use std::env;

use thirtyfour::{DesiredCapabilities, WebDriver};
use tokio::sync::Mutex;

mod handler;
mod model;

pub async fn selenium() -> (handler::Selenium, WebDriver) {
    let api_key = env::var("API_KEY").expect("API_KEY must be set");

    let caps = DesiredCapabilities::chrome();
    let web_driver = match WebDriver::new("http://localhost:9515", caps).await {
        Ok(d) => d,
        Err(e) => {
            panic!("Failed to create session: {:?}", e)
        }
    };

    let driver = Mutex::new(web_driver.clone());

    let selenium_api = handler::Selenium::new(driver, api_key);
    return (selenium_api, web_driver);
}
