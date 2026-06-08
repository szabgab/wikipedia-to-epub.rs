use crate::cache::PageResponse;
use crate::config::{BookConfig, LinksToExcludedPages, Metadata, current_utc_date_string};
use crate::error::AppResult;
use crate::image::{ImageRegistry, ResolvedImage};
use crate::{
    InternalLinks, TemplateSkipCounts, cleanup_inline_markup, normalize_lookup_key,
    render_wikitext_with_template_counts_and_excluded_links,
};
use html_escape::{encode_double_quoted_attribute, encode_text};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tracing::info;
use zip::{
    CompressionMethod, ZipWriter,
    write::{FileOptions, SimpleFileOptions},
};

#[derive(Debug)]
pub struct Chapter {
    pub file_name: String,
    pub title: String,
    pub content: String,
    pub template_skip_counts: TemplateSkipCounts,
}

#[derive(Debug, Clone)]
pub struct TocNode {
    pub title: String,
    pub file_name: String,
    pub children: Vec<TocNode>,
}

pub fn write_epub(
    config: &BookConfig,
    chapters: &[Chapter],
    images: &[ResolvedImage],
    wikipedia_language: &str,
    toc_nodes: &[TocNode],
    cover_image: &Option<(Vec<u8>, String, &'static str)>,
) -> AppResult<()> {
    let generated_date = current_utc_date_string();

    if let Some(parent) = config.output_file.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    let identifier = config.id.clone().unwrap_or_else(book_identifier);
    info!(identifier = identifier, "book id");
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

    if let Some((bytes, ext, _media_type)) = cover_image {
        let cover_xhtml = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="{wikipedia_language}">
  <head>
    <title>Cover</title>
    <style type="text/css">
      body {{
        margin: 0;
        padding: 0;
        text-align: center;
        background-color: #ffffff;
      }}
      img {{
        max-width: 100%;
        height: auto;
      }}
    </style>
  </head>
  <body>
    <div style="text-align: center; page-break-inside: avoid;">
      <img src="cover_image.{ext}" alt="Cover" />
    </div>
  </body>
</html>
"#,
        );
        zip.start_file("OEBPS/cover.xhtml", deflated)?;
        zip.write_all(cover_xhtml.as_bytes())?;

        zip.start_file(format!("OEBPS/cover_image.{}", ext), deflated)?;
        zip.write_all(bytes)?;
    }

    let frontmatter = frontmatter_xhtml(&config.metadata, wikipedia_language, &generated_date);
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

    let nav = nav_xhtml(toc_nodes, wikipedia_language);
    zip.start_file("OEBPS/nav.xhtml", deflated)?;
    zip.write_all(nav.as_bytes())?;

    let toc = toc_ncx(&identifier, config, toc_nodes);
    zip.start_file("OEBPS/toc.ncx", deflated)?;
    zip.write_all(toc.as_bytes())?;

    let cover_info = cover_image
        .as_ref()
        .map(|(_, ext, media_type)| (ext.as_str(), *media_type));
    let package = content_opf(
        &identifier,
        config,
        chapters,
        images,
        cover_info,
        &generated_date,
    );
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

fn frontmatter_xhtml(
    metadata: &Metadata,
    wikipedia_language: &str,
    generated_date: &str,
) -> String {
    let internal_links = InternalLinks::new();
    let license = metadata
        .license
        .as_deref()
        .map(|license| cleanup_inline_markup(license, &internal_links, wikipedia_language))
        .unwrap_or_default();
    let date = cleanup_inline_markup(generated_date, &internal_links, wikipedia_language);
    let edition = cleanup_inline_markup(&metadata.edition, &internal_links, wikipedia_language);

    let mut details = vec![format!(
        "<p><strong>Author:</strong> {}</p>",
        encode_text(&metadata.author)
    )];

    details.push(format!("<p><strong>Edition:</strong> {edition}</p>"));

    details.push(format!("<p><strong>Date:</strong> {date}</p>"));

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

fn render_nav_node(node: &TocNode) -> String {
    if node.children.is_empty() {
        format!(
            r#"<li><a href="{}">{}</a></li>"#,
            encode_text(&node.file_name),
            encode_text(&node.title)
        )
    } else {
        let child_items = node
            .children
            .iter()
            .map(render_nav_node)
            .collect::<Vec<_>>()
            .join("\n          ");
        format!(
            r#"<li>
          <a href="{}">{}</a>
          <ol>
            {}
          </ol>
        </li>"#,
            encode_text(&node.file_name),
            encode_text(&node.title),
            child_items
        )
    }
}

fn nav_xhtml(toc_nodes: &[TocNode], language: &str) -> String {
    let mut items = vec![format!(
        r#"<li><a href="frontmatter.xhtml">Front matter</a></li>"#
    )];

    for node in toc_nodes {
        items.push(render_nav_node(node));
    }

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
        {}
      </ol>
    </nav>
  </body>
</html>
"#,
        items.join("\n        "),
        language_attributes = html_language_attributes(language),
    )
}

fn content_opf(
    identifier: &str,
    config: &BookConfig,
    chapters: &[Chapter],
    images: &[ResolvedImage],
    cover_info: Option<(&str, &str)>,
    generated_date: &str,
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

    if let Some((ext, media_type)) = cover_info {
        manifest_items.push(
            r#"<item id="cover" href="cover.xhtml" media-type="application/xhtml+xml"/>"#
                .to_string(),
        );
        manifest_items.push(format!(
            r#"<item id="cover-image" href="cover_image.{ext}" media-type="{media_type}" properties="cover-image"/>"#
        ));
        spine_items.insert(0, r#"<itemref idref="cover"/>"#.to_string());
    }

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
    let date_line = format!("<dc:date>{}</dc:date>", encode_text(generated_date));

    let meta_line = if cover_info.is_some() {
        "\n    <meta name=\"cover\" content=\"cover-image\"/>".to_string()
    } else {
        String::new()
    };

    let guide_section = if cover_info.is_some() {
        r#"
  <guide>
    <reference type="cover" title="Cover" href="cover.xhtml"/>
  </guide>"#
    } else {
        ""
    };

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<package version="2.0" unique-identifier="bookid" xmlns="http://www.idpf.org/2007/opf">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="bookid">{identifier}</dc:identifier>
    <dc:title>{title}</dc:title>
    <dc:creator>{creator}</dc:creator>
    <dc:language>{language}</dc:language>
    {date_line}
    {rights_line}{meta_line}
  </metadata>
  <manifest>
    {manifest}
  </manifest>
  <spine toc="ncx">
    <itemref idref="nav" linear="no"/>
    {spine}
  </spine>{guide_section}
</package>
"#,
        title = encode_text(&config.metadata.title),
        creator = encode_text(&config.metadata.author),
        language = encode_text(&config.metadata.language),
        date_line = date_line,
        rights_line = if rights.is_empty() {
            String::new()
        } else {
            format!("<dc:rights>{rights}</dc:rights>")
        },
        manifest = manifest_items.join("\n    "),
        spine = spine_items.join("\n    "),
    )
}

fn render_ncx_nav_point(node: &TocNode, play_order: &mut usize) -> String {
    *play_order += 1;
    let order = *play_order;
    let id_suffix = node
        .file_name
        .strip_prefix("chapter-")
        .unwrap_or(&node.file_name)
        .strip_suffix(".xhtml")
        .unwrap_or(&node.file_name);
    let id = format!("chapter-{id_suffix}");

    if node.children.is_empty() {
        format!(
            r#"<navPoint id="{id}" playOrder="{order}">
      <navLabel><text>{title}</text></navLabel>
      <content src="{file}"/>
    </navPoint>"#,
            title = encode_text(&node.title),
            file = encode_text(&node.file_name),
        )
    } else {
        let mut child_navs = Vec::new();
        for child in &node.children {
            child_navs.push(render_ncx_nav_point(child, play_order));
        }
        format!(
            r#"<navPoint id="{id}" playOrder="{order}">
      <navLabel><text>{title}</text></navLabel>
      <content src="{file}"/>
      {}
    </navPoint>"#,
            child_navs.join("\n      "),
            title = encode_text(&node.title),
            file = encode_text(&node.file_name),
        )
    }
}

fn toc_ncx(identifier: &str, config: &BookConfig, toc_nodes: &[TocNode]) -> String {
    let mut play_order = 1;
    let mut nav_points = vec![
        r#"<navPoint id="frontmatter" playOrder="1">
      <navLabel><text>Front matter</text></navLabel>
      <content src="frontmatter.xhtml"/>
    </navPoint>"#
            .to_string(),
    ];

    for node in toc_nodes {
        nav_points.push(render_ncx_nav_point(node, &mut play_order));
    }

    fn get_max_depth(nodes: &[TocNode]) -> usize {
        nodes
            .iter()
            .map(|node| 1 + get_max_depth(&node.children))
            .max()
            .unwrap_or(0)
    }
    let depth = get_max_depth(toc_nodes).max(1);

    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="{identifier}"/>
    <meta name="dtb:depth" content="{depth}"/>
    <meta name="dtb:totalPageCount" content="0"/>
    <meta name="dtb:maxPageNumber" content="0"/>
  </head>
  <docTitle><text>{title}</text></docTitle>
  <navMap>
    {}
  </navMap>
</ncx>
"#,
        nav_points.join("\n    "),
        identifier = encode_text(identifier),
        title = encode_text(&config.metadata.title),
        depth = depth,
    )
}

