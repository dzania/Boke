mod articles;
mod feeds;
mod folders;

pub use articles::ArticleService;
pub use feeds::{FeedService, RefreshResult};
pub use folders::FolderService;

/// Reads a response body into a string, failing once it exceeds `bytes_max`.
pub(crate) async fn read_body_limited(
    mut response: reqwest::Response,
    bytes_max: usize,
) -> anyhow::Result<String> {
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if body.len() + chunk.len() > bytes_max {
            anyhow::bail!("Response exceeds {bytes_max} bytes");
        }
        body.extend_from_slice(&chunk);
    }
    Ok(String::from_utf8_lossy(&body).into_owned())
}
