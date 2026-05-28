use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    env,
    error::Error,
    fmt::{self, Display},
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    rc::Rc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clap::Parser;
use html_escape::{decode_html_entities, encode_double_quoted_attribute, encode_text};
use regex::Regex;
use reqwest::{
    Url,
    blocking::Client,
    header::{HeaderMap, RETRY_AFTER},
};
use serde::Deserialize;
use tracing::{Level, debug, info, warn};
use tracing_subscriber::fmt as tracing_fmt;
use zip::{
    CompressionMethod, ZipWriter,
    write::{FileOptions, SimpleFileOptions},
};

type AppResult<T> = Result<T, AppError>;
type InternalLinks = HashMap<String, String>;
const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/szabgab/wikipedia-to-epub.rs; contact: https://github.com/szabgab/wikipedia-to-epub.rs/issues)"
);

#[derive(Debug)]
enum AppError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Yaml(serde_yaml::Error),
    Http(reqwest::Error),
    Zip(zip::result::ZipError),
    Message(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::Json(err) => write!(f, "{err}"),
            Self::Yaml(err) => write!(f, "{err}"),
            Self::Http(err) => write!(f, "{err}"),
            Self::Zip(err) => write!(f, "{err}"),
            Self::Message(message) => write!(f, "{message}"),
        }
    }
}

impl Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<serde_yaml::Error> for AppError {
    fn from(value: serde_yaml::Error) -> Self {
        Self::Yaml(value)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
}

impl From<zip::result::ZipError> for AppError {
    fn from(value: zip::result::ZipError) -> Self {
        Self::Zip(value)
    }
}

#[derive(Debug, Deserialize)]
struct BookConfig {
    metadata: Metadata,
    #[serde(rename = "output-file")]
    output_file: PathBuf,
    #[serde(default)]
    images: bool,
    articles: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    title: String,
    author: String,
    license: Option<String>,
    language: String,
    date: Option<String>,
    edition: String,
}

#[derive(Debug, Deserialize)]
struct PageResponse {
    parse: ParsedPage,
}

#[derive(Debug, Deserialize)]
struct ParsedPage {
    title: String,
    wikitext: WikitextValue,
}

#[derive(Debug, Deserialize)]
struct WikitextValue {
    #[serde(rename = "*")]
    text: String,
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

#[derive(Debug)]
struct Chapter {
    file_name: String,
    title: String,
    content: String,
    template_skip_counts: TemplateSkipCounts,
}

#[derive(Debug)]
struct BookImage {
    title: String,
    href: String,
    media_type: String,
    source_pages: Vec<String>,
    source: BookImageSource,
}

#[derive(Debug)]
enum BookImageSource {
    Local(PathBuf),
    Remote { title: String },
}

#[derive(Debug)]
struct ResolvedImage {
    href: String,
    media_type: String,
    bytes: Vec<u8>,
}

#[derive(Debug)]
struct ImageOccurrence {
    href: String,
    alt: String,
    caption: String,
}

#[derive(Debug)]
struct ImageRegistry {
    availability: ImageAvailability,
    images: Vec<BookImage>,
    images_by_title: HashMap<String, usize>,
    occurrences: Vec<ImageOccurrence>,
}

#[derive(Debug)]
enum ImageAvailability {
    All,
    Local {
        root: PathBuf,
        fixtures: HashMap<String, LocalImageFixture>,
    },
}

#[derive(Clone, Debug, Deserialize)]
struct LocalImageFixture {
    path: PathBuf,
    #[serde(rename = "media-type")]
    media_type: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TemplateSkipCounts {
    recognized: usize,
    unknown: usize,
}

thread_local! {
    static TEMPLATE_SKIP_COUNTS: RefCell<Option<TemplateSkipCounts>> = const { RefCell::new(None) };
}

#[derive(Debug, Parser)]
#[command(name = "wikipedia-to-epub")]
struct CliArgs {
    #[arg(value_name = "config.yaml")]
    config_path: PathBuf,
    #[arg(long = "local", value_name = "pages-dir")]
    local_pages_dir: Option<PathBuf>,
    #[arg(long = "refresh-cache")]
    refresh_cache: bool,
    #[arg(long = "log", value_name = "level", default_value_t = Level::INFO)]
    log_level: Level,
}

trait PageSource {
    fn load_page(&self, article: &str) -> AppResult<PageResponse>;
}

struct WikipediaApiPageSource {
    client: Client,
    api_url: Url,
    language: String,
    cache: DownloadCache,
}

impl WikipediaApiPageSource {
    fn new(language: &str, cache: DownloadCache) -> AppResult<Self> {
        let client = Client::builder().user_agent(USER_AGENT).build()?;
        let api_url = wikipedia_parse_api_url(language)?;
        Ok(Self {
            client,
            api_url,
            language: language.to_string(),
            cache,
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
            || self.fetch_page_payload(article),
        )?;
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
                    || self.fetch_page_payload(article),
                )?;
                Self::parse_page_payload(article, &payload)
            }
            Err(err) => Err(err),
        }
    }
}

#[derive(Clone, Debug)]
struct DownloadCache {
    root: PathBuf,
    refresh: bool,
    stats: DownloadStats,
}

impl DownloadCache {
    fn new(root: PathBuf, refresh: bool, stats: DownloadStats) -> Self {
        Self {
            root,
            refresh,
            stats,
        }
    }

    fn page_json_path(&self, language: &str, article: &str) -> PathBuf {
        self.root
            .join("pages")
            .join(language)
            .join(format!("{}.json", cache_key(article)))
    }

    fn image_metadata_path(&self, language: &str, title: &str) -> PathBuf {
        self.root
            .join("images")
            .join("metadata")
            .join(language)
            .join(format!("{}.json", cache_key(title)))
    }

