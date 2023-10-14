use crate::utils::{ApiTags, MyResponse, ResponseObject};
use poem::Request;
use poem_openapi::{OpenApi, param::Query};
use thirtyfour::prelude::*;

pub struct Selenium {
}


#[OpenApi(
    prefix_path = "/browser/",
    request_header(
        name = "API-Key",
        ty = "String",
        description = "Private API Key",
    ),
    tag = "ApiTags::Selenium"
)]
impl Selenium {
    // create new instance
    pub fn new() -> Self {
        Self { }
    }

    /// get rendered html
    #[oai(path = "/html/", method = "get", operation_id = "browser::get_html")]
    async fn get_html(
        &self,
        req: &Request,
        /// url of the page to render
        url: Query<String>,
    ) -> MyResponse<String> {
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

        let caps = DesiredCapabilities::chrome();
        let driver = match WebDriver::new("http://localhost:9515", caps).await {
            Ok(d) => d,
            Err(e) => {
                println!("Failed to create session: {:?}", e);
                return ResponseObject::internal_server_error("Failed to create session");
            }
        };
        match driver.goto(url.to_string()).await {
            Ok(_) => (),
            Err(e) => {
                println!("Failed to navigate to URL: {:?}", e);
                return ResponseObject::internal_server_error("Failed to navigate to URL");
            }
        }
        
        // Wait for page to be loaded.
        match driver.find(By::Tag("span")).await {
            Ok(_) => (),
            Err(e) => {
                println!("Failed to find span: {:?}", e);
                return ResponseObject::internal_server_error("Failed to find span");
            }
        }
        match driver.find(By::Tag("img")).await {
            Ok(_) => (),
            Err(e) => {
                println!("Failed to find img: {:?}", e);
                return ResponseObject::internal_server_error("Failed to find img");
            }
        }
        // sleep for 500ms
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let html = match driver.source().await {
            Ok(h) => h,
            Err(e) => {
                println!("Failed to get page source: {:?}", e);
                return ResponseObject::internal_server_error("Failed to get page source");
            }
        };

        return ResponseObject::ok(html);
    }
    

}
