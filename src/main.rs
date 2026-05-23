use std::{
    collections::HashMap,
    env,
    error::Error,
    fmt::{self, Display},
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
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
}

#[derive(Debug, Parser)]
#[command(name = "wikipedia-to-epub")]
struct CliArgs {
    #[arg(value_name = "config.yaml")]
    config_path: PathBuf,
    #[arg(long = "local", value_name = "pages-dir")]
    local_pages_dir: Option<PathBuf>,
    #[arg(long = "log", value_name = "level", default_value_t = Level::INFO)]
    log_level: Level,
}

trait PageSource {
    fn load_page(&self, article: &str) -> AppResult<PageResponse>;
}

struct WikipediaApiPageSource {
    client: Client,
    api_url: Url,
}

impl WikipediaApiPageSource {
    fn new(language: &str) -> AppResult<Self> {
        let client = Client::builder().user_agent(USER_AGENT).build()?;
        let api_url = wikipedia_parse_api_url(language)?;
        Ok(Self { client, api_url })
    }
}

impl PageSource for WikipediaApiPageSource {
    fn load_page(&self, article: &str) -> AppResult<PageResponse> {
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

        let page = serde_json::from_str::<PageResponse>(&payload).map_err(|err| {
            AppError::Message(format!(
                "failed to parse Wikipedia response for '{article}': {err}"
            ))
        })?;
        info!(article = article, "downloaded page");

        Ok(page)
    }
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

    let page_source: Box<dyn PageSource> = if let Some(pages_dir) = args.local_pages_dir {
        Box::new(FixturePageSource::new(pages_dir))
    } else {
        Box::new(WikipediaApiPageSource::new(&wikipedia_language)?)
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
            )
        })
        .collect::<AppResult<Vec<_>>>()?;

    write_epub(&config, &chapters, &wikipedia_language)?;
    println!("Created {}", config.output_file.display());

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

fn read_config(path: &Path) -> AppResult<BookConfig> {
    let content = fs::read_to_string(path)?;
    Ok(serde_yaml::from_str(&content)?)
}