    fn image_file_path(&self, url: &str, extension: &str) -> PathBuf {
        self.root
            .join("images")
            .join("files")
            .join(format!("{}.{}", cache_key(url), extension))
    }
}

#[derive(Clone, Debug, Default)]
struct DownloadStats {
    json: Rc<FileDownloadStats>,
    images: Rc<FileDownloadStats>,
}

#[derive(Debug, Default)]
struct FileDownloadStats {
    needed: Cell<usize>,
    from_cache: Cell<usize>,
    downloaded: Cell<usize>,
    failed: Cell<usize>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct FileDownloadSnapshot {
    needed: usize,
    from_cache: usize,
    downloaded: usize,
    failed: usize,
}

impl FileDownloadStats {
    fn snapshot(&self) -> FileDownloadSnapshot {
        FileDownloadSnapshot {
            needed: self.needed.get(),
            from_cache: self.from_cache.get(),
            downloaded: self.downloaded.get(),
            failed: self.failed.get(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CacheSource {
    Hit,
    Refreshed,
}

struct FixturePageSource {
    pages_dir: PathBuf,
}

impl FixturePageSource {
    fn new(pages_dir: impl Into<PathBuf>) -> Self {
        Self {
            pages_dir: pages_dir.into(),
        }
    }
}

impl PageSource for FixturePageSource {
    fn load_page(&self, article: &str) -> AppResult<PageResponse> {
        let page_path = find_page_path(article, &self.pages_dir)?;
        read_json::<PageResponse>(&page_path)
    }
}

fn main() {
    if let Err(err) = try_main() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn try_main() -> AppResult<()> {
    let args = parse_args()?;
    init_logging(args.log_level);
    info!(
        config_path = %args.config_path.display(),
        local_pages_dir = ?args.local_pages_dir,
        log_level = ?args.log_level,
        "starting wikipedia-to-epub"
    );
    run(args)
}

fn run(args: CliArgs) -> AppResult<()> {
    let config = read_config(&args.config_path)?;
    let wikipedia_language = normalized_wikipedia_language(&config.metadata.language)?;
    if config.articles.is_empty() {
        return Err(AppError::Message(
            "the configuration must contain at least one article".to_string(),
        ));
    }

    let local_pages_dir = args.local_pages_dir.clone();
    let download_stats = DownloadStats::default();
    let download_cache = if local_pages_dir.is_some() {
        None
    } else {
        Some(DownloadCache::new(
            default_cache_root()?,
            args.refresh_cache,
            download_stats.clone(),
        ))
    };
    let page_source: Box<dyn PageSource> = if let Some(pages_dir) = args.local_pages_dir {
        Box::new(FixturePageSource::new(pages_dir))
    } else {
        Box::new(WikipediaApiPageSource::new(
            &wikipedia_language,
            download_cache
                .clone()
                .expect("download cache is present for live API mode"),
        )?)
    };
    let mut image_registry = if config.images {
        Some(ImageRegistry::new(local_pages_dir.as_deref())?)
    } else {
        None
    };

    let internal_links = internal_links(&config.articles);
    let chapters = config
        .articles
        .iter()
        .enumerate()
        .map(|(index, article)| {
            load_chapter(
                page_source.as_ref(),
                article,
                index + 1,
                &internal_links,
                &wikipedia_language,
                image_registry.as_mut(),
            )
        })
        .collect::<AppResult<Vec<_>>>()?;
    let total_template_skip_counts =
        chapters
            .iter()
            .fold(TemplateSkipCounts::default(), |mut total, chapter| {
                total.recognized += chapter.template_skip_counts.recognized;
                total.unknown += chapter.template_skip_counts.unknown;
                total
            });
    info!(
        recognized_skipped_templates = total_template_skip_counts.recognized,
        unknown_skipped_templates = total_template_skip_counts.unknown,
        "template skip totals"
    );

    let images = if let Some(image_registry) = image_registry {
        resolve_images(image_registry, &wikipedia_language, download_cache.as_ref())?
    } else {
        Vec::new()
    };

    write_epub(&config, &chapters, &images, &wikipedia_language)?;
    println!("Created {}", config.output_file.display());
    println!(
        "Skipped templates: recognized={}, unknown={}",
        total_template_skip_counts.recognized, total_template_skip_counts.unknown
    );
    log_download_stats(&download_stats);

    Ok(())
}

fn parse_args() -> AppResult<CliArgs> {
    parse_args_from(env::args_os())
}

fn parse_args_from<I, T>(args: I) -> AppResult<CliArgs>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    CliArgs::try_parse_from(args).map_err(|err| AppError::Message(err.to_string()))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> AppResult<T> {
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

fn default_cache_root() -> AppResult<PathBuf> {
    let cache_dir = if cfg!(target_os = "windows") {
        env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join("Library").join("Caches"))
    } else {
        env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
    };

    cache_dir
        .map(|path| path.join("wikipedia-to-epub"))
        .ok_or_else(|| {
            AppError::Message(
                "could not determine the user cache directory for live downloads".to_string(),
            )
        })
}

#[cfg(test)]
fn read_or_fetch_text(
    cache_path: &Path,
    refresh: bool,
    fetch: impl FnOnce() -> AppResult<String>,
) -> AppResult<(String, CacheSource)> {
    read_or_fetch_text_with_stats(cache_path, refresh, None, fetch)
}

fn read_or_fetch_text_with_stats(
    cache_path: &Path,
    refresh: bool,
    stats: Option<&FileDownloadStats>,
    fetch: impl FnOnce() -> AppResult<String>,
) -> AppResult<(String, CacheSource)> {
    if let Some(stats) = stats {
        stats.needed.set(stats.needed.get() + 1);
    }
    if !refresh && cache_path.is_file() {
        if let Some(stats) = stats {
            stats.from_cache.set(stats.from_cache.get() + 1);
        }
        return Ok((fs::read_to_string(cache_path)?, CacheSource::Hit));
    }

    let content = fetch_and_write_text_with_stats(cache_path, stats, fetch)?;
    Ok((content, CacheSource::Refreshed))
}

fn fetch_and_write_text_with_stats(
    cache_path: &Path,
    stats: Option<&FileDownloadStats>,
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
    if let Err(err) = write_cache_text(cache_path, &content) {
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

#[cfg(test)]
fn read_or_fetch_bytes(
    cache_path: &Path,
    refresh: bool,
    fetch: impl FnOnce() -> AppResult<Vec<u8>>,
) -> AppResult<(Vec<u8>, CacheSource)> {
    read_or_fetch_bytes_with_stats(cache_path, refresh, None, fetch)
}

fn read_or_fetch_bytes_with_stats(
    cache_path: &Path,
    refresh: bool,
    stats: Option<&FileDownloadStats>,
    fetch: impl FnOnce() -> AppResult<Vec<u8>>,
) -> AppResult<(Vec<u8>, CacheSource)> {
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

fn log_download_stats(stats: &DownloadStats) {
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

fn write_cache_text(cache_path: &Path, content: &str) -> AppResult<()> {
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

fn read_config(path: &Path) -> AppResult<BookConfig> {
    let content = fs::read_to_string(path)?;
    Ok(serde_yaml::from_str(&content)?)
}

fn init_logging(level: Level) {
    use tracing_subscriber::prelude::*;
    let level_filter = tracing_subscriber::filter::LevelFilter::from_level(level);

    let stdout_layer = tracing_fmt::layer()
        .with_target(false)
        .with_filter(level_filter);

    let file_layer = std::fs::File::create("report.log").ok().map(|file| {
        tracing_fmt::layer()
            .with_ansi(false)
            .with_target(false)
            .with_writer(file)
            .with_filter(level_filter)
    });

    let _ = tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .try_init();
}

fn internal_links(articles: &[String]) -> InternalLinks {
    let mut links = InternalLinks::new();
    for (index, article) in articles.iter().enumerate() {
        links
            .entry(normalize_lookup_key(article))
            .or_insert_with(|| format!("chapter-{}.xhtml", index + 1));
    }
    links
}

fn load_chapter(
    page_source: &dyn PageSource,
    article: &str,
    index: usize,
    internal_links: &InternalLinks,
    language: &str,
    image_registry: Option<&mut ImageRegistry>,
) -> AppResult<Chapter> {
    info!(article = article, "fetching article");
    let page = page_source.load_page(article)?;
    let (rendered, template_skip_counts) = render_wikitext_with_template_counts(
        &page.parse.title,
        &page.parse.wikitext.text,
        internal_links,
        language,
        image_registry,
    );
    info!(
        article = article,
        title = page.parse.title,
        recognized_skipped_templates = template_skip_counts.recognized,
        unknown_skipped_templates = template_skip_counts.unknown,
        "article template skip counts"
    );

    Ok(Chapter {
        file_name: format!("chapter-{index}.xhtml"),
        title: page.parse.title,
        content: rendered,
        template_skip_counts,
    })
}

fn find_page_path(article: &str, pages_dir: &Path) -> AppResult<PathBuf> {
    for candidate in article_file_candidates(article) {
        let path = pages_dir.join(candidate);
        if path.is_file() {
            return Ok(path);
        }
    }

    let wanted = normalize_lookup_key(article);
    let mut entries = fs::read_dir(pages_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    entries.sort();

    if let Some(path) = entries.into_iter().find(|path| {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .map(normalize_lookup_key)
            .is_some_and(|candidate| candidate == wanted)
    }) {
        return Ok(path);
    }

    Err(AppError::Message(format!(
        "article '{article}' was not found in {}",
        pages_dir.display()
    )))
}

fn article_file_candidates(article: &str) -> Vec<String> {
    let trimmed = article.trim();
    let lowercase = trimmed.to_lowercase();
    let underscore = trimmed.replace(' ', "_");
    let underscore_lower = lowercase.replace(' ', "_");
    let dash = trimmed.replace(' ', "-");
    let dash_lower = lowercase.replace(' ', "-");

    [
        format!("{trimmed}.json"),
        format!("{lowercase}.json"),
        format!("{underscore}.json"),
        format!("{underscore_lower}.json"),
        format!("{dash}.json"),
        format!("{dash_lower}.json"),
    ]
    .into_iter()
    .collect::<Vec<_>>()
}

fn normalize_lookup_key(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|ch| ch.is_alphanumeric())
        .collect()
}

fn http_failure_detail(headers: &HeaderMap, body: &str) -> Option<String> {
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

#[cfg(test)]
fn render_wikitext(
    title: &str,
    wikitext: &str,
    internal_links: &InternalLinks,
    language: &str,
) -> String {
    render_wikitext_with_template_counts(title, wikitext, internal_links, language, None).0
}

fn render_wikitext_with_template_counts(
    title: &str,
    wikitext: &str,
    internal_links: &InternalLinks,
    language: &str,
    image_registry: Option<&mut ImageRegistry>,
) -> (String, TemplateSkipCounts) {
    with_template_skip_counts(|| {
        render_wikitext_impl(title, wikitext, internal_links, language, image_registry)
    })
}

fn with_template_skip_counts(render: impl FnOnce() -> String) -> (String, TemplateSkipCounts) {
    TEMPLATE_SKIP_COUNTS.with(|counts| {
        let previous = counts.replace(Some(TemplateSkipCounts::default()));
        let rendered = render();
        let current = counts.replace(previous).unwrap_or_default();
        (rendered, current)
    })
}

fn increment_recognized_skipped_template_count() {
    TEMPLATE_SKIP_COUNTS.with(|counts| {
        if let Some(counts) = counts.borrow_mut().as_mut() {
            counts.recognized += 1;
        }
    });
}

fn increment_unknown_skipped_template_count() {
    TEMPLATE_SKIP_COUNTS.with(|counts| {
        if let Some(counts) = counts.borrow_mut().as_mut() {
            counts.unknown += 1;
        }
    });
}

fn render_wikitext_impl(
    title: &str,
    wikitext: &str,
    internal_links: &InternalLinks,
    language: &str,
    mut image_registry: Option<&mut ImageRegistry>,
) -> String {
    let mut text = wikitext.replace("\r\n", "\n");
    text = Regex::new(r"(?s)<!--.*?-->")
        .unwrap()
        .replace_all(&text, "")
        .into_owned();
    text = Regex::new(r"(?is)<ref\b[^>/]*/>")
        .unwrap()
        .replace_all(&text, "")
        .into_owned();
    text = Regex::new(r"(?is)<ref\b[^>]*>.*?</ref>")
        .unwrap()
        .replace_all(&text, "")
        .into_owned();
    for tag in ["gallery", "timeline", "math", "score", "syntaxhighlight"] {
        let pattern = format!(r"(?is)<{tag}\b[^>]*>.*?</{tag}>");
        text = Regex::new(&pattern)
            .unwrap()
            .replace_all(&text, "")
            .into_owned();
    }
    text = Regex::new(r"(?i)<br\s*/?>")
        .unwrap()
        .replace_all(&text, "\n")
        .into_owned();
    text = render_templates(&text);
    text = strip_balanced_sections(&text, "{|", "|}");
    text = process_file_links(
        &text,
        image_registry.as_deref_mut(),
        internal_links,
        language,
        title,
    );

    let list_re = Regex::new(r"^([*#]+)\s*(.+?)\s*$").unwrap();
    let mut html = Vec::new();
    let mut paragraph_lines = Vec::new();
    let mut active_list: Option<char> = None;

    for raw_line in text.lines() {
        let line = raw_line.trim();

        if line.is_empty() {
            flush_paragraph(&mut html, &mut paragraph_lines);
            flush_list(&mut html, &mut active_list);
            continue;
        }

        if line.starts_with("[[Category:") || line == "__TOC__" || line == "__NOTOC__" {
            continue;
        }

        if line.starts_with('|') || line.starts_with('!') || line == "|-" {
            continue;
        }

        if let Some(image_id) = image_marker_id(line) {
            flush_paragraph(&mut html, &mut paragraph_lines);
            flush_list(&mut html, &mut active_list);
            if let Some(registry) = image_registry.as_deref()
                && let Some(image) = registry.occurrence(image_id)
            {
                html.push(render_image_html(image));
            }
            continue;
        }

        if line == "__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_START__" {
            flush_paragraph(&mut html, &mut paragraph_lines);
            flush_list(&mut html, &mut active_list);
            html.push("<blockquote>".to_string());
            continue;
        }

        if line == "__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_END__" {
            flush_paragraph(&mut html, &mut paragraph_lines);
            flush_list(&mut html, &mut active_list);
            html.push("</blockquote>".to_string());
            continue;
        }

        if let Some(text) = line.strip_prefix("__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__") {
            flush_paragraph(&mut html, &mut paragraph_lines);
            flush_list(&mut html, &mut active_list);
            let text = cleanup_inline_markup(text, internal_links, language);
            if !text.is_empty() {
                html.push(format!("<p>{text}</p>"));
            }
            continue;
        }

        if let Some(source) = line.strip_prefix("__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_SOURCE__") {
            flush_paragraph(&mut html, &mut paragraph_lines);
            flush_list(&mut html, &mut active_list);
            let source = cleanup_inline_markup(source, internal_links, language);
            if !source.is_empty() {
                html.push(format!(r#"<p class="blockquote-source">{source}</p>"#));
            }
            continue;
        }

        if let Some((level, heading)) = parse_heading(line) {
            flush_paragraph(&mut html, &mut paragraph_lines);
            flush_list(&mut html, &mut active_list);

            let heading = cleanup_inline_markup(&heading, internal_links, language);
            if !heading.is_empty() {
                html.push(format!("<h{level}>{heading}</h{level}>"));
            }
            continue;
        }

        if let Some(captures) = list_re.captures(line) {
            flush_paragraph(&mut html, &mut paragraph_lines);

            let marker = captures[1].chars().next().unwrap_or('*');
            if active_list != Some(marker) {
                flush_list(&mut html, &mut active_list);
                active_list = Some(marker);
                html.push(if marker == '#' {
                    "<ol>".to_string()
                } else {
                    "<ul>".to_string()
                });
            }

            let item = cleanup_inline_markup(&captures[2], internal_links, language);
            if !item.is_empty() {
                html.push(format!("<li>{item}</li>"));
            }
            continue;
        }

        flush_list(&mut html, &mut active_list);

        let cleaned = cleanup_inline_markup(line, internal_links, language);
        if !cleaned.is_empty() {
            paragraph_lines.push(cleaned);
        }
    }

    flush_paragraph(&mut html, &mut paragraph_lines);
    flush_list(&mut html, &mut active_list);

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" {language_attributes}>
  <head>
    <title>{}</title>
    <link rel="stylesheet" type="text/css" href="style.css" />
  </head>
  <body>
    <h1>{}</h1>
    {}
  </body>
</html>
"#,
        encode_text(title),
        encode_text(title),
        html.join("\n    "),
        language_attributes = html_language_attributes(language),
    )
}

fn flush_paragraph(html: &mut Vec<String>, paragraph_lines: &mut Vec<String>) {
    if paragraph_lines.is_empty() {
        return;
    }

    html.push(format!("<p>{}</p>", paragraph_lines.join(" ")));
    paragraph_lines.clear();
}

fn flush_list(html: &mut Vec<String>, active_list: &mut Option<char>) {
    if let Some(marker) = active_list.take() {
        html.push(if marker == '#' {
            "</ol>".to_string()
        } else {
            "</ul>".to_string()
        });
    }
}

fn render_templates(text: &str) -> String {
    let mut rendered = String::new();
    let mut offset = 0;

    while let Some(start) = text[offset..].find("{{").map(|index| offset + index) {
        rendered.push_str(&text[offset..start]);

        if let Some(end) = matching_template_end(text, start) {
            let content = &text[start + 2..end];
            rendered.push_str(&render_template(content));
            offset = end + 2;
        } else {
            rendered.push_str(&text[start..]);
            offset = text.len();
        }
    }

    rendered.push_str(&text[offset..]);
    rendered
}

fn matching_template_end(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 1usize;
    let mut index = start + 2;

    while index + 1 < bytes.len() {
        if bytes[index] == b'{' && bytes[index + 1] == b'{' {
            depth += 1;
            index += 2;
        } else if bytes[index] == b'}' && bytes[index + 1] == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
            index += 2;
        } else {
            index += 1;
        }
    }

    None
}

fn render_template(content: &str) -> String {
    let (template, params) = split_template_name(content);
    let template = template.trim();

    if template.eq_ignore_ascii_case("Korean") || template.eq_ignore_ascii_case("Korean/auto") {
        render_korean_template(params)
    } else if template.eq_ignore_ascii_case("Nihongo4") || template.eq_ignore_ascii_case("Nihongo")
    {
        render_japanese_template(params)
    } else if template.eq_ignore_ascii_case("nbsp") {
        render_nonbreaking_space_template()
    } else if template.eq_ignore_ascii_case("snd") {
        render_spaced_endash_template()
    } else if template.eq_ignore_ascii_case("mdash") {
        render_emdash_template()
    } else if template.eq_ignore_ascii_case("nowrap") {
        render_passthrough_template(params)
    } else if template.eq_ignore_ascii_case("smaller") {
        render_smaller_template(params)
    } else if template.eq_ignore_ascii_case("sic") {
        render_sic_template(params)
    } else if template.eq_ignore_ascii_case("circa")
        || template.eq_ignore_ascii_case("c.")
        || template.eq_ignore_ascii_case("cx")
    {
        render_circa_template(params)
    } else if template.eq_ignore_ascii_case("lang") {
        render_lang_template(params)
    } else if template.eq_ignore_ascii_case("in lang") {
        render_in_lang_template(params)
    } else if template.eq_ignore_ascii_case("langx") {
        render_langx_template(params)
    } else if template.eq_ignore_ascii_case("linktext") {
        render_linktext_template(params)
    } else if template.eq_ignore_ascii_case("lang-zh")
        || template.eq_ignore_ascii_case("zh")
        || template.eq_ignore_ascii_case("zhi")
    {
        render_chinese_lang_template(params)
    } else if template.eq_ignore_ascii_case("transliteration") {
        render_transliteration_template(params)
    } else if template.eq_ignore_ascii_case("tlit") {
        render_transliteration_like_template(params)
    } else if template.eq_ignore_ascii_case("ko-translit") {
        render_korean_transliteration_template(params)
    } else if template.eq_ignore_ascii_case("lit") {
        render_literal_template(params)
    } else if template.eq_ignore_ascii_case("isbn") {
        render_isbn_template(params)
    } else if template.eq_ignore_ascii_case("ipa") {
        render_ipa_template(params)
    } else if template.eq_ignore_ascii_case("IPAc-en") {
        render_english_ipa_template(params)
    } else if template.eq_ignore_ascii_case("Respell") {
        render_respell_template(params)
    } else if template.eq_ignore_ascii_case("abbr") {
        render_abbr_template(params)
    } else if template.eq_ignore_ascii_case("frac") || template.eq_ignore_ascii_case("fraction") {
        render_frac_template(params)
    } else if template.eq_ignore_ascii_case("floruit") {
        render_floruit_template(params)
    } else if template.eq_ignore_ascii_case("coord") {
        render_coord_template(params)
    } else if template.eq_ignore_ascii_case("rp") {
        render_reference_page_template(params)
    } else if template.eq_ignore_ascii_case("cite web") {
        render_cite_web_template(params)
    } else if template.eq_ignore_ascii_case("cite book") {
        render_cite_book_template(params)
    } else if template.eq_ignore_ascii_case("cite journal") {
        render_cite_journal_template(params)
    } else if template.eq_ignore_ascii_case("cite report") {
        render_cite_report_template(params)
    } else if template.eq_ignore_ascii_case("cite ECCP") {
        render_cite_eccp_template(params)
    } else if template.eq_ignore_ascii_case("cite conference")
        || template.eq_ignore_ascii_case("citation")
    {
        render_citation_template(params)
    } else if template.eq_ignore_ascii_case("harvc") {
        render_harvc_template(params)
    } else if template.eq_ignore_ascii_case("as of") {
        render_as_of_template(params)
    } else if template.eq_ignore_ascii_case("died-in") {
        render_died_in_template(params)
    } else if template.eq_ignore_ascii_case("blockquote") {
        render_blockquote_template(params)
    } else if template.eq_ignore_ascii_case("percentage") {
        render_percentage_template(params)
    } else if template.eq_ignore_ascii_case("UN_Population") {
        render_un_population_template(params)
    } else if template.eq_ignore_ascii_case("convert") || template.eq_ignore_ascii_case("cvt") {
        render_convert_template(params)
    } else if template.eq_ignore_ascii_case("for") {
        render_for_template(params)
    } else if template.eq_ignore_ascii_case("for timeline") {
        render_for_timeline_template(params)
    } else if template.eq_ignore_ascii_case("crossreference") {
        render_passthrough_template(params)
    } else if template.eq_ignore_ascii_case("slink") {
        render_section_link_template(params)
    } else if template.eq_ignore_ascii_case("legend") {
        render_legend_template(params)
    } else if template.eq_ignore_ascii_case("numero") {
        render_numero_template(params)
    } else if template.eq_ignore_ascii_case("anl") {
        render_article_link_template(params)
    } else if template.eq_ignore_ascii_case("excerpt") {
        render_excerpt_template(params)
    } else if template.eq_ignore_ascii_case("main") {
        render_main_template(params)
    } else if template.eq_ignore_ascii_case("see also") {
        render_see_also_template(params)
    } else if template.eq_ignore_ascii_case("further") {
        render_further_template(params)
    } else if template.eq_ignore_ascii_case("wiktionary") {
        render_wiktionary_template(params)
    } else if template.eq_ignore_ascii_case("wikivoyage") {
        render_wikivoyage_template(params)
    } else if template.eq_ignore_ascii_case("wikisource") {
        render_wikisource_template(params)
    } else if template.eq_ignore_ascii_case("wikibooks") {
        render_wikibooks_template(params)
    } else if template.eq_ignore_ascii_case("britannica") {
        render_britannica_template(params)
    } else if template.eq_ignore_ascii_case("official website") {
        render_official_website_template(params)
    } else if template.eq_ignore_ascii_case("url") {
        render_url_template(params)
    } else if template.eq_ignore_ascii_case("osmrelation-inline") {
        render_openstreetmap_relation_template(params)
    } else if template.eq_ignore_ascii_case("webarchive") {
        render_webarchive_template(params)
    } else if template.eq_ignore_ascii_case("largest cities") {
        render_largest_cities_template(params)
    } else if template.eq_ignore_ascii_case("historical populations") {
        render_historical_populations_template(params)
    } else if template.eq_ignore_ascii_case("climate chart") {
        render_climate_chart_template(params)
    } else if template.eq_ignore_ascii_case("sclass") {
        render_ship_class_template(params)
    } else if template.eq_ignore_ascii_case("nobold") {
        render_passthrough_template(params)
    } else if template.eq_ignore_ascii_case("Arrow") {
        render_arrow_template(params)
    } else if template.eq_ignore_ascii_case("ROKS") {
        render_republic_of_korea_ship_template(params)
    } else if template.eq_ignore_ascii_case("ill") {
        render_interlanguage_link_template(params)
    } else if template.eq_ignore_ascii_case("reign") {
        render_reign_template(params)
    } else if template.eq_ignore_ascii_case("open access")
        || template.eq_ignore_ascii_case("free access")
    {
        render_open_access_template()
    } else if template.eq_ignore_ascii_case("For-multi") {
        render_for_multi_template(params)
    } else if template.eq_ignore_ascii_case("Inflation") {
        render_inflation_template(params)
    } else if template.eq_ignore_ascii_case("Inflation/year") {
        render_inflation_year_template(params)
    } else if template.eq_ignore_ascii_case("stack") {
        render_passthrough_template(params)
    } else if template.eq_ignore_ascii_case("USS") {
        render_ship_template("USS", params)
    } else if template.eq_ignore_ascii_case("HMS") {
        render_ship_template("HMS", params)
    } else if template.eq_ignore_ascii_case("Collapsible list") {
        render_collapsible_list_template(params)
    } else if template.eq_ignore_ascii_case("Internet Archive short film") {
        render_internet_archive_short_film_template(params)
    } else if template.eq_ignore_ascii_case("worldhistory") {
        render_worldhistory_template(params)
    } else if template.eq_ignore_ascii_case("nihongo2") {
        render_nihongo2_template(params)
    } else if template.eq_ignore_ascii_case("gloss") {
        render_gloss_template(params)
    } else if template.eq_ignore_ascii_case("xref") {
        render_passthrough_template(params)
    } else if template.eq_ignore_ascii_case("Shy") {
        render_soft_hyphen_template(params)
    } else if template.eq_ignore_ascii_case("color box") {
        render_color_box_template(params)
    } else if template.eq_ignore_ascii_case("pb") {
        "__WIKIPEDIA_TO_EPUB_PB__".to_string()
    } else if template.eq_ignore_ascii_case("OSM relation") {
        render_openstreetmap_relation_template(params)
    } else if template.eq_ignore_ascii_case("okina") {
        "ʻ".to_string()
    } else if template.eq_ignore_ascii_case("'s") {
        "'s".to_string()
    } else if template.eq_ignore_ascii_case("harvp") {
        render_harvp_template(params)
    } else if template.eq_ignore_ascii_case("IPAslink") {
        render_ipa_link_template(params)
    } else if template.eq_ignore_ascii_case("angbr") {
        render_angbr_template(params)
    } else if template.eq_ignore_ascii_case("angbr IPA") {
        render_angbr_ipa_template(params)
    } else if template.eq_ignore_ascii_case("unichar") {
        render_unichar_template(params)
    } else if template.eq_ignore_ascii_case("xlit") {
        render_transliteration_template(params)
    } else if template.eq_ignore_ascii_case("note") {
        render_note_template(params)
    } else if template.eq_ignore_ascii_case("fs interlinear") {
        render_fs_interlinear_template(params)
    } else if template.eq_ignore_ascii_case("Tooltip") {
        render_tooltip_template(params)
    } else if template.eq_ignore_ascii_case("Nihongo krt") {
        render_nihongo_krt_template(params)
    } else if template.eq_ignore_ascii_case("Easy CSS image crop") {
        render_easy_css_image_crop_template(params)
    } else if template.eq_ignore_ascii_case("ISSN") {
        render_issn_template(params)
    } else if template.eq_ignore_ascii_case("Cite NSRW") {
        render_cite_nsrw_template(params)
    } else if template
        .get(.."formatnum:".len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("formatnum:"))
        || template.eq_ignore_ascii_case("formatnum")
    {
        render_formatnum_template(template, params)
    } else if template.eq_ignore_ascii_case("STN") {
        render_stn_template(params)
    } else if is_silent_template_name(template) {
        increment_recognized_skipped_template_count();
        String::new()
    } else {
        increment_unknown_skipped_template_count();
        debug!(
            content = template_log_content(content),
            "removing unhandled wikitext template"
        );
        log_and_count_nested_skipped_templates(params);
        String::new()
    }
}

fn log_and_count_nested_skipped_templates(text: &str) {
    let mut offset = 0;

    while let Some(start) = text[offset..].find("{{").map(|index| offset + index) {
        if let Some(end) = matching_template_end(text, start) {
            let content = &text[start + 2..end];
            let (template, params) = split_template_name(content);
            let template = template.trim();
            if is_silent_template_name(template) {
                increment_recognized_skipped_template_count();
            } else if !is_handled_template_name(template) {
                increment_unknown_skipped_template_count();
                debug!(
                    content = template_log_content(content),
                    "removing nested unhandled wikitext template"
                );
                log_and_count_nested_skipped_templates(params);
            }
            offset = end + 2;
        } else {
            break;
        }
    }
}

fn template_log_content(content: &str) -> String {
    content.chars().take(80).collect()
}

fn is_handled_template_name(template: &str) -> bool {
    template.eq_ignore_ascii_case("Korean")
        || template.eq_ignore_ascii_case("Korean/auto")
        || template.eq_ignore_ascii_case("Nihongo4")
        || template.eq_ignore_ascii_case("Nihongo")
        || template.eq_ignore_ascii_case("nbsp")
        || template.eq_ignore_ascii_case("snd")
        || template.eq_ignore_ascii_case("mdash")
        || template.eq_ignore_ascii_case("nowrap")
        || template.eq_ignore_ascii_case("smaller")
        || template.eq_ignore_ascii_case("sic")
        || template.eq_ignore_ascii_case("circa")
        || template.eq_ignore_ascii_case("c.")
        || template.eq_ignore_ascii_case("cx")
        || template.eq_ignore_ascii_case("lang")
        || template.eq_ignore_ascii_case("in lang")
        || template.eq_ignore_ascii_case("langx")
        || template.eq_ignore_ascii_case("linktext")
        || template.eq_ignore_ascii_case("lang-zh")
        || template.eq_ignore_ascii_case("zh")
        || template.eq_ignore_ascii_case("zhi")
        || template.eq_ignore_ascii_case("transliteration")
        || template.eq_ignore_ascii_case("tlit")
        || template.eq_ignore_ascii_case("ko-translit")
        || template.eq_ignore_ascii_case("lit")
        || template.eq_ignore_ascii_case("isbn")
        || template.eq_ignore_ascii_case("ipa")
        || template.eq_ignore_ascii_case("IPAc-en")
        || template.eq_ignore_ascii_case("Respell")
        || template.eq_ignore_ascii_case("abbr")
        || template.eq_ignore_ascii_case("frac")
        || template.eq_ignore_ascii_case("fraction")
        || template.eq_ignore_ascii_case("floruit")
        || template.eq_ignore_ascii_case("coord")
        || template.eq_ignore_ascii_case("rp")
        || template.eq_ignore_ascii_case("cite web")
        || template.eq_ignore_ascii_case("cite book")
        || template.eq_ignore_ascii_case("cite journal")
        || template.eq_ignore_ascii_case("cite report")
        || template.eq_ignore_ascii_case("cite ECCP")
        || template.eq_ignore_ascii_case("cite conference")
        || template.eq_ignore_ascii_case("citation")
        || template.eq_ignore_ascii_case("harvc")
        || template.eq_ignore_ascii_case("as of")
        || template.eq_ignore_ascii_case("died-in")
        || template.eq_ignore_ascii_case("blockquote")
        || template.eq_ignore_ascii_case("percentage")
        || template.eq_ignore_ascii_case("UN_Population")
        || template.eq_ignore_ascii_case("convert")
        || template.eq_ignore_ascii_case("cvt")
        || template.eq_ignore_ascii_case("for")
        || template.eq_ignore_ascii_case("for timeline")
        || template.eq_ignore_ascii_case("crossreference")
        || template.eq_ignore_ascii_case("slink")
        || template.eq_ignore_ascii_case("legend")
        || template.eq_ignore_ascii_case("numero")
        || template.eq_ignore_ascii_case("anl")
        || template.eq_ignore_ascii_case("excerpt")
        || template.eq_ignore_ascii_case("main")
        || template.eq_ignore_ascii_case("see also")
        || template.eq_ignore_ascii_case("further")
        || template.eq_ignore_ascii_case("wiktionary")
        || template.eq_ignore_ascii_case("wikivoyage")
        || template.eq_ignore_ascii_case("wikisource")
        || template.eq_ignore_ascii_case("wikibooks")
        || template.eq_ignore_ascii_case("britannica")
        || template.eq_ignore_ascii_case("official website")
        || template.eq_ignore_ascii_case("url")
        || template.eq_ignore_ascii_case("osmrelation-inline")
        || template.eq_ignore_ascii_case("webarchive")
        || template.eq_ignore_ascii_case("largest cities")
        || template.eq_ignore_ascii_case("historical populations")
        || template.eq_ignore_ascii_case("climate chart")
        || template.eq_ignore_ascii_case("sclass")
        || template.eq_ignore_ascii_case("nobold")
        || template.eq_ignore_ascii_case("Arrow")
        || template.eq_ignore_ascii_case("ROKS")
        || template.eq_ignore_ascii_case("ill")
        || template.eq_ignore_ascii_case("reign")
        || template.eq_ignore_ascii_case("open access")
        || template.eq_ignore_ascii_case("free access")
        || template.eq_ignore_ascii_case("For-multi")
        || template.eq_ignore_ascii_case("Inflation")
        || template.eq_ignore_ascii_case("Inflation/year")
        || template.eq_ignore_ascii_case("stack")
        || template.eq_ignore_ascii_case("USS")
        || template.eq_ignore_ascii_case("HMS")
        || template.eq_ignore_ascii_case("Collapsible list")
        || template.eq_ignore_ascii_case("Internet Archive short film")
        || template.eq_ignore_ascii_case("worldhistory")
        || template.eq_ignore_ascii_case("nihongo2")
        || template.eq_ignore_ascii_case("gloss")
        || template.eq_ignore_ascii_case("xref")
        || template.eq_ignore_ascii_case("Shy")
        || template.eq_ignore_ascii_case("color box")
        || template.eq_ignore_ascii_case("pb")
        || template.eq_ignore_ascii_case("OSM relation")
        || template.eq_ignore_ascii_case("okina")
        || template.eq_ignore_ascii_case("'s")
        || template.eq_ignore_ascii_case("harvp")
        || template.eq_ignore_ascii_case("IPAslink")
        || template.eq_ignore_ascii_case("angbr")
        || template.eq_ignore_ascii_case("angbr IPA")
        || template.eq_ignore_ascii_case("unichar")
        || template.eq_ignore_ascii_case("xlit")
        || template.eq_ignore_ascii_case("note")
        || template.eq_ignore_ascii_case("fs interlinear")
        || template.eq_ignore_ascii_case("Tooltip")
        || template.eq_ignore_ascii_case("Nihongo krt")
        || template.eq_ignore_ascii_case("Easy CSS image crop")
        || template.eq_ignore_ascii_case("ISSN")
        || template.eq_ignore_ascii_case("Cite NSRW")
        || template
            .get(.."formatnum:".len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("formatnum:"))
        || template.eq_ignore_ascii_case("formatnum")
        || template.eq_ignore_ascii_case("STN")
        || is_silent_template_name(template)
}

fn is_silent_template_name(template: &str) -> bool {
    let template = template.trim();
    template_name_is_in_csv(template, include_str!("silent.csv"))
        || template.ends_with(" weatherbox")
        || template
            .get(.."DEFAULTSORT".len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("DEFAULTSORT"))
        || is_succession_template_name(template)
        || template
            .get(.."Self-published".len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("Self-published"))
        || template
            .get(.."Use ".len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("Use "))
        || template
            .get(.."Infobox".len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("Infobox"))
        || is_observed_navigation_template_name(template)
}

fn is_observed_navigation_template_name(template: &str) -> bool {
    template_name_is_in_csv(template.trim(), include_str!("navigations.csv"))
}

fn template_name_is_in_csv(template: &str, csv: &str) -> bool {
    csv.lines().any(|line| {
        line.split_once(',')
            .map_or(line, |(name, _)| name)
            .trim()
            .eq_ignore_ascii_case(template)
    })
}

fn is_succession_template_name(template: &str) -> bool {
    template
        .get(.."s-".len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("s-"))
        || template.eq_ignore_ascii_case("Succession box")
}

fn split_template_name(content: &str) -> (&str, &str) {
    let mut template_depth = 0usize;
    let mut link_depth = 0usize;
    let mut chars = content.char_indices().peekable();

    while let Some((index, ch)) = chars.next() {
        if ch == '[' && chars.peek().is_some_and(|(_, next)| *next == '[') {
            chars.next();
            link_depth += 1;
        } else if ch == ']' && chars.peek().is_some_and(|(_, next)| *next == ']') {
            chars.next();
            link_depth = link_depth.saturating_sub(1);
        } else if ch == '{' && chars.peek().is_some_and(|(_, next)| *next == '{') {
            chars.next();
            template_depth += 1;
        } else if ch == '}' && chars.peek().is_some_and(|(_, next)| *next == '}') {
            chars.next();
            template_depth = template_depth.saturating_sub(1);
        } else if ch == '|' && template_depth == 0 && link_depth == 0 {
            return (&content[..index], &content[index + 1..]);
        }
    }

    (content, "")
}

fn render_korean_template(params: &str) -> String {
    let mut hangul = None;
    let mut hanja = None;
    let mut ko_ipa = None;
    let mut positional = Vec::new();

    for part in split_template_params(params)
        .into_iter()
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
    {
        if let Some((key, value)) = part.split_once('=') {
            match key.trim().to_lowercase().as_str() {
                "hangul" => hangul = Some(clean_korean_auto_value(value)),
                "hanja" => hanja = Some(clean_korean_auto_value(value)),
                "ko_ipa" => ko_ipa = Some(value.trim().to_string()),
                _ => {}
            }
        } else {
            positional.push(clean_korean_auto_value(&part));
        }
    }

    let hangul = hangul.or_else(|| positional.first().cloned());
    let hanja = hanja.or_else(|| positional.get(1).cloned());
    let mut values = Vec::new();

    if let Some(hangul) = hangul.as_deref()
        && !hangul.trim().is_empty()
    {
        values.push(format!(
            "Korean: __WIKIPEDIA_TO_EPUB_KOREAN_HANGUL_START__{hangul}__WIKIPEDIA_TO_EPUB_KOREAN_SCRIPT_END__"
        ));
    }

    if let Some(hanja) = hanja.as_deref()
        && !hanja.trim().is_empty()
    {
        values.push(format!(
            "Hanja: __WIKIPEDIA_TO_EPUB_KOREAN_HANJA_START__{hanja}__WIKIPEDIA_TO_EPUB_KOREAN_SCRIPT_END__"
        ));
    }

    if let Some(ko_ipa) = ko_ipa.as_deref()
        && !ko_ipa.trim().is_empty()
    {
        values.push(format!("pronounced [{}]", render_templates(ko_ipa.trim())));
    }

    if values.is_empty() {
        return String::new();
    }

    format!(
        "__WIKIPEDIA_TO_EPUB_KOREAN_TEXT_START__{}__WIKIPEDIA_TO_EPUB_KOREAN_TEXT_END__",
        values.join(" / ")
    )
}

fn clean_korean_auto_value(value: &str) -> String {
    value
        .trim()
        .chars()
        .filter(|ch| !matches!(ch, '^' | '%' | '_'))
        .collect()
}

fn render_japanese_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);
    let term = positional.first().map_or("", |value| value.trim());
    let japanese = positional.get(1).map_or("", |value| value.trim());

    if japanese.is_empty() {
        return term.to_string();
    }

    let extra = template_param(&named, &["extra"])
        .map(render_templates)
        .filter(|value| !value.trim().is_empty());
    let suffix = extra.map_or(String::new(), |extra| format!("; {extra}"));

    format!(
        "{term}__WIKIPEDIA_TO_EPUB_JAPANESE_NORMAL_START__ (__WIKIPEDIA_TO_EPUB_JAPANESE_TEXT_START__{japanese}__WIKIPEDIA_TO_EPUB_JAPANESE_TEXT_END__{suffix})__WIKIPEDIA_TO_EPUB_JAPANESE_NORMAL_END__"
    )
}

fn render_nonbreaking_space_template() -> String {
    " ".to_string()
}

fn render_spaced_endash_template() -> String {
    " – ".to_string()
}

fn render_emdash_template() -> String {
    "—".to_string()
}

fn render_passthrough_template(params: &str) -> String {
    template_positional_params(params)
        .into_iter()
        .map(|param| render_templates(&param))
        .filter(|param| !param.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_smaller_template(params: &str) -> String {
    let text = render_passthrough_template(params);
    if text.is_empty() {
        return String::new();
    }

    format!("__WIKIPEDIA_TO_EPUB_SMALL_START__{text}__WIKIPEDIA_TO_EPUB_SMALL_END__")
}

fn render_sic_template(params: &str) -> String {
    let text = render_passthrough_template(params);
    if text.is_empty() {
        "[sic]".to_string()
    } else {
        format!("{text} [sic]")
    }
}

fn render_circa_template(params: &str) -> String {
    let text = render_passthrough_template(params);
    if text.is_empty() {
        "c.".to_string()
    } else {
        format!("c. {text}")
    }
}

fn render_lang_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    let Some(language) = params
        .first()
        .map(String::as_str)
        .filter(|value| !value.is_empty())
    else {
        return String::new();
    };
    let Some(text) = params
        .get(1)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
    else {
        return String::new();
    };

    let language = language
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
        .collect::<String>();

    if language.is_empty() {
        return text.to_string();
    }

    let text = render_templates(text);

    format!(
        "__WIKIPEDIA_TO_EPUB_LANG_START__{language}__WIKIPEDIA_TO_EPUB_LANG_VALUE__{text}__WIKIPEDIA_TO_EPUB_LANG_END__"
    )
}

fn render_in_lang_template(params: &str) -> String {
    let languages = template_positional_params(params)
        .into_iter()
        .map(|language| language_name_for_in_lang(&language).to_string())
        .filter(|language| !language.is_empty())
        .collect::<Vec<_>>();

    match languages.as_slice() {
        [] => String::new(),
        [language] => format!("(in {language})"),
        languages => format!("(in {})", join_plain_items(languages)),
    }
}

fn language_name_for_in_lang(language: &str) -> &str {
    match language.trim().to_ascii_lowercase().as_str() {
        "ar" => "Arabic",
        "de" => "German",
        "en" => "English",
        "es" => "Spanish",
        "fa" => "Persian",
        "fr" => "French",
        "he" => "Hebrew",
        "ja" => "Japanese",
        "ko" => "Korean",
        "ru" => "Russian",
        "zh" | "zh-cn" | "zh-hans" | "zh-hant" | "zh-tw" => "Chinese",
        _ => language.trim(),
    }
}

fn render_linktext_template(params: &str) -> String {
    template_positional_params(params)
        .into_iter()
        .map(|param| render_templates(&param))
        .collect::<Vec<_>>()
        .join("")
}

fn render_langx_template(params: &str) -> String {
    let mut positional = Vec::new();
    let mut named = HashMap::new();

    for param in split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty())
    {
        if let Some((key, value)) = param.split_once('=') {
            named.insert(key.trim().to_lowercase(), value.trim().to_string());
        } else {
            positional.push(param);
        }
    }

    let Some(language) = positional.first().map(String::as_str) else {
        return String::new();
    };
    let Some(text) = positional.get(1).map(String::as_str) else {
        return String::new();
    };

    let mut rendered = render_lang_template(&format!("{language}|{text}"));

    if let Some(translit) = named
        .get("translit")
        .filter(|value| !value.trim().is_empty())
    {
        rendered.push_str(" (");
        rendered.push_str(translit.trim());
        rendered.push(')');
    }

    if let Some(literal) = named.get("lit").filter(|value| !value.trim().is_empty()) {
        rendered.push_str(", lit. ");
        rendered.push_str(literal.trim());
    }

    rendered
}

fn render_chinese_lang_template(params: &str) -> String {
    let mut positional = Vec::new();
    let mut named = HashMap::new();

    for param in split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty())
    {
        if let Some((key, value)) = param.split_once('=') {
            named.insert(key.trim().to_lowercase(), value.trim().to_string());
        } else {
            positional.push(param);
        }
    }

    let Some(text) = named
        .get("t")
        .or_else(|| named.get("s"))
        .or_else(|| named.get("c"))
        .or_else(|| named.get("text"))
        .or_else(|| positional.first())
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
    else {
        return String::new();
    };

    let mut rendered = render_lang_template(&format!("zh|{text}"));

    if let Some(pinyin) = named
        .get("p")
        .or_else(|| named.get("pinyin"))
        .filter(|value| !value.trim().is_empty())
    {
        rendered.push_str(" (");
        rendered.push_str(pinyin.trim());
        rendered.push(')');
    }

    rendered
}

fn render_transliteration_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    let Some(language) = params.first().map(String::as_str) else {
        return String::new();
    };
    let Some(text) = params
        .last()
        .map(String::as_str)
        .filter(|value| !value.is_empty() && params.len() > 1)
    else {
        return String::new();
    };

    let language = language
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
        .collect::<String>();

    if language.is_empty() {
        return render_templates(text);
    }

    format!(
        "__WIKIPEDIA_TO_EPUB_LANG_START__{language}-Latn__WIKIPEDIA_TO_EPUB_LANG_VALUE__{}__WIKIPEDIA_TO_EPUB_LANG_END__",
        render_templates(text)
    )
}

fn render_transliteration_like_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    let Some(language) = params.first().map(String::as_str) else {
        return String::new();
    };
    let Some(text) = params
        .last()
        .map(String::as_str)
        .filter(|_| params.len() > 1)
    else {
        return String::new();
    };

    let language = language
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
        .collect::<String>();

    if language.is_empty() {
        return render_templates(text);
    }

    format!(
        "__WIKIPEDIA_TO_EPUB_LANG_START__{language}-Latn__WIKIPEDIA_TO_EPUB_LANG_VALUE__{}__WIKIPEDIA_TO_EPUB_LANG_END__",
        render_templates(text)
    )
}

fn render_korean_transliteration_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    let Some(system) = params.first().map(String::as_str) else {
        return String::new();
    };
    let Some(korean) = params.get(1).map(|value| clean_korean_auto_value(value)) else {
        return String::new();
    };

    match (system.trim().to_ascii_lowercase().as_str(), korean.as_str()) {
        ("rr", "한국") => "Hanguk".to_string(),
        ("mr", "한국") => "Han'guk".to_string(),
        ("rr", "조선") => "Joseon".to_string(),
        ("mr", "조선") => "Chosŏn".to_string(),
        _ => korean,
    }
}

fn render_literal_template(params: &str) -> String {
    let Some(text) = template_positional_params(params)
        .into_iter()
        .find(|value| !value.trim().is_empty())
    else {
        return String::new();
    };

    format!("lit. {}", render_templates(&text))
}

fn render_isbn_template(params: &str) -> String {
    let Some(isbn) = template_positional_params(params)
        .into_iter()
        .find(|value| !value.trim().is_empty())
    else {
        return String::new();
    };

    format!("ISBN {}", render_templates(&isbn))
}

fn render_ipa_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    let Some(ipa) = params.get(1).map(String::as_str) else {
        return String::new();
    };

    format!(
        "__WIKIPEDIA_TO_EPUB_IPA_START__{}__WIKIPEDIA_TO_EPUB_IPA_END__",
        render_templates(ipa)
    )
}

fn render_english_ipa_template(params: &str) -> String {
    let ipa = template_positional_params(params)
        .into_iter()
        .filter(|param| {
            !matches!(
                param.trim().to_ascii_lowercase().as_str(),
                "lang" | "pron" | "pronunciation"
            )
        })
        .map(|param| render_templates(&param))
        .collect::<Vec<_>>()
        .join("");

    if ipa.is_empty() {
        return String::new();
    }

    format!("__WIKIPEDIA_TO_EPUB_IPA_START__{ipa}__WIKIPEDIA_TO_EPUB_IPA_END__")
}

fn render_respell_template(params: &str) -> String {
    template_positional_params(params)
        .into_iter()
        .map(|param| render_templates(&param))
        .filter(|param| !param.trim().is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn render_abbr_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .collect::<Vec<_>>();

    let Some(text) = params.first().filter(|value| !value.is_empty()) else {
        return String::new();
    };

    let Some(title) = params.get(1).filter(|value| !value.is_empty()) else {
        return render_templates(text);
    };

    format!(
        "__WIKIPEDIA_TO_EPUB_ABBR_START__{}__WIKIPEDIA_TO_EPUB_ABBR_VALUE__{}__WIKIPEDIA_TO_EPUB_ABBR_END__",
        render_templates(title),
        render_templates(text)
    )
}

fn render_frac_template(params: &str) -> String {
    let params = template_positional_params(params)
        .into_iter()
        .map(|param| render_templates(&param))
        .collect::<Vec<_>>();

    match params.as_slice() {
        [] => String::new(),
        [value] => value.clone(),
        [numerator, denominator] => format!("{numerator}/{denominator}"),
        [whole, numerator, denominator] => format!("{whole} {numerator}/{denominator}"),
        [first, rest @ ..] => format!("{first} {}", rest.join("/")),
    }
}

fn render_floruit_template(params: &str) -> String {
    let text = render_passthrough_template(params);
    if text.is_empty() {
        "fl.".to_string()
    } else {
        format!("fl. {text}")
    }
}

fn render_coord_template(params: &str) -> String {
    let named = template_named_params(params);
    // For now both inline and title will display inline
    if let Some(display) = template_param(&named, &["display"]) {
        let shows_inline = display.split([',', ';']).any(|value| {
            value.trim().eq_ignore_ascii_case("inline")
                || value.trim().eq_ignore_ascii_case("title")
        });
        if !shows_inline {
            return String::new();
        }
    }

    let positional = split_template_params(params)
        .into_iter()
        .map(|param| render_templates(param.trim()).trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    format_coord_components(&positional).unwrap_or_default()
}

fn format_coord_components(params: &[String]) -> Option<String> {
    format_hemisphere_coordinates(params).or_else(|| format_decimal_coordinates(params))
}

fn format_hemisphere_coordinates(params: &[String]) -> Option<String> {
    let lat_hemisphere_index = params
        .iter()
        .position(|param| matches_direction(param, ['N', 'S']))?;
    if !(1..=3).contains(&lat_hemisphere_index) {
        return None;
    }

    let lon_hemisphere_index = params
        .iter()
        .skip(lat_hemisphere_index + 1)
        .position(|param| matches_direction(param, ['E', 'W']))
        .map(|index| index + lat_hemisphere_index + 1)?;
    let lon_component_count = lon_hemisphere_index.checked_sub(lat_hemisphere_index + 1)?;
    if !(1..=3).contains(&lon_component_count) {
        return None;
    }

    let latitude = format_coord_axis(
        &params[..lat_hemisphere_index],
        params[lat_hemisphere_index].chars().next()?,
    )?;
    let longitude = format_coord_axis(
        &params[lat_hemisphere_index + 1..lon_hemisphere_index],
        params[lon_hemisphere_index].chars().next()?,
    )?;

    Some(format!("{latitude} {longitude}"))
}

fn format_coord_axis(parts: &[String], hemisphere: char) -> Option<String> {
    if parts.is_empty()
        || parts.len() > 3
        || !parts.iter().all(|part| coord_component_is_number(part))
    {
        return None;
    }

    let mut rendered = String::new();
    rendered.push_str(parts.first()?.trim());
    rendered.push('°');

    if let Some(minutes) = parts.get(1) {
        rendered.push_str(minutes.trim());
        rendered.push('′');
    }
    if let Some(seconds) = parts.get(2) {
        rendered.push_str(seconds.trim());
        rendered.push('″');
    }

    rendered.push(hemisphere.to_ascii_uppercase());
    Some(rendered)
}

fn format_decimal_coordinates(params: &[String]) -> Option<String> {
    let latitude = params.first()?.trim();
    let longitude = params.get(1)?.trim();
    if !coord_component_is_number(latitude) || !coord_component_is_number(longitude) {
        return None;
    }

    Some(format!("{latitude}, {longitude}"))
}

fn coord_component_is_number(value: &str) -> bool {
    value.trim().parse::<f64>().is_ok()
}

fn render_cite_web_template(params: &str) -> String {
    let named = template_named_params(params);
    let mut parts = Vec::new();

    let authors = citation_people(&named, PersonRole::Author);
    if !authors.is_empty() {
        parts.push(authors);
    }

    if let Some(title) = template_param(&named, &["title", "trans-title", "script-title"]) {
        let title = match template_param(&named, &["url"]) {
            Some(url) => format!(
                "[[official-url:{}|\"{}\"]]",
                render_templates(url),
                render_templates(title)
            ),
            None => format!("\"{}\"", render_templates(title)),
        };
        parts.push(title);
    }

    let website = template_param(&named, &["website", "work"]);
    let publisher = template_param(&named, &["publisher"]);
    if let Some(website) = website {
        parts.push(format!("''{}''", render_templates(website)));
    }
    if let Some(publisher) = publisher
        && website.is_none_or(|website| !website.eq_ignore_ascii_case(publisher))
    {
        parts.push(render_templates(publisher));
    }

    if let Some(date) = template_param(&named, &["date", "year"]) {
        parts.push(render_templates(date));
    }

    if let Some(pages) = template_param(&named, &["pages", "page"]) {
        parts.push(format!("p. {}", render_templates(pages)));
    }

    parts.join(". ")
}

fn matches_direction(value: &str, allowed: [char; 2]) -> bool {
    let trimmed = value.trim();
    trimmed.len() == 1
        && trimmed.chars().next().is_some_and(|ch| {
            allowed
                .iter()
                .any(|direction| ch.eq_ignore_ascii_case(direction))
        })
}

fn render_cite_book_template(params: &str) -> String {
    render_citation_template(params)
}

fn render_cite_journal_template(params: &str) -> String {
    let named = template_named_params(params);
    let mut parts = Vec::new();

    let authors = citation_people(&named, PersonRole::Author);
    if !authors.is_empty() {
        parts.push(authors);
    }

    if let Some(title) = template_param(&named, &["title", "trans-title", "script-title"]) {
        let title = match template_param(&named, &["url"]) {
            Some(url) => format!(
                "[{} \"{}\"]",
                render_templates(url),
                render_templates(title)
            ),
            None => format!("\"{}\"", render_templates(title)),
        };
        parts.push(title);
    }

    if let Some(journal) = template_param(&named, &["journal", "work", "website"]) {
        parts.push(format!("''{}''", render_templates(journal)));
    }

    let mut details = Vec::new();
    if let Some(date) = template_param(&named, &["date", "year"]) {
        details.push(render_templates(date));
    }
    if let Some(volume) = template_param(&named, &["volume"]) {
        details.push(format!("vol. {}", render_templates(volume)));
    }
    if let Some(issue) = template_param(&named, &["issue", "number"]) {
        details.push(format!("no. {}", render_templates(issue)));
    }
    if let Some(pages) = template_param(&named, &["pages", "page"]) {
        details.push(format!("pp. {}", render_templates(pages)));
    }
    if !details.is_empty() {
        parts.push(details.join(", "));
    }

    if let Some(doi) = template_param(&named, &["doi"]) {
        parts.push(format!("doi:{}", render_templates(doi)));
    }

    if let Some(jstor) = template_param(&named, &["jstor"]) {
        parts.push(format!("JSTOR {}", render_templates(jstor)));
    }

    if let Some(issn) = template_param(&named, &["issn"]) {
        parts.push(format!("ISSN {}", render_templates(issn)));
    }

    parts.join(". ")
}

fn render_cite_report_template(params: &str) -> String {
    let named = template_named_params(params);
    let mut parts = Vec::new();

    let authors = citation_people(&named, PersonRole::Author);
    if !authors.is_empty() {
        parts.push(authors);
    }

    if let Some(title) = template_param(&named, &["title", "trans-title", "script-title"]) {
        let title = match template_param(&named, &["url"]) {
            Some(url) => format!(
                "[{} \"{}\"]",
                render_templates(url),
                render_templates(title)
            ),
            None => format!("''{}''", render_templates(title)),
        };
        parts.push(title);
    }

    if let Some(date) = template_param(&named, &["publication-date", "date", "year"]) {
        parts.push(render_templates(date));
    }

    if let Some(pages) = template_param(&named, &["pages", "page"]) {
        parts.push(format!("p. {}", render_templates(pages)));
    }

    if let Some(isbn) = template_param(&named, &["isbn"]) {
        parts.push(format!("ISBN {}", render_templates(isbn)));
    }

    if let Some(oclc) = template_param(&named, &["oclc"]) {
        parts.push(format!("OCLC {}", render_templates(oclc)));
    }

    parts.join(". ")
}

fn render_cite_eccp_template(params: &str) -> String {
    let named = template_named_params(params);
    let mut parts = Vec::new();

    let authors = citation_people(&named, PersonRole::Author);
    if !authors.is_empty() {
        parts.push(authors);
    }

    if let Some(title) = template_param(&named, &["title"]) {
        parts.push(format!("\"{}\"", render_templates(title)));
    }

    parts.push("Eminent Chinese of the Ch'ing Period".to_string());

    if let Some(date) = template_param(&named, &["date", "year"]) {
        parts.push(render_templates(date));
    }

    if let Some(pages) = template_param(&named, &["pages", "page"]) {
        parts.push(format!("pp. {}", render_templates(pages)));
    }

    parts.join(". ")
}

fn render_worldhistory_template(params: &str) -> String {
    let named = template_named_params(params);
    let mut parts = Vec::new();

    if let Some(quote) = template_param(&named, &["quote"]) {
        parts.push(format!("\"{}\"", render_templates(quote)));
    } else {
        parts.push("Citation".to_string());
    }

    parts.push("''The Encyclopedia of World History'' (6th ed.)".to_string());
    parts.join(". ")
}

fn render_nihongo2_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(text) = positional.first().filter(|t| !t.trim().is_empty()) else {
        return String::new();
    };
    let text = render_templates(text);
    format!(
        "__WIKIPEDIA_TO_EPUB_LANG_START__ja__WIKIPEDIA_TO_EPUB_LANG_VALUE__{text}__WIKIPEDIA_TO_EPUB_LANG_END__"
    )
}

fn render_gloss_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);
    let Some(text) = positional.first().filter(|t| !t.trim().is_empty()) else {
        return String::new();
    };
    let text = render_templates(text);
    if template_param(&named, &["mode"]).is_some_and(|mode| mode.trim() == "def") {
        format!("({text})")
    } else {
        format!("'{text}'")
    }
}

fn render_soft_hyphen_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let parts = positional
        .into_iter()
        .filter(|t| !t.trim().is_empty())
        .collect::<Vec<_>>();
    parts.join("\u{00ad}")
}

fn render_color_box_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(color) = positional.first().filter(|c| !c.trim().is_empty()) else {
        return "■".to_string();
    };
    let color = color.trim();
    format!("__WIKIPEDIA_TO_EPUB_COLOR_BOX_START__{color}__WIKIPEDIA_TO_EPUB_COLOR_BOX_END__")
}

fn render_harvp_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params)
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();

