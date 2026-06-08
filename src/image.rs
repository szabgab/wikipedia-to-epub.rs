use crate::USER_AGENT;
use crate::cache::{
    CacheSource, DownloadCache, fetch_and_write_text_with_stats, read_json,
    read_or_fetch_bytes_with_stats, read_or_fetch_text_with_stats, wikipedia_parse_api_url,
};
use crate::error::{AppError, AppResult};
use html_escape::encode_double_quoted_attribute;
use regex::Regex;
use reqwest::{Url, blocking::Client};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{debug, info, warn};

#[derive(Debug)]
pub enum BookImageSource {
    Local(PathBuf),
    Remote { title: String },
}

#[derive(Debug)]
pub struct ResolvedImage {
    pub href: String,
    pub media_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub struct ImageOccurrence {
    pub href: String,
    pub alt: String,
    pub caption: String,
}

#[derive(Debug)]
pub struct ImageRegistry {
    pub availability: ImageAvailability,
    pub images: Vec<BookImage>,
    pub images_by_title: HashMap<String, usize>,
    pub occurrences: Vec<ImageOccurrence>,
}

#[derive(Debug)]
pub enum ImageAvailability {
    All,
    Local {
        root: PathBuf,
        fixtures: HashMap<String, LocalImageFixture>,
    },
}

#[derive(Clone, Debug, Deserialize)]
pub struct LocalImageFixture {
    pub path: PathBuf,
    #[serde(rename = "media-type")]
    pub media_type: String,
}

#[derive(Debug)]
pub struct BookImage {
    pub title: String,
    pub href: String,
    pub media_type: String,
    pub source_pages: Vec<String>,
    pub source: BookImageSource,
}

#[derive(Debug)]
pub struct ParsedFileLink {
    pub title: String,
    pub caption: String,
    pub alt: String,
}

impl ImageRegistry {
    pub fn new(local_pages_dir: Option<&Path>) -> AppResult<Self> {
        let availability = match local_pages_dir {
            Some(pages_dir) => {
                let manifest_path = pages_dir.join("images").join("manifest.json");
                let fixtures = if manifest_path.is_file() {
                    let fixtures = read_json::<HashMap<String, LocalImageFixture>>(&manifest_path)?;
                    fixtures
                        .into_iter()
                        .map(|(title, fixture)| (normalize_image_title(&title), fixture))
                        .collect::<HashMap<_, _>>()
                } else {
                    HashMap::new()
                };
                ImageAvailability::Local {
                    root: pages_dir.join("images"),
                    fixtures,
                }
            }
            None => ImageAvailability::All,
        };

        Ok(Self {
            availability,
            images: Vec::new(),
            images_by_title: HashMap::new(),
            occurrences: Vec::new(),
        })
    }

    pub fn register(&mut self, file_link: ParsedFileLink, source_page: &str) -> Option<usize> {
        let key = normalize_image_title(&file_link.title);
        let image_index = if let Some(index) = self.images_by_title.get(&key).copied() {
            let image = &mut self.images[index];
            if !image.source_pages.iter().any(|page| page == source_page) {
                image.source_pages.push(source_page.to_string());
            }
            index
        } else {
            let image = match &self.availability {
                ImageAvailability::All => {
                    let href = format!(
                        "images/image-{}.{}",
                        self.images.len() + 1,
                        image_extension(&file_link.title)
                    );
                    BookImage {
                        title: file_link.title.clone(),
                        href,
                        media_type: media_type_from_title(&file_link.title).to_string(),
                        source_pages: vec![source_page.to_string()],
                        source: BookImageSource::Remote {
                            title: file_link.title.clone(),
                        },
                    }
                }
                ImageAvailability::Local { root, fixtures } => {
                    let Some(fixture) = fixtures.get(&key) else {
                        warn!(
                            image = file_link.title,
                            "image fixture is missing; omitting image"
                        );
                        return None;
                    };
                    let href = format!(
                        "images/image-{}.{}",
                        self.images.len() + 1,
                        path_extension(&fixture.path)
                            .unwrap_or_else(|| image_extension(&file_link.title))
                    );
                    BookImage {
                        title: file_link.title.clone(),
                        href,
                        media_type: fixture.media_type.clone(),
                        source_pages: vec![source_page.to_string()],
                        source: BookImageSource::Local(root.join(&fixture.path)),
                    }
                }
            };

            let index = self.images.len();
            self.images.push(image);
            self.images_by_title.insert(key, index);
            index
        };

        let image = &self.images[image_index];
        let occurrence_id = self.occurrences.len();
        self.occurrences.push(ImageOccurrence {
            href: image.href.clone(),
            alt: file_link.alt,
            caption: file_link.caption,
        });
        Some(occurrence_id)
    }

