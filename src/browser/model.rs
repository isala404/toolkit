use poem_openapi::Object;
use serde::Serialize;

/// Image on the page
#[derive(Debug, Object, Clone, Serialize)]
pub struct Image {
    /// URL of the image
    pub url: String,
    /// Alt text of the image
    pub alt: Option<String>,
    /// Width of the image
    pub width: f64,
    /// Height of the image
    pub height: f64,
    /// Size of the image
    pub size: f64,
}
