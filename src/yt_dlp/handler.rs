use super::model::Metadata;
use crate::utils::{self, verify_apikey, ApiTags, JsonError, JsonSuccess, ResponseObject};
use poem::Request;
use poem_openapi::{
    param::Query,
    payload::{Attachment, AttachmentType},
    OpenApi,
};
use std::{collections::HashMap, fs::DirEntry, path::Path};
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
    async fn get_metadata(
        &self,
        req: &Request,
        url: Query<String>,
    ) -> Result<JsonSuccess<Metadata>, JsonError<String>> {
        match verify_apikey(req).await {
            Ok(_) => (),
            Err(e) => {
                return Err(ResponseObject::unauthorized(e));
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
                    return Err(ResponseObject::bad_request("Unsupported URL".to_string()));
                }
                return Err(ResponseObject::internal_server_error(
                    "Failed to get metadata".to_string(),
                ));
            }
        };

        let video = match output.into_single_video() {
            Some(video) => video,
            None => {
                error!(url = %url.0, "Failed to get metadata");
                return Err(ResponseObject::bad_request(
                    "Failed to get metadata".to_string(),
                ));
            }
        };

        Ok(ResponseObject::ok(Metadata {
            description: video.description,
            duration: video.duration,
            extractor: video.extractor,
            is_live: video.is_live,
            tags: video.tags,
            title: video.title,
            upload_date: video.upload_date,
            url: video.url,
            webpage_url: video.webpage_url,
        }))
    }

    #[oai(path = "/download", method = "get", operation_id = "yt_dlp::download")]
    async fn download(
        &self,
        req: &Request,
        url: Query<String>,
        format: Query<String>,
    ) -> Result<Attachment<Vec<u8>>, JsonError<String>> {
        match verify_apikey(req).await {
            Ok(_) => (),
            Err(e) => {
                return Err(ResponseObject::unauthorized(e));
            }
        }

        let temp_dir = match tempdir() {
            Ok(temp_dir) => temp_dir,
            Err(error) => {
                error!(url = %url.0, error = %error, "Failed to get download directory");
                return Err(ResponseObject::internal_server_error(
                    "Failed to get download directory".to_string(),
                ));
            }
        };

        let dir_path = temp_dir.path();

        let file = match self
            ._download(url.0.clone(), format.0.clone(), dir_path)
            .await
        {
            Ok(file_contents) => file_contents,
            Err(error) => {
                return Err(error);
            }
        };

        // TODO: Read this as a stream instead of reading the whole file into memory
        // read file contents as bytes
        let file_contents = match std::fs::read(file.path()) {
            Ok(file_contents) => file_contents,
            Err(error) => {
                error!(url = %url.0, error = %error, "Failed to get download directory");
                return Err(ResponseObject::internal_server_error(
                    "Failed to get download directory".to_string(),
                ));
            }
        };

        let attachment = Attachment::new(file_contents)
            .attachment_type(AttachmentType::Attachment)
            .filename(file.file_name().to_str().unwrap().to_string());

        Ok(attachment)
    }

    #[oai(
        path = "/transcribe",
        method = "get",
        operation_id = "yt_dlp::transcribe"
    )]
    async fn get_transcription(
        &self,
        req: &Request,
        url: Query<String>,
    ) -> Result<JsonSuccess<String>, JsonError<String>> {
        match verify_apikey(req).await {
            Ok(_) => (),
            Err(e) => {
                return Err(ResponseObject::unauthorized(e));
            }
        }

        let temp_dir = match tempdir() {
            Ok(temp_dir) => temp_dir,
            Err(error) => {
                error!(url = %url.0, error = %error, "Failed to get download directory");
                return Err(ResponseObject::internal_server_error(
                    "Failed to get download directory".to_string(),
                ));
            }
        };

        let dir_path = temp_dir.path();

        let file = match self
            ._download(url.0.clone(), "w".to_string(), dir_path)
            .await
        {
            Ok(file_contents) => file_contents,
            Err(error) => {
                return Err(error);
            }
        };

        // convert to mp3 using ffmpeg command
        match std::process::Command::new("ffmpeg")
            .arg("-i")
            .arg(file.path())
            .arg(dir_path.join("audio.mp3"))
            .output()
        {
            Ok(output) => output,
            Err(error) => {
                error!(url = %url.0, error = %error, "Failed to convert to mp3");
                return Err(ResponseObject::internal_server_error(
                    "Failed to convert to mp3".to_string(),
                ));
            }
        };

        // TODO: Read this as a stream instead of reading the whole file into memory
        // read file contents as bytes
        let file_contents = match std::fs::read(dir_path.join("audio.mp3")) {
            Ok(file_contents) => file_contents,
            Err(error) => {
                error!(url = %url.0, error = %error, "Failed to read the downloaded file");
                return Err(ResponseObject::internal_server_error(
                    "Failed to read the downloaded file".to_string(),
                ));
            }
        };

        let client = match reqwest::Client::builder().build() {
            Ok(client) => client,
            Err(error) => {
                error!(url = %url.0, error = %error, "Failed to build reqwest client");
                return Err(ResponseObject::internal_server_error(
                    "Failed to transcribe audio".to_string(),
                ));
            }
        };

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", utils::OPENAI_API_KEY.to_string())
                .parse()
                .unwrap(),
        );

        let form = reqwest::multipart::Form::new()
            .part(
                "file",
                reqwest::multipart::Part::bytes(file_contents).file_name("audio.mp3"),
            )
            .text("model", "whisper-1");

        let request = client
            .request(
                reqwest::Method::POST,
                "https://api.openai.com/v1/audio/transcriptions",
            )
            .headers(headers)
            .multipart(form);

        let response = match request.send().await {
            Ok(response) => response,
            Err(error) => {
                error!(url = %url.0, error = %error, "Failed to send request to OpenAI");
                return Err(ResponseObject::internal_server_error(
                    "Failed to transcribe audio".to_string(),
                ));
            }
        };
        if !response.status().is_success() {
            let body = response.text().await.unwrap();
            error!(url = %url.0, body= %body, "OpenAI returned an error");
            return Err(ResponseObject::internal_server_error(
                "Failed to transcribe audio".to_string(),
            ));
        }
        let body: HashMap<String, String> = match response.json().await {
            Ok(body) => body,
            Err(error) => {
                error!(url = %url.0, error = %error, "Failed to parse response from OpenAI");
                return Err(ResponseObject::internal_server_error(
                    "Failed to transcribe audio".to_string(),
                ));
            }
        };

        let transcription = match body.get("text") {
            Some(transcription) => transcription,
            None => {
                error!(url = %url.0, "Failed to get transcription from response");
                return Err(ResponseObject::internal_server_error(
                    "Failed to transcribe audio".to_string(),
                ));
            }
        };

        Ok(ResponseObject::ok(transcription.to_string()))
    }

    async fn _download(
        &self,
        url: String,
        format: String,
        dir_path: &Path,
    ) -> Result<DirEntry, JsonError<String>> {
        let output = YoutubeDl::new(url.clone())
            .format(format)
            .output_template("%(id)s.%(ext)s")
            .download_to_async(dir_path.to_str().unwrap())
            .await;

        match output {
            Ok(output) => output,
            Err(error) => {
                error!(url = %url, error = %error, "Failed to get metadata");
                if error.to_string().contains("ERROR: Unsupported URL") {
                    return Err(ResponseObject::bad_request("Unsupported URL".to_string()));
                }
                return Err(ResponseObject::internal_server_error(
                    "Failed to get metadata".to_string(),
                ));
            }
        };

        // list files in directory
        let files = match std::fs::read_dir(dir_path) {
            Ok(files) => files,
            Err(error) => {
                error!(url = %url, error = %error, "Failed to get download directory");
                return Err(ResponseObject::internal_server_error(
                    "Failed to get download directory".to_string(),
                ));
            }
        };

        // get first file
        let file = match files.into_iter().next() {
            Some(file) => file,
            None => {
                error!(url = %url, "Failed to get download directory");
                return Err(ResponseObject::bad_request(
                    "Failed to get download directory".to_string(),
                ));
            }
        };

        let file = match file {
            Ok(file) => file,
            Err(error) => {
                error!(url = %url, error = %error, "Failed to get download directory");
                return Err(ResponseObject::internal_server_error(
                    "Failed to get download directory".to_string(),
                ));
            }
        };

        Ok(file)
    }
}