    pub fn occurrence(&self, id: usize) -> Option<&ImageOccurrence> {
        self.occurrences.get(id)
    }
}

pub fn normalize_image_title(title: &str) -> String {
    title.trim().replace('_', " ").to_ascii_lowercase()
}

pub fn image_extension(title: &str) -> String {
    title
        .rsplit_once('.')
        .map(|(_, extension)| sanitize_extension(extension))
        .filter(|extension| !extension.is_empty())
        .unwrap_or_else(|| "img".to_string())
}

pub fn path_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(sanitize_extension)
        .filter(|extension| !extension.is_empty())
}

pub fn sanitize_extension(extension: &str) -> String {
    extension
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

pub fn media_type_from_title(title: &str) -> &'static str {
    match image_extension(title).as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}

pub fn resolve_images(
    registry: ImageRegistry,
    wikipedia_language: &str,
    cache: Option<&DownloadCache>,
) -> AppResult<Vec<ResolvedImage>> {
    let client = Client::builder().user_agent(USER_AGENT).build()?;
    let api_url = wikipedia_parse_api_url(wikipedia_language)?;
    registry
        .images
        .into_iter()
        .filter_map(|image| {
            match resolve_image(image, &client, &api_url, wikipedia_language, cache) {
                Ok(image) => Some(Ok(image)),
                Err(err) => {
                    warn!(error = %err, "image could not be resolved; omitting image asset");
                    None
                }
            }
        })
        .collect()
}

fn resolve_image(
    image: BookImage,
    client: &Client,
    api_url: &Url,
    wikipedia_language: &str,
    cache: Option<&DownloadCache>,
) -> AppResult<ResolvedImage> {
    match image.source {
        BookImageSource::Local(path) => Ok(ResolvedImage {
            href: image.href,
            media_type: image.media_type,
            bytes: fs::read(path)?,
        }),
        BookImageSource::Remote { ref title } => {
            let info = load_remote_image_info(client, api_url, title, wikipedia_language, cache)?;
            let cache_path = cache.map(|cache| {
                cache.image_file_path(
                    &info.url,
                    &image_cache_extension(&info.url, &info.media_type, &image.title),
                )
            });
            let bytes = if let Some(cache_path) = cache_path {
                let image_download_request_count = cache.map_or(1, |cache| {
                    cache.stats.images.downloaded.get() + cache.stats.images.failed.get() + 1
                });
                let (bytes, source) = read_or_fetch_bytes_with_stats(
                    &cache_path,
                    cache.is_some_and(|cache| cache.refresh),
                    cache.map(|cache| cache.stats.images.as_ref()),
                    cache.is_some_and(|cache| cache.enabled),
                    || download_image_bytes(client, &image, &info, image_download_request_count),
                )?;
                if source == CacheSource::Hit {
                    debug!(
                        image_url = %info.url,
                        cached_filename = %cache_path.display(),
                        "using cached image"
                    );
                }
                bytes
            } else {
                download_image_bytes(client, &image, &info, 1)?
            };
            Ok(ResolvedImage {
                href: image.href,
                media_type: info.media_type,
                bytes,
            })
        }
    }
}

fn download_image_bytes(
    client: &Client,
    image: &BookImage,
    info: &RemoteImageInfo,
    image_download_request_count: usize,
) -> AppResult<Vec<u8>> {
    // Without sleep the requests are rate-limited to 10 request then we get 429 Too Many Requests errors.
    // With 1 sec sleep we did not get such error.
    // Maybe less sleep time would also work. We can optimize this later if needed.
    std::thread::sleep(Duration::from_secs(1));
    info!(
        image_url = %info.url,
        source_pages = %image.source_pages.join(", "),
        image_download_request_count = image_download_request_count,
        "downloading image"
    );
    let response = client.get(&info.url).send()?;
    if !response.status().is_success() {
        return Err(AppError::Message(format!(
            "image download for '{}' failed with status {}",
            image.title,
            response.status()
        )));
    }
    Ok(response.bytes()?.to_vec())
}

#[derive(Debug)]
struct RemoteImageInfo {
    url: String,
    media_type: String,
}

