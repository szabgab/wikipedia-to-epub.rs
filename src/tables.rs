use html_escape::encode_double_quoted_attribute;
use regex::Regex;

use crate::InternalLinks;
use crate::cleanup_inline_markup_with_excluded_links;
use crate::config::LinksToExcludedPages;
use crate::image::ImageRegistry;

struct Cell {
    is_header: bool,
    content: String,
}

pub(crate) fn extract_class_attr(attrs: &str) -> Option<String> {
    let re = Regex::new(r#"(?i)\bclass\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#).unwrap();
    if let Some(caps) = re.captures(attrs)
        && let Some(c) = caps.get(1).or_else(|| caps.get(2)).or_else(|| caps.get(3))
    {
        return Some(c.as_str().to_string());
    }
    None
}

pub(crate) fn table_marker_id(line: &str) -> Option<usize> {
    let line = line.trim();
    if line.starts_with("__WIKIPEDIA_TO_EPUB_TABLE_") && line.ends_with("__") {
        let number_str = &line["__WIKIPEDIA_TO_EPUB_TABLE_".len()..line.len() - 2];
        number_str.parse::<usize>().ok()
    } else {
        None
    }
}

#[cfg(test)]
pub(crate) fn render_wikitext_tables(
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
        None,
        "",
    )
}

pub(crate) fn render_wikitext_tables_with_excluded_links(
    text: &str,
    tables: &mut Vec<String>,
    internal_links: &InternalLinks,
    language: &str,
    links_to_excluded_pages: LinksToExcludedPages,
    mut image_registry: Option<&mut ImageRegistry>,
    source_page: &str,
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

            if is_wikitable_attrs(attrs_line) || extract_class_attr(attrs_line).is_none() {
                // Render the wikitable block (everything between {| and |})
                let inner = &text[block_start + 2..block_end - 2];
                let rendered = render_wikitable(
                    inner,
                    attrs_line,
                    internal_links,
                    language,
                    links_to_excluded_pages,
                    image_registry.as_deref_mut(),
                    source_page,
                );
                let table_id = tables.len();
                tables.push(rendered);
                output.push_str(&format!("__WIKIPEDIA_TO_EPUB_TABLE_{}__", table_id));
                output.push('\n');
            } else {
                let class_str = extract_class_attr(attrs_line).unwrap_or_default();
                tracing::warn!(class = %class_str, "Skipping table with unrecognized class: {}", class_str);
            }
            continue;
        }

        let ch = remaining.chars().next().unwrap();
        output.push(ch);
        index += ch.len_utf8();
    }

    output
}

pub(crate) fn is_wikitable_attrs(attrs: &str) -> bool {
    // Match class=wikitable (unquoted) or class="...wikitable..." (quoted)
    let re = Regex::new(
        r#"(?i)class\s*=\s*(?:"[^"]*wikitable[^"]*"|'[^']*wikitable[^']*'|wikitable\b)"#,
    )
    .unwrap();
    re.is_match(attrs)
}