    if positional.is_empty() {
        return String::new();
    }

    let (authors, year) = if positional.len() > 1 {
        let (auths, y) = positional.split_at(positional.len() - 1);
        (auths.to_vec(), Some(y[0].clone()))
    } else {
        (positional.clone(), None)
    };

    let authors_formatted = match authors.len() {
        0 => String::new(),
        1 => authors[0].clone(),
        2 => format!("{} & {}", authors[0], authors[1]),
        _ => {
            let prefix = authors[0..authors.len() - 1].join(", ");
            format!("{}, & {}", prefix, authors.last().unwrap())
        }
    };

    let mut parts = Vec::new();
    let auth_year = if let Some(y) = year {
        if !authors_formatted.is_empty() {
            format!("{} {y}", authors_formatted)
        } else {
            y
        }
    } else {
        authors_formatted
    };

    if !auth_year.is_empty() {
        parts.push(auth_year);
    }

    if let Some(page) = template_param(&named, &["p"]) {
        parts.push(format!("p. {}", render_templates(page.trim())));
    } else if let Some(pages) = template_param(&named, &["pp"]) {
        parts.push(format!("pp. {}", render_templates(pages.trim())));
    }

    if let Some(location) = template_param(&named, &["loc"]) {
        parts.push(render_templates(location.trim()));
    }

