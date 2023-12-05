use anyhow::Context;
use regex::Regex;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde_json::Value;
use tracing::debug;
use std::{fs::DirEntry, io, path::Path};
use url::Url;

pub async fn download_instgram_video(
    url: String,
    dir_path: &Path,
) -> Result<DirEntry, anyhow::Error> {
    let post_id = get_post_id(&url)?;
    debug!("Post ID: {}", post_id);


    let mut headers = HeaderMap::new();
    headers.insert("Accept", HeaderValue::from_static("*/*"));
    headers.insert(
        "Accept-Language",
        HeaderValue::from_static("en-US,en;q=0.5"),
    );
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/x-www-form-urlencoded"),
    );
    headers.insert(
        "X-FB-Friendly-Name",
        HeaderValue::from_static("PolarisPostActionLoadPostQueryQuery"),
    );
    headers.insert(
        "X-CSRFToken",
        HeaderValue::from_static("RVDUooU5MYsBbS1CNN3CzVAuEP8oHB52"),
    );
    headers.insert("X-IG-App-ID", HeaderValue::from_static("1217981644879628"));
    headers.insert("X-FB-LSD", HeaderValue::from_static("AVqbxe3J_YA"));
    headers.insert("X-ASBD-ID", HeaderValue::from_static("129477"));
    headers.insert("Sec-Fetch-Dest", HeaderValue::from_static("empty"));
    headers.insert("Sec-Fetch-Mode", HeaderValue::from_static("cors"));
    headers.insert("Sec-Fetch-Site", HeaderValue::from_static("same-origin"));
    headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Linux; Android 11; SAMSUNG SM-G973U) AppleWebKit/537.36 (KHTML, like Gecko) SamsungBrowser/14.2 Chrome/87.0.4280.141 Mobile Safari/537.36"));

    let url_encoded_string = format!("av=0&__d=www&__user=0&__a=1&__req=3&__hs=19624.HYP%3Ainstagram_web_pkg.2.1..0.0&dpr=3&__ccg=UNKNOWN&__rev=1008824440&__s=xf44ne%3Azhh75g%3Axr51e7&__hsi=7282217488877343271&__dyn=7xeUmwlEnwn8K2WnFw9-2i5U4e0yoW3q32360CEbo1nEhw2nVE4W0om78b87C0yE5ufz81s8hwGwQwoEcE7O2l0Fwqo31w9a9x-0z8-U2zxe2GewGwso88cobEaU2eUlwhEe87q7-0iK2S3qazo7u1xwIw8O321LwTwKG1pg661pwr86C1mwraCg&__csr=gZ3yFmJkillQvV6ybimnG8AmhqujGbLADgjyEOWz49z9XDlAXBJpC7Wy-vQTSvUGWGh5u8KibG44dBiigrgjDxGjU0150Q0848azk48N09C02IR0go4SaR70r8owyg9pU0V23hwiA0LQczA48S0f-x-27o05NG0fkw&__comet_req=7&lsd=AVqbxe3J_YA&jazoest=2957&__spin_r=1008824440&__spin_b=trunk&__spin_t=1695523385&fb_api_caller_class=RelayModern&fb_api_req_friendly_name=PolarisPostActionLoadPostQueryQuery&variables=%7B%22shortcode%22%3A%22{}%22%2C%22fetch_comment_count%22%3A%22null%22%2C%22fetch_related_profile_media_count%22%3A%22null%22%2C%22parent_comment_count%22%3A%22null%22%2C%22child_comment_count%22%3A%22null%22%2C%22fetch_like_count%22%3A%22null%22%2C%22fetch_tagged_user_count%22%3A%22null%22%2C%22fetch_preview_comment_count%22%3A%22null%22%2C%22has_threaded_comments%22%3A%22false%22%2C%22hoisted_comment_id%22%3A%22null%22%2C%22hoisted_reply_id%22%3A%22null%22%7D&server_timestamps=true&doc_id=10015901848480474", post_id);

    dbg!(&url_encoded_string);

    let client = Client::new();
    let response = client
        .post("https://www.instagram.com/api/graphql")
        .headers(headers)
        .body(url_encoded_string)
        .send()
        .await?;

    if response.status().is_success() {
        let response_data: Value = response.json().await?;

        let is_video = response_data["data"]["xdt_shortcode_media"]["is_video"]
            .as_bool()
            .context("Failed to get is_video")?;

        if !is_video {
            return Err(anyhow::anyhow!("Only instagram reels are supported"));
        }
        
        let video_url = response_data["data"]["xdt_shortcode_media"]["video_url"]
            .as_str()
            .context("Failed to get video URL")?;

        debug!(url=%video_url, "Downloading video");

        let video_response = client.get(video_url).send().await?;
        let video_bytes = video_response.bytes().await?;
        let mut video_file = std::fs::File::create(dir_path.join(format!("{}.mp4", post_id)))?;
        io::copy(&mut video_bytes.as_ref(), &mut video_file)?;
        video_file.sync_all()?;

        let video_file = std::fs::read_dir(dir_path)?
            .next()
            .context("Failed to get video file")??;
        Ok(video_file)
    } else {
        return Err(anyhow::anyhow!("Failed to get video URL"));
    }
}

pub fn get_post_id(post_url: &str) -> Result<String, anyhow::Error> {
    let post_regex = Regex::new(r"^https://(?:www\.)?instagram\.com/p/([a-zA-Z0-9_-]+)/?")?;
    let reel_regex = Regex::new(r"^https://(?:www\.)?instagram\.com/reels?/([a-zA-Z0-9_-]+)/?")?;
    let mut post_id = String::new();

    if let Some(captures) = post_regex.captures(post_url) {
        post_id = captures.get(1).map_or("", |m| m.as_str()).to_string();
    }

    if let Some(captures) = reel_regex.captures(post_url) {
        post_id = captures.get(1).map_or("", |m| m.as_str()).to_string();
    }

    if post_id.is_empty() {
        return Err(anyhow::anyhow!("Instagram post/reel ID was not found"));
    }

    Ok(post_id)
}

pub fn is_instagram_url(url: &str) -> Result<bool, anyhow::Error> {
    let parsed_url = Url::parse(url)?;
    let domain = parsed_url.domain().context("Failed to get domain")?;
    Ok(domain.contains("instagram.com"))
}