pub(crate) fn render_wikitable(
    inner: &str,
    attrs_line: &str,
    internal_links: &InternalLinks,
    language: &str,
    links_to_excluded_pages: LinksToExcludedPages,
    mut image_registry: Option<&mut ImageRegistry>,
    source_page: &str,
) -> String {
    // Split into lines, skipping the opening attrs line (first line of inner)
    let lines: Vec<&str> = inner.lines().collect();

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

    let class_attr = extract_class_attr(attrs_line).unwrap_or_else(|| "wikitable".to_string());
    let mut html = String::new();

    // Determine the maximum column count across all rows
    let col_count = rows.iter().map(|r| r.len()).max().unwrap_or(0);

    if col_count == 1 {
        // 1 column: Simple Bulleted List
        html.push_str(&format!(
            "<ul class=\"responsive-list {}\">\n",
            encode_double_quoted_attribute(&class_attr)
        ));
        for row in &rows {
            for cell in row {
                let cleaned = cleanup_inline_markup_with_excluded_links(
                    &cell.content,
                    image_registry.as_deref_mut(),
                    internal_links,
                    language,
                    links_to_excluded_pages,
                    source_page,
                );
                html.push_str(&format!("  <li>{}</li>\n", cleaned));
            }
        }
        html.push_str("</ul>\n");
    } else if col_count == 2 {
        // 2 columns: Description List
        html.push_str(&format!(
            "<dl class=\"responsive-dl {}\">\n",
            encode_double_quoted_attribute(&class_attr)
        ));
        for row in &rows {
            if row.is_empty() {
                continue;
            }
            if row.len() == 1 {
                let val = cleanup_inline_markup_with_excluded_links(
                    &row[0].content,
                    image_registry.as_deref_mut(),
                    internal_links,
                    language,
                    links_to_excluded_pages,
                    source_page,
                );
                html.push_str(&format!("  <dd>{}</dd>\n", val));
            } else {
                let key = cleanup_inline_markup_with_excluded_links(
                    &row[0].content,
                    image_registry.as_deref_mut(),
                    internal_links,
                    language,
                    links_to_excluded_pages,
                    source_page,
                );
                html.push_str(&format!("  <dt>{}</dt>\n", key));
                for cell in &row[1..] {
                    let val = cleanup_inline_markup_with_excluded_links(
                        &cell.content,
                        image_registry.as_deref_mut(),
                        internal_links,
                        language,
                        links_to_excluded_pages,
                        source_page,
                    );
                    html.push_str(&format!("  <dd>{}</dd>\n", val));
                }
            }
        }
        html.push_str("</dl>\n");
    } else {
        // 3+ columns: Linearized Card Layout
        html.push_str(&format!(
            "<div class=\"responsive-card-container {}\">\n",
            encode_double_quoted_attribute(&class_attr)
        ));

        // Determine if the first row is all-header to wrap/treat as headers
        let has_headers = rows
            .first()
            .map(|r| r.iter().all(|c| c.is_header))
            .unwrap_or(false);

        let headers: Vec<String> = if has_headers {
            rows[0]
                .iter()
                .map(|cell| {
                    cleanup_inline_markup_with_excluded_links(
                        &cell.content,
                        image_registry.as_deref_mut(),
                        internal_links,
                        language,
                        links_to_excluded_pages,
                        source_page,
                    )
                })
                .collect()
        } else {
            Vec::new()
        };

        let start_idx = if has_headers { 1 } else { 0 };
        for row in &rows[start_idx..] {
            if row.is_empty() {
                continue;
            }

            html.push_str("  <div class=\"responsive-card\">\n");

            // Primary column acts as card title
            let title = cleanup_inline_markup_with_excluded_links(
                &row[0].content,
                image_registry.as_deref_mut(),
                internal_links,
                language,
                links_to_excluded_pages,
                source_page,
            );
            html.push_str(&format!("    <div class=\"card-title\">{}</div>\n", title));

            if row.len() > 1 {
                html.push_str("    <ul class=\"card-list\">\n");
                for (i, cell) in row.iter().skip(1).enumerate() {
                    let label = headers.get(i + 1).cloned().unwrap_or_default();
                    let cleaned_val = cleanup_inline_markup_with_excluded_links(
                        &cell.content,
                        image_registry.as_deref_mut(),
                        internal_links,
                        language,
                        links_to_excluded_pages,
                        source_page,
                    );
                    html.push_str("      <li>");
                    if !label.is_empty() {
                        html.push_str(&format!("<strong>{}:</strong> ", label));
                    }
                    html.push_str(&format!("{}</li>\n", cleaned_val));
                }
                html.push_str("    </ul>\n");
            }
            html.push_str("  </div>\n");
        }
        html.push_str("</div>\n");
    }

    html
}

pub(crate) fn extract_cell_content(cell: &str) -> &str {
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
