use crate::cache::normalize_lookup_key;
use crate::error::{AppError, AppResult};
use clap::Parser;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::Level;

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CachingMode {
    None,
    Local,
    Central,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ArticleType {
    Section,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum ArticleConfig {
    Simple(String),
    Detailed(Box<DetailedArticle>),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DetailedArticle {
    pub title: String,
    #[serde(rename = "type")]
    pub r#type: Option<ArticleType>,
    #[serde(default)]
    pub articles: Vec<ArticleConfig>,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ChapterStyle {
    #[default]
    Title,
    NumberedTitle,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum LinksToExcludedPages {
    Display,
    Emphasize,
    Disregard,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BookConfig {
    pub id: Option<String>,
    pub chapters: ChapterStyle,
    pub metadata: Metadata,
    #[serde(rename = "output-file")]
    pub output_file: PathBuf,
    #[serde(default)]
    pub images: bool,
    #[serde(default)]
    pub resources: bool,
    pub links_to_pages: bool,
    pub links_to_excluded_pages: LinksToExcludedPages,
    pub cover: Option<String>,
    pub caching: CachingMode,
    pub depth: usize,
    #[serde(default, alias = "front_matter", alias = "front-matter")]
    pub front_mater: Vec<PathBuf>,
    pub articles: Vec<ArticleConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Metadata {
    pub title: String,
    pub author: String,
    pub license: Option<String>,
    pub language: String,
    pub edition: String,
}

#[derive(Debug, Parser)]
#[command(
    name = "wikipedia-to-epub",
    version = concat!(env!("CARGO_PKG_VERSION"), " (SHA: ", env!("GIT_SHA"), ")"),
    disable_version_flag = true
)]
pub(crate) struct CliArgs {
    #[arg(value_name = "config.yaml")]
    pub config_path: PathBuf,
    #[arg(long = "local", value_name = "pages-dir")]
    pub local_pages_dir: Option<PathBuf>,
    #[arg(long = "refresh-cache")]
    pub refresh_cache: bool,
    #[arg(long = "log", value_name = "level", default_value_t = Level::INFO)]
    pub log_level: Level,
    #[arg(long = "images", conflicts_with = "no_images")]
    pub images: bool,
    #[arg(long = "no-images", conflicts_with = "images")]
    pub no_images: bool,
    #[arg(long = "logfile", value_name = "path")]
    pub logfile: Option<PathBuf>,
    #[arg(long = "caching", value_name = "mode")]
    pub caching: Option<CachingMode>,
    #[arg(short = 'o', long = "output", value_name = "output.epub")]
    pub output: Option<PathBuf>,
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    pub version: Option<bool>,
}

pub(crate) fn parse_args() -> AppResult<CliArgs> {
    parse_args_from(std::env::args_os())
}

pub(crate) fn parse_args_from<I, T>(args: I) -> AppResult<CliArgs>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    CliArgs::try_parse_from(args).map_err(|err| AppError::Message(err.to_string()))
}

pub(crate) fn read_config(path: &Path) -> AppResult<BookConfig> {
    let content = fs::read_to_string(path).map_err(|err| {
        AppError::Message(format!(
            "failed to read configuration file {}: {err}",
            path.display()
        ))
    })?;
    parse_config_str(path, &content)
}

pub(crate) fn parse_config_str(path: &Path, content: &str) -> AppResult<BookConfig> {
    let config = serde_yaml::from_str(content).map_err(|err| {
        let mut message = format!("invalid configuration in {}", path.display());
        if let Some(location) = err.location() {
            message.push_str(&format!(
                " at line {}, column {}",
                location.line(),
                location.column()
            ));
        }
        message.push_str(&format!(": {err}"));
        AppError::Message(message)
    })?;

    validate_unique_articles(path, &config)?;
    Ok(config)
}

fn validate_unique_articles(path: &Path, config: &BookConfig) -> AppResult<()> {
    let mut seen = HashMap::new();
    collect_duplicate_articles(path, &config.articles, &mut seen)
}

fn collect_duplicate_articles(
    path: &Path,
    articles: &[ArticleConfig],
    seen: &mut HashMap<String, String>,
) -> AppResult<()> {
    for article in articles {
        match article {
            ArticleConfig::Simple(title) => record_article_title(path, title, seen)?,
            ArticleConfig::Detailed(detailed) => {
                record_article_title(path, &detailed.title, seen)?;
                collect_duplicate_articles(path, &detailed.articles, seen)?;
            }
        }
    }

    Ok(())
}

fn record_article_title(
    path: &Path,
    title: &str,
    seen: &mut HashMap<String, String>,
) -> AppResult<()> {
    let lookup_key = normalize_lookup_key(title);
    if let Some(first_title) = seen.get(&lookup_key) {
        return Err(AppError::Message(format!(
            "invalid configuration in {}: duplicate page `{title}` (already included as `{first_title}`)",
            path.display()
        )));
    }

    seen.insert(lookup_key, title.to_string());
    Ok(())
}

pub(crate) fn current_utc_date() -> (i32, i32, i32) {
    if let Ok(mock_date) = std::env::var("WIKIPEDIA_TO_EPUB_MOCK_DATE") {
        let mock_date = mock_date.trim();
        let parts: Vec<&str> = mock_date.split('-').collect();
        if parts.len() == 3 {
            let parsed = (
                parts[0].parse::<i32>(),
                parts[1].parse::<i32>(),
                parts[2].parse::<i32>(),
            );
            if let (Ok(y), Ok(m), Ok(d)) = parsed {
                return (y, m, d);
            }
        }
        if let Some((y, m, d)) = parse_date_string(mock_date) {
            return (y, m, d);
        }
    }
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0));
    let secs = duration.as_secs();

    let days_since_epoch = secs / 86400;

    let mut days = days_since_epoch as i32;
    let mut year = 1970;
    loop {
        let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_year = if is_leap { 366 } else { 365 };
        if days >= days_in_year {
            days -= days_in_year;
            year += 1;
        } else {
            break;
        }
    }

    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let month_lengths = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for &length in &month_lengths {
        if days >= length {
            days -= length;
            month += 1;
        } else {
            break;
        }
    }

    let day = days + 1;
    (year, month, day)
}

pub(crate) fn current_utc_date_string() -> String {
    let (year, month, day) = current_utc_date();
    format!("{year:04}-{month:02}-{day:02}")
}

pub(crate) fn parse_date_string(s: &str) -> Option<(i32, i32, i32)> {
    let s = s.trim().replace(',', "");
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 3 {
        return None;
    }

    let months = [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ];

    // Case 1: Month Day Year (e.g. April 26 2001)
    if let Some(month_idx) = months
        .iter()
        .position(|&m| m.eq_ignore_ascii_case(parts[0]))
    {
        let month = month_idx as i32 + 1;
        let day = parts[1].parse::<i32>().ok()?;
        let year = parts[2].parse::<i32>().ok()?;
        return Some((year, month, day));
    }

    // Case 2: Day Month Year (e.g. 1 October 2024)
    if let Some(month_idx) = months
        .iter()
        .position(|&m| m.eq_ignore_ascii_case(parts[1]))
    {
        let month = month_idx as i32 + 1;
        let day = parts[0].parse::<i32>().ok()?;
        let year = parts[2].parse::<i32>().ok()?;
        return Some((year, month, day));
    }

    None
}