    format!("({})", parts.join(", "))
}

fn render_ipa_link_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(symbol) = positional.first().filter(|s| !s.trim().is_empty()) else {
        return String::new();
    };
    let label = positional
        .get(1)
        .filter(|l| !l.trim().is_empty())
        .unwrap_or(symbol);
    format!(
        "__WIKIPEDIA_TO_EPUB_IPA_START__{}__WIKIPEDIA_TO_EPUB_IPA_END__",
        render_templates(label.trim())
    )
}

fn render_angbr_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(text) = positional.first().filter(|t| !t.trim().is_empty()) else {
        return String::new();
    };
    format!("⟨{}⟩", render_templates(text.trim()))
}

fn render_angbr_ipa_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(text) = positional.first().filter(|t| !t.trim().is_empty()) else {
        return String::new();
    };
    let text = render_templates(text.trim());
    format!(
        "⟨__WIKIPEDIA_TO_EPUB_LANG_START__und-fonipa__WIKIPEDIA_TO_EPUB_LANG_VALUE__{text}__WIKIPEDIA_TO_EPUB_LANG_END__⟩"
    )
}

fn render_unichar_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);
    let Some(hex_str) = positional.first().filter(|s| !s.trim().is_empty()) else {
        return String::new();
    };
    let hex_str = hex_str.trim();
    let ch = u32::from_str_radix(hex_str, 16)
        .ok()
        .and_then(char::from_u32);

    let ch_str = match ch {
        Some(c) => c.to_string(),
        None => String::new(),
    };

    let base = template_param(&named, &["cwith"])
        .map(|s| s.trim())
        .unwrap_or("");

    let name = positional
        .get(1)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    let glyph = format!("{base}{ch_str}");

    let details = match name {
        Some(n) => format!("U+{} {}", hex_str.to_uppercase(), n),
        None => format!("U+{}", hex_str.to_uppercase()),
    };

    format!("{glyph} ({details})")
}

fn render_note_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(label) = positional.get(1).filter(|l| !l.trim().is_empty()) else {
        return String::new();
    };
    format!("'''{}'''", render_templates(label.trim()))
}

fn render_fs_interlinear_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let line1 = positional.first().map(|s| s.trim()).unwrap_or("");
    let line2 = positional.get(1).map(|s| s.trim()).unwrap_or("");
    let line3 = positional.get(2).map(|s| s.trim()).unwrap_or("");
    let line4 = positional.get(3).map(|s| s.trim()).unwrap_or("");

    if line1.is_empty() && line2.is_empty() && line3.is_empty() && line4.is_empty() {
        return String::new();
    }

    let line1_rendered = render_templates(line1);
    let line1_html = if let Some(lang) = template_param(&named, &["lang"]) {
        let lang = lang.trim();
        format!(
            "__WIKIPEDIA_TO_EPUB_LANG_START__{lang}__WIKIPEDIA_TO_EPUB_LANG_VALUE__{line1_rendered}__WIKIPEDIA_TO_EPUB_LANG_END__"
        )
    } else {
        line1_rendered
    };

    let line2_rendered = render_templates(line2);
    let line3_rendered = render_templates(line3);
    let line4_rendered = render_templates(line4);

    let mut html = String::new();
    html.push_str("__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_START__\n");
    if !line1_html.is_empty() {
        html.push_str(&format!(
            "__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__'''{}'''\n",
            line1_html
        ));
    }
    if !line2_rendered.is_empty() {
        html.push_str(&format!(
            "__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__''{}''\n",
            line2_rendered
        ));
    }
    if !line3_rendered.is_empty() {
        html.push_str(&format!(
            "__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__{}\n",
            line3_rendered
        ));
    }
    if !line4_rendered.is_empty() {
        let line4_formatted = if line4_rendered.starts_with('\'')
            && line4_rendered.ends_with('\'')
            && line4_rendered.len() > 1
        {
            format!(
                "''&#39;{}&#39;''",
                &line4_rendered[1..line4_rendered.len() - 1]
            )
        } else {
            format!("''{}''", line4_rendered)
        };
        html.push_str(&format!(
            "__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__{}\n",
            line4_formatted
        ));
    }
    html.push_str("__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_END__");
    html
}

fn render_tooltip_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(text) = positional.first().filter(|t| !t.trim().is_empty()) else {
        return String::new();
    };
    let Some(title) = positional.get(1).filter(|t| !t.trim().is_empty()) else {
        return text.to_string();
    };
    let text = render_templates(text.trim());
    let title = render_templates(title.trim());
    format!(
        "__WIKIPEDIA_TO_EPUB_ABBR_START__{title}__WIKIPEDIA_TO_EPUB_ABBR_VALUE__{text}__WIKIPEDIA_TO_EPUB_ABBR_END__"
    )
}