fn book_identifier() -> String {
    let id = uuid::Uuid::new_v4();
    format!("urn:uuid:{id}")
}

pub fn sanitize_chapter_filename(title: &str) -> String {
    let ascii_title = any_ascii::any_ascii(title);
    let sanitized: String = ascii_title
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("{}.xhtml", sanitized)
}

pub fn internal_links(articles: &[String]) -> InternalLinks {
    let mut links = InternalLinks::new();
    for article in articles {
        links
            .entry(normalize_lookup_key(article))
            .or_insert_with(|| sanitize_chapter_filename(article));
    }
    links
}

pub fn load_markdown_chapter(path: &Path, language: &str) -> AppResult<Chapter> {
    let content = fs::read_to_string(path)?;
    let mut title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Front Matter")
        .to_string();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(header) = trimmed.strip_prefix("# ") {
            title = header.trim().to_string();
            break;
        }
    }

    let file_name = format!(
        "{}.xhtml",
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("front-page")
    );

    let parser = pulldown_cmark::Parser::new(&content);
    let mut html_content = String::new();
    pulldown_cmark::html::push_html(&mut html_content, parser);

    let xhtml = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="{}">
  <head>
    <title>{}</title>
    <link rel="stylesheet" type="text/css" href="style.css" />
  </head>
  <body>
    {}
  </body>
