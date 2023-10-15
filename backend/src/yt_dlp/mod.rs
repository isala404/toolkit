mod handler;
mod model;

pub async fn yt_dlp() -> handler::YoutubeDL {
    handler::YoutubeDL::default()
}