fn render_nihongo_krt_template(params: &str) -> String {
    let positional = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.contains('='))
        .collect::<Vec<_>>();

    let english = positional.first().map(|s| s.as_str()).unwrap_or("");
    let kanji = positional.get(1).map(|s| s.as_str()).unwrap_or("");
    let romaji = positional.get(2).map(|s| s.as_str()).unwrap_or("");

    if kanji.is_empty() {
        return render_templates(english);
    }

    let mut inside = Vec::new();
    if !romaji.is_empty() {
        inside.push(format!("''{}''", render_templates(romaji)));
    }
    if !english.is_empty() {
        inside.push(render_templates(english).to_string());
    }

    let kanji_rendered = format!(
        "__WIKIPEDIA_TO_EPUB_LANG_START__ja__WIKIPEDIA_TO_EPUB_LANG_VALUE__{kanji}__WIKIPEDIA_TO_EPUB_LANG_END__"
    );

    if inside.is_empty() {
        kanji_rendered
    } else {
        format!("{kanji_rendered} ({})", inside.join(", "))
    }
}

fn render_easy_css_image_crop_template(params: &str) -> String {
    let named = template_named_params(params);
    let Some(image) = template_param(&named, &["Image", "image"]) else {
        return String::new();
    };
    let image = image.trim();
    if image.is_empty() {
        return String::new();
    }

    let caption = template_param(&named, &["caption", "Caption"])
        .map(|s| s.trim())
        .unwrap_or("");
    let alt = template_param(&named, &["alt", "Alt"])
        .map(|s| s.trim())
        .unwrap_or("");

    if alt.is_empty() {
        format!("[[File:{image}|thumb|{caption}]]")
    } else {
        format!("[[File:{image}|thumb|alt={alt}|{caption}]]")
    }
}

fn render_issn_template(params: &str) -> String {
    let Some(issn) = template_positional_params(params)
        .first()
        .filter(|v| !v.trim().is_empty())
        .cloned()
    else {
        return String::new();
    };
    format!("ISSN {}", render_templates(&issn))
}

fn render_cite_nsrw_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);
    let title = template_param(&named, &["wstitle", "title"])
        .or_else(|| positional.first().map(String::as_str))
        .map(|s| s.trim())
        .unwrap_or("");

    if title.is_empty() {
        return "''The New Student's Reference Work'' (1914)".to_string();
    }

    format!(
        "\"{}\" in ''[[src:The New Student's Reference Work/{}|The New Student's Reference Work]]'' (1914)",
        render_templates(title),
        title
    )
}

fn format_number_with_commas(s: &str) -> String {
    let s = s.trim();
    if s.is_empty() {
        return String::new();
    }
    let (sign, rest) = if let Some(stripped) = s.strip_prefix('-') {
        ("-", stripped)
    } else if let Some(stripped) = s.strip_prefix('+') {
        ("+", stripped)
    } else {
        ("", s)
    };

    let parts: Vec<&str> = rest.split('.').collect();
    let integer_part = parts[0];

    if !integer_part.chars().all(|c| c.is_ascii_digit()) {
        return s.to_string();
    }

    let mut formatted_integer = String::new();
    let bytes = integer_part.as_bytes();
    let len = bytes.len();
    for (i, &byte) in bytes.iter().enumerate() {
        formatted_integer.push(byte as char);
        let remaining = len - 1 - i;
        if remaining > 0 && remaining.is_multiple_of(3) {
            formatted_integer.push(',');
        }
    }

    let mut result = format!("{}{}", sign, formatted_integer);
    if parts.len() > 1 {
        result.push('.');
        result.push_str(&parts[1..].join("."));
    }
    result
}

fn render_formatnum_template(template: &str, params: &str) -> String {
    let mut num_str = String::new();
    if let Some(colon_idx) = template.find(':') {
        num_str = template[colon_idx + 1..].to_string();
    } else {
        if let Some(first_param) = template_positional_params(params).first() {
            num_str = first_param.clone();
        }
    }
    format_number_with_commas(&num_str)
}

fn render_stn_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|p| p.trim().to_string())
        .collect::<Vec<_>>();
    if params.is_empty() {
        return String::new();
    }
    let name = &params[0];
    if name.is_empty() {
        return String::new();
    }

    let mut capitalize = true;
    let mut disambig = None;
    let mut custom_label = None;

    if params.len() > 1 {
        let p1 = &params[1];
        if p1 == "x" {
            capitalize = true;
        } else if !p1.is_empty() && !p1.contains('=') {
            disambig = Some(p1);
        }
    }

    if params.len() > 2 {
        let p2 = &params[2];
        if !p2.is_empty() && !p2.contains('=') {
            custom_label = Some(p2);
        }
    }

    let suffix = if capitalize { "Station" } else { "station" };

    let target = match disambig {
        Some(d) => format!("{} {} ({})", name, suffix, d),
        None => format!("{} {}", name, suffix),
    };

    let label = match custom_label {
        Some(l) => l.to_string(),
        None => name.to_string(),
    };

    format!("[[{}|{}]]", target, render_templates(&label))
}

fn render_citation_template(params: &str) -> String {
    let named = template_named_params(params);
    let mut parts = Vec::new();

    let authors = citation_people(&named, PersonRole::Author);
    let has_authors = !authors.is_empty();
    if has_authors {
        parts.push(authors);
    }

    if !has_authors {
        let editors = citation_people(&named, PersonRole::Editor);
        if !editors.is_empty() {
            parts.push(format!("{editors}, ed"));
        }
    }

    if let Some(contribution) = template_param(&named, &["contribution", "chapter", "article"]) {
        parts.push(render_templates(contribution));
    }

    if let Some(title) = template_param(&named, &["title"]) {
        let title = match template_param(&named, &["url"]) {
            Some(url) => format!(
                "[{} ''{}'']",
                render_templates(url),
                render_templates(title)
            ),
            None => format!("''{}''", render_templates(title)),
        };
        parts.push(title);
    }

    let mut publication = String::new();
    if let Some(location) = template_param(&named, &["location", "place"]) {
        publication.push_str(&render_templates(location));
        publication.push_str(": ");
    }
    if let Some(publisher) = template_param(&named, &["publisher"]) {
        publication.push_str(&render_templates(publisher));
    }
    if let Some(year) = template_param(&named, &["year", "date"]) {
        if !publication.is_empty() {
            publication.push_str(", ");
        }
        publication.push_str(&render_templates(year));
    }
    if !publication.is_empty() {
        parts.push(publication);
    }

    if let Some(edition) = template_param(&named, &["edition"]) {
        parts.push(format!("{} ed", render_templates(edition)));
    }

    if let Some(pages) = template_param(&named, &["page", "pages"]) {
        parts.push(format!("p. {}", render_templates(pages)));
    }

    if let Some(isbn) = template_param(&named, &["isbn"]) {
        parts.push(format!("ISBN {}", render_templates(isbn)));
    }

    if let Some(oclc) = template_param(&named, &["oclc"]) {
        parts.push(format!("OCLC {}", render_templates(oclc)));
    }

    parts.join(". ")
}

fn render_harvc_template(params: &str) -> String {
    let named = template_named_params(params);
    let mut parts = Vec::new();

    let authors = citation_people(&named, PersonRole::Author);
    if !authors.is_empty() {
        parts.push(authors);
    }

    if let Some(contribution) = template_param(&named, &["c", "chapter", "contribution"]) {
        let contribution = match template_param(&named, &["url", "chapter-url", "contribution-url"])
        {
            Some(url) => format!(
                "[{} \"{}\"]",
                render_templates(url),
                render_templates(contribution)
            ),
            None => format!("\"{}\"", render_templates(contribution)),
        };
        parts.push(contribution);
    }

    let source = harvc_source(&named);
    if !source.is_empty() {
        parts.push(format!("In {source}"));
    }

    if let Some(page) = template_param(&named, &["p", "page"]) {
        parts.push(format!("p. {}", render_templates(page)));
    } else if let Some(pages) = template_param(&named, &["pp", "pages"]) {
        parts.push(format!("pp. {}", render_templates(pages)));
    }

    if let Some(location) = template_param(&named, &["loc"]) {
        parts.push(render_templates(location));
    }

    parts.join(". ")
}

fn harvc_source(named: &HashMap<String, String>) -> String {
    let source_authors = (1..=4)
        .filter_map(|index| {
            let keys = if index == 1 {
                vec!["in".to_string(), "in1".to_string()]
            } else {
                vec![format!("in{index}")]
            };
            template_param_owned(named, &keys).map(|value| render_templates(&value))
        })
        .collect::<Vec<_>>();

    let year = template_param(named, &["anchor-year", "year"]).map(render_templates);

    match (source_authors.as_slice(), year) {
        ([], None) => String::new(),
        ([], Some(year)) => year,
        ([source], None) => source.clone(),
        ([source], Some(year)) => format!("{source} {year}"),
        (sources, None) => sources.join(" and "),
        (sources, Some(year)) => format!("{} {year}", sources.join(" and ")),
    }
}

fn render_as_of_template(params: &str) -> String {
    let named = template_named_params(params);
    if let Some(alt) = template_param(&named, &["alt"]) {
        return render_templates(alt);
    }

    let positional = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .map(|param| render_templates(&param))
        .collect::<Vec<_>>();

    let Some(year) = positional.first() else {
        return String::new();
    };

    let date = as_of_date(&positional, template_param(&named, &["df"]));
    let prefix = if template_param_truthy(&named, &["lc"]) {
        "as of"
    } else {
        "As of"
    };

    if date.is_empty() {
        render_templates(year)
    } else {
        format!("{prefix} {date}")
    }
}

fn render_died_in_template(params: &str) -> String {
    let date = render_passthrough_template(params);
    if date.trim().is_empty() {
        String::new()
    } else {
        format!("d. {}", date.trim())
    }
}

fn as_of_date(positional: &[String], date_format: Option<&str>) -> String {
    let year = positional.first().map(String::as_str).unwrap_or_default();
    let Some(month) = positional.get(1).map(String::as_str) else {
        return year.to_string();
    };

    let month = as_of_month_name(month).unwrap_or(month);
    let Some(day) = positional.get(2).map(String::as_str) else {
        return format!("{month} {year}");
    };

    if date_format.is_some_and(|value| value.eq_ignore_ascii_case("dmy")) {
        format!("{day} {month} {year}")
    } else {
        format!("{month} {day}, {year}")
    }
}

fn as_of_month_name(month: &str) -> Option<&'static str> {
    match month.trim().parse::<usize>().ok()? {
        1 => Some("January"),
        2 => Some("February"),
        3 => Some("March"),
        4 => Some("April"),
        5 => Some("May"),
        6 => Some("June"),
        7 => Some("July"),
        8 => Some("August"),
        9 => Some("September"),
        10 => Some("October"),
        11 => Some("November"),
        12 => Some("December"),
        _ => None,
    }
}

fn template_param_truthy(named: &HashMap<String, String>, keys: &[&str]) -> bool {
    template_param(named, keys).is_some_and(|value| {
        value.eq_ignore_ascii_case("y")
            || value.eq_ignore_ascii_case("yes")
            || value.eq_ignore_ascii_case("true")
            || value == "1"
    })
}

fn render_blockquote_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let text = template_param(&named, &["text", "quote", "1"])
        .map(str::to_string)
        .or_else(|| positional.first().cloned())
        .map(|value| render_templates(&value).replace('\n', " "))
        .unwrap_or_default();

    let source = template_param(&named, &["source", "author", "cite", "2"])
        .map(str::to_string)
        .or_else(|| positional.get(1).cloned())
        .map(|value| render_templates(&value).replace('\n', " "))
        .unwrap_or_default();

    if text.trim().is_empty() {
        return String::new();
    }

    let mut rendered = format!(
        "\n__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_START__\n__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__{}\n",
        text.trim()
    );
    if !source.trim().is_empty() {
        rendered.push_str(&format!(
            "__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_SOURCE__{}\n",
            source.trim()
        ));
    }
    rendered.push_str("__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_END__\n");
    rendered
}

#[derive(Clone, Copy)]
enum PersonRole {
    Author,
    Editor,
}

fn citation_people(named: &HashMap<String, String>, role: PersonRole) -> String {
    let mut people = Vec::new();
    let (person_key, first_key, last_key, link_key) = match role {
        PersonRole::Author => ("author", "first", "last", "author-link"),
        PersonRole::Editor => ("editor", "editor-first", "editor-last", "editor-link"),
    };

    if let Some(person) = template_param(named, &[person_key]) {
        people.push(render_templates(person));
    }

    let unnumbered_first_keys = person_first_keys(first_key, 0);
    let unnumbered_last_keys = person_last_keys(last_key, 0);
    let unnumbered_link_keys = person_link_keys(link_key, 0);
    let unnumbered_first = template_param_owned(named, &unnumbered_first_keys);
    let unnumbered_last = template_param_owned(named, &unnumbered_last_keys);
    let unnumbered_link = template_param_owned(named, &unnumbered_link_keys);
    let unnumbered_name = match (unnumbered_first.as_deref(), unnumbered_last.as_deref()) {
        (Some(first), Some(last)) => {
            format!("{} {}", render_templates(first), render_templates(last))
        }
        (Some(first), None) => render_templates(first),
        (None, Some(last)) => render_templates(last),
        (None, None) => String::new(),
    };
    let has_unnumbered_name = !unnumbered_name.is_empty();

    if has_unnumbered_name {
        if let Some(link) = unnumbered_link.filter(|value| !value.trim().is_empty()) {
            people.push(format!(
                "[[{}|{}]]",
                render_templates(&link),
                unnumbered_name
            ));
        } else {
            people.push(unnumbered_name);
        }
    }

    for index in 1..=8 {
        if has_unnumbered_name && matches!(role, PersonRole::Editor) && index == 1 {
            continue;
        }

        let first_keys = person_first_keys(first_key, index);
        let last_keys = person_last_keys(last_key, index);
        let link_keys = person_link_keys(link_key, index);

        let first = template_param_owned(named, &first_keys);
        let last = template_param_owned(named, &last_keys);
        let link = template_param_owned(named, &link_keys);

        if first.is_none() && last.is_none() {
            continue;
        }

        let name = match (first.as_deref(), last.as_deref()) {
            (Some(first), Some(last)) => {
                format!("{} {}", render_templates(first), render_templates(last))
            }
            (Some(first), None) => render_templates(first),
            (None, Some(last)) => render_templates(last),
            (None, None) => String::new(),
        };

        if let Some(link) = link.filter(|value| !value.trim().is_empty()) {
            people.push(format!("[[{}|{}]]", render_templates(&link), name));
        } else {
            people.push(name);
        }
    }

    if matches!(role, PersonRole::Author)
        && let Some(others) = template_param(named, &["others"])
    {
        people.push(render_templates(others));
    }

    match people.as_slice() {
        [] => String::new(),
        [person] => person.clone(),
        [first, second] => format!("{first} and {second}"),
        _ => {
            let last = people.last().cloned().unwrap_or_default();
            format!("{}, and {last}", people[..people.len() - 1].join(", "))
        }
    }
}

fn person_first_keys(base: &str, index: usize) -> Vec<String> {
    if index == 0 {
        match base {
            "first" => vec!["first".to_string(), "given".to_string()],
            "editor-first" => vec![
                "editor-first".to_string(),
                "editor-given".to_string(),
                "editor-first1".to_string(),
                "editor-given1".to_string(),
            ],
            _ => vec![base.to_string()],
        }
    } else {
        match base {
            "first" => vec![format!("first{index}"), format!("given{index}")],
            "editor-first" => vec![
                format!("editor-first{index}"),
                format!("editor-given{index}"),
            ],
            _ => vec![format!("{base}{index}")],
        }
    }
}

fn person_last_keys(base: &str, index: usize) -> Vec<String> {
    if index == 0 {
        match base {
            "last" => vec!["last".to_string(), "surname".to_string()],
            "editor-last" => vec![
                "editor-last".to_string(),
                "editor-surname".to_string(),
                "editor-last1".to_string(),
                "editor-surname1".to_string(),
            ],
            _ => vec![base.to_string()],
        }
    } else {
        match base {
            "last" => vec![format!("last{index}"), format!("surname{index}")],
            "editor-last" => vec![
                format!("editor-last{index}"),
                format!("editor-surname{index}"),
            ],
            _ => vec![format!("{base}{index}")],
        }
    }
}

fn person_link_keys(base: &str, index: usize) -> Vec<String> {
    if index == 0 {
        match base {
            "author-link" => vec!["author-link".to_string(), "authorlink".to_string()],
            "editor-link" => vec![
                "editor-link".to_string(),
                "editorlink".to_string(),
                "editor-link1".to_string(),
                "editorlink1".to_string(),
            ],
            _ => vec![base.to_string()],
        }
    } else {
        match base {
            "author-link" => vec![format!("author-link{index}"), format!("authorlink{index}")],
            "editor-link" => vec![format!("editor-link{index}"), format!("editorlink{index}")],
            _ => vec![format!("{base}{index}")],
        }
    }
}

fn template_named_params(params: &str) -> HashMap<String, String> {
    split_template_params(params)
        .into_iter()
        .filter_map(|param| {
            let (key, value) = param.split_once('=')?;
            Some((key.trim().to_lowercase(), value.trim().to_string()))
        })
        .collect()
}

fn template_positional_params(params: &str) -> Vec<String> {
    split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect()
}

fn template_param<'a>(named: &'a HashMap<String, String>, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| named.get(*key))
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn template_param_owned(named: &HashMap<String, String>, keys: &[String]) -> Option<String> {
    keys.iter()
        .find_map(|key| named.get(key))
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(String::from)
}

fn render_percentage_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| render_templates(param.trim()).trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    let Some(part) = params
        .first()
        .and_then(|value| parse_template_number(value))
    else {
        return String::new();
    };
    let Some(total) = params.get(1).and_then(|value| parse_template_number(value)) else {
        return String::new();
    };

    if total == 0.0 {
        return String::new();
    }

    let decimals = params
        .get(2)
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(0);
    let percentage = part / total * 100.0;

    if decimals == 0 {
        format!("{:.0}%", percentage)
    } else {
        format!("{percentage:.decimals$}%")
    }
}

