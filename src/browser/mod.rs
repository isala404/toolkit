use thirtyfour::{DesiredCapabilities, WebDriver};
use tokio::sync::Mutex;
use crate::utils;

mod handler;
mod model;

pub async fn selenium() -> (handler::Selenium, WebDriver) {

    let caps = DesiredCapabilities::chrome();
    let web_driver = match WebDriver::new(utils::CHROME_DRIVER_ENDPOINT.as_str(), caps).await {
        Ok(d) => d,
        Err(e) => {
            panic!("Failed to create session: {:?}", e)
        }
    };

    let driver = Mutex::new(web_driver.clone());
    let selenium_api = handler::Selenium::new(driver);

    return (selenium_api, web_driver);
}