fn init_logging(level: Level) {
    let _ = tracing_fmt()
        .with_max_level(level)
        .with_target(false)
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
) -> AppResult<Chapter> {
    info!(article = article, "fetching article");
    let page = page_source.load_page(article)?;
    let rendered = render_wikitext(
        &page.parse.title,
        &page.parse.wikitext.text,
        internal_links,
        language,
    );

    Ok(Chapter {
        file_name: format!("chapter-{index}.xhtml"),
        title: page.parse.title,
        content: rendered,
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

fn render_wikitext(
    title: &str,
    wikitext: &str,
    internal_links: &InternalLinks,
    language: &str,
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
    text = strip_file_links(&text);

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
    } else if template.eq_ignore_ascii_case("Nihongo4") {
        render_japanese_template(params)
    } else if template.eq_ignore_ascii_case("lang") {
        render_lang_template(params)
    } else if template.eq_ignore_ascii_case("langx") {
        render_langx_template(params)
    } else if template.eq_ignore_ascii_case("ko-translit") {
        render_korean_transliteration_template(params)
    } else if template.eq_ignore_ascii_case("percentage") {
        render_percentage_template(params)
    } else if template.eq_ignore_ascii_case("UN_Population") {
        render_un_population_template(params)
    } else if template.eq_ignore_ascii_case("convert") {
        render_convert_template(params)
    } else if template.eq_ignore_ascii_case("main") {
        render_main_template(params)
    } else if template.eq_ignore_ascii_case("see also") {
        render_see_also_template(params)
    } else if template.eq_ignore_ascii_case("ill") {
        render_interlanguage_link_template(params)
    } else if template.eq_ignore_ascii_case("reign") {
        render_reign_template(params)
    } else if is_silent_template_name(template) {
        String::new()
    } else {
        debug!(
            content = template_log_content(content),
            "removing unhandled wikitext template"
        );
        log_unhandled_nested_template_instructions(params);
        String::new()
    }
}

fn log_unhandled_nested_template_instructions(text: &str) {
    let mut offset = 0;

    while let Some(start) = text[offset..].find("{{").map(|index| offset + index) {
        if let Some(end) = matching_template_end(text, start) {
            let content = &text[start + 2..end];
            let (template, params) = split_template_name(content);
            let template = template.trim();
            if !is_handled_template_name(template) {
                debug!(
                    content = template_log_content(content),
                    "removing nested unhandled wikitext template"
                );
                log_unhandled_nested_template_instructions(params);
            }
            offset = end + 2;
        } else {
            break;
        }
    }
}

fn template_log_content(content: &str) -> String {
    content.chars().take(20).collect()
}

fn is_handled_template_name(template: &str) -> bool {
    template.eq_ignore_ascii_case("Korean")
        || template.eq_ignore_ascii_case("Korean/auto")
        || template.eq_ignore_ascii_case("Nihongo4")
        || template.eq_ignore_ascii_case("lang")
        || template.eq_ignore_ascii_case("langx")
        || template.eq_ignore_ascii_case("ko-translit")
        || template.eq_ignore_ascii_case("percentage")
        || template.eq_ignore_ascii_case("UN_Population")
        || template.eq_ignore_ascii_case("convert")
        || template.eq_ignore_ascii_case("main")
        || template.eq_ignore_ascii_case("see also")
        || template.eq_ignore_ascii_case("ill")
        || template.eq_ignore_ascii_case("reign")
        || is_silent_template_name(template)
}

fn is_silent_template_name(template: &str) -> bool {
    let template = template.trim();
    template.eq_ignore_ascii_case("Distinguish")
        || template.eq_ignore_ascii_case("Pp-move")
        || template.eq_ignore_ascii_case("Protection padlock")
        || template.eq_ignore_ascii_case("Short description")
        || template.eq_ignore_ascii_case("About")
        || template.eq_ignore_ascii_case("Redirect")
        || template.eq_ignore_ascii_case("pp-semi-indef")
        || template.eq_ignore_ascii_case("Sfn")
        || template.eq_ignore_ascii_case("efn")
        || template
            .get(.."Infobox".len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("Infobox"))
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
            "__WIKIPEDIA_TO_EPUB_KOREAN_HANGUL_START__{hangul}__WIKIPEDIA_TO_EPUB_KOREAN_SCRIPT_END__"
        ));
    }

    if let Some(hanja) = hanja.as_deref()
        && !hanja.trim().is_empty()
    {
        values.push(format!(
            "__WIKIPEDIA_TO_EPUB_KOREAN_HANJA_START__{hanja}__WIKIPEDIA_TO_EPUB_KOREAN_SCRIPT_END__"
        ));
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
    let params = split_template_params(params);
    let term = params.first().map_or("", |value| value.trim());
    let japanese = params.get(1).map_or("", |value| value.trim());

    if japanese.is_empty() {
        return term.to_string();
    }

    format!(
        "{term}__WIKIPEDIA_TO_EPUB_JAPANESE_NORMAL_START__ (__WIKIPEDIA_TO_EPUB_JAPANESE_TEXT_START__{japanese}__WIKIPEDIA_TO_EPUB_JAPANESE_TEXT_END__)__WIKIPEDIA_TO_EPUB_JAPANESE_NORMAL_END__"
    )
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

    format!(
        "__WIKIPEDIA_TO_EPUB_LANG_START__{language}__WIKIPEDIA_TO_EPUB_LANG_VALUE__{text}__WIKIPEDIA_TO_EPUB_LANG_END__"
    )
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

    match links.as_slice() {
        [] => String::new(),
        [link] => link.clone(),
        [first, second] => format!("{first} and {second}"),
        _ => {
            let last = links.last().cloned().unwrap_or_default();
            let leading = &links[..links.len() - 1];
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
        .replace(',', "")
        .replace(' ', "")
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

    restore_lang_template_spans(&html)
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
    if let Some(href) = wiktionary_article_url(target) {
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

fn wiktionary_article_url(target: &str) -> Option<String> {
    let title = target.strip_prefix("wikt:")?.trim().replace(' ', "_");
    let mut url = Url::parse("https://en.wiktionary.org").unwrap();
    url.path_segments_mut().unwrap().push("wiki").push(&title);
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

fn strip_file_links(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut index = 0usize;

    while index < text.len() {
        let remaining = &text[index..];

        if remaining.starts_with("[[") && is_file_link_start(&text[index + 2..]) {
            if let Some(end) = balanced_wiki_link_end(text, index) {
                index = end;
                continue;
            }
        }

        let ch = remaining.chars().next().unwrap();
        output.push(ch);
        index += ch.len_utf8();
    }

    output
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

    let nav = nav_xhtml(chapters, wikipedia_language);
    zip.start_file("OEBPS/nav.xhtml", deflated)?;
    zip.write_all(nav.as_bytes())?;

    let toc = toc_ncx(&identifier, config, chapters);
    zip.start_file("OEBPS/toc.ncx", deflated)?;
    zip.write_all(toc.as_bytes())?;

    let package = content_opf(&identifier, config, chapters);
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

fn content_opf(identifier: &str, config: &BookConfig, chapters: &[Chapter]) -> String {
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
mod tests {
    use super::*;
    use reqwest::header::HeaderValue;

    #[test]
    fn article_candidates_cover_common_file_names() {
        let candidates = article_file_candidates("North Korea");
        assert!(candidates.contains(&"North Korea.json".to_string()));
        assert!(candidates.contains(&"north korea.json".to_string()));
        assert!(candidates.contains(&"North_Korea.json".to_string()));
        assert!(candidates.contains(&"north_korea.json".to_string()));
        assert!(candidates.contains(&"North-Korea.json".to_string()));
        assert!(candidates.contains(&"north-korea.json".to_string()));
    }

    #[test]
    fn render_wikitext_handles_sections_links_and_lists() {
        let internal_links = internal_links(&["Sample".to_string(), "Seoul".to_string()]);
        let rendered = render_wikitext(
            "Sample",
            r#"Intro with [[Link target|visible text]] and '''bold''' text. See [[Seoul]].

== History ==
* First item
* Second [https://example.com link]
[[Category:Hidden]]
{{Infobox|ignored=yes}}
<ref>omit this</ref>
"#,
            &internal_links,
            "en",
        );

        assert!(
            rendered.contains(
                r#"<p>Intro with <a href="https://en.wikipedia.org/wiki/Link_target">visible text</a><span class="external-link">↗</span> and <strong>bold</strong> text. See <a href="chapter-2.xhtml">Seoul</a>.</p>"#
            )
        );
        assert!(rendered.contains("<h2>History</h2>"));
        assert!(rendered.contains("<ul>"));
        assert!(rendered.contains("<li>First item</li>"));
        assert!(rendered.contains("<li>Second link</li>"));
        assert!(!rendered.contains("Category:Hidden"));
        assert!(!rendered.contains("Infobox"));
        assert!(!rendered.contains("omit this"));
    }

    #[test]
    fn render_wikitext_silently_skips_metadata_templates() {
        let rendered = render_wikitext(
            "Sample",
            r#"{{Short description|Sample page}}
{{About|the sample|other uses|Sample (disambiguation)}}
{{Distinguish|Example}}
{{Pp-move}}
{{Protection padlock|small=yes}}
{{Redirect|Sample}}
{{pp-semi-indef}}
{{Sfn|Author|2024|p=1}}
{{efn|Footnote text}}
{{Infobox settlement|name=Sample}}
Visible text."#,
            &InternalLinks::new(),
            "en",
        );

        assert!(rendered.contains("<h1>Sample</h1>"));
        assert!(rendered.contains("<p>Visible text.</p>"));
        assert!(!rendered.contains("Short description"));
        assert!(!rendered.contains("About"));
        assert!(!rendered.contains("Distinguish"));
        assert!(!rendered.contains("Pp-move"));
        assert!(!rendered.contains("Protection padlock"));
        assert!(!rendered.contains("Redirect"));
        assert!(!rendered.contains("pp-semi-indef"));
        assert!(!rendered.contains("Author"));
        assert!(!rendered.contains("Footnote text"));
        assert!(!rendered.contains("Infobox"));
    }

    #[test]
    fn template_log_content_is_limited_to_twenty_characters() {
        assert_eq!(
            template_log_content("Unhandled template with a long body"),
            "Unhandled template w"
        );
        assert_eq!(
            template_log_content("短いtemplate content"),
            "短いtemplate content"
        );
    }

    #[test]
    fn render_wikitext_formats_italic_markup() {
        let rendered = render_wikitext(
            "Sample",
            "Intro with ''italic text'' and [[Fortune Global 500|''Fortune'' Global 500]].",
            &InternalLinks::new(),
            "en",
        );

        assert!(rendered.contains(
            r#"<p>Intro with <em>italic text</em> and <a href="https://en.wikipedia.org/wiki/Fortune_Global_500"><em>Fortune</em> Global 500</a><span class="external-link">↗</span>.</p>"#
        ));
    }

    #[test]
    fn render_wikitext_parses_example_file() {
        let rendered = render_wikitext(
            "Sample",
            r#"''{{ill|Hyangyakchips\u014fngbang|ko|\ud5a5\uc57d\uc9d1\uc131\ubc29}}''"#,
            &InternalLinks::new(),
            "en",
        );

        assert!(rendered.contains("<h1>Sample</h1>"));
        assert!(rendered.contains(
            r#"<p><em><a href="https://en.wikipedia.org/wiki/Hyangyakchips\u014fngbang">Hyangyakchips\u014fngbang</a><span class="external-link">↗</span> [ko]</em></p>"#
        ));
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("ill|"));
    }

    #[test]
    fn render_wikitext_parses_empty_template_inside_italics() {
        let rendered = render_wikitext("Sample", "''{{  }}''", &InternalLinks::new(), "en");

        assert!(rendered.contains("<p><em></em></p>"));
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("}}"));
    }

    #[test]
    fn render_wikitext_formats_korean_templates() {
        let rendered = render_wikitext(
            "Sample",
            "Traditionally, ''seoul'' ({{Korean|hangul=서울|labels=no}}) meant capital. Earlier {{Korean|labels=no|위례성|慰禮城}} was nearby. He was called {{Korean/auto|hangul=^해동_^요순|hanja=海東堯舜|mr=yes|labels=no}}.",
            &InternalLinks::new(),
            "en",
        );

        assert!(rendered.contains(
            r#"<p>Traditionally, <em>seoul</em> (<span title="Korean-language text"><span lang="ko-Hang">서울</span></span>) meant capital. Earlier <span title="Korean-language text"><span lang="ko-Hang">위례성</span> / <span lang="ko-Hani">慰禮城</span></span> was nearby. He was called <span title="Korean-language text"><span lang="ko-Hang">해동요순</span> / <span lang="ko-Hani">海東堯舜</span></span>.</p>"#
        ));
    }

    #[test]
    fn render_wikitext_formats_japanese_nihongo4_templates() {
        let rendered = render_wikitext(
            "Sample",
            "The city was formerly {{Nihongo4|''[[Edo (Tokyo)|Edo]]''|[[wikt:江戸|江戸]]}}.",
            &InternalLinks::new(),
            "en",
        );

        assert!(
            rendered.contains(
                r#"<p>The city was formerly <em><a href="https://en.wikipedia.org/wiki/Edo_(Tokyo)">Edo</a><span class="external-link">↗</span></em><span> (<span title="Japanese-language text"><span lang="ja"><a href="https://en.wiktionary.org/wiki/%E6%B1%9F%E6%88%B8">江戸</a><span class="external-link">↗</span></span></span>)</span>.</p>"#
            ),
            "{rendered}"
        );
    }

    #[test]
    fn render_wikitext_formats_lang_templates() {
        let cases = [
            ("{{lang|ko|서울}}", r#"<p><span lang="ko">서울</span></p>"#),
            (
                "{{lang|ja|''Edo''}}",
                r#"<p><span lang="ja"><em>Edo</em></span></p>"#,
            ),
            (
                "{{lang|ko-Hang|[[Seoul|서울]]}}",
                r#"<p><span lang="ko-Hang"><a href="https://en.wikipedia.org/wiki/Seoul">서울</a><span class="external-link">↗</span></span></p>"#,
            ),
            ("{{lang|ko}}", "<h1>Sample</h1>"),
            ("{{lang|!|서울}}", "<p>서울</p>"),
        ];

        for (template, expected) in cases {
            let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
            assert!(
                rendered.contains(expected),
                "lang template {template:?} rendered unexpectedly:\n{rendered}"
            );
            assert!(!rendered.contains("{{"));
            assert!(!rendered.contains("lang|"));
        }
    }

    #[test]
    fn render_wikitext_formats_langx_templates() {
        let cases = [
            (
                "{{langx|ko|溝樓|lit=Walled City|label=none}}",
                r#"<p><span lang="ko">溝樓</span>, lit. Walled City</p>"#,
            ),
            (
                "{{langx|ko|가우리|lit=Center|label=none}}",
                r#"<p><span lang="ko">가우리</span>, lit. Center</p>"#,
            ),
            (
                "{{Langx|ja|朝鮮|translit=Chōsen|label=none}}",
                r#"<p><span lang="ja">朝鮮</span> (Chōsen)</p>"#,
            ),
        ];

        for (template, expected) in cases {
            let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
            assert!(
                rendered.contains(expected),
                "langx template {template:?} rendered unexpectedly:\n{rendered}"
            );
            assert!(!rendered.contains("{{"));
            assert!(!rendered.contains("langx|"));
        }
    }

    #[test]
    fn render_wikitext_formats_korean_transliteration_templates() {
        let cases = [
            ("{{Ko-translit|rr|^한국}}", "Hanguk"),
            ("{{Ko-translit|mr|^한국}}", "Han'guk"),
            ("{{ko-translit|rr|^조선}}", "Joseon"),
            ("{{ko-translit|mr|^조선}}", "Chosŏn"),
        ];

        for (template, expected) in cases {
            let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
            assert!(
                rendered.contains(&format!("<p>{expected}</p>")),
                "Ko-translit template {template:?} rendered unexpectedly:\n{rendered}"
            );
            assert!(!rendered.contains("{{"));
            assert!(!rendered.contains("translit|"));
        }
    }

    #[test]
    fn render_wikitext_formats_percentage_templates() {
        let cases = [
            ("{{Percentage|1|4}}", "25%"),
            ("{{Percentage|1280000|26100000|1}}", "4.9%"),
            (
                "{{Percentage|7769000|{{UN_Population|Dem. People's Republic of Korea}}}}",
                "30%",
            ),
            (
                "{{Percentage|1280000|{{UN_Population|Dem. People's Republic of Korea}}|1}}",
                "4.9%",
            ),
        ];

        for (template, expected) in cases {
            let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
            assert!(
                rendered.contains(&format!("<p>{expected}</p>")),
                "percentage template {template:?} rendered unexpectedly:\n{rendered}"
            );
        }
    }

    #[test]
    fn render_wikitext_formats_un_population_templates() {
        let cases = [
            (
                "{{UN_Population|Dem. People's Republic of Korea}}",
                "<p>26,100,000</p>",
            ),
            ("{{UN_Population|ref}}", "<h1>Sample</h1>"),
        ];

        for (template, expected) in cases {
            let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
            assert!(
                rendered.contains(expected),
                "UN_Population template {template:?} rendered unexpectedly:\n{rendered}"
            );
            assert!(!rendered.contains("{{"));
            assert!(!rendered.contains("UN_Population|"));
        }
    }

    #[test]
    fn render_wikitext_formats_convert_templates() {
        let cases = [
            ("{{convert|1100|km|abbr=on}}", "1100 km"),
            ("{{convert|30|°C|°F}}", "30 °C"),
            ("{{Convert|24|ug/m3||sp=us}}", "24 ug/m³"),
            ("{{convert|&minus;3|°C|1|disp=or}}", "−3 °C"),
            ("{{convert|10|to|47|km2|disp=or|abbr=on}}", "10 to 47 km²"),
            ("{{convert|15|km|0|abbr=on}}", "15 km"),
            ("{{convert|2.1|and|−5.5|C|F|1}}", "2.1 °C and −5.5 °C"),
            ("{{convert|250|km|0|abbr=on}}", "250 km"),
            ("{{convert|268|km2|mi2|sp=us|abbr=on}}", "268 km²"),
            ("{{convert|30.0|and|22.9|C|F|0}}", "30.0 °C and 22.9 °C"),
            ("{{convert|300|km/h|0|abbr=on}}", "300 km/h"),
            ("{{convert|40|C|F|1}}", "40 °C"),
            ("{{convert|4|km|mile|sp=us|abbr=on}}", "4 km"),
            ("{{convert|605.25|km2|sqmi|abbr=unit}}", "605.25 km²"),
            ("{{convert|613|km2|mi2|sp=us|abbr=on}}", "613 km²"),
            ("{{convert|940|km|abbr=on}}", "940 km"),
            ("{{convert|−10|C}}", "−10 °C"),
            ("{{convert|−15|C}}", "−15 °C"),
            ("{{convert|−20|C}}", "−20 °C"),
        ];

        for (template, expected) in cases {
            let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
            assert!(
                rendered.contains(&format!("<p>{expected}</p>")),
                "convert template {template:?} rendered unexpectedly:\n{rendered}"
            );
        }
    }

    #[test]
    fn render_wikitext_formats_main_templates() {
        let mut internal_links = InternalLinks::new();
        internal_links.insert("namesofkorea".to_string(), "chapter-2.xhtml".to_string());

        let rendered = render_wikitext(
            "Sample",
            "{{Main|Names of Korea}}\n{{Main|Korean cuisine|Korean tea ceremony}}",
            &internal_links,
            "en",
        );

        assert!(
            rendered.contains(r#"Main article: <a href="chapter-2.xhtml">Names of Korea</a>"#),
            "{rendered}"
        );
        assert!(rendered.contains(
            r#"Main articles: <a href="https://en.wikipedia.org/wiki/Korean_cuisine">Korean cuisine</a><span class="external-link">↗</span> and <a href="https://en.wikipedia.org/wiki/Korean_tea_ceremony">Korean tea ceremony</a><span class="external-link">↗</span>"#
        ));
    }

    #[test]
    fn render_wikitext_formats_see_also_templates() {
        let mut internal_links = InternalLinks::new();
        internal_links.insert("seoul".to_string(), "chapter-2.xhtml".to_string());

        let rendered = render_wikitext(
            "Sample",
            "{{See also|Seoul}}\n{{See also|Korean tea ceremony|Korean royal court cuisine}}",
            &internal_links,
            "en",
        );

        assert!(
            rendered.contains(r#"See also: <a href="chapter-2.xhtml">Seoul</a>"#),
            "{rendered}"
        );
        assert!(rendered.contains(
            r#"See also: <a href="https://en.wikipedia.org/wiki/Korean_tea_ceremony">Korean tea ceremony</a><span class="external-link">↗</span> and <a href="https://en.wikipedia.org/wiki/Korean_royal_court_cuisine">Korean royal court cuisine</a><span class="external-link">↗</span>"#
        ));
    }

    #[test]
    fn render_wikitext_formats_interlanguage_link_templates() {
        let rendered = render_wikitext(
            "Sample",
            "Known as ''{{ill|Hyangyakchips\u{014f}ngbang|ko|향약집성방}}'' and {{ill|Seoul|ko|서울|lt=the capital}}.",
            &InternalLinks::new(),
            "en",
        );

        assert!(
            rendered.contains(
                r#"<p>Known as <em><a href="https://en.wikipedia.org/wiki/Hyangyakchipsŏngbang">Hyangyakchipsŏngbang</a><span class="external-link">↗</span> [ko]</em> and <a href="https://en.wikipedia.org/wiki/Seoul">the capital</a><span class="external-link">↗</span> [ko].</p>"#
            ),
            "{rendered}"
        );
    }

    #[test]
    fn render_wikitext_formats_reign_templates() {
        let cases = [
            ("{{Reign}}", "r."),
            ("{{Reign|1207|1272}}", "r. 1207–1272"),
            (
                "{{Reign |1 October 1207 |1272}}",
                "r. 1 October 1207 – 1272",
            ),
            ("{{Reign|1207|present}}", "r. 1207–present"),
            ("{{Reign||940}}", "r. ?–940"),
            ("{{Reign|89|67|era=BCE}}", "r. 89–67 BCE"),
            ("{{Reign|single=1872}}", "r. 1872"),
            ("{{Reign|1962|present|show=word}}", "reigned 1962–present"),
            ("{{Reign|1962|present|show=colon}}", "reign: 1962–present"),
            ("{{Reign|1962|present|show=blank}}", "1962–present"),
            ("{{Reign|label=ruled|1967|1969}}", "ruled 1967–1969"),
            ("{{Reign|1267|1272|post-date=1275}}", "r. 1267–1272, 1275"),
        ];

        for (template, expected) in cases {
            let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
            assert!(
                rendered.contains(&format!("<p>{expected}</p>")),
                "reign template {template:?} rendered unexpectedly:\n{rendered}"
            );
        }
    }

    #[test]
    fn strip_balanced_sections_removes_nested_templates() {
        let cleaned = strip_balanced_sections("before {{a {{nested}} value}} after", "{{", "}}");
        assert_eq!(cleaned, "before  after");
    }

    #[test]
    fn strip_file_links_removes_nested_caption_links() {
        let cleaned = strip_file_links(
            "before [[File:Hangul.svg|thumb|[[Hangul]], afterwards called [[Korean alphabet]]]] after",
        );

        assert_eq!(cleaned, "before  after");
    }

    #[test]
    fn render_wikitext_omits_file_links_without_leaking_closing_markup() {
        let internal_links = InternalLinks::new();
        let rendered = render_wikitext(
            "Sample",
            "[[File:Gimjang.jpg|thumb|[[Gimjang]], the process for making [[kimchi]]]] Koreans traditionally believe in spices.",
            &internal_links,
            "en",
        );

        assert!(rendered.contains("<p>Koreans traditionally believe in spices.</p>"));
        assert!(!rendered.contains("[[File:"));
        assert!(!rendered.contains("]]"));
        assert!(!rendered.contains("Gimjang"));
    }

    #[test]
    fn fixture_page_source_uses_local_page_dumps() {
        let source = FixturePageSource::new("pages");
        let page = source.load_page("Korea").expect("fixture page should load");

        assert_eq!(page.parse.title, "Korea");
        assert!(page.parse.wikitext.text.contains("East Asia"));
    }

    #[test]
    fn wikipedia_urls_use_configured_language() {
        assert_eq!(
            wikipedia_parse_api_url("es")
                .expect("Spanish API URL should build")
                .as_str(),
            "https://es.wikipedia.org/w/api.php"
        );
        assert_eq!(
            wikipedia_article_url("Corea del Sur", "es"),
            "https://es.wikipedia.org/wiki/Corea_del_Sur"
        );
    }

    #[test]
    fn wikipedia_language_rejects_invalid_hostname_labels() {
        let err = normalized_wikipedia_language("en.example.com")
            .expect_err("invalid language should fail");

        assert!(err.to_string().contains("invalid Wikipedia language code"));
    }

    #[test]
    fn hebrew_html_uses_right_to_left_direction() {
        assert_eq!(html_language_attributes("he"), r#"xml:lang="he" dir="rtl""#);
        assert_eq!(html_language_attributes("en"), r#"xml:lang="en""#);
    }

    #[test]
    fn parse_args_accepts_local_pages_dir() {
        let args = parse_args_from(["wikipedia-to-epub", "books/korea.yaml", "--local", "pages"])
            .expect("args should parse");

        assert_eq!(args.config_path, PathBuf::from("books/korea.yaml"));
        assert_eq!(args.local_pages_dir, Some(PathBuf::from("pages")));
        assert_eq!(args.log_level, Level::INFO);
    }

    #[test]
    fn parse_args_accepts_explicit_log_level() {
        let args = parse_args_from(["wikipedia-to-epub", "books/korea.yaml", "--log", "debug"])
            .expect("args should parse");

        assert_eq!(args.log_level, Level::DEBUG);
    }

    #[test]
    fn parse_args_rejects_unknown_flags() {
        let err = parse_args_from(["wikipedia-to-epub", "books/korea.yaml", "--bogus"])
            .expect_err("unknown flags should fail");

        let err_message = err.to_string();
        assert!(err_message.contains("unexpected argument"));
        assert!(err_message.contains("--bogus"));
    }

    #[test]
    fn http_failure_detail_prefers_wikipedia_error_body() {
        let detail = http_failure_detail(
            &HeaderMap::new(),
            r#"{"error":{"code":"ratelimited","info":"Slow down"}}"#,
        );

        assert_eq!(detail.as_deref(), Some("ratelimited: Slow down"));
    }

    #[test]
    fn http_failure_detail_falls_back_to_retry_after_header() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("60"));

        let detail = http_failure_detail(&headers, "");

        assert_eq!(detail.as_deref(), Some("retry-after: 60"));
    }

    #[test]
    fn user_agent_includes_contact_information() {
        assert!(USER_AGENT.contains('/'));
        assert!(USER_AGENT.contains("github.com/szabgab/wikipedia-to-epub.rs"));
        assert!(USER_AGENT.contains("contact:"));
    }
}
