mod handler;


pub async fn selenium() -> handler::Selenium {
    let selenium_api = handler::Selenium::new();
    return selenium_api;
}