fn load_remote_image_info(
    client: &Client,
    api_url: &Url,
    title: &str,
    wikipedia_language: &str,
    cache: Option<&DownloadCache>,
) -> AppResult<RemoteImageInfo> {
    let cache_path = cache.map(|cache| cache.image_metadata_path(wikipedia_language, title));
    let (payload, source) = if let Some(cache_path) = cache_path.as_deref() {
        read_or_fetch_text_with_stats(
            cache_path,
            cache.is_some_and(|cache| cache.refresh),
            cache.map(|cache| cache.stats.json.as_ref()),
            cache.is_some_and(|cache| cache.enabled),
            || fetch_remote_image_metadata_payload(client, api_url, title),
        )?
    } else {
        (
            fetch_remote_image_metadata_payload(client, api_url, title)?,
            CacheSource::Refreshed,
        )
    };
    match parse_remote_image_info(title, &payload) {
        Ok(info) => Ok(info),
        Err(err) if source == CacheSource::Hit => {
            let cache_path = cache_path.expect("cache path is present for cache hit");
            warn!(
                image = title,
                cache_path = %cache_path.display(),
                error = %err,
                "cached image metadata JSON could not be parsed; refreshing cache"
            );
            let payload = fetch_and_write_text_with_stats(
                &cache_path,
                cache.map(|cache| cache.stats.json.as_ref()),
                cache.is_some_and(|cache| cache.enabled),
                || fetch_remote_image_metadata_payload(client, api_url, title),
            )?;
            parse_remote_image_info(title, &payload)
        }
        Err(err) => Err(err),
    }
}

fn fetch_remote_image_metadata_payload(
    client: &Client,
    api_url: &Url,
    title: &str,
) -> AppResult<String> {
    let title_param = format!("File:{title}");
    let response = client
        .get(api_url.clone())
        .query(&[
            ("action", "query"),
            ("prop", "imageinfo"),
            ("iiprop", "url|mime"),
            ("iiurlwidth", "800"),
            ("format", "json"),
            ("titles", title_param.as_str()),
        ])
        .send()?;
    if !response.status().is_success() {
        return Err(AppError::Message(format!(
            "image metadata request for '{title}' failed with status {}",
            response.status()
        )));
    }
    Ok(response.text()?)
}

fn parse_remote_image_info(title: &str, payload: &str) -> AppResult<RemoteImageInfo> {
    let value = serde_json::from_str::<serde_json::Value>(payload)?;
    let pages = value
        .get("query")
        .and_then(|query| query.get("pages"))
        .and_then(|pages| pages.as_object())
        .ok_or_else(|| AppError::Message(format!("image metadata for '{title}' is missing")))?;
    let imageinfo = pages
        .values()
        .find_map(|page| page.get("imageinfo"))
        .and_then(|imageinfo| imageinfo.as_array())
        .and_then(|imageinfo| imageinfo.first())
        .ok_or_else(|| AppError::Message(format!("image metadata for '{title}' is missing")))?;
    let url = imageinfo
        .get("thumburl")
        .or_else(|| imageinfo.get("url"))
        .and_then(|url| url.as_str())
        .ok_or_else(|| AppError::Message(format!("image URL for '{title}' is missing")))?
        .to_string();
    let media_type = imageinfo
        .get("thumbmime")
        .or_else(|| imageinfo.get("mime"))
        .and_then(|mime| mime.as_str())
        .unwrap_or_else(|| media_type_from_title(title))
        .to_string();

    Ok(RemoteImageInfo { url, media_type })
}

fn image_cache_extension(url: &str, media_type: &str, fallback_title: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|url| {
            Path::new(url.path())
                .extension()
                .and_then(|extension| extension.to_str())
                .map(sanitize_extension)
        })
        .filter(|extension| !extension.is_empty())
        .or_else(|| image_extension_from_media_type(media_type).map(str::to_string))
        .unwrap_or_else(|| image_extension(fallback_title))
}

fn image_extension_from_media_type(media_type: &str) -> Option<&'static str> {
    match media_type.split(';').next().map(str::trim) {
        Some("image/jpeg") => Some("jpg"),
        Some("image/png") => Some("png"),
        Some("image/gif") => Some("gif"),
        Some("image/svg+xml") => Some("svg"),
        Some("image/webp") => Some("webp"),
        _ => None,
    }
}

pub fn image_marker_id(line: &str) -> Option<usize> {
    line.strip_prefix("__WIKIPEDIA_TO_EPUB_IMAGE_")?
        .strip_suffix("__")?
        .parse()
        .ok()
}

pub fn render_image_html(image: &ImageOccurrence) -> String {
    let caption = if image.caption.trim().is_empty() {
        String::new()
    } else {
        format!(r#"<p class="caption">{}</p>"#, image.caption)
    };

    format!(
        r#"<div class="image"><img src="{}" alt="{}" />{caption}</div>"#,
        encode_double_quoted_attribute(&image.href),
        encode_double_quoted_attribute(&plain_text_from_html(&image.alt)),
    )
}

fn plain_text_from_html(value: &str) -> String {
    let value = Regex::new(r#"(?is)<span class="external-link">.*?</span>"#)
        .unwrap()
        .replace_all(value, "");
    Regex::new(r"(?is)<[^>]+>")
        .unwrap()
        .replace_all(&value, "")
        .into_owned()
}