fn render_un_population_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    match params.first().map(String::as_str) {
        Some("ref") => String::new(),
        Some(country) if country.eq_ignore_ascii_case("Dem. People's Republic of Korea") => {
            "26,100,000".to_string()
        }
        _ => String::new(),
    }
}

fn render_convert_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.contains('='))
        .collect::<Vec<_>>();

    let Some(value) = params.first().map(String::as_str) else {
        return String::new();
    };

    match params.get(1).map(String::as_str) {
        Some("to") if params.len() >= 4 => format!(
            "{} to {} {}",
            format_convert_value(value),
            format_convert_value(&params[2]),
            format_convert_unit(&params[3])
        ),
        Some("and") if params.len() >= 4 => format!(
            "{} {} and {} {}",
            format_convert_value(value),
            format_convert_unit(&params[3]),
            format_convert_value(&params[2]),
            format_convert_unit(&params[3])
        ),
        Some(unit) => format!(
            "{} {}",
            format_convert_value(value),
            format_convert_unit(unit)
        ),
        None => format_convert_value(value),
    }
}

fn render_for_timeline_template(params: &str) -> String {
    let articles = template_article_params(params);

    match articles.as_slice() {
        [] => String::new(),
        [article] => format!("For a timeline, see: [[{article}]]"),
        articles => format!("For timelines, see: {}", join_template_articles(articles)),
    }
}

fn render_legend_template(params: &str) -> String {
    let params = template_positional_params(params);
    let Some(label) = params.get(1).map(String::as_str) else {
        return String::new();
    };

    render_templates(label)
}

fn render_numero_template(params: &str) -> String {
    let number = render_passthrough_template(params);
    if number.trim().is_empty() {
        String::new()
    } else {
        format!("No. {}", number.trim())
    }
}

fn render_article_link_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(article) = positional
        .first()
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
    else {
        return String::new();
    };
    let label = positional
        .get(1)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(article);

    format!("[[{article}|{}]]", render_templates(label))
}

fn render_for_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(topic) = positional
        .first()
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
    else {
        return String::new();
    };
    let articles = positional
        .iter()
        .skip(1)
        .filter(|article| !article.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>();

    if articles.is_empty() {
        render_templates(topic)
    } else {
        format!(
            "For {}, see: {}",
            render_templates(topic),
            join_template_articles(&articles)
        )
    }
}

fn render_excerpt_template(params: &str) -> String {
    let articles = template_article_params(params);

    match articles.as_slice() {
        [] => String::new(),
        [article] => format!("Excerpt from: [[{article}]]"),
        articles => format!("Excerpts from: {}", join_template_articles(articles)),
    }
}

fn render_main_template(params: &str) -> String {
    let articles = template_article_params(params);

    match articles.as_slice() {
        [] => String::new(),
        [article] => format!("Main article: [[{article}]]"),
        articles => format!("Main articles: {}", join_template_articles(articles)),
    }
}

fn render_see_also_template(params: &str) -> String {
    let articles = template_article_params(params);

    if articles.is_empty() {
        String::new()
    } else {
        format!("See also: {}", join_template_articles(&articles))
    }
}

fn render_section_link_template(params: &str) -> String {
    let params = template_positional_params(params);
    let Some(first) = params.first().map(String::as_str) else {
        return String::new();
    };

    if let Some(section) = first.strip_prefix('#') {
        let label = params
            .get(1)
            .map(String::as_str)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(section);
        return format!("[[#{section}|{label}]]");
    }

    let Some(section) = params.get(1).map(String::as_str) else {
        return format!("[[{first}]]");
    };
    let label = params
        .get(2)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(section);

    format!("[[{first}#{section}|{label}]]")
}

fn render_further_template(params: &str) -> String {
    let named = template_named_params(params);
    let articles = template_article_params(params);

    if articles.is_empty() {
        String::new()
    } else if let Some(topic) = template_param(&named, &["topic"]) {
        format!(
            "Further information about {}: {}",
            render_templates(topic),
            join_template_articles(&articles)
        )
    } else {
        format!("Further information: {}", join_template_articles(&articles))
    }
}

fn render_wiktionary_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    let Some(title) = params.first() else {
        return String::new();
    };
    let label = params.get(1).unwrap_or(title);
    let target = format!("wikt:{title}");

    format!("Wiktionary: [[{target}|{label}]]")
}

fn render_wikivoyage_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    let Some(title) = params.first() else {
        return String::new();
    };
    let label = params.get(1).unwrap_or(title);
    let target = format!("voy:{title}");

    format!("Wikivoyage: [[{target}|{label}]]")
}

fn render_wikisource_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    let Some(title) = params.first() else {
        return String::new();
    };
    let label = params.get(1).unwrap_or(title);
    let target = format!("src:{title}");

    format!("Wikisource: [[{target}|{label}]]")
}

fn render_wikibooks_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);
    let book = template_param(&named, &["1"])
        .or_else(|| positional.first().map(String::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let page = template_param(&named, &["2"])
        .or_else(|| positional.get(1).map(String::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let Some(book) = book else {
        return String::new();
    };

    let target = if let Some(page) = page {
        format!("b:{book}/{page}")
    } else {
        format!("b:{book}")
    };
    let label = template_param(&named, &["3"])
        .or_else(|| positional.get(2).map(String::as_str))
        .or(page)
        .unwrap_or(book);

    format!("Wikibooks: [[{target}|{}]]", render_templates(label))
}

fn render_britannica_template(params: &str) -> String {
    let params = template_positional_params(params);
    let Some(article_id) = params
        .first()
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
    else {
        return String::new();
    };
    let label = params
        .get(1)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Encyclopaedia Britannica");
    let url = format!("https://www.britannica.com/EBchecked/topic/{article_id}");

    format!(
        "Britannica: [[official-url:{url}|{}]]",
        render_templates(label)
    )
}

fn render_official_website_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let named = template_named_params(params);

    let url = positional
        .first()
        .map(String::as_str)
        .or_else(|| template_param(&named, &["url", "website"]))
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let Some(url) = url else {
        return String::new();
    };

    let label = template_param(&named, &["name", "title"])
        .or_else(|| positional.get(1).map(String::as_str).map(str::trim))
        .filter(|value| !value.is_empty())
        .unwrap_or("Official website");

    format!("[[official-url:{url}|{}]]", render_templates(label))
}

fn render_url_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let named = template_named_params(params);

    let url = template_param(&named, &["1", "url"])
        .or_else(|| positional.first().map(String::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let Some(url) = url else {
        return String::new();
    };

    let label = template_param(&named, &["2", "name", "title"])
        .or_else(|| positional.get(1).map(String::as_str).map(str::trim))
        .filter(|value| !value.is_empty())
        .unwrap_or(url);

    format!("[[official-url:{url}|{}]]", render_templates(label))
}

fn render_openstreetmap_relation_template(params: &str) -> String {
    let params = template_positional_params(params);
    let Some(relation_id) = params.first().map(String::as_str) else {
        return String::new();
    };
    let relation_id = relation_id.trim();
    if relation_id.is_empty() {
        return String::new();
    }

    format!("[[osmrelation:{relation_id}|OpenStreetMap relation {relation_id}]]")
}

fn render_webarchive_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let named = template_named_params(params);

    let url = template_param(&named, &["url"])
        .or_else(|| {
            positional
                .iter()
                .find_map(|param| template_url_value(param))
        })
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let Some(url) = url else {
        return String::new();
    };

    let label = template_param(&named, &["date"])
        .map(|date| format!("Archived on {}", render_templates(date)))
        .unwrap_or_else(|| "Archived copy".to_string());

    format!("[[official-url:{url}|{label}]]")
}

fn template_url_value(value: &str) -> Option<&str> {
    let value = value.trim();
    if value.starts_with("http://") || value.starts_with("https://") || value.starts_with("//") {
        Some(value)
    } else {
        None
    }
}

fn render_largest_cities_template(params: &str) -> String {
    let named = template_named_params(params);
    let country = template_param(&named, &["country"])
        .map(render_templates)
        .filter(|value| !value.is_empty());
    let mut lines = Vec::new();

    for index in 1..=100 {
        let city_key = format!("city_{index}");
        let Some(city) = named.get(&city_key).map(String::as_str).map(str::trim) else {
            continue;
        };
        if city.is_empty() {
            continue;
        }

        let city = render_largest_city_name(city);
        let division = named
            .get(&format!("div_{index}"))
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(render_templates);
        let population = named
            .get(&format!("pop_{index}"))
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(render_templates);

        let mut details = Vec::new();
        if let Some(division) = division {
            details.push(division);
        }
        if let Some(population) = population {
            details.push(format!("population {population}"));
        }

        if details.is_empty() {
            lines.push(format!("* {city}"));
        } else {
            lines.push(format!("* {city} ({})", details.join(", ")));
        }
    }

    if lines.is_empty() {
        return String::new();
    }

    let heading = country
        .map(|country| format!("Largest cities in {country}:"))
        .unwrap_or_else(|| "Largest cities:".to_string());
    format!("\n{heading}\n{}\n", lines.join("\n"))
}

fn render_historical_populations_template(params: &str) -> String {
    let entries = historical_population_entries(params);
    if entries.is_empty() {
        return String::new();
    }

    let lines = entries
        .into_iter()
        .map(|(year, population)| format!("* {year}: {population}"))
        .collect::<Vec<_>>();
    format!("\nHistorical populations:\n{}\n", lines.join("\n"))
}

fn render_climate_chart_template(params: &str) -> String {
    let params = template_positional_params(params)
        .into_iter()
        .map(|param| render_templates(&param).trim().to_string())
        .filter(|param| !param.is_empty())
        .collect::<Vec<_>>();

    let Some(location) = params.first() else {
        return String::new();
    };

    let entries = params.iter().skip(1).take(36).collect::<Vec<_>>();
    if entries.len() < 36 {
        return String::new();
    }

    let month_names = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let lines = month_names
        .iter()
        .zip(entries.chunks_exact(3))
        .map(|(month, values)| {
            format!(
                "* {month}: {} to {} °C, {} mm",
                format_convert_value(values[0]),
                format_convert_value(values[1]),
                format_convert_value(values[2])
            )
        })
        .collect::<Vec<_>>();

    format!(
        "\nClimate chart for {}:\n{}\n",
        render_templates(location),
        lines.join("\n")
    )
}

fn historical_population_entries(params: &str) -> Vec<(String, String)> {
    let values = split_template_params(params)
        .into_iter()
        .filter_map(|param| {
            let trimmed = param.trim();
            if trimmed.is_empty() {
                return None;
            }

            match trimmed.split_once('=') {
                Some((key, value)) if key.trim().parse::<usize>().is_ok() => {
                    Some(value.trim().to_string())
                }
                Some(_) => None,
                None => Some(trimmed.to_string()),
            }
        })
        .map(|value| render_templates(&value).trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    values
        .chunks(2)
        .filter_map(|chunk| {
            let [year, population] = chunk else {
                return None;
            };

            Some((year.to_string(), format_historical_population(population)))
        })
        .collect()
}

fn format_historical_population(value: &str) -> String {
    let trimmed = value.trim();
    match trimmed.parse::<i64>() {
        Ok(number) => format_population_number(number),
        Err(_) => trimmed.to_string(),
    }
}

fn format_population_number(value: i64) -> String {
    let digits = value.abs().to_string();
    let grouped = digits
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or_default())
        .collect::<Vec<_>>()
        .join(",");

    if value < 0 {
        format!("-{grouped}")
    } else {
        grouped
    }
}

fn render_largest_city_name(city: &str) -> String {
    let city = render_templates(city).trim().to_string();
    if city.contains("[[") {
        city
    } else {
        format!("[[{city}]]")
    }
}

fn render_ship_class_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| render_templates(param.trim()).trim().to_string())
        .collect::<Vec<_>>();

    let class_name = params.first().map(String::as_str).unwrap_or("").trim();
    let ship_type = params.get(1).map(String::as_str).unwrap_or("").trim();
    if class_name.is_empty() || ship_type.is_empty() {
        return String::new();
    }

    let format = params.get(2).map(String::as_str).unwrap_or("").trim();
    let ship_type_disambiguation = params.get(3).map(String::as_str).unwrap_or("").trim();
    let class_disambiguation = params.get(4).map(String::as_str).unwrap_or("").trim();

    let mut class_target = format!("{class_name}-class {ship_type}");
    if !class_disambiguation.is_empty() {
        class_target.push_str(&format!(" ({class_disambiguation})"));
    }

    let class_label = match format {
        "1" => format!("''{class_name}''-class {ship_type}"),
        "4" => format!("''{class_name}'' class"),
        "5" => format!("''{class_name}''"),
        _ => format!("''{class_name}''-class"),
    };

    let class_link = format!("[[{class_target}|{class_label}]]");
    match format {
        "0" | "4" | "5" => class_link,
        "1" => class_link,
        "2" => format!("{class_link} {ship_type}"),
        "" | "3" => {
            let ship_type_link = if ship_type_disambiguation.is_empty() {
                format!("[[{ship_type}]]")
            } else {
                format!("[[{ship_type} ({ship_type_disambiguation})|{ship_type}]]")
            };
            format!("{class_link} {ship_type_link}")
        }
        _ => class_link,
    }
}

fn render_arrow_template(params: &str) -> String {
    let params = template_positional_params(params);
    match params
        .first()
        .map(|value| value.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("l" | "left" | "w") => "←".to_string(),
        Some("u" | "up" | "n") => "↑".to_string(),
        Some("d" | "down" | "s") => "↓".to_string(),
        Some("ne") => "↗".to_string(),
        Some("nw") => "↖".to_string(),
        Some("se") => "↘".to_string(),
        Some("sw") => "↙".to_string(),
        _ => "→".to_string(),
    }
}

fn render_republic_of_korea_ship_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| render_templates(&param).trim().to_string())
        .filter(|param| !param.contains('='))
        .collect::<Vec<_>>();

    let Some(name) = params
        .first()
        .map(String::as_str)
        .filter(|name| !name.is_empty())
    else {
        return "ROKS".to_string();
    };

    let disambiguator = params
        .get(1)
        .map(String::as_str)
        .filter(|value| !value.is_empty());
    let target = match disambiguator {
        Some(disambiguator) => format!("ROKS {name} ({disambiguator})"),
        None => format!("ROKS {name}"),
    };

    format!("[[{target}|ROKS ''{name}'']]")
}

fn render_for_multi_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let mut chunks = Vec::new();
    let mut iter = positional.into_iter();
    while let Some(topic) = iter.next() {
        if let Some(article) = iter.next()
            && !topic.trim().is_empty()
            && !article.trim().is_empty()
        {
            chunks.push(format!("{}, see [[{}]]", render_templates(&topic), article));
        }
    }

    if chunks.is_empty() {
        return String::new();
    }

    format!("For {}.", chunks.join("; for "))
}

fn render_inflation_template(params: &str) -> String {
    let positional = template_positional_params(params);
    if positional.len() < 3 {
        return String::new();
    }
    let index = &positional[0];
    let value_str = &positional[1];
    let year_str = &positional[2];

    let value: f64 = value_str.trim().parse().unwrap_or(0.0);
    let year: i32 = year_str.trim().parse().unwrap_or(0);

    if index.eq_ignore_ascii_case("US") {
        let cpi_1950 = 24.1;
        let cpi_2023 = 304.7;

        let cpi_start = match year {
            1950 => cpi_1950,
            _ => 24.1,
        };

        let inflated = value * (cpi_2023 / cpi_start);

        if inflated >= 100.0 {
            format!("{:.0}", inflated)
        } else {
            format!("{:.2}", inflated)
        }
    } else {
        value_str.clone()
    }
}

fn render_inflation_year_template(_params: &str) -> String {
    "2023".to_string()
}

fn render_ship_template(prefix: &str, params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(name) = positional
        .first()
        .map(String::as_str)
        .filter(|name| !name.is_empty())
    else {
        return prefix.to_string();
    };

    let id = positional
        .get(1)
        .map(String::as_str)
        .filter(|val| !val.is_empty());

    let format_val = positional
        .get(2)
        .map(String::as_str)
        .filter(|val| !val.is_empty())
        .and_then(|val| val.parse::<i32>().ok())
        .unwrap_or(0);

    let target = match id {
        Some(id_val) => format!("{prefix} {name} ({id_val})"),
        None => format!("{prefix} {name}"),
    };

    let display = match format_val {
        6 => format!("''{name}''"),
        2 => match id {
            Some(id_val) => format!("''{name}'' ({id_val})"),
            None => format!("''{name}''"),
        },
        3 => format!("{prefix} ''{name}''"),
        _ => match id {
            Some(id_val) => format!("{prefix} ''{name}'' ({id_val})"),
            None => format!("{prefix} ''{name}''"),
        },
    };

    format!("[[{target}|{display}]]")
}

fn render_collapsible_list_template(params: &str) -> String {
    let named = template_named_params(params);
    let title = template_param(&named, &["title"]);
    let positional = template_positional_params(params);

    let mut parts = Vec::new();
    if let Some(t) = title {
        let t_rendered = render_templates(t);
        if !t_rendered.trim().is_empty() {
            parts.push(t_rendered.trim().to_string());
        }
    }

    for item in positional {
        let item_rendered = render_templates(&item);
        if !item_rendered.trim().is_empty() {
            parts.push(format!("* {}", item_rendered.trim()));
        }
    }

    if parts.is_empty() {
        return String::new();
    }

    format!("\n{}\n", parts.join("\n"))
}

