mod handler;
mod model;
mod utils;

pub async fn yt_dlp() -> handler::YoutubeDL {
    handler::YoutubeDL::default()
}
