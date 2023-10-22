use poem_openapi::Object;
use serde::Serialize;
use serde_json::Value;

/// Metadata of the video
#[derive(Debug, Object, Clone, Serialize)]
pub struct Metadata {
    /// The title of the media.
    pub title: Option<String>,
    /// A description of the media.
    pub description: Option<String>,
    /// The URL of the media.
    pub url: Option<String>,
    /// The webpage URL of the media.
    pub webpage_url: Option<String>,
    /// The upload date of the media.
    pub upload_date: Option<String>,
    /// The duration of the media.
    pub duration: Option<Value>,
    /// Whether the media is live or not.
    pub is_live: Option<bool>,
    /// The name of the extractor used to extract the media.
    pub extractor: Option<String>,
    /// The tags associated with the media.
    pub tags: Option<Vec<Option<String>>>,
}