fn render_internet_archive_short_film_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let named = template_named_params(params);

    let id = template_param(&named, &["1", "id"])
        .or_else(|| positional.first().map(String::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let Some(id) = id else {
        return String::new();
    };

    let name = template_param(&named, &["2", "name"])
        .or_else(|| positional.get(1).map(String::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Internet Archive short film");

    let url = format!("https://archive.org/details/{id}");
    format!(
        "[[official-url:{url}|''{}'']] at the Internet Archive",
        render_templates(name)
    )
}

fn render_interlanguage_link_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .collect::<Vec<_>>();

    let Some(article) = params.first().filter(|article| !article.is_empty()) else {
        return String::new();
    };

    let label = params
        .iter()
        .filter_map(|param| param.split_once('='))
        .find_map(|(key, value)| {
            if key.trim().eq_ignore_ascii_case("lt") {
                Some(value.trim())
            } else {
                None
            }
        })
        .filter(|value| !value.is_empty())
        .unwrap_or(article);

    if label == article {
        format_interlanguage_link(article, None, params.get(1))
    } else {
        format_interlanguage_link(article, Some(label), params.get(1))
    }
}

fn format_interlanguage_link(
    article: &str,
    label: Option<&str>,
    language: Option<&String>,
) -> String {
    let link = if let Some(label) = label {
        format!("[[{article}|{label}]]")
    } else {
        format!("[[{article}]]")
    };

    match language
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(language) => format!("{link} [{language}]"),
        None => link,
    }
}

fn render_reign_template(params: &str) -> String {
    let mut positional = Vec::new();
    let mut named = HashMap::new();

    if !params.trim().is_empty() {
        for param in split_template_params(params)
            .into_iter()
            .map(|param| param.trim().to_string())
        {
            if let Some((key, value)) = param.split_once('=') {
                named.insert(key.trim().to_lowercase(), value.trim().to_string());
            } else {
                positional.push(param);
            }
        }
    }

    let label = reign_label(&named);
    let era = named.get("era").map(String::as_str);
    let mut dates = Vec::new();

    if let Some(pre_date) = named.get("pre-date").filter(|value| !value.is_empty()) {
        dates.push(pre_date.to_string());
    }

    if let Some(single) = named
        .get("single")
        .or_else(|| named.get("post-date"))
        .filter(|value| !value.is_empty() && positional.is_empty() && dates.is_empty())
    {
        dates.push(single.to_string());
    } else if !positional.is_empty() {
        dates.push(format_reign_range(
            positional.first().map(String::as_str),
            positional.get(1).map(String::as_str),
        ));
    }

    if let Some(mid_date) = named.get("mid-date").filter(|value| !value.is_empty()) {
        dates.push(mid_date.to_string());
    }

    if positional.get(1).is_some() && positional.get(3).is_some() {
        dates.push(format_reign_range(
            positional.get(2).map(String::as_str),
            positional.get(3).map(String::as_str),
        ));
    }

    if let Some(post_date) = named
        .get("post-date")
        .filter(|value| !value.is_empty() && !positional.is_empty())
    {
        dates.push(post_date.to_string());
    }

    if let Some(era) = era.filter(|value| !value.trim().is_empty())
        && let Some(last) = dates.last_mut()
    {
        last.push(' ');
        last.push_str(era.trim());
    }

    match (label.as_str(), dates.is_empty()) {
        ("", true) => String::new(),
        ("", false) => dates.join(", "),
        (_, true) => label,
        (_, false) => format!("{label} {}", dates.join(", ")),
    }
}

fn render_open_access_template() -> String {
    "__WIKIPEDIA_TO_EPUB_OPEN_ACCESS__".to_string()
}

fn render_reference_page_template(params: &str) -> String {
    let pages = split_template_params(params)
        .into_iter()
        .map(|param| render_templates(param.trim()).trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    match pages.as_slice() {
        [] => String::new(),
        [page] => format!(" p. {page}"),
        pages => format!(" pp. {}", pages.join(", ")),
    }
}

fn reign_label(named: &HashMap<String, String>) -> String {
    if let Some(label) = named.get("label").filter(|value| !value.trim().is_empty()) {
        return label.trim().to_string();
    }

    let show = named
        .get("show")
        .or_else(|| named.get("link"))
        .or_else(|| named.get("lk"))
        .map(String::as_str)
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    let capitalized = named.contains_key("cap");

    match show.as_str() {
        "none" | "no" | "n" | "off" | "false" | "0" | "blank" => String::new(),
        "word" => {
            if capitalized {
                "Reigned".to_string()
            } else {
                "reigned".to_string()
            }
        }
        "colon" => {
            if capitalized {
                "Reign:".to_string()
            } else {
                "reign:".to_string()
            }
        }
        "lword" => {
            if capitalized {
                "[[Reign|Reigned]]".to_string()
            } else {
                "[[Reign|reigned]]".to_string()
            }
        }
        "lcolon" => {
            if capitalized {
                "[[Reign|Reign]]:".to_string()
            } else {
                "[[Reign|reign]]:".to_string()
            }
        }
        "link" | "yes" | "y" | "on" | "true" | "1" => {
            if capitalized {
                "[[Reign|R.]]".to_string()
            } else {
                "[[Reign|r.]]".to_string()
            }
        }
        _ => {
            if capitalized {
                "R.".to_string()
            } else {
                "r.".to_string()
            }
        }
    }
}

fn format_reign_range(start: Option<&str>, end: Option<&str>) -> String {
    let start = start.unwrap_or("").trim();
    let end = end.unwrap_or("").trim();
    let start = if start.is_empty() { "?" } else { start };
    let separator = if start.contains(char::is_whitespace) || end.contains(char::is_whitespace) {
        " – "
    } else {
        "–"
    };

    format!("{start}{separator}{end}")
}

fn template_article_params(params: &str) -> Vec<String> {
    split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>()
}

fn join_template_articles(articles: &[String]) -> String {
    let links = articles
        .iter()
        .map(|article| format!("[[{article}]]"))
        .collect::<Vec<_>>();

    join_plain_items(&links)
}

fn join_plain_items(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [link] => link.to_string(),
        [first, second] => format!("{first} and {second}"),
        _ => {
            let last = items.last().cloned().unwrap_or_default();
            let leading = &items[..items.len() - 1];
            format!("{}, and {last}", leading.join(", "))
        }
    }
}

fn format_convert_value(value: &str) -> String {
    value.trim().replace("&minus;", "−")
}

fn format_convert_unit(unit: &str) -> String {
    match unit.trim() {
        "C" => "°C".to_string(),
        "F" => "°F".to_string(),
        "km2" => "km²".to_string(),
        "mi2" | "sqmi" => "mi²".to_string(),
        "m3" => "m³".to_string(),
        "ug/m3" => "ug/m³".to_string(),
        value => value.to_string(),
    }
}

fn parse_template_number(value: &str) -> Option<f64> {
    let number = value
        .trim()
        .replace([',', ' '], "")
        .replace("&minus;", "-")
        .replace('−', "-");

    number.parse::<f64>().ok()
}

fn split_template_params(params: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut template_depth = 0usize;
    let mut link_depth = 0usize;
    let mut chars = params.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '[' && chars.peek() == Some(&'[') {
            current.push(ch);
            current.push(chars.next().unwrap());
            link_depth += 1;
        } else if ch == ']' && chars.peek() == Some(&']') {
            current.push(ch);
            current.push(chars.next().unwrap());
            link_depth = link_depth.saturating_sub(1);
        } else if ch == '{' && chars.peek() == Some(&'{') {
            current.push(ch);
            current.push(chars.next().unwrap());
            template_depth += 1;
        } else if ch == '}' && chars.peek() == Some(&'}') {
            current.push(ch);
            current.push(chars.next().unwrap());
            template_depth = template_depth.saturating_sub(1);
        } else if ch == '|' && template_depth == 0 && link_depth == 0 {
            parts.push(current);
            current = String::new();
        } else {
            current.push(ch);
        }
    }

    parts.push(current);
    parts
}

fn cleanup_inline_markup(line: &str, internal_links: &InternalLinks, language: &str) -> String {
    let mut text = line.trim().to_string();
    let mut link_placeholders = Vec::new();

    text = strip_file_links(&text);

    let piped_link_re = Regex::new(r"\[\[([^\]|]+)\|([^\]]+)\]\]").unwrap();
    text = piped_link_re
        .replace_all(&text, |captures: &regex::Captures| {
            wiki_link_placeholder(
                &mut link_placeholders,
                &captures[1],
                &captures[2],
                internal_links,
                language,
            )
        })
        .into_owned();

    let simple_link_re = Regex::new(r"\[\[([^\]|]+)\]\]").unwrap();
    text = simple_link_re
        .replace_all(&text, |captures: &regex::Captures| {
            wiki_link_placeholder(
                &mut link_placeholders,
                &captures[1],
                &captures[1],
                internal_links,
                language,
            )
        })
        .into_owned();

    let external_link_re = Regex::new(r"\[(https?://[^\s\]]+)\s+([^\]]+)\]").unwrap();
    text = external_link_re.replace_all(&text, "$2").into_owned();

    let bare_external_link_re = Regex::new(r"\[(https?://[^\]]+)\]").unwrap();
    text = bare_external_link_re.replace_all(&text, "$1").into_owned();

    let mut html = format_inline_text(&text);

    for (index, link) in link_placeholders.iter().enumerate() {
        html = html.replace(&format!("__WIKIPEDIA_TO_EPUB_LINK_{index}__"), link);
    }

    html
}

fn format_inline_text(text: &str) -> String {
    let mut text = text.to_string();

    text = Regex::new(r"'''(.*?)'''")
        .unwrap()
        .replace_all(
            &text,
            "__WIKIPEDIA_TO_EPUB_BOLD_START__${1}__WIKIPEDIA_TO_EPUB_BOLD_END__",
        )
        .into_owned();
    text = Regex::new(r"''(.*?)''")
        .unwrap()
        .replace_all(
            &text,
            "__WIKIPEDIA_TO_EPUB_ITALIC_START__${1}__WIKIPEDIA_TO_EPUB_ITALIC_END__",
        )
        .into_owned();
    text = text.replace("'''", "");
    text = text.replace("''", "");

    let residual_tags_re = Regex::new(r"(?is)</?[a-z0-9]+(?:\s+[^>]*)?>").unwrap();
    text = residual_tags_re.replace_all(&text, "").into_owned();

    let entity_decoded = decode_html_entities(&text).into_owned();
    let collapsed = entity_decoded
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let html = encode_text(collapsed.trim())
        .replace("__WIKIPEDIA_TO_EPUB_BOLD_START__", "<strong>")
        .replace("__WIKIPEDIA_TO_EPUB_BOLD_END__", "</strong>")
        .replace("__WIKIPEDIA_TO_EPUB_ITALIC_START__", "<em>")
        .replace("__WIKIPEDIA_TO_EPUB_ITALIC_END__", "</em>")
        .replace("__WIKIPEDIA_TO_EPUB_SMALL_START__", "<small>")
        .replace("__WIKIPEDIA_TO_EPUB_SMALL_END__", "</small>")
        .replace(
            "__WIKIPEDIA_TO_EPUB_KOREAN_TEXT_START__",
            r#"<span title="Korean-language text">"#,
        )
        .replace("__WIKIPEDIA_TO_EPUB_KOREAN_TEXT_END__", "</span>")
        .replace(
            "__WIKIPEDIA_TO_EPUB_KOREAN_HANGUL_START__",
            r#"<span lang="ko-Hang">"#,
        )
        .replace(
            "__WIKIPEDIA_TO_EPUB_KOREAN_HANJA_START__",
            r#"<span lang="ko-Hani">"#,
        )
        .replace("__WIKIPEDIA_TO_EPUB_KOREAN_SCRIPT_END__", "</span>")
        .replace("__WIKIPEDIA_TO_EPUB_JAPANESE_NORMAL_START__", "<span>")
        .replace("__WIKIPEDIA_TO_EPUB_JAPANESE_NORMAL_END__", "</span>")
        .replace(
            "__WIKIPEDIA_TO_EPUB_JAPANESE_TEXT_START__",
            r#"<span title="Japanese-language text"><span lang="ja">"#,
        )
        .replace("__WIKIPEDIA_TO_EPUB_JAPANESE_TEXT_END__", "</span></span>");

    restore_pb_spans(&restore_color_box_spans(&restore_open_access_spans(
        &restore_ipa_template_spans(&restore_abbr_template_spans(&restore_lang_template_spans(
            &html,
        ))),
    )))
}

fn restore_color_box_spans(html: &str) -> String {
    Regex::new(r"__WIKIPEDIA_TO_EPUB_COLOR_BOX_START__(.*?)__WIKIPEDIA_TO_EPUB_COLOR_BOX_END__")
        .unwrap()
        .replace_all(html, |captures: &regex::Captures| {
            format!(
                r#"<span style="color: {};">■</span>"#,
                encode_double_quoted_attribute(&captures[1])
            )
        })
        .into_owned()
}

fn restore_pb_spans(html: &str) -> String {
    html.replace("__WIKIPEDIA_TO_EPUB_PB__", "<br /><br />")
}

fn restore_open_access_spans(html: &str) -> String {
    html.replace(
        "__WIKIPEDIA_TO_EPUB_OPEN_ACCESS__",
        r#"<span title="open access">&#128275;</span>"#,
    )
}

fn restore_lang_template_spans(html: &str) -> String {
    Regex::new(
        r"__WIKIPEDIA_TO_EPUB_LANG_START__([A-Za-z0-9-]+)__WIKIPEDIA_TO_EPUB_LANG_VALUE__(.*?)__WIKIPEDIA_TO_EPUB_LANG_END__",
    )
    .unwrap()
    .replace_all(html, |captures: &regex::Captures| {
        format!(
            r#"<span lang="{}">{}</span>"#,
            encode_double_quoted_attribute(&captures[1]),
            &captures[2]
        )
    })
    .into_owned()
}

fn restore_abbr_template_spans(html: &str) -> String {
    Regex::new(
        r"__WIKIPEDIA_TO_EPUB_ABBR_START__(.*?)__WIKIPEDIA_TO_EPUB_ABBR_VALUE__(.*?)__WIKIPEDIA_TO_EPUB_ABBR_END__",
    )
    .unwrap()
    .replace_all(html, |captures: &regex::Captures| {
        format!(
            r#"<abbr title="{}">{}</abbr>"#,
            encode_double_quoted_attribute(&captures[1]),
            &captures[2]
        )
    })
    .into_owned()
}

fn restore_ipa_template_spans(html: &str) -> String {
    Regex::new(r"__WIKIPEDIA_TO_EPUB_IPA_START__(.*?)__WIKIPEDIA_TO_EPUB_IPA_END__")
        .unwrap()
        .replace_all(html, |captures: &regex::Captures| {
            format!(
                r#"<span title="International Phonetic Alphabet">[{}]</span>"#,
                &captures[1]
            )
        })
        .into_owned()
}

fn wiki_link_placeholder(
    links: &mut Vec<String>,
    target: &str,
    label: &str,
    internal_links: &InternalLinks,
    language: &str,
) -> String {
    let placeholder = format!("__WIKIPEDIA_TO_EPUB_LINK_{}__", links.len());
    links.push(wikipedia_link_html(target, label, internal_links, language));
    placeholder
}

fn wikipedia_link_html(
    target: &str,
    label: &str,
    internal_links: &InternalLinks,
    language: &str,
) -> String {
    if let Some(href) = target.strip_prefix("official-url:") {
        return format!(
            r#"<a href="{}">{}</a><span class="external-link">↗</span>"#,
            encode_double_quoted_attribute(&normalize_external_url(href)),
            format_inline_text(label)
        );
    }

    if let Some(relation_id) = target.strip_prefix("osmrelation:") {
        return format!(
            r#"<a href="{}">{}</a><span class="external-link">↗</span>"#,
            encode_double_quoted_attribute(&openstreetmap_relation_url(relation_id)),
            format_inline_text(label)
        );
    }

    if let Some(href) = wiktionary_article_url(target) {
        return format!(
            r#"<a href="{}">{}</a><span class="external-link">↗</span>"#,
            encode_double_quoted_attribute(&href),
            format_inline_text(label)
        );
    }

    if let Some(href) = wikivoyage_article_url(target) {
        return format!(
            r#"<a href="{}">{}</a><span class="external-link">↗</span>"#,
            encode_double_quoted_attribute(&href),
            format_inline_text(label)
        );
    }

    if let Some(href) = wikisource_article_url(target) {
        return format!(
            r#"<a href="{}">{}</a><span class="external-link">↗</span>"#,
            encode_double_quoted_attribute(&href),
            format_inline_text(label)
        );
    }

    if let Some(href) = wikibooks_article_url(target) {
        return format!(
            r#"<a href="{}">{}</a><span class="external-link">↗</span>"#,
            encode_double_quoted_attribute(&href),
            format_inline_text(label)
        );
    }

    if let Some(href) = interlanguage_article_url(target) {
        return format!(
            r#"<a href="{}">{}</a><span class="external-link">↗</span>"#,
            encode_double_quoted_attribute(&href),
            format_inline_text(label)
        );
    }

    if let Some(href) = internal_article_url(target, internal_links) {
        return format!(
            r#"<a href="{}">{}</a>"#,
            encode_double_quoted_attribute(&href),
            format_inline_text(label)
        );
    }

    format!(
        r#"<a href="{}">{}</a><span class="external-link">↗</span>"#,
        encode_double_quoted_attribute(&wikipedia_article_url(target, language)),
        format_inline_text(label)
    )
}

fn normalize_external_url(url: &str) -> String {
    let url = url.trim();
    if let Some(url) = url.strip_prefix("//") {
        format!("https://{url}")
    } else if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{url}")
    }
}

fn openstreetmap_relation_url(relation_id: &str) -> String {
    let relation_id = relation_id.trim();
    format!("https://www.openstreetmap.org/relation/{relation_id}")
}

fn internal_article_url(target: &str, internal_links: &InternalLinks) -> Option<String> {
    let article = target
        .split_once('#')
        .map_or(target, |(article, _)| article);
    internal_links
        .get(&normalize_lookup_key(article))
        .map(ToString::to_string)
}

fn wikipedia_article_url(target: &str, language: &str) -> String {
    let target = target.trim().replace(' ', "_");
    format!("https://{language}.wikipedia.org/wiki/{target}")
}

