use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
};

use regex::Regex;

use html_escape::{decode_html_entities, encode_double_quoted_attribute, encode_text};
use reqwest::Url;
use tracing::{Level, debug, info, warn};
use tracing_subscriber::fmt as tracing_fmt;

pub(crate) mod cache;
pub(crate) mod config;
pub(crate) mod epub;
pub(crate) mod error;
pub(crate) mod image;
pub(crate) mod templates;

pub(crate) use cache::*;
pub(crate) use config::*;
pub(crate) use epub::*;
pub(crate) use error::*;
pub(crate) use image::*;
pub(crate) use templates::*;

type InternalLinks = HashMap<String, String>;
const USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (https://github.com/szabgab/wikipedia-to-epub.rs; contact: https://github.com/szabgab/wikipedia-to-epub.rs/issues)"
);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TemplateSkipCounts {
    recognized: usize,
    unknown: usize,
}

thread_local! {
    static TEMPLATE_SKIP_COUNTS: RefCell<Option<TemplateSkipCounts>> = const { RefCell::new(None) };
}

fn main() {
    if let Err(err) = try_main() {
        let msg = err.to_string();
        if (msg.contains("wikipedia-to-epub") && msg.contains("SHA:"))
            || msg.starts_with("Usage:")
            || msg.contains("Usage:\n")
            || msg.contains("Options:")
        {
            print!("{msg}");
            std::process::exit(0);
        } else {
            eprintln!("error: {err}");
            std::process::exit(1);
        }
    }
}

fn try_main() -> AppResult<()> {
    let args = parse_args()?;
    init_logging(args.log_level, args.logfile.as_deref());
    info!(
        config_path = %args.config_path.display(),
        local_pages_dir = ?args.local_pages_dir,
        log_level = ?args.log_level,
        "starting wikipedia-to-epub"
    );
    run(args)
}

#[allow(clippy::too_many_arguments)]
fn generate_chapters_hierarchical(
    entries: &[ArticleConfig],
    wikipedia_language: &str,
    loaded_pages: &std::collections::HashMap<String, PageResponse>,
    page_source: &dyn PageSource,
    internal_links: &InternalLinks,
    links_to_excluded_pages: LinksToExcludedPages,
    image_registry: &mut Option<ImageRegistry>,
    added_article_keys: &mut std::collections::HashSet<String>,
    chapters: &mut Vec<Chapter>,
    chapter_style: ChapterStyle,
    parent_prefix: &[usize],
) -> AppResult<Vec<TocNode>> {
    let mut nodes = Vec::new();
    let mut sibling_index = 1;
    for entry in entries {
        match entry {
            ArticleConfig::Simple(title) => {
                let lookup_key = normalize_lookup_key(title);
                if let Some(page) = loaded_pages.get(&lookup_key) {
                    if !page_source.is_cache_hit(title) {
                        info!(article = page.parse.title, "fetching article");
                    }
                    let mut current_prefix = parent_prefix.to_vec();
                    current_prefix.push(sibling_index);
                    let prefix_str = current_prefix
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(".");
                    sibling_index += 1;

                    let display_title = match chapter_style {
                        ChapterStyle::NumberedTitle => format!("{prefix_str} {}", page.parse.title),
                        ChapterStyle::Title => page.parse.title.clone(),
                    };

                    let chapter = load_chapter(
                        page,
                        display_title,
                        internal_links,
                        wikipedia_language,
                        links_to_excluded_pages,
                        image_registry.as_mut(),
                    )?;
                    let file_name = chapter.file_name.clone();
                    let chapter_title = chapter.title.clone();
                    chapters.push(chapter);
                    added_article_keys.insert(lookup_key);

                    nodes.push(TocNode {
                        title: chapter_title,
                        file_name,
                        children: Vec::new(),
                    });
                }
            }
            ArticleConfig::Detailed(detailed) => {
                if let Some(ArticleType::Section) = detailed.r#type {
                    let title = &detailed.title;
                    let mut current_prefix = parent_prefix.to_vec();
                    current_prefix.push(sibling_index);
                    let prefix_str = current_prefix
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<String>>()
                        .join(".");
                    sibling_index += 1;

                    let display_title = match chapter_style {
                        ChapterStyle::NumberedTitle => format!("{prefix_str} {title}"),
                        ChapterStyle::Title => title.clone(),
                    };

                    let content = format!(
                        r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="{}">
  <head>
    <title>{}</title>
    <link rel="stylesheet" type="text/css" href="style.css" />
  </head>
  <body>
    <h1>{}</h1>
  </body>
</html>
"#,
                        wikipedia_language, display_title, display_title
                    );
                    let file_name = sanitize_chapter_filename(title);
                    chapters.push(Chapter {
                        title: display_title.clone(),
                        file_name: file_name.clone(),
                        content,
                        template_skip_counts: TemplateSkipCounts::default(),
                    });

                    let children = generate_chapters_hierarchical(
                        &detailed.articles,
                        wikipedia_language,
                        loaded_pages,
                        page_source,
                        internal_links,
                        links_to_excluded_pages,
                        image_registry,
                        added_article_keys,
                        chapters,
                        chapter_style,
                        &current_prefix,
                    )?;

                    nodes.push(TocNode {
                        title: display_title,
                        file_name,
                        children,
                    });
                } else {
                    let title = &detailed.title;
                    let lookup_key = normalize_lookup_key(title);
                    if let Some(page) = loaded_pages.get(&lookup_key) {
                        if !page_source.is_cache_hit(title) {
                            info!(article = page.parse.title, "fetching article");
                        }
                        let mut current_prefix = parent_prefix.to_vec();
                        current_prefix.push(sibling_index);
                        let prefix_str = current_prefix
                            .iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>()
                            .join(".");
                        sibling_index += 1;

                        let display_title = match chapter_style {
                            ChapterStyle::NumberedTitle => {
                                format!("{prefix_str} {}", page.parse.title)
                            }
                            ChapterStyle::Title => page.parse.title.clone(),
                        };

                        let chapter = load_chapter(
                            page,
                            display_title,
                            internal_links,
                            wikipedia_language,
                            links_to_excluded_pages,
                            image_registry.as_mut(),
                        )?;
                        let file_name = chapter.file_name.clone();
                        let chapter_title = chapter.title.clone();
                        chapters.push(chapter);
                        added_article_keys.insert(lookup_key);

                        let children = generate_chapters_hierarchical(
                            &detailed.articles,
                            wikipedia_language,
                            loaded_pages,
                            page_source,
                            internal_links,
                            links_to_excluded_pages,
                            image_registry,
                            added_article_keys,
                            chapters,
                            chapter_style,
                            &current_prefix,
                        )?;

                        nodes.push(TocNode {
                            title: chapter_title,
                            file_name,
                            children,
                        });
                    }
                }
            }
        }
    }
    Ok(nodes)
}

