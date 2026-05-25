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
    } else if template.eq_ignore_ascii_case("Nihongo4") {
        render_japanese_template(params)
    } else if template.eq_ignore_ascii_case("lang") {
        render_lang_template(params)
    } else if template.eq_ignore_ascii_case("langx") {
        render_langx_template(params)
    } else if template.eq_ignore_ascii_case("lang-zh") {
        render_chinese_lang_template(params)
    } else if template.eq_ignore_ascii_case("transliteration") {
        render_transliteration_template(params)
    } else if template.eq_ignore_ascii_case("ko-translit") {
        render_korean_transliteration_template(params)
    } else if template.eq_ignore_ascii_case("ipa") {
        render_ipa_template(params)
    } else if template.eq_ignore_ascii_case("abbr") {
        render_abbr_template(params)
    } else if template.eq_ignore_ascii_case("rp") {
        render_reference_page_template(params)
    } else if template.eq_ignore_ascii_case("cite book") {
        render_cite_book_template(params)
    } else if template.eq_ignore_ascii_case("cite journal") {
        render_cite_journal_template(params)
    } else if template.eq_ignore_ascii_case("cite report") {
        render_cite_report_template(params)
    } else if template.eq_ignore_ascii_case("citation") {
        render_citation_template(params)
    } else if template.eq_ignore_ascii_case("harvc") {
        render_harvc_template(params)
    } else if template.eq_ignore_ascii_case("as of") {
        render_as_of_template(params)
    } else if template.eq_ignore_ascii_case("blockquote") {
        render_blockquote_template(params)
    } else if template.eq_ignore_ascii_case("percentage") {
        render_percentage_template(params)
    } else if template.eq_ignore_ascii_case("UN_Population") {
        render_un_population_template(params)
    } else if template.eq_ignore_ascii_case("convert") {
        render_convert_template(params)
    } else if template.eq_ignore_ascii_case("for timeline") {
        render_for_timeline_template(params)
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
    } else if template.eq_ignore_ascii_case("sclass") {
        render_ship_class_template(params)
    } else if template.eq_ignore_ascii_case("ill") {
        render_interlanguage_link_template(params)
    } else if template.eq_ignore_ascii_case("reign") {
        render_reign_template(params)
    } else if template.eq_ignore_ascii_case("open access")
        || template.eq_ignore_ascii_case("free access")
    {
        render_open_access_template()
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
    content.chars().take(50).collect()
}

fn is_handled_template_name(template: &str) -> bool {
    template.eq_ignore_ascii_case("Korean")
        || template.eq_ignore_ascii_case("Korean/auto")
        || template.eq_ignore_ascii_case("Nihongo4")
        || template.eq_ignore_ascii_case("lang")
        || template.eq_ignore_ascii_case("langx")
        || template.eq_ignore_ascii_case("lang-zh")
        || template.eq_ignore_ascii_case("transliteration")
        || template.eq_ignore_ascii_case("ko-translit")
        || template.eq_ignore_ascii_case("ipa")
        || template.eq_ignore_ascii_case("abbr")
        || template.eq_ignore_ascii_case("rp")
        || template.eq_ignore_ascii_case("cite book")
        || template.eq_ignore_ascii_case("cite journal")
        || template.eq_ignore_ascii_case("cite report")
        || template.eq_ignore_ascii_case("citation")
        || template.eq_ignore_ascii_case("harvc")
        || template.eq_ignore_ascii_case("as of")
        || template.eq_ignore_ascii_case("blockquote")
        || template.eq_ignore_ascii_case("percentage")
        || template.eq_ignore_ascii_case("UN_Population")
        || template.eq_ignore_ascii_case("convert")
        || template.eq_ignore_ascii_case("for timeline")
        || template.eq_ignore_ascii_case("main")
        || template.eq_ignore_ascii_case("see also")
        || template.eq_ignore_ascii_case("further")
        || template.eq_ignore_ascii_case("wiktionary")
        || template.eq_ignore_ascii_case("wikivoyage")
        || template.eq_ignore_ascii_case("sclass")
        || template.eq_ignore_ascii_case("ill")
        || template.eq_ignore_ascii_case("reign")
        || template.eq_ignore_ascii_case("open access")
        || template.eq_ignore_ascii_case("free access")
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
        || template.eq_ignore_ascii_case("sfnm")
        || template.eq_ignore_ascii_case("efn")
        || template.eq_ignore_ascii_case("refn")
        || template.eq_ignore_ascii_case("reflist")
        || template.eq_ignore_ascii_case("notelist")
        || template.eq_ignore_ascii_case("Refbegin")
        || template.eq_ignore_ascii_case("Refend")
        || template.eq_ignore_ascii_case("flagicon")
        || template.eq_ignore_ascii_case("unreferenced section")
        || template.eq_ignore_ascii_case("Excessive citations inline")
        || template.eq_ignore_ascii_case("Portal bar")
        || template.eq_ignore_ascii_case("Authority control")
        || template.eq_ignore_ascii_case("Portal")
        || template.eq_ignore_ascii_case("Commons category")
        || template.eq_ignore_ascii_case("location map+")
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
    template.eq_ignore_ascii_case("History of Korea")
        || template.eq_ignore_ascii_case("Korea topics")
        || template.eq_ignore_ascii_case("East Asian topics")
        || template.eq_ignore_ascii_case("Joseon monarchs")
        || template.eq_ignore_ascii_case("Grand princes of Joseon")
        || template.eq_ignore_ascii_case("House of Yi")
        || template.eq_ignore_ascii_case("Seoul")
        || template.eq_ignore_ascii_case("Seoul weatherbox")
        || template.eq_ignore_ascii_case("Seoul landmarks")
        || template.eq_ignore_ascii_case("Navboxes")
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

    restore_open_access_spans(&restore_ipa_template_spans(&restore_abbr_template_spans(
        &restore_lang_template_spans(&html),
    )))
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

fn wikivoyage_article_url(target: &str) -> Option<String> {
    let title = target.strip_prefix("voy:")?.trim().replace(' ', "_");
    let mut url = Url::parse("https://en.wikivoyage.org").unwrap();
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
mod tests;