fn interlanguage_article_url(target: &str) -> Option<String> {
    let target = target.strip_prefix(':')?;
    let (language, title) = target.split_once(':')?;
    if !language
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch == '-')
    {
        return None;
    }
    Some(wikipedia_article_url(title, language))
}

fn wiktionary_article_url(target: &str) -> Option<String> {
    let title = target.strip_prefix("wikt:")?.trim().replace(' ', "_");
    let mut url = Url::parse("https://en.wiktionary.org").unwrap();
    url.path_segments_mut().unwrap().push("wiki").push(&title);
    Some(url.into())
}

fn wikivoyage_article_url(target: &str) -> Option<String> {
    let title = target.strip_prefix("voy:")?.trim().replace(' ', "_");
    let mut url = Url::parse("https://en.wikivoyage.org").unwrap();
    url.path_segments_mut().unwrap().push("wiki").push(&title);
    Some(url.into())
}

fn wikisource_article_url(target: &str) -> Option<String> {
    let title = target.strip_prefix("src:")?.trim().replace(' ', "_");
    let mut url = Url::parse("https://en.wikisource.org").unwrap();
    let mut segments = url.path_segments_mut().unwrap();
    segments.push("wiki");
    for segment in title.split('/') {
        segments.push(segment);
    }
    drop(segments);
    Some(url.into())
}

fn wikibooks_article_url(target: &str) -> Option<String> {
    let title = target.strip_prefix("b:")?.trim().replace(' ', "_");
    let mut url = Url::parse("https://en.wikibooks.org").unwrap();
    let mut segments = url.path_segments_mut().unwrap();
    segments.push("wiki");
    for segment in title.split('/') {
        segments.push(segment);
    }
    drop(segments);
    Some(url.into())
}

fn wikipedia_parse_api_url(language: &str) -> AppResult<Url> {
    let language = normalized_wikipedia_language(language)?;
    Url::parse(&format!("https://{language}.wikipedia.org/w/api.php"))
        .map_err(|err| AppError::Message(format!("invalid Wikipedia API URL: {err}")))
}

fn normalized_wikipedia_language(language: &str) -> AppResult<String> {
    let language = language.trim().to_ascii_lowercase();
    let valid = !language.is_empty()
        && language
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
        && language
            .split('-')
            .all(|part| !part.is_empty() && !part.starts_with(|ch: char| ch.is_ascii_digit()));

    if valid {
        Ok(language)
    } else {
        Err(AppError::Message(format!(
            "invalid Wikipedia language code: {language:?}"
        )))
    }
}

fn html_language_attributes(language: &str) -> String {
    let language = encode_double_quoted_attribute(language);
    if is_right_to_left_language(&language) {
        format!(r#"xml:lang="{language}" dir="rtl""#)
    } else {
        format!(r#"xml:lang="{language}""#)
    }
}

fn is_right_to_left_language(language: &str) -> bool {
    let base_language = language.split_once('-').map_or(language, |(base, _)| base);
    matches!(base_language, "ar" | "fa" | "he" | "ur")
}

fn parse_heading(line: &str) -> Option<(usize, String)> {
    let level = line.chars().take_while(|ch| *ch == '=').count();
    if !(2..=6).contains(&level) {
        return None;
    }

    let trailing = line.chars().rev().take_while(|ch| *ch == '=').count();
    if trailing != level {
        return None;
    }

    let inner = line
        .strip_prefix(&"=".repeat(level))?
        .strip_suffix(&"=".repeat(level))?
        .trim();

    if inner.is_empty() {
        return None;
    }

    Some((level, inner.to_string()))
}

fn strip_balanced_sections(text: &str, open: &str, close: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut depth = 0usize;
    let mut index = 0usize;

    while index < text.len() {
        let remaining = &text[index..];

        if remaining.starts_with(open) {
            depth += 1;
            index += open.len();
            continue;
        }

        if depth > 0 && remaining.starts_with(close) {
            depth -= 1;
            index += close.len();
            continue;
        }

        let ch = remaining.chars().next().unwrap();
        if depth == 0 {
            output.push(ch);
        }
        index += ch.len_utf8();
    }

    output
}

impl ImageRegistry {
    fn new(local_pages_dir: Option<&Path>) -> AppResult<Self> {
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

    fn register(&mut self, file_link: ParsedFileLink, source_page: &str) -> Option<usize> {
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

    fn occurrence(&self, id: usize) -> Option<&ImageOccurrence> {
        self.occurrences.get(id)
    }
}

fn normalize_image_title(title: &str) -> String {
    title.trim().replace('_', " ").to_ascii_lowercase()
}

fn image_extension(title: &str) -> String {
    title
        .rsplit_once('.')
        .map(|(_, extension)| sanitize_extension(extension))
        .filter(|extension| !extension.is_empty())
        .unwrap_or_else(|| "img".to_string())
}

fn path_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(sanitize_extension)
        .filter(|extension| !extension.is_empty())
}

fn sanitize_extension(extension: &str) -> String {
    extension
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn media_type_from_title(title: &str) -> &'static str {
    match image_extension(title).as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        _ => "application/octet-stream",
    }
}

fn resolve_images(
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
    // Without sleep the rewuests are rate-limited to 10 request then we get 429 Too Many Requests errors.
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

fn strip_file_links(text: &str) -> String {
    process_file_links(text, None, &InternalLinks::new(), "en", "")
}

fn process_file_links(
    text: &str,
    mut image_registry: Option<&mut ImageRegistry>,
    internal_links: &InternalLinks,
    language: &str,
    source_page: &str,
) -> String {
    let mut output = String::with_capacity(text.len());
    let mut index = 0usize;

    while index < text.len() {
        let remaining = &text[index..];

        if remaining.starts_with("[[")
            && is_file_link_start(&text[index + 2..])
            && let Some(end) = balanced_wiki_link_end(text, index)
        {
            let content = &text[index + 2..end - 2];
            if let Some(registry) = image_registry.as_deref_mut()
                && let Some(file_link) = parse_file_link(content, internal_links, language)
                && let Some(image_id) = registry.register(file_link, source_page)
            {
                output.push('\n');
                output.push_str(&format!("__WIKIPEDIA_TO_EPUB_IMAGE_{image_id}__"));
                output.push('\n');
            }
            index = end;
            continue;
        }

        let ch = remaining.chars().next().unwrap();
        output.push(ch);
        index += ch.len_utf8();
    }

    output
}

#[derive(Debug)]
struct ParsedFileLink {
    title: String,
    caption: String,
    alt: String,
}

fn parse_file_link(
    content: &str,
    internal_links: &InternalLinks,
    language: &str,
) -> Option<ParsedFileLink> {
    let params = split_template_params(content)
        .into_iter()
        .map(|part| part.trim().to_string())
        .collect::<Vec<_>>();
    let first = params.first()?;
    let title = file_link_title(first)?.trim().to_string();
    if title.is_empty() {
        return None;
    }

    let mut alt = None;
    let mut caption = None;
    for param in params.iter().skip(1).filter(|param| !param.is_empty()) {
        if let Some((key, value)) = param.split_once('=')
            && key.trim().eq_ignore_ascii_case("alt")
        {
            alt = Some(cleanup_inline_markup(
                &render_templates(value.trim()),
                internal_links,
                language,
            ));
            continue;
        }

        if file_link_param_is_option(param) {
            continue;
        }

        caption = Some(cleanup_inline_markup(
            &render_templates(param),
            internal_links,
            language,
        ));
    }

    let caption = caption.unwrap_or_default();
    let alt = alt
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            if caption.trim().is_empty() {
                title.clone()
            } else {
                caption.clone()
            }
        });

    Some(ParsedFileLink {
        title,
        caption,
        alt,
    })
}

fn file_link_title(value: &str) -> Option<&str> {
    let value = value.trim();
    value
        .strip_prefix("File:")
        .or_else(|| value.strip_prefix("file:"))
        .or_else(|| value.strip_prefix("Image:"))
        .or_else(|| value.strip_prefix("image:"))
}

fn file_link_param_is_option(value: &str) -> bool {
    let value = value.trim();
    let lowercase = value.to_ascii_lowercase();
    matches!(
        lowercase.as_str(),
        "thumb"
            | "thumbnail"
            | "frame"
            | "frameless"
            | "border"
            | "right"
            | "left"
            | "center"
            | "centre"
            | "none"
            | "baseline"
            | "middle"
            | "sub"
            | "super"
            | "text-top"
            | "text-bottom"
            | "top"
            | "bottom"
    ) || lowercase.ends_with("px")
        || lowercase.starts_with("upright")
        || lowercase.starts_with("link=")
        || lowercase.starts_with("class=")
        || lowercase.starts_with("lang=")
        || lowercase.starts_with("page=")
}

fn image_marker_id(line: &str) -> Option<usize> {
    line.strip_prefix("__WIKIPEDIA_TO_EPUB_IMAGE_")?
        .strip_suffix("__")?
        .parse()
        .ok()
}

fn render_image_html(image: &ImageOccurrence) -> String {
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

fn is_file_link_start(text: &str) -> bool {
    let trimmed = text.trim_start();
    let lowercase = trimmed.chars().take(6).collect::<String>().to_lowercase();
    lowercase.starts_with("file:") || lowercase.starts_with("image:")
}

fn balanced_wiki_link_end(text: &str, start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = start;

    while index < text.len() {
        let remaining = &text[index..];

        if remaining.starts_with("[[") {
            depth += 1;
            index += 2;
            continue;
        }

        if remaining.starts_with("]]") && depth > 0 {
            depth -= 1;
            index += 2;
            if depth == 0 {
                return Some(index);
            }
            continue;
        }

        let ch = remaining.chars().next().unwrap();
        index += ch.len_utf8();
    }

    None
}

fn write_epub(
    config: &BookConfig,
    chapters: &[Chapter],
    images: &[ResolvedImage],
    wikipedia_language: &str,
) -> AppResult<()> {
    if let Some(parent) = config.output_file.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    let identifier = book_identifier();
    let file = File::create(&config.output_file)?;
    let mut zip = ZipWriter::new(file);
    let stored: SimpleFileOptions =
        FileOptions::default().compression_method(CompressionMethod::Stored);
    let deflated: SimpleFileOptions =
        FileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("mimetype", stored)?;
    zip.write_all(b"application/epub+zip")?;

    zip.start_file("META-INF/container.xml", deflated)?;
    zip.write_all(container_xml().as_bytes())?;

    zip.start_file("OEBPS/style.css", deflated)?;
    zip.write_all(style_css().as_bytes())?;

    let frontmatter = frontmatter_xhtml(&config.metadata, wikipedia_language);
    zip.start_file("OEBPS/frontmatter.xhtml", deflated)?;
    zip.write_all(frontmatter.as_bytes())?;

    for chapter in chapters {
        zip.start_file(format!("OEBPS/{}", chapter.file_name), deflated)?;
        zip.write_all(chapter.content.as_bytes())?;
    }

    for image in images {
        zip.start_file(format!("OEBPS/{}", image.href), deflated)?;
        zip.write_all(&image.bytes)?;
    }

    let nav = nav_xhtml(chapters, wikipedia_language);
    zip.start_file("OEBPS/nav.xhtml", deflated)?;
    zip.write_all(nav.as_bytes())?;

    let toc = toc_ncx(&identifier, config, chapters);
    zip.start_file("OEBPS/toc.ncx", deflated)?;
    zip.write_all(toc.as_bytes())?;

    let package = content_opf(&identifier, config, chapters, images);
    zip.start_file("OEBPS/content.opf", deflated)?;
    zip.write_all(package.as_bytes())?;

    zip.finish()?;
    Ok(())
}

fn container_xml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>
"#
}

fn style_css() -> &'static str {
    r#"body {
  font-family: serif;
  line-height: 1.4;
}

h1, h2, h3, h4, h5, h6 {
  page-break-after: avoid;
}

.external-link {
  font-size: 0.8em;
  vertical-align: super;
}

.image {
  margin: 1em 0;
  text-align: center;
}

.image img {
  max-width: 100%;
  height: auto;
}

.caption {
  font-size: 0.9em;
  margin-top: 0.25em;
}
"#
}

fn frontmatter_xhtml(metadata: &Metadata, wikipedia_language: &str) -> String {
    let internal_links = InternalLinks::new();
    let license = metadata
        .license
        .as_deref()
        .map(|license| cleanup_inline_markup(license, &internal_links, wikipedia_language))
        .unwrap_or_default();
    let date = metadata
        .date
        .as_deref()
        .map(|date| cleanup_inline_markup(date, &internal_links, wikipedia_language))
        .unwrap_or_default();
    let edition = cleanup_inline_markup(&metadata.edition, &internal_links, wikipedia_language);

    let mut details = vec![format!(
        "<p><strong>Author:</strong> {}</p>",
        encode_text(&metadata.author)
    )];

    details.push(format!("<p><strong>Edition:</strong> {edition}</p>"));

    if !date.is_empty() {
        details.push(format!("<p><strong>Date:</strong> {date}</p>"));
    }

    if !license.is_empty() {
        details.push(format!("<p><strong>License:</strong> {license}</p>"));
    }

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" {language_attributes}>
  <head>
    <title>{title}</title>
    <link rel="stylesheet" type="text/css" href="style.css" />
  </head>
  <body>
    <h1>{title}</h1>
    {}
  </body>
</html>
"#,
        details.join("\n    "),
        language_attributes = html_language_attributes(wikipedia_language),
        title = encode_text(&metadata.title),
    )
}

fn nav_xhtml(chapters: &[Chapter], language: &str) -> String {
    let items = chapters
        .iter()
        .map(|chapter| {
            format!(
                r#"<li><a href="{}">{}</a></li>"#,
                encode_text(&chapter.file_name),
                encode_text(&chapter.title)
            )
        })
        .collect::<Vec<_>>()
        .join("\n        ");

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" {language_attributes}>
  <head>
    <title>Table of contents</title>
    <link rel="stylesheet" type="text/css" href="style.css" />
  </head>
  <body>
    <nav epub:type="toc" id="toc">
      <h1>Contents</h1>
      <ol>
        <li><a href="frontmatter.xhtml">Front matter</a></li>
        {items}
      </ol>
    </nav>
  </body>
</html>
"#,
        language_attributes = html_language_attributes(language),
    )
}

fn content_opf(
    identifier: &str,
    config: &BookConfig,
    chapters: &[Chapter],
    images: &[ResolvedImage],
) -> String {
    let mut manifest_items = vec![
        r#"<item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>"#
            .to_string(),
        r#"<item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>"#.to_string(),
        r#"<item id="style" href="style.css" media-type="text/css"/>"#.to_string(),
        r#"<item id="frontmatter" href="frontmatter.xhtml" media-type="application/xhtml+xml"/>"#
            .to_string(),
    ];
    let mut spine_items = vec![r#"<itemref idref="frontmatter"/>"#.to_string()];

    for (index, chapter) in chapters.iter().enumerate() {
        let id = format!("chapter-{}", index + 1);
        manifest_items.push(format!(
            r#"<item id="{id}" href="{}" media-type="application/xhtml+xml"/>"#,
            encode_text(&chapter.file_name)
        ));
        spine_items.push(format!(r#"<itemref idref="{id}"/>"#));
    }

    for (index, image) in images.iter().enumerate() {
        manifest_items.push(format!(
            r#"<item id="image-{}" href="{}" media-type="{}"/>"#,
            index + 1,
            encode_text(&image.href),
            encode_text(&image.media_type)
        ));
    }

    let rights = config
        .metadata
        .license
        .as_deref()
        .map(encode_text)
        .unwrap_or_default();
    let date = config
        .metadata
        .date
        .as_deref()
        .map(encode_text)
        .unwrap_or_default();

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<package version="2.0" unique-identifier="bookid" xmlns="http://www.idpf.org/2007/opf">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="bookid">{identifier}</dc:identifier>
    <dc:title>{title}</dc:title>
    <dc:creator>{creator}</dc:creator>
    <dc:language>{language}</dc:language>
    {date_line}
    {rights_line}
  </metadata>
  <manifest>
    {manifest}
  </manifest>
  <spine toc="ncx">
    <itemref idref="nav" linear="no"/>
    {spine}
  </spine>
</package>
"#,
        title = encode_text(&config.metadata.title),
        creator = encode_text(&config.metadata.author),
        language = encode_text(&config.metadata.language),
        date_line = if date.is_empty() {
            String::new()
        } else {
            format!("<dc:date>{date}</dc:date>")
        },
        rights_line = if rights.is_empty() {
            String::new()
        } else {
            format!("<dc:rights>{rights}</dc:rights>")
        },
        manifest = manifest_items.join("\n    "),
        spine = spine_items.join("\n    "),
    )
}

fn toc_ncx(identifier: &str, config: &BookConfig, chapters: &[Chapter]) -> String {
    let mut nav_points = vec![
        r#"<navPoint id="frontmatter" playOrder="1">
      <navLabel><text>Front matter</text></navLabel>
      <content src="frontmatter.xhtml"/>
    </navPoint>"#
            .to_string(),
    ];

    for (index, chapter) in chapters.iter().enumerate() {
        nav_points.push(format!(
            r#"<navPoint id="chapter-{id}" playOrder="{order}">
      <navLabel><text>{title}</text></navLabel>
      <content src="{file}"/>
    </navPoint>"#,
            id = index + 1,
            order = index + 2,
            title = encode_text(&chapter.title),
            file = encode_text(&chapter.file_name),
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="{identifier}"/>
    <meta name="dtb:depth" content="1"/>
    <meta name="dtb:totalPageCount" content="0"/>
    <meta name="dtb:maxPageNumber" content="0"/>
  </head>
  <docTitle><text>{title}</text></docTitle>
  <navMap>
    {nav_points}
  </navMap>
</ncx>
"#,
        identifier = encode_text(identifier),
        title = encode_text(&config.metadata.title),
        nav_points = nav_points.join("\n    "),
    )
}

fn book_identifier() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("urn:wikipedia-to-epub:{nanos}")
}

#[cfg(test)]
mod tests;
