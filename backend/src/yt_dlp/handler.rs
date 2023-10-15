use super::model::Metadata;
use crate::utils::{verify_apikey, ApiTags, MyResponse, ResponseObject};
use poem::Request;
use poem_openapi::{
    param::Query,
    payload::{Attachment, AttachmentType},
    OpenApi,
};
use tempfile::tempdir;
use tracing::error;
use youtube_dl::YoutubeDl;

#[derive(Default)]
pub struct YoutubeDL;

#[OpenApi(
    prefix_path = "/yt-dlp/",
    request_header(name = "API-Key", ty = "String", description = "Private API Key"),
    tag = "ApiTags::YoutubeDL"
)]
impl YoutubeDL {
    #[oai(
        path = "/metadata",
        method = "get",
        operation_id = "yt_dlp::get_metadata"
    )]
    async fn get_metadata(&self, req: &Request, url: Query<String>) -> MyResponse<Metadata> {
        match verify_apikey(req).await {
            Ok(_) => (),
            Err(e) => {
                return ResponseObject::unauthorized(e);
            }
        }

        let output = YoutubeDl::new(url.0.clone())
            .socket_timeout("15")
            .run_async()
            .await;

        let output = match output {
            Ok(output) => output,
            Err(error) => {
                error!(url = %url.0, error = %error, "Failed to get metadata");
                if error.to_string().contains("ERROR: Unsupported URL") {
                    return ResponseObject::bad_request("Unsupported URL".to_string());
                }
                return ResponseObject::internal_server_error("Failed to get metadata".to_string());
            }
        };

        let video = match output.into_single_video() {
            Some(video) => video,
            None => {
                error!(url = %url.0, "Failed to get metadata");
                return ResponseObject::internal_server_error("Failed to get metadata".to_string());
            }
        };

        ResponseObject::ok(Metadata {
            description: video.description,
            duration: video.duration,
            extractor: video.extractor,
            is_live: video.is_live,
            tags: video.tags,
            title: video.title,
            upload_date: video.upload_date,
            url: video.url,
            webpage_url: video.webpage_url,
        })
    }

    #[oai(path = "/download", method = "get", operation_id = "yt_dlp::download")]
    async fn download(
        &self,
        req: &Request,
        url: Query<String>,
        format: Query<String>,
    ) -> MyResponse<String> {
        match verify_apikey(req).await {
            Ok(_) => (),
            Err(e) => {
                return ResponseObject::unauthorized(e);
            }
        }

        let temp_dir = match tempdir() {
            Ok(temp_dir) => temp_dir,
            Err(error) => {
                error!(url = %url.0, error = %error, "Failed to get download directory");
                return ResponseObject::internal_server_error(
                    "Failed to get download directory".to_string(),
                );
            }
        };

        let dir_path = temp_dir.path();

        let output = YoutubeDl::new(url.0.clone())
            .format(format.0)
            .output_template("%(id)s.%(ext)s")
            .download_to_async(dir_path.to_str().unwrap())
            .await;

        match output {
            Ok(output) => output,
            Err(error) => {
                error!(url = %url.0, error = %error, "Failed to get metadata");
                if error.to_string().contains("ERROR: Unsupported URL") {
                    return ResponseObject::bad_request("Unsupported URL".to_string());
                }
                return ResponseObject::internal_server_error("Failed to get metadata".to_string());
            }
        };

        // list files in directory
        let files = match std::fs::read_dir(dir_path) {
            Ok(files) => files,
            Err(error) => {
                error!(url = %url.0, error = %error, "Failed to get download directory");
                return ResponseObject::internal_server_error(
                    "Failed to get download directory".to_string(),
                );
            }
        };

        // get first file
        let file = match files.into_iter().next() {
            Some(file) => file,
            None => {
                error!(url = %url.0, "Failed to get download directory");
                return ResponseObject::internal_server_error(
                    "Failed to get download directory".to_string(),
                );
            }
        };

        let file = match file {
            Ok(file) => file,
            Err(error) => {
                error!(url = %url.0, error = %error, "Failed to get download directory");
                return ResponseObject::internal_server_error(
                    "Failed to get download directory".to_string(),
                );
            }
        };

        // read file contents as bytes
        let file_contents = match std::fs::read(file.path()) {
            Ok(file_contents) => file_contents,
            Err(error) => {
                error!(url = %url.0, error = %error, "Failed to get download directory");
                return ResponseObject::internal_server_error(
                    "Failed to get download directory".to_string(),
                );
            }
        };

        let attachment = Attachment::new(file_contents)
            .attachment_type(AttachmentType::Attachment)
            .filename(file.file_name().to_str().unwrap().to_string());

        ResponseObject::file_response(attachment)
    }
}
