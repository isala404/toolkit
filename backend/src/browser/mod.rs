use thirtyfour::{DesiredCapabilities, WebDriver};
use tokio::sync::Mutex;

mod handler;

pub async fn selenium() -> (handler::Selenium, WebDriver) {
    let caps = DesiredCapabilities::chrome();
    let web_driver = match WebDriver::new("http://localhost:9515", caps).await {
        Ok(d) => d,
        Err(e) => {
            panic!("Failed to create session: {:?}", e)
        }
    };

    let driver = Mutex::new(web_driver.clone());

    let selenium_api = handler::Selenium::new(driver);
    return (selenium_api, web_driver);
}
