use std::{
    collections::HashMap,
    env,
    error::Error,
    fmt::{self, Display},
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clap::Parser;
use html_escape::{decode_html_entities, encode_double_quoted_attribute, encode_text};
use regex::Regex;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, RETRY_AFTER},
};
use serde::Deserialize;
use tracing::{Level, info, warn};
use tracing_subscriber::fmt as tracing_fmt;
use zip::{
    CompressionMethod, ZipWriter,
    write::{FileOptions, SimpleFileOptions},
};

type AppResult<T> = Result<T, AppError>;
type InternalLinks = HashMap<String, String>;
const WIKIPEDIA_PARSE_API_URL: &str = "https://en.wikipedia.org/w/api.php";
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
    #[arg(long = "log", value_name = "level", default_value_t = Level::WARN)]
    log_level: Level,
}

trait PageSource {
    fn load_page(&self, article: &str) -> AppResult<PageResponse>;
}

struct WikipediaApiPageSource {
    client: Client,
}

impl WikipediaApiPageSource {
    fn new() -> AppResult<Self> {
        let client = Client::builder().user_agent(USER_AGENT).build()?;
        Ok(Self { client })
    }
}

impl PageSource for WikipediaApiPageSource {
    fn load_page(&self, article: &str) -> AppResult<PageResponse> {
        let response = self
            .client
            .get(WIKIPEDIA_PARSE_API_URL)
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
        thread::sleep(Duration::from_secs(1));

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
    if config.articles.is_empty() {
        return Err(AppError::Message(
            "the configuration must contain at least one article".to_string(),
        ));
    }

    let page_source: Box<dyn PageSource> = if let Some(pages_dir) = args.local_pages_dir {
        Box::new(FixturePageSource::new(pages_dir))
    } else {
        Box::new(WikipediaApiPageSource::new()?)
    };

    let internal_links = internal_links(&config.articles);
    let chapters = config
        .articles
        .iter()
        .enumerate()
        .map(|(index, article)| {
            load_chapter(page_source.as_ref(), article, index + 1, &internal_links)
        })
        .collect::<AppResult<Vec<_>>>()?;

    write_epub(&config, &chapters)?;
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
) -> AppResult<Chapter> {
    info!(article = article, "fetching article");
    let page = page_source.load_page(article)?;
    let rendered = render_wikitext(&page.parse.title, &page.parse.wikitext.text, internal_links);

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

fn render_wikitext(title: &str, wikitext: &str, internal_links: &InternalLinks) -> String {
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
    text = strip_balanced_sections(&text, "{{", "}}");
    text = strip_balanced_sections(&text, "{|", "|}");

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

            let heading = cleanup_inline_markup(&heading, internal_links);
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

            let item = cleanup_inline_markup(&captures[2], internal_links);
            if !item.is_empty() {
                html.push(format!("<li>{item}</li>"));
            }
            continue;
        }

        flush_list(&mut html, &mut active_list);

        let cleaned = cleanup_inline_markup(line, internal_links);
        if !cleaned.is_empty() {
            paragraph_lines.push(cleaned);
        }
    }