fn run(args: CliArgs) -> AppResult<()> {
    let mut config = read_config(&args.config_path)?;
    if let Some(output) = args.output {
        config.output_file = output;
    }
    let mut cover_image = None;
    if let Some(ref cover_str) = config.cover
        && cover_str != "None"
        && !cover_str.is_empty()
    {
        let path = PathBuf::from(cover_str);
        let resolved_path = if path.is_absolute() {
            path
        } else {
            let config_parent = args.config_path.parent().unwrap_or_else(|| Path::new("."));
            config_parent.join(path)
        };
        if !resolved_path.is_file() {
            return Err(AppError::Message(format!(
                "cover image file not found: {}",
                resolved_path.display()
            )));
        }
        let bytes = fs::read(&resolved_path)?;
        let ext = resolved_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        let media_type = match ext.as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            _ => "image/jpeg",
        };
        cover_image = Some((bytes, ext, media_type));
    }

    let wikipedia_language = normalized_wikipedia_language(&config.metadata.language)?;
    if config.articles.is_empty() {
        return Err(AppError::Message(
            "the configuration must contain at least one article".to_string(),
        ));
    }

    let images = if args.images {
        true
    } else if args.no_images {
        false
    } else {
        config.images
    };

    let local_pages_dir = args.local_pages_dir.clone();
    let download_stats = DownloadStats::default();
    let download_cache = if local_pages_dir.is_some() {
        None
    } else {
        let caching = args.caching.unwrap_or(config.caching);
        let enabled = caching != CachingMode::None;
        let root = match caching {
            CachingMode::Central => default_cache_root()?,
            CachingMode::Local => std::env::current_dir()?.join(".cache"),
            CachingMode::None => default_cache_root()?,
        };
        Some(DownloadCache::new(
            root,
            args.refresh_cache,
            download_stats.clone(),
            enabled,
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
    let mut image_registry = if images {
        Some(ImageRegistry::new(local_pages_dir.as_deref())?)
    } else {
        None
    };

    let mut visited = std::collections::HashSet::new();
    let mut ordered_articles = Vec::new();
    let mut loaded_pages = HashMap::new();

    fn visit_hierarchical_articles(
        entries: &[ArticleConfig],
        depth: usize,
        page_source: &dyn PageSource,
        visited: &mut std::collections::HashSet<String>,
        ordered_articles: &mut Vec<String>,
        loaded_pages: &mut HashMap<String, PageResponse>,
    ) -> AppResult<()> {
        for entry in entries {
            match entry {
                ArticleConfig::Simple(title) => {
                    dfs_visit(
                        title,
                        0,
                        depth,
                        page_source,
                        visited,
                        ordered_articles,
                        loaded_pages,
                    )?;
                }
                ArticleConfig::Detailed(detailed) => {
                    if detailed.r#type.is_none() {
                        dfs_visit(
                            &detailed.title,
                            0,
                            depth,
                            page_source,
                            visited,
                            ordered_articles,
                            loaded_pages,
                        )?;
                    }
                    visit_hierarchical_articles(
                        &detailed.articles,
                        depth,
                        page_source,
                        visited,
                        ordered_articles,
                        loaded_pages,
                    )?;
                }
            }
        }
        Ok(())
    }

    visit_hierarchical_articles(
        &config.articles,
        config.depth,
        page_source.as_ref(),
        &mut visited,
        &mut ordered_articles,
        &mut loaded_pages,
    )?;

    let internal_links = internal_links(&ordered_articles);

    let mut chapters = Vec::new();
    let mut added_article_keys = std::collections::HashSet::new();

    let mut toc_nodes = generate_chapters_hierarchical(
        &config.articles,
        &wikipedia_language,
        &loaded_pages,
        page_source.as_ref(),
        &internal_links,
        config.links_to_excluded_pages,
        &mut image_registry,
        &mut added_article_keys,
        &mut chapters,
        config.chapters,
        &[],
    )?;

    // Now append any recursively crawled articles (depth > 0)
    let mut next_top_level = toc_nodes.len() + 1;
    for article in &ordered_articles {
        let lookup_key = normalize_lookup_key(article);
        if !added_article_keys.contains(&lookup_key) {
            let page_opt = loaded_pages.get(&lookup_key);
            if let Some(page) = page_opt {
                if !page_source.is_cache_hit(article) {
                    info!(article = page.parse.title, "fetching article");
                }
                let display_title = match config.chapters {
                    ChapterStyle::NumberedTitle => {
                        format!("{next_top_level} {}", page.parse.title)
                    }
                    ChapterStyle::Title => page.parse.title.clone(),
                };
                let chapter = load_chapter(
                    page,
                    display_title,
                    &internal_links,
                    &wikipedia_language,
                    config.links_to_excluded_pages,
                    image_registry.as_mut(),
                )?;
                let file_name = chapter.file_name.clone();
                let chapter_title = chapter.title.clone();
                chapters.push(chapter);
                added_article_keys.insert(lookup_key);
                next_top_level += 1;

                toc_nodes.push(TocNode {
                    title: chapter_title,
                    file_name,
                    children: Vec::new(),
                });
            }
        }
    }

    if config.resources {
        let mut resources_list = String::new();
        for article in &ordered_articles {
            let lookup_key = normalize_lookup_key(article);
            if let Some(page) = loaded_pages.get(&lookup_key) {
                let canonical_title = &page.parse.title;
                let url = wikipedia_article_url(canonical_title, &wikipedia_language);
                resources_list.push_str(&format!(
                    "      <li><a href=\"{}\">{}</a></li>\n",
                    encode_text(&url),
                    encode_text(canonical_title)
                ));
            }
        }

        let content = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" {language_attributes}>
  <head>
    <title>Resources</title>
    <link rel="stylesheet" type="text/css" href="style.css" />
  </head>
  <body>
    <h1>Resources</h1>
    <ul>
{}    </ul>
  </body>
</html>
"#,
            resources_list,
            language_attributes = html_language_attributes(&wikipedia_language),
        );

        let file_name = sanitize_chapter_filename("Resources");
        chapters.push(Chapter {
            title: "Resources".to_string(),
            file_name: file_name.clone(),
            content,
            template_skip_counts: TemplateSkipCounts::default(),
        });

        toc_nodes.push(TocNode {
            title: "Resources".to_string(),
            file_name,
            children: Vec::new(),
        });
    }

    if config.links_to_pages {
        let mut appendix_list = String::new();
        for article in &ordered_articles {
            let lookup_key = normalize_lookup_key(article);
            if let Some(page) = loaded_pages.get(&lookup_key) {
                let canonical_title = &page.parse.title;
                let url = wikipedia_article_url(canonical_title, &wikipedia_language);
                appendix_list.push_str(&format!(
                    "      <li><a href=\"{}\">{}</a></li>\n",
                    encode_text(&url),
                    encode_text(canonical_title)
                ));
            }
        }

        let content = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" {language_attributes}>
  <head>
    <title>Appendix A</title>
    <link rel="stylesheet" type="text/css" href="style.css" />
  </head>
  <body>
    <h1>Appendix A</h1>
    <ul>
{}    </ul>
  </body>
</html>
"#,
            appendix_list,
            language_attributes = html_language_attributes(&wikipedia_language),
        );

        let file_name = sanitize_chapter_filename("Appendix A");
        chapters.push(Chapter {
            title: "Appendix A".to_string(),
            file_name: file_name.clone(),
            content,
            template_skip_counts: TemplateSkipCounts::default(),
        });

        toc_nodes.push(TocNode {
            title: "Appendix A".to_string(),
            file_name,
            children: Vec::new(),
        });
    }

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

    let mut front_matter_chapters = Vec::new();
    let mut front_matter_toc_nodes = Vec::new();
    let config_parent = args.config_path.parent().unwrap_or_else(|| Path::new("."));
    for md_file in &config.front_mater {
        let resolved_path = if md_file.is_absolute() {
            md_file.clone()
        } else {
            config_parent.join(md_file)
        };
        if !resolved_path.is_file() {
            return Err(AppError::Message(format!(
                "front matter file not found: {}",
                resolved_path.display()
            )));
        }
        let chapter = load_markdown_chapter(&resolved_path, &wikipedia_language)?;
        let file_name = chapter.file_name.clone();
        let title = chapter.title.clone();
        front_matter_chapters.push(chapter);
        front_matter_toc_nodes.push(TocNode {
            title,
            file_name,
            children: Vec::new(),
        });
    }

    if !front_matter_chapters.is_empty() {
        front_matter_chapters.extend(chapters);
        chapters = front_matter_chapters;

        front_matter_toc_nodes.extend(toc_nodes);
        toc_nodes = front_matter_toc_nodes;
    }

    write_epub(
        &config,
        &chapters,
        &images,
        &wikipedia_language,
        &toc_nodes,
        &cover_image,
    )?;
    let report_path = html_report_path(&config.output_file);
    write_html_report(
        &report_path,
        &config.metadata.title,
        &wikipedia_language,
        &toc_nodes,
        &loaded_pages,
        &ordered_articles,
    )?;
    println!("Created {}", config.output_file.display());
    println!("Created {}", report_path.display());
    println!(
        "Skipped templates: recognized={}, unknown={}",
        total_template_skip_counts.recognized, total_template_skip_counts.unknown
    );
    log_download_stats(&download_stats);

    Ok(())
}

