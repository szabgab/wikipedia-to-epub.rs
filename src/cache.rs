use crate::USER_AGENT;
use crate::error::{AppError, AppResult};
use reqwest::{
    Url,
    blocking::Client,
    header::{HeaderMap, RETRY_AFTER},
};
use serde::Deserialize;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use tracing::{info, warn};

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct PageResponse {
    pub(crate) parse: ParsedPage,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct ParsedPage {
    pub(crate) title: String,
    pub(crate) wikitext: WikitextValue,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct WikitextValue {
    #[serde(rename = "*")]
    pub(crate) text: String,
}

#[derive(Debug, Deserialize)]
struct WikipediaErrorResponse {
    error: Option<WikipediaError>,
}

#[derive(Debug, Deserialize)]
struct WikipediaError {
    code: String,
    info: String,
}

pub(crate) trait PageSource {
    fn load_page(&self, article: &str) -> AppResult<PageResponse>;
    fn is_cache_hit(&self, article: &str) -> bool;
}

pub(crate) struct WikipediaApiPageSource {
    client: Client,
    api_url: Url,
    language: String,
    cache: DownloadCache,
    cache_hits: RefCell<HashSet<String>>,
}

impl WikipediaApiPageSource {
    pub(crate) fn new(language: &str, cache: DownloadCache) -> AppResult<Self> {
        let client = Client::builder().user_agent(USER_AGENT).build()?;
        let api_url = wikipedia_parse_api_url(language)?;
        Ok(Self {
            client,
            api_url,
            language: language.to_string(),
            cache,
            cache_hits: RefCell::new(HashSet::new()),
        })
    }

    fn fetch_page_payload(&self, article: &str) -> AppResult<String> {
        let response = self
            .client
            .get(self.api_url.clone())
            .query(&[
                ("action", "parse"),
                ("prop", "wikitext"),
                ("redirects", "true"),
                ("format", "json"),
                ("page", article),
            ])
            .send()?;

        let status = response.status();
        let headers = response.headers().clone();
        let payload = response.text()?;
        if !status.is_success() {
            let detail = http_failure_detail(&headers, &payload);
            if let Some(detail) = detail.as_deref() {
                warn!(
                    article = article,
                    %status,
                    detail = detail,
                    "Wikipedia API request failed"
                );
            } else {
                warn!(article = article, %status, "Wikipedia API request failed");
            }

            let mut message =
                format!("Wikipedia API request for '{article}' failed with status {status}");
            if let Some(detail) = detail {
                message.push_str(": ");
                message.push_str(&detail);
            }
            return Err(AppError::Message(message));
        }

        info!(article = article, "downloaded page");
        Ok(payload)
    }

    fn parse_page_payload(article: &str, payload: &str) -> AppResult<PageResponse> {
        serde_json::from_str::<PageResponse>(payload).map_err(|err| {
            AppError::Message(format!(
                "failed to parse Wikipedia response for '{article}': {err}"
            ))
        })
    }
}

impl PageSource for WikipediaApiPageSource {
    fn load_page(&self, article: &str) -> AppResult<PageResponse> {
        let cache_path = self.cache.page_json_path(&self.language, article);
        let (payload, source) = read_or_fetch_text_with_stats(
            &cache_path,
            self.cache.refresh,
            Some(&self.cache.stats.json),
            self.cache.enabled,
            || self.fetch_page_payload(article),
        )?;
        if source == CacheSource::Hit {
            let filename = cache_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            info!(
                article = article,
                filename = filename,
                "loaded page from cache"
            );
            self.cache_hits
                .borrow_mut()
                .insert(normalize_lookup_key(article));
        }
        match Self::parse_page_payload(article, &payload) {
            Ok(page) => Ok(page),
            Err(err) if source == CacheSource::Hit => {
                warn!(
                    article = article,
                    cache_path = %cache_path.display(),
                    error = %err,
                    "cached page JSON could not be parsed; refreshing cache"
                );
                let payload = fetch_and_write_text_with_stats(
                    &cache_path,
                    Some(&self.cache.stats.json),
                    self.cache.enabled,
                    || self.fetch_page_payload(article),
                )?;
                Self::parse_page_payload(article, &payload)
            }
            Err(err) => Err(err),
        }
    }

    fn is_cache_hit(&self, article: &str) -> bool {
        self.cache_hits
            .borrow()
            .contains(&normalize_lookup_key(article))
    }
}

pub(crate) fn http_failure_detail(headers: &HeaderMap, body: &str) -> Option<String> {
    if let Ok(error) = serde_json::from_str::<WikipediaErrorResponse>(body)
        && let Some(error) = error.error
    {
        return Some(format!("{}: {}", error.code, error.info));
    }

    let body = body.split_whitespace().collect::<Vec<_>>().join(" ");
    if !body.is_empty() {
        let mut shortened = body.chars().take(240).collect::<String>();
        if body.chars().count() > 240 {
            shortened.push('…');
        }
        return Some(shortened);
    }

    headers
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .map(|value| format!("retry-after: {value}"))
}

#[derive(Clone, Debug)]
pub(crate) struct DownloadCache {
    root: PathBuf,
    pub(crate) refresh: bool,
    pub(crate) stats: DownloadStats,
    pub(crate) enabled: bool,
}

impl DownloadCache {
    pub(crate) fn new(root: PathBuf, refresh: bool, stats: DownloadStats, enabled: bool) -> Self {
        Self {
            root,
            refresh,
            stats,
            enabled,
        }
    }

    pub(crate) fn page_json_path(&self, language: &str, article: &str) -> PathBuf {
        self.root
            .join("pages")
            .join(language)
            .join(format!("{}.json", cache_key(article)))
    }

    pub(crate) fn image_metadata_path(&self, language: &str, title: &str) -> PathBuf {
        self.root
            .join("images")
            .join("metadata")
            .join(language)
            .join(format!("{}.json", cache_key(title)))
    }

    pub(crate) fn image_file_path(&self, url: &str, extension: &str) -> PathBuf {
        self.root
            .join("images")
            .join("files")
            .join(format!("{}.{}", cache_key(url), extension))
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct DownloadStats {
    pub(crate) json: Rc<FileDownloadStats>,
    pub(crate) images: Rc<FileDownloadStats>,
}

#[derive(Debug, Default)]
pub(crate) struct FileDownloadStats {
    pub(crate) needed: Cell<usize>,
    pub(crate) from_cache: Cell<usize>,
    pub(crate) downloaded: Cell<usize>,
    pub(crate) failed: Cell<usize>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct FileDownloadSnapshot {
    pub(crate) needed: usize,
    pub(crate) from_cache: usize,
    pub(crate) downloaded: usize,
    pub(crate) failed: usize,
}

impl FileDownloadStats {
    pub(crate) fn snapshot(&self) -> FileDownloadSnapshot {
        FileDownloadSnapshot {
            needed: self.needed.get(),
            from_cache: self.from_cache.get(),
            downloaded: self.downloaded.get(),
            failed: self.failed.get(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CacheSource {
    Hit,
    Refreshed,
}

pub(crate) struct FixturePageSource {
    pages_dir: PathBuf,
}

impl FixturePageSource {
    pub(crate) fn new(pages_dir: impl Into<PathBuf>) -> Self {
        Self {
            pages_dir: pages_dir.into(),
        }
    }
}

impl PageSource for FixturePageSource {
    fn load_page(&self, article: &str) -> AppResult<PageResponse> {
        let page_path = crate::find_page_path(article, &self.pages_dir)?;
        read_json::<PageResponse>(&page_path)
    }

    fn is_cache_hit(&self, _article: &str) -> bool {
        false
    }
}

pub(crate) fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> AppResult<T> {
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

pub(crate) fn default_cache_root() -> AppResult<PathBuf> {
    let cache_dir = if cfg!(target_os = "windows") {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join("Library").join("Caches"))
    } else {
        std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
    };

    cache_dir
        .map(|path| path.join("wikipedia-to-epub"))
        .ok_or_else(|| {
            AppError::Message(
                "could not determine the user cache directory for live downloads".to_string(),
            )
        })
}

pub(crate) fn read_or_fetch_text_with_stats(
    cache_path: &Path,
    refresh: bool,
    stats: Option<&FileDownloadStats>,
    enabled: bool,
    fetch: impl FnOnce() -> AppResult<String>,
) -> AppResult<(String, CacheSource)> {
    if !enabled {
        let content = fetch()?;
        if let Some(stats) = stats {
            stats.needed.set(stats.needed.get() + 1);
            stats.downloaded.set(stats.downloaded.get() + 1);
        }
        return Ok((content, CacheSource::Refreshed));
    }
    if let Some(stats) = stats {
        stats.needed.set(stats.needed.get() + 1);
    }
    if !refresh && cache_path.is_file() {
        if let Some(stats) = stats {
            stats.from_cache.set(stats.from_cache.get() + 1);
        }
        return Ok((fs::read_to_string(cache_path)?, CacheSource::Hit));
    }

    let content = fetch_and_write_text_with_stats(cache_path, stats, enabled, fetch)?;
    Ok((content, CacheSource::Refreshed))
}

pub(crate) fn fetch_and_write_text_with_stats(
    cache_path: &Path,
    stats: Option<&FileDownloadStats>,
    enabled: bool,
    fetch: impl FnOnce() -> AppResult<String>,
) -> AppResult<String> {
    let content = match fetch() {
        Ok(content) => content,
        Err(err) => {
            if let Some(stats) = stats {
                stats.failed.set(stats.failed.get() + 1);
            }
            return Err(err);
        }
    };
    if enabled && let Err(err) = write_cache_text(cache_path, &content) {
        if let Some(stats) = stats {
            stats.failed.set(stats.failed.get() + 1);
        }
        return Err(err);
    }
    if let Some(stats) = stats {
        stats.downloaded.set(stats.downloaded.get() + 1);
    }
    Ok(content)
}

pub(crate) fn read_or_fetch_bytes_with_stats(
    cache_path: &Path,
    refresh: bool,
    stats: Option<&FileDownloadStats>,
    enabled: bool,
    fetch: impl FnOnce() -> AppResult<Vec<u8>>,
) -> AppResult<(Vec<u8>, CacheSource)> {
    if !enabled {
        let content = fetch()?;
        if let Some(stats) = stats {
            stats.needed.set(stats.needed.get() + 1);
            stats.downloaded.set(stats.downloaded.get() + 1);
        }
        return Ok((content, CacheSource::Refreshed));
    }
    if let Some(stats) = stats {
        stats.needed.set(stats.needed.get() + 1);
    }
    if !refresh && cache_path.is_file() {
        if let Some(stats) = stats {
            stats.from_cache.set(stats.from_cache.get() + 1);
        }
        return Ok((fs::read(cache_path)?, CacheSource::Hit));
    }

    let content = match fetch() {
        Ok(content) => content,
        Err(err) => {
            if let Some(stats) = stats {
                stats.failed.set(stats.failed.get() + 1);
            }
            return Err(err);
        }
    };
    if let Err(err) = write_cache_bytes(cache_path, &content) {
        if let Some(stats) = stats {
            stats.failed.set(stats.failed.get() + 1);
        }
        return Err(err);
    }
    if let Some(stats) = stats {
        stats.downloaded.set(stats.downloaded.get() + 1);
    }
    Ok((content, CacheSource::Refreshed))
}

pub(crate) fn log_download_stats(stats: &DownloadStats) {
    let json = stats.json.snapshot();
    let images = stats.images.snapshot();
    info!(
        json_needed = json.needed,
        json_from_cache = json.from_cache,
        json_downloaded = json.downloaded,
        json_failed = json.failed,
        image_needed = images.needed,
        image_from_cache = images.from_cache,
        image_downloaded = images.downloaded,
        image_failed = images.failed,
        "download cache report"
    );
}

pub(crate) fn write_cache_text(cache_path: &Path, content: &str) -> AppResult<()> {
    write_cache_bytes(cache_path, content.as_bytes())
}

fn write_cache_bytes(cache_path: &Path, content: &[u8]) -> AppResult<()> {
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(cache_path, content)?;
    Ok(())
}

fn cache_key(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

pub(crate) fn wikipedia_parse_api_url(language: &str) -> AppResult<Url> {
    let language = normalized_wikipedia_language(language)?;
    Url::parse(&format!("https://{language}.wikipedia.org/w/api.php"))
        .map_err(|err| AppError::Message(format!("invalid Wikipedia API URL: {err}")))
}

pub(crate) fn normalized_wikipedia_language(language: &str) -> AppResult<String> {
    let language = language.trim().to_ascii_lowercase();
    let valid = !language.is_empty()
        && language
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-');
    if !valid {
        return Err(AppError::Message(format!(
            "invalid Wikipedia language code: '{language}'"
        )));
    }
    Ok(language)
}

pub(crate) fn normalize_lookup_key(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|ch| ch.is_alphanumeric())
        .collect()
}