    flush_paragraph(&mut html, &mut paragraph_lines);
    flush_list(&mut html, &mut active_list);

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en">
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
        html.join("\n    ")
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

fn cleanup_inline_markup(line: &str, internal_links: &InternalLinks) -> String {
    let mut text = line.trim().to_string();
    let mut link_placeholders = Vec::new();

    let file_link_re = Regex::new(r"\[\[(?:File|Image):[^\]]+\]\]").unwrap();
    text = file_link_re.replace_all(&text, "").into_owned();

    let piped_link_re = Regex::new(r"\[\[([^\]|]+)\|([^\]]+)\]\]").unwrap();
    text = piped_link_re
        .replace_all(&text, |captures: &regex::Captures| {
            wiki_link_placeholder(
                &mut link_placeholders,
                &captures[1],
                &captures[2],
                internal_links,
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
            )
        })
        .into_owned();

    let external_link_re = Regex::new(r"\[(https?://[^\s\]]+)\s+([^\]]+)\]").unwrap();
    text = external_link_re.replace_all(&text, "$2").into_owned();

    let bare_external_link_re = Regex::new(r"\[(https?://[^\]]+)\]").unwrap();
    text = bare_external_link_re.replace_all(&text, "$1").into_owned();

    text = Regex::new(r"'''(.*?)'''")
        .unwrap()
        .replace_all(
            &text,
            "__WIKIPEDIA_TO_EPUB_BOLD_START__${1}__WIKIPEDIA_TO_EPUB_BOLD_END__",
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
    let mut html = encode_text(collapsed.trim())
        .replace("__WIKIPEDIA_TO_EPUB_BOLD_START__", "<strong>")
        .replace("__WIKIPEDIA_TO_EPUB_BOLD_END__", "</strong>");

    for (index, link) in link_placeholders.iter().enumerate() {
        html = html.replace(&format!("__WIKIPEDIA_TO_EPUB_LINK_{index}__"), link);
    }

    html
}

fn wiki_link_placeholder(
    links: &mut Vec<String>,
    target: &str,
    label: &str,
    internal_links: &InternalLinks,
) -> String {
    let placeholder = format!("__WIKIPEDIA_TO_EPUB_LINK_{}__", links.len());
    links.push(wikipedia_link_html(target, label, internal_links));
    placeholder
}

fn wikipedia_link_html(target: &str, label: &str, internal_links: &InternalLinks) -> String {
    if let Some(href) = internal_article_url(target, internal_links) {
        return format!(
            r#"<a href="{}">{}</a>"#,
            encode_double_quoted_attribute(&href),
            encode_text(decode_html_entities(label).trim())
        );
    }

    format!(
        r#"<a href="{}">{}</a><span class="external-link">↗</span>"#,
        encode_double_quoted_attribute(&wikipedia_article_url(target)),
        encode_text(decode_html_entities(label).trim())
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

fn wikipedia_article_url(target: &str) -> String {
    let target = target.trim().replace(' ', "_");
    format!("https://en.wikipedia.org/wiki/{target}")
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

fn write_epub(config: &BookConfig, chapters: &[Chapter]) -> AppResult<()> {
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

    let frontmatter = frontmatter_xhtml(&config.metadata);
    zip.start_file("OEBPS/frontmatter.xhtml", deflated)?;
    zip.write_all(frontmatter.as_bytes())?;

    for chapter in chapters {
        zip.start_file(format!("OEBPS/{}", chapter.file_name), deflated)?;
        zip.write_all(chapter.content.as_bytes())?;
    }

    let nav = nav_xhtml(chapters);
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

fn frontmatter_xhtml(metadata: &Metadata) -> String {
    let internal_links = InternalLinks::new();
    let license = metadata
        .license
        .as_deref()
        .map(|license| cleanup_inline_markup(license, &internal_links))
        .unwrap_or_default();
    let date = metadata
        .date
        .as_deref()
        .map(|date| cleanup_inline_markup(date, &internal_links))
        .unwrap_or_default();

    let mut details = vec![format!(
        "<p><strong>Author:</strong> {}</p>",
        encode_text(&metadata.author)
    )];

    if !date.is_empty() {
        details.push(format!("<p><strong>Date:</strong> {date}</p>"));
    }

    if !license.is_empty() {
        details.push(format!("<p><strong>License:</strong> {license}</p>"));
    }

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="{language}">
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
        language = encode_text(&metadata.language),
        title = encode_text(&metadata.title),
    )
}

fn nav_xhtml(chapters: &[Chapter]) -> String {
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
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" xml:lang="en">
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
"#
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
    fn strip_balanced_sections_removes_nested_templates() {
        let cleaned = strip_balanced_sections("before {{a {{nested}} value}} after", "{{", "}}");
        assert_eq!(cleaned, "before  after");
    }

    #[test]
    fn fixture_page_source_uses_local_page_dumps() {
        let source = FixturePageSource::new("pages");
        let page = source.load_page("Korea").expect("fixture page should load");

        assert_eq!(page.parse.title, "Korea");
        assert!(page.parse.wikitext.text.contains("East Asia"));
    }

    #[test]
    fn parse_args_accepts_local_pages_dir() {
        let args = parse_args_from(["wikipedia-to-epub", "books/korea.yaml", "--local", "pages"])
            .expect("args should parse");

        assert_eq!(args.config_path, PathBuf::from("books/korea.yaml"));
        assert_eq!(args.local_pages_dir, Some(PathBuf::from("pages")));
        assert_eq!(args.log_level, Level::WARN);
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