fn init_logging(level: Level, logfile: Option<&Path>) {
    use tracing_subscriber::prelude::*;
    let level_filter = tracing_subscriber::filter::LevelFilter::from_level(level);

    let stdout_layer = tracing_fmt::layer()
        .with_target(false)
        .with_filter(level_filter);

    let log_path = logfile.unwrap_or_else(|| Path::new("report.log"));
    let file_layer = std::fs::File::create(log_path).ok().map(|file| {
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

fn is_valid_internal_article_link(target: &str) -> bool {
    let target = target.trim();
    if target.is_empty() {
        return false;
    }
    if target.starts_with(':') {
        return false;
    }
    if let Some((prefix, _)) = target.split_once(':') {
        let prefix_lower = prefix.to_lowercase();
        // Check if prefix is a namespace or language code or interwiki
        let ignored_namespaces = [
            "category",
            "file",
            "image",
            "talk",
            "wikipedia",
            "wp",
            "template",
            "help",
            "portal",
            "special",
            "media",
            "draft",
            "user",
            "book",
            "module",
        ];
        if ignored_namespaces.contains(&prefix_lower.as_str()) {
            return false;
        }
        // If the prefix has length 2 or 3 and is all ASCII alphabetic, it is likely a language code
        if (2..=3).contains(&prefix_lower.len())
            && prefix_lower.chars().all(|c| c.is_ascii_alphabetic())
        {
            return false;
        }
        // Interwiki prefixes
        let interwikis = ["wikt", "voy", "s", "b", "m", "meta", "commons", "wikidata"];
        if interwikis.contains(&prefix_lower.as_str()) {
            return false;
        }
    }
    true
}

fn extract_internal_links(wikitext: &str) -> Vec<String> {
    let link_re = Regex::new(r"\[\[([^\]|]+)(?:\|[^\]]*)?\]\]").unwrap();
    let mut targets = Vec::new();
    for caps in link_re.captures_iter(wikitext) {
        if let Some(target_match) = caps.get(1) {
            let target = target_match.as_str();
            if is_valid_internal_article_link(target) {
                targets.push(target.to_string());
            }
        }
    }
    targets
}

fn dfs_visit(
    article: &str,
    current_depth: usize,
    max_depth: usize,
    page_source: &dyn PageSource,
    visited: &mut std::collections::HashSet<String>,
    ordered_articles: &mut Vec<String>,
    loaded_pages: &mut HashMap<String, PageResponse>,
) -> AppResult<()> {
    let norm = normalize_lookup_key(article);
    if visited.contains(&norm) {
        return Ok(());
    }

    // Load the page
    let page = match page_source.load_page(article) {
        Ok(p) => p,
        Err(err) => {
            if current_depth > 0 {
                warn!(
                    article = article,
                    error = %err,
                    "Skipping recursively followed link that failed to load"
                );
                return Ok(());
            } else {
                return Err(err);
            }
        }
    };

    let actual_title = page.parse.title.clone();
    let actual_norm = normalize_lookup_key(&actual_title);
    if visited.contains(&actual_norm) {
        visited.insert(norm);
        return Ok(());
    }

    visited.insert(norm);
    visited.insert(actual_norm.clone());

    ordered_articles.push(actual_title);
    loaded_pages.insert(actual_norm, page.clone());

    if current_depth < max_depth {
        // Extract all valid internal links from the page wikitext
        let targets = extract_internal_links(&page.parse.wikitext.text);
        for target in targets {
            dfs_visit(
                &target,
                current_depth + 1,
                max_depth,
                page_source,
                visited,
                ordered_articles,
                loaded_pages,
            )?;
        }
    }

    Ok(())
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

#[cfg(test)]
fn render_wikitext_with_template_counts(
    title: &str,
    wikitext: &str,
    internal_links: &InternalLinks,
    language: &str,
    image_registry: Option<&mut ImageRegistry>,
) -> (String, TemplateSkipCounts) {
    render_wikitext_with_template_counts_and_excluded_links(
        title,
        wikitext,
        internal_links,
        language,
        LinksToExcludedPages::Emphasize,
        image_registry,
    )
}

pub(crate) fn render_wikitext_with_template_counts_and_excluded_links(
    title: &str,
    wikitext: &str,
    internal_links: &InternalLinks,
    language: &str,
    links_to_excluded_pages: LinksToExcludedPages,
    image_registry: Option<&mut ImageRegistry>,
) -> (String, TemplateSkipCounts) {
    with_template_skip_counts(|| {
        render_wikitext_impl(
            title,
            wikitext,
            internal_links,
            language,
            links_to_excluded_pages,
            image_registry,
        )
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
    links_to_excluded_pages: LinksToExcludedPages,
    mut image_registry: Option<&mut ImageRegistry>,
) -> String {
    let mut text = wikitext.replace("\r\n", "\n");
    text = Regex::new(r"(?s)<!--.*?-->")
        .unwrap()
        .replace_all(&text, "")
        .into_owned();
    let reference_groups = collect_reference_groups(&text);
    let mut reflists = Vec::new();
    text = replace_reflist_templates(
        &text,
        &reference_groups,
        &mut reflists,
        internal_links,
        language,
        links_to_excluded_pages,
    );
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
    let mut tables = Vec::new();
    text = render_wikitext_tables_with_excluded_links(
        &text,
        &mut tables,
        internal_links,
        language,
        links_to_excluded_pages,
    );
    text = process_file_links_with_excluded_links(
        &text,
        image_registry.as_deref_mut(),
        internal_links,
        language,
        links_to_excluded_pages,
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

        if let Some(table_id) = table_marker_id(line) {
            flush_paragraph(&mut html, &mut paragraph_lines);
            flush_list(&mut html, &mut active_list);
            if let Some(table_html) = tables.get(table_id) {
                html.push(table_html.clone());
            }
            continue;
        }

        if let Some(reflist_id) = reflist_marker_id(line) {
            flush_paragraph(&mut html, &mut paragraph_lines);
            flush_list(&mut html, &mut active_list);
            if let Some(reflist_html) = reflists.get(reflist_id) {
                html.push(reflist_html.clone());
            }
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
            let text = cleanup_inline_markup_with_excluded_links(
                text,
                internal_links,
                language,
                links_to_excluded_pages,
            );
            if !text.is_empty() {
                html.push(format!("<p>{text}</p>"));
            }
            continue;
        }

        if let Some(source) = line.strip_prefix("__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_SOURCE__") {
            flush_paragraph(&mut html, &mut paragraph_lines);
            flush_list(&mut html, &mut active_list);
            let source = cleanup_inline_markup_with_excluded_links(
                source,
                internal_links,
                language,
                links_to_excluded_pages,
            );
            if !source.is_empty() {
                html.push(format!(r#"<p class="blockquote-source">{source}</p>"#));
            }
            continue;
        }

        if let Some((level, heading)) = parse_heading(line) {
            flush_paragraph(&mut html, &mut paragraph_lines);
            flush_list(&mut html, &mut active_list);

            let heading = cleanup_inline_markup_with_excluded_links(
                &heading,
                internal_links,
                language,
                links_to_excluded_pages,
            );
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

            let item = cleanup_inline_markup_with_excluded_links(
                &captures[2],
                internal_links,
                language,
                links_to_excluded_pages,
            );
            if !item.is_empty() {
                html.push(format!("<li>{item}</li>"));
            }
            continue;
        }

        flush_list(&mut html, &mut active_list);

        let cleaned = cleanup_inline_markup_with_excluded_links(
            line,
            internal_links,
            language,
            links_to_excluded_pages,
        );
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
    cleanup_inline_markup_with_excluded_links(
        line,
        internal_links,
        language,
        LinksToExcludedPages::Emphasize,
    )
}

fn cleanup_inline_markup_with_excluded_links(
    line: &str,
    internal_links: &InternalLinks,
    language: &str,
    links_to_excluded_pages: LinksToExcludedPages,
) -> String {
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
                links_to_excluded_pages,
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
                links_to_excluded_pages,
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

    let html = restore_lang_template_spans(&html);
    let html = restore_abbr_template_spans(&html);
    let html = restore_ipa_template_spans(&html);
    let html = restore_open_access_spans(&html);
    let html = restore_color_box_spans(&html);
    let html = restore_color_spans(&html);
    let html = restore_route_box_spans(&html);
    let html = restore_pb_spans(&html);
    let html = restore_br_spans(&html);
    let html = restore_sub_spans(&html);
    restore_sup_spans(&html)
}

#[derive(Clone, Debug)]
struct ReferenceTag {
    group: String,
    name: Option<String>,
    content: Option<String>,
}

fn normalize_reference_attr(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

fn parse_reference_tags(text: &str) -> Vec<ReferenceTag> {
    let ref_re = Regex::new(r#"(?is)<ref\b([^>/]*?)/>|<ref\b([^>]*)>(.*?)</ref>"#).unwrap();
    let name_re = Regex::new(r#"(?i)\bname\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#).unwrap();
    let group_re = Regex::new(r#"(?i)\bgroup\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#).unwrap();

    ref_re
        .captures_iter(text)
        .map(|captures| {
            let attrs = captures
                .get(1)
                .or_else(|| captures.get(2))
                .map(|m| m.as_str())
                .unwrap_or("");
            let name = name_re
                .captures(attrs)
                .and_then(|caps| caps.get(1).or_else(|| caps.get(2)).or_else(|| caps.get(3)))
                .map(|m| normalize_reference_attr(m.as_str()))
                .filter(|value| !value.is_empty());
            let group = group_re
                .captures(attrs)
                .and_then(|caps| caps.get(1).or_else(|| caps.get(2)).or_else(|| caps.get(3)))
                .map(|m| normalize_reference_attr(m.as_str()))
                .unwrap_or_default();
            let content = captures
                .get(3)
                .map(|m| m.as_str().trim().to_string())
                .filter(|value| !value.is_empty());

            ReferenceTag {
                group,
                name,
                content,
            }
        })
        .collect()
}

fn strip_reflist_templates(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut offset = 0usize;

    while let Some(start) = text[offset..].find("{{").map(|index| offset + index) {
        output.push_str(&text[offset..start]);
        if let Some(end) = matching_template_end(text, start) {
            let content = &text[start + 2..end];
            let (template, _) = split_template_name(content);
            if template.trim().eq_ignore_ascii_case("reflist") {
                offset = end + 2;
                continue;
            }
        }
        output.push_str("{{");
        offset = start + 2;
    }

    output.push_str(&text[offset..]);
    output
}

fn collect_reference_groups(text: &str) -> HashMap<String, Vec<String>> {
    let mut named_definitions = HashMap::<(String, String), String>::new();
    for tag in parse_reference_tags(text) {
        if let (Some(name), Some(content)) = (tag.name, tag.content) {
            named_definitions.insert((tag.group, name), content);
        }
    }

    let mut groups = HashMap::<String, Vec<String>>::new();
    let mut seen_named = HashSet::<(String, String)>::new();
    let occurrence_text = strip_reflist_templates(text);

    for tag in parse_reference_tags(&occurrence_text) {
        match (tag.name, tag.content) {
            (Some(name), Some(content)) => {
                if seen_named.insert((tag.group.clone(), name)) {
                    groups.entry(tag.group).or_default().push(content);
                }
            }
            (Some(name), None) => {
                let key = (tag.group.clone(), name.clone());
                if seen_named.insert(key.clone())
                    && let Some(content) = named_definitions.get(&key)
                {
                    groups.entry(tag.group).or_default().push(content.clone());
                }
            }
            (None, Some(content)) => {
                groups.entry(tag.group).or_default().push(content);
            }
            (None, None) => {}
        }
    }

    groups
}

fn render_reference_list(
    refs: &[String],
    internal_links: &InternalLinks,
    language: &str,
    links_to_excluded_pages: LinksToExcludedPages,
) -> String {
    if refs.is_empty() {
        return String::new();
    }

    let items = refs
        .iter()
        .filter_map(|reference| {
            let without_refs = Regex::new(r"(?is)<ref\b[^>/]*/>|<ref\b[^>]*>.*?</ref>")
                .unwrap()
                .replace_all(reference, "")
                .into_owned();
            let rendered_templates = render_templates(&without_refs);
            let cleaned = cleanup_inline_markup_with_excluded_links(
                &rendered_templates,
                internal_links,
                language,
                links_to_excluded_pages,
            );
            if cleaned.trim().is_empty() {
                None
            } else {
                Some(format!("<li>{cleaned}</li>"))
            }
        })
        .collect::<Vec<_>>();

    if items.is_empty() {
        String::new()
    } else {
        format!(r#"<ol class="references">{}</ol>"#, items.join(""))
    }
}

fn replace_reflist_templates(
    text: &str,
    reference_groups: &HashMap<String, Vec<String>>,
    reflists: &mut Vec<String>,
    internal_links: &InternalLinks,
    language: &str,
    links_to_excluded_pages: LinksToExcludedPages,
) -> String {
    let mut output = String::with_capacity(text.len());
    let mut offset = 0usize;

    while let Some(start) = text[offset..].find("{{").map(|index| offset + index) {
        output.push_str(&text[offset..start]);
        if let Some(end) = matching_template_end(text, start) {
            let content = &text[start + 2..end];
            let (template, params) = split_template_name(content);
            if template.trim().eq_ignore_ascii_case("reflist") {
                let named = template_named_params(params);
                let group = template_param(&named, &["group"])
                    .map(normalize_reference_attr)
                    .unwrap_or_default();
                let rendered = render_reference_list(
                    reference_groups
                        .get(&group)
                        .map(Vec::as_slice)
                        .unwrap_or(&[]),
                    internal_links,
                    language,
                    links_to_excluded_pages,
                );
                if !rendered.is_empty() {
                    let reflist_id = reflists.len();
                    reflists.push(rendered);
                    output.push('\n');
                    output.push_str(&format!("__WIKIPEDIA_TO_EPUB_REFLIST_{reflist_id}__"));
                    output.push('\n');
                }
                offset = end + 2;
                continue;
            }
        }
        output.push_str("{{");
        offset = start + 2;
    }

    output.push_str(&text[offset..]);
    output
}

fn reflist_marker_id(line: &str) -> Option<usize> {
    let line = line.trim();
    if line.starts_with("__WIKIPEDIA_TO_EPUB_REFLIST_") && line.ends_with("__") {
        let number_str = &line["__WIKIPEDIA_TO_EPUB_REFLIST_".len()..line.len() - 2];
        number_str.parse::<usize>().ok()
    } else {
        None
    }
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

fn restore_color_spans(html: &str) -> String {
    Regex::new(r"__WIKIPEDIA_TO_EPUB_COLOR_START__(.*?)__WIKIPEDIA_TO_EPUB_COLOR_MID__(.*?)__WIKIPEDIA_TO_EPUB_COLOR_END__")
        .unwrap()
        .replace_all(html, |captures: &regex::Captures| {
            format!(
                r#"<span style="color: {};">{}</span>"#,
                encode_double_quoted_attribute(&captures[1]),
                &captures[2]
            )
        })
        .into_owned()
}

fn restore_route_box_spans(html: &str) -> String {
    Regex::new(r"__WIKIPEDIA_TO_EPUB_ROUTE_BOX_START__(.*?)__WIKIPEDIA_TO_EPUB_ROUTE_BOX_MID__(.*?)__WIKIPEDIA_TO_EPUB_ROUTE_BOX_TEXT__(.*?)__WIKIPEDIA_TO_EPUB_ROUTE_BOX_END__")
        .unwrap()
        .replace_all(html, |captures: &regex::Captures| {
            format!(
                r#"<span style="background-color: {}; color: {}; padding: 1px 4px; border-radius: 2px; font-weight: bold; font-size: 0.9em; display: inline-block;">{}</span>"#,
                encode_double_quoted_attribute(&captures[1]),
                encode_double_quoted_attribute(&captures[2]),
                &captures[3]
            )
        })
        .into_owned()
}

fn restore_pb_spans(html: &str) -> String {
    html.replace("__WIKIPEDIA_TO_EPUB_PB__", "<br /><br />")
}

fn restore_br_spans(html: &str) -> String {
    html.replace("__WIKIPEDIA_TO_EPUB_BR__", "<br />")
}

fn restore_sup_spans(html: &str) -> String {
    html.replace("__WIKIPEDIA_TO_EPUB_SUP_START__", "<sup>")
        .replace("__WIKIPEDIA_TO_EPUB_SUP_END__", "</sup>")
}

fn restore_sub_spans(html: &str) -> String {
    html.replace("__WIKIPEDIA_TO_EPUB_SUB_START__", "<sub>")
        .replace("__WIKIPEDIA_TO_EPUB_SUB_END__", "</sub>")
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
    links_to_excluded_pages: LinksToExcludedPages,
) -> String {
    let placeholder = format!("__WIKIPEDIA_TO_EPUB_LINK_{}__", links.len());
    links.push(wikipedia_link_html(
        target,
        label,
        internal_links,
        language,
        links_to_excluded_pages,
    ));
    placeholder
}

fn wikipedia_link_html(
    target: &str,
    label: &str,
    internal_links: &InternalLinks,
    language: &str,
    links_to_excluded_pages: LinksToExcludedPages,
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

    if let Some(way_id) = target.strip_prefix("osmway:") {
        return format!(
            r#"<a href="{}">{}</a><span class="external-link">↗</span>"#,
            encode_double_quoted_attribute(&openstreetmap_way_url(way_id)),
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

    match links_to_excluded_pages {
        LinksToExcludedPages::Display => format!(
            r#"<a href="{}">{}</a>"#,
            encode_double_quoted_attribute(&wikipedia_article_url(target, language)),
            format_inline_text(label)
        ),
        LinksToExcludedPages::Emphasize => format!(
            r#"<a href="{}">{}</a><span class="external-link">↗</span>"#,
            encode_double_quoted_attribute(&wikipedia_article_url(target, language)),
            format_inline_text(label)
        ),
        LinksToExcludedPages::Disregard => format_inline_text(label),
    }
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

fn html_report_path(output_file: &Path) -> PathBuf {
    output_file.with_extension("html")
}

fn collect_excluded_article_links(
    loaded_pages: &HashMap<String, PageResponse>,
    ordered_articles: &[String],
) -> Vec<(String, Vec<String>)> {
    let included = ordered_articles
        .iter()
        .map(|title| normalize_lookup_key(title))
        .collect::<std::collections::HashSet<_>>();

    let mut excluded: BTreeMap<String, (String, BTreeSet<String>)> = BTreeMap::new();

    for article in ordered_articles {
        let lookup_key = normalize_lookup_key(article);
        let Some(page) = loaded_pages.get(&lookup_key) else {
            continue;
        };

        for target in extract_internal_links(&page.parse.wikitext.text) {
            let article_target = target
                .split_once('#')
                .map_or(target.as_str(), |(article, _)| article)
                .trim();
            if article_target.is_empty() {
                continue;
            }

            let target_key = normalize_lookup_key(article_target);
            if included.contains(&target_key) {
                continue;
            }

            let display_title = article_target.replace('_', " ");
            excluded
                .entry(target_key)
                .or_insert_with(|| (display_title, BTreeSet::new()))
                .1
                .insert(page.parse.title.clone());
        }
    }

    excluded
        .into_values()
        .map(|(title, sources)| (title, sources.into_iter().collect()))
        .collect()
}

fn render_report_hierarchy(
    nodes: &[TocNode],
    included_page_urls: &HashMap<String, String>,
) -> String {
    fn should_include_node(node: &TocNode, included_page_urls: &HashMap<String, String>) -> bool {
        included_page_urls.contains_key(&node.file_name)
            || node
                .children
                .iter()
                .any(|child| should_include_node(child, included_page_urls))
    }

    if nodes.is_empty() {
        return "<p>No included pages.</p>".to_string();
    }

    fn render_node(node: &TocNode, included_page_urls: &HashMap<String, String>) -> String {
        let label = if let Some(url) = included_page_urls.get(&node.file_name) {
            format!(
                r#"<a href="{}">{}</a>"#,
                encode_double_quoted_attribute(url),
                encode_text(&node.title)
            )
        } else {
            encode_text(&node.title).into_owned()
        };

        if node.children.is_empty() {
            format!("<li>{label}</li>")
        } else {
            let children = node
                .children
                .iter()
                .filter(|child| should_include_node(child, included_page_urls))
                .map(|child| render_node(child, included_page_urls))
                .collect::<Vec<_>>()
                .join("\n");
            format!("<li>{label}\n<ul>\n{children}\n</ul>\n</li>")
        }
    }

    let items = nodes
        .iter()
        .filter(|node| should_include_node(node, included_page_urls))
        .map(|node| render_node(node, included_page_urls))
        .collect::<Vec<_>>()
        .join("\n");
    format!("<ul>\n{items}\n</ul>")
}

fn write_html_report(
    report_path: &Path,
    book_title: &str,
    wikipedia_language: &str,
    toc_nodes: &[TocNode],
    loaded_pages: &HashMap<String, PageResponse>,
    ordered_articles: &[String],
) -> AppResult<()> {
    if let Some(parent) = report_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    let generated_date = current_utc_date_string();
    let included_page_urls = loaded_pages
        .values()
        .map(|page| {
            (
                sanitize_chapter_filename(&page.parse.title),
                wikipedia_article_url(&page.parse.title, wikipedia_language),
            )
        })
        .collect::<HashMap<_, _>>();
    let included_hierarchy = render_report_hierarchy(toc_nodes, &included_page_urls);
    let excluded_articles = collect_excluded_article_links(loaded_pages, ordered_articles);

    let excluded_section = if excluded_articles.is_empty() {
        "<p>No excluded Wikipedia pages were linked from the included pages.</p>".to_string()
    } else {
        let items = excluded_articles
            .iter()
            .map(|(title, sources)| {
                let linked_from = sources
                    .iter()
                    .map(|source| encode_text(source).into_owned())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    r#"<li><a href="{}">{}</a> <small>(linked from: {})</small></li>"#,
                    encode_double_quoted_attribute(&wikipedia_article_url(
                        title,
                        wikipedia_language
                    )),
                    encode_text(title),
                    linked_from
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("<ul>\n{items}\n</ul>")
    };

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="{language}">
  <head>
    <meta charset="utf-8" />
    <title>{title} report</title>
    <style>
      body {{ font-family: sans-serif; line-height: 1.5; margin: 2rem; }}
      h1, h2 {{ margin-bottom: 0.5rem; }}
      ul {{ margin-top: 0.5rem; }}
      small {{ color: #555; }}
    </style>
  </head>
  <body>
    <h1>{title} report</h1>
    <p>Generated on {generated_date}.</p>
    <p>This report lists the included book hierarchy and the same-language Wikipedia pages that were linked but not included in the book.</p>
    <h2>Included pages</h2>
    {included_hierarchy}
    <h2>Linked Wikipedia pages not included</h2>
    {excluded_section}
  </body>
</html>
"#,
        language = encode_double_quoted_attribute(wikipedia_language),
        title = encode_text(book_title),
    );

    fs::write(report_path, html)?;
    Ok(())
}
fn openstreetmap_relation_url(relation_id: &str) -> String {
    let relation_id = relation_id.trim();
    format!("https://www.openstreetmap.org/relation/{relation_id}")
}

fn openstreetmap_way_url(way_id: &str) -> String {
    let way_id = way_id.trim();
    format!("https://www.openstreetmap.org/way/{way_id}")
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

fn extract_class_attr(attrs: &str) -> Option<String> {
    let re = Regex::new(r#"(?i)\bclass\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#).unwrap();
    if let Some(caps) = re.captures(attrs)
        && let Some(c) = caps.get(1).or_else(|| caps.get(2)).or_else(|| caps.get(3))
    {
        return Some(c.as_str().to_string());
    }
    None
}

fn table_marker_id(line: &str) -> Option<usize> {
    let line = line.trim();
    if line.starts_with("__WIKIPEDIA_TO_EPUB_TABLE_") && line.ends_with("__") {
        let number_str = &line["__WIKIPEDIA_TO_EPUB_TABLE_".len()..line.len() - 2];
        number_str.parse::<usize>().ok()
    } else {
        None
    }
}

#[cfg(test)]
fn render_wikitext_tables(
    text: &str,
    tables: &mut Vec<String>,
    internal_links: &InternalLinks,
    language: &str,
) -> String {
    render_wikitext_tables_with_excluded_links(
        text,
        tables,
        internal_links,
        language,
        LinksToExcludedPages::Emphasize,
    )
}

fn render_wikitext_tables_with_excluded_links(
    text: &str,
    tables: &mut Vec<String>,
    internal_links: &InternalLinks,
    language: &str,
    links_to_excluded_pages: LinksToExcludedPages,
) -> String {
    let mut output = String::with_capacity(text.len());
    let mut index = 0usize;

    while index < text.len() {
        let remaining = &text[index..];

        if let Some(after_open) = remaining.strip_prefix("{|") {
            // Collect the full balanced table block (depth-track nested tables)
            let block_start = index;
            let mut depth = 1usize;
            let mut scan = index + 2;
            while scan < text.len() {
                if text[scan..].starts_with("{|") {
                    depth += 1;
                    scan += 2;
                } else if text[scan..].starts_with("|}") {
                    depth -= 1;
                    scan += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    scan += text[scan..].chars().next().map_or(1, |c| c.len_utf8());
                }
            }
            let block_end = scan; // points to the char after |}
            index = block_end;

            // Extract the opening attribute string (first line after {|)
            let attrs_line = after_open.lines().next().unwrap_or("").trim();

            if is_wikitable_attrs(attrs_line) {
                // Render the wikitable block (everything between {| and |})
                let inner = &text[block_start + 2..block_end - 2];
                let rendered = render_wikitable(
                    inner,
                    attrs_line,
                    internal_links,
                    language,
                    links_to_excluded_pages,
                );
                let table_id = tables.len();
                tables.push(rendered);
                output.push_str(&format!("__WIKIPEDIA_TO_EPUB_TABLE_{}__", table_id));
                output.push('\n');
            } else {
                if let Some(class_str) = extract_class_attr(attrs_line) {
                    warn!(class = %class_str, "Skipping table with unrecognized class: {}", class_str);
                } else {
                    debug!(
                        attrs = attrs_line,
                        "skipping non-wikitable table with no class"
                    );
                }
            }
            continue;
        }

        let ch = remaining.chars().next().unwrap();
        output.push(ch);
        index += ch.len_utf8();
    }

    output
}

/// Returns true when the table opening attribute string declares a `wikitable` class.
fn is_wikitable_attrs(attrs: &str) -> bool {
    // Match class=wikitable (unquoted) or class="...wikitable..." (quoted)
    let re = Regex::new(
        r#"(?i)class\s*=\s*(?:"[^"]*wikitable[^"]*"|'[^']*wikitable[^']*'|wikitable\b)"#,
    )
    .unwrap();
    re.is_match(attrs)
}

/// Renders the interior of a `{| ... |}` wikitable block as an XHTML `<table>`.
///
/// `inner` is everything between the opening `{|attrs` line and the closing `|}`.
/// Each cell's text content is cleaned through `cleanup_inline_markup`.
fn render_wikitable(
    inner: &str,
    attrs_line: &str,
    internal_links: &InternalLinks,
    language: &str,
    links_to_excluded_pages: LinksToExcludedPages,
) -> String {
    // Split into lines, skipping the opening attrs line (first line of inner)
    let lines: Vec<&str> = inner.lines().collect();

    // We build a list of rows; each row is a list of (is_header, content) cells.
    struct Cell {
        is_header: bool,
        content: String,
    }

    let mut rows: Vec<Vec<Cell>> = Vec::new();
    let mut current_row: Vec<Cell> = Vec::new();
    // skip_line_index: first line is the attrs line, skip it
    let start = if lines.first().map(|l| {
        !l.trim_start().starts_with('|')
            && !l.trim_start().starts_with('!')
            && !l.trim_start().starts_with('{')
    }) == Some(true)
    {
        1
    } else {
        0
    };

    for line in &lines[start..] {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with("{|") {
            // Nested table open — skip (it's inside the block, already depth-tracked)
            continue;
        }

        if trimmed.starts_with("|}") {
            // Nested table close — skip
            continue;
        }

        if trimmed == "|-" || trimmed.starts_with("|-") {
            // Row separator: commit current row if non-empty
            if !current_row.is_empty() {
                rows.push(std::mem::take(&mut current_row));
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("!!") {
            // Continuation header cells on the same line (rare, but handle it)
            for cell_raw in rest.split("!!") {
                let content = extract_cell_content(cell_raw);
                current_row.push(Cell {
                    is_header: true,
                    content: content.to_string(),
                });
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix('!') {
            // Header cell(s): split on !!
            for cell_raw in rest.split("!!") {
                let content = extract_cell_content(cell_raw);
                current_row.push(Cell {
                    is_header: true,
                    content: content.to_string(),
                });
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("||") {
            // Continuation data cells on the same line
            for cell_raw in rest.split("||") {
                let content = extract_cell_content(cell_raw);
                current_row.push(Cell {
                    is_header: false,
                    content: content.to_string(),
                });
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix('|') {
            // Data cell(s): split on ||
            for cell_raw in rest.split("||") {
                let content = extract_cell_content(cell_raw);
                current_row.push(Cell {
                    is_header: false,
                    content: content.to_string(),
                });
            }
            continue;
        }

        // Caption or other — skip
    }

    // Commit the final row
    if !current_row.is_empty() {
        rows.push(current_row);
    }

    if rows.is_empty() {
        return String::new();
    }

    // Render to XHTML
    let class_attr = extract_class_attr(attrs_line).unwrap_or_else(|| "wikitable".to_string());
    let mut html = String::new();
    html.push_str(&format!(
        "<table class=\"{}\">\n",
        encode_double_quoted_attribute(&class_attr)
    ));

    // Determine if the first row is all-header to wrap in <thead>
    let first_all_header = rows
        .first()
        .map(|r| r.iter().all(|c| c.is_header))
        .unwrap_or(false);
    let (header_rows, body_rows) = if first_all_header {
        (&rows[..1], &rows[1..])
    } else {
        (&rows[..0], &rows[..])
    };

    if !header_rows.is_empty() {
        html.push_str("  <thead>\n");
        for row in header_rows {
            html.push_str("    <tr>\n");
            for cell in row {
                let cleaned = cleanup_inline_markup_with_excluded_links(
                    &cell.content,
                    internal_links,
                    language,
                    links_to_excluded_pages,
                );
                let tag = if cell.is_header { "th" } else { "td" };
                html.push_str(&format!("      <{tag}>{cleaned}</{tag}>\n"));
            }
            html.push_str("    </tr>\n");
        }
        html.push_str("  </thead>\n");
    }

    if !body_rows.is_empty() {
        html.push_str("  <tbody>\n");
        for row in body_rows {
            html.push_str("    <tr>\n");
            for cell in row {
                let cleaned = cleanup_inline_markup_with_excluded_links(
                    &cell.content,
                    internal_links,
                    language,
                    links_to_excluded_pages,
                );
                let tag = if cell.is_header { "th" } else { "td" };
                html.push_str(&format!("      <{tag}>{cleaned}</{tag}>\n"));
            }
            html.push_str("    </tr>\n");
        }
        html.push_str("  </tbody>\n");
    }

    html.push_str("</table>");
    html
}

/// Extracts the visible cell content from a wikitext cell string.
///
/// Wikitext cells can have the form `attrs | content` where `attrs` contains
/// HTML-like attributes (e.g. `align="right"`, `rowspan=5`).  When a bare `|`
/// separator is present the part after it is the content; otherwise the whole
/// string is the content.
fn extract_cell_content(cell: &str) -> &str {
    let trimmed = cell.trim();
    // A cell attribute prefix contains `=` (e.g. `align="right"`, `rowspan=2`).
    // Split on the FIRST `|`; if the left part looks like attributes (contains `=`)
    // use the right part as content.  Otherwise treat the whole string as content.
    if let Some(pipe_pos) = trimmed.find('|') {
        let possible_attrs = &trimmed[..pipe_pos];
        if possible_attrs.contains('=') {
            return trimmed[pipe_pos + 1..].trim();
        }
    }
    trimmed
}

fn strip_file_links(text: &str) -> String {
    process_file_links(text, None, &InternalLinks::new(), "en", "")
}

fn process_file_links(
    text: &str,
    image_registry: Option<&mut ImageRegistry>,
    internal_links: &InternalLinks,
    language: &str,
    source_page: &str,
) -> String {
    process_file_links_with_excluded_links(
        text,
        image_registry,
        internal_links,
        language,
        LinksToExcludedPages::Emphasize,
        source_page,
    )
}

fn process_file_links_with_excluded_links(
    text: &str,
    mut image_registry: Option<&mut ImageRegistry>,
    internal_links: &InternalLinks,
    language: &str,
    links_to_excluded_pages: LinksToExcludedPages,
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
                && let Some(file_link) =
                    parse_file_link(content, internal_links, language, links_to_excluded_pages)
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

fn parse_file_link(
    content: &str,
    internal_links: &InternalLinks,
    language: &str,
    links_to_excluded_pages: LinksToExcludedPages,
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
            alt = Some(cleanup_inline_markup_with_excluded_links(
                &render_templates(value.trim()),
                internal_links,
                language,
                links_to_excluded_pages,
            ));
            continue;
        }

        if file_link_param_is_option(param) {
            continue;
        }

        caption = Some(cleanup_inline_markup_with_excluded_links(
            &render_templates(param),
            internal_links,
            language,
            links_to_excluded_pages,
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

#[cfg(test)]
mod tests;