</html>
"#,
        language,
        encode_text(&title),
        html_content,
    );

    Ok(Chapter {
        file_name,
        title,
        content: xhtml,
        template_skip_counts: TemplateSkipCounts::default(),
    })
}

pub fn load_chapter(
    page: &PageResponse,
    display_title: String,
    internal_links: &InternalLinks,
    language: &str,
    links_to_excluded_pages: LinksToExcludedPages,
    image_registry: Option<&mut ImageRegistry>,
) -> AppResult<Chapter> {
    let (rendered, template_skip_counts) = render_wikitext_with_template_counts_and_excluded_links(
        &display_title,
        &page.parse.wikitext.text,
        internal_links,
        language,
        links_to_excluded_pages,
        image_registry,
    );
    info!(
        article = page.parse.title,
        title = page.parse.title,
        recognized_skipped_templates = template_skip_counts.recognized,
        unknown_skipped_templates = template_skip_counts.unknown,
        "article template skip counts"
    );

    let file_name = sanitize_chapter_filename(&page.parse.title);
    Ok(Chapter {
        title: display_title,
        file_name,
        content: rendered,
        template_skip_counts,
    })
}

pub fn html_language_attributes(language: &str) -> String {
    let language = encode_double_quoted_attribute(language);
    if is_right_to_left_language(&language) {
        format!(r#"xml:lang="{language}" dir="rtl""#)
    } else {
        format!(r#"xml:lang="{language}""#)
    }
}

pub fn is_right_to_left_language(language: &str) -> bool {
    let base_language = language.split_once('-').map_or(language, |(base, _)| base);
    matches!(base_language, "ar" | "fa" | "he" | "ur")
}
