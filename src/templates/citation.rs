use std::collections::HashMap;

use crate::types::{DispatchTable, PersonRole, TemplateHandler};

use crate::tools::{
    person_first_keys, person_last_keys, person_link_keys, template_named_params, template_param,
    template_param_owned, template_positional_params,
};

use crate::split_template_params;
use crate::templates::render_templates;

pub(crate) fn citation_people(named: &HashMap<String, String>, role: PersonRole) -> String {
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

/// [citation needed span](https://en.wikipedia.org/wiki/Template:Citation_needed_span)
fn render_citation_needed_span_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let text = named
        .get("1")
        .map(|s| s.as_str())
        .or_else(|| positional.first().map(|s| s.as_str()))
        .unwrap_or("");

    render_templates(text)
}

/// [cite web](https://en.wikipedia.org/wiki/Template:Cite_web)
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

/// [cite book](https://en.wikipedia.org/wiki/Template:Cite_book)
fn render_cite_book_template(params: &str) -> String {
    render_citation_template(params)
}

/// [cite journal](https://en.wikipedia.org/wiki/Template:Cite_journal)
/// [cite magazine](https://en.wikipedia.org/wiki/Template:Cite_magazine)
/// [cite news](https://en.wikipedia.org/wiki/Template:Cite_news)
/// [cite encyclopedia](https://en.wikipedia.org/wiki/Template:Cite_encyclopedia)
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

    if let Some(journal) = template_param(
        &named,
        &[
            "journal",
            "work",
            "website",
            "magazine",
            "newspaper",
            "periodical",
            "encyclopedia",
        ],
    ) {
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

/// [cite report](https://en.wikipedia.org/wiki/Template:Cite_report)
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

/// [cite ECCP](https://en.wikipedia.org/wiki/Template:Cite_ECCP)
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

/// [cite gvp](https://en.wikipedia.org/wiki/Template:Cite_gvp)
fn render_cite_gvp_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let name = template_param(&named, &["name", "1"])
        .map(|s| s.to_string())
        .or_else(|| positional.first().cloned())
        .unwrap_or_default();

    let access_date = template_param(&named, &["access-date", "accessdate"])
        .map(|s| s.to_string())
        .unwrap_or_default();

    let name = name.trim();
    let access_date = access_date.trim();

    if name.is_empty() {
        return String::new();
    }

    let mut parts = Vec::new();
    parts.push(format!("\"{}\"", render_templates(name)));
    parts.push("''Global Volcanism Program''".to_string());
    parts.push("Smithsonian Institution".to_string());

    if !access_date.is_empty() {
        parts.push(format!("Retrieved {}", render_templates(access_date)));
    }

    parts.join(". ")
}

fn format_harvard_citation(params: &str) -> String {
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

    parts.join(", ")
}

/// [harvp](https://en.wikipedia.org/wiki/Template:Harvp)
/// [harv](https://en.wikipedia.org/wiki/Template:Harv)
fn render_harvp_template(params: &str) -> String {
    let formatted = format_harvard_citation(params);
    if formatted.is_empty() {
        String::new()
    } else {
        format!("({formatted})")
    }
}

/// [harvnb](https://en.wikipedia.org/wiki/Template:Harvnb)
fn render_harvnb_template(params: &str) -> String {
    format_harvard_citation(params)
}

/// [harvtxt](https://en.wikipedia.org/wiki/Template:Harvtxt)
fn render_harvtxt_template(params: &str) -> String {
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

    let mut inner_parts = Vec::new();
    if let Some(y) = year {
        inner_parts.push(y);
    }
    if let Some(page) = template_param(&named, &["p"]) {
        inner_parts.push(format!("p. {}", render_templates(page.trim())));
    } else if let Some(pages) = template_param(&named, &["pp"]) {
        inner_parts.push(format!("pp. {}", render_templates(pages.trim())));
    }
    if let Some(location) = template_param(&named, &["loc"]) {
        inner_parts.push(render_templates(location.trim()));
    }

    let inner_formatted = inner_parts.join(", ");

    if authors_formatted.is_empty() {
        if inner_formatted.is_empty() {
            String::new()
        } else {
            format!("({inner_formatted})")
        }
    } else if inner_formatted.is_empty() {
        authors_formatted
    } else {
        format!("{} ({})", authors_formatted, inner_formatted)
    }
}

/// [Cite NSRW](https://en.wikipedia.org/wiki/Template:Cite_NSRW)
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

/// [cite conference](https://en.wikipedia.org/wiki/Template:Cite_conference)
/// [citation](https://en.wikipedia.org/wiki/Template:Citation)
/// [cite thesis](https://en.wikipedia.org/wiki/Template:Cite_thesis)
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

/// [harvc](https://en.wikipedia.org/wiki/Template:Harvc)
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

/// [Cite EB1911](https://en.wikipedia.org/wiki/Template:Cite_EB1911)
fn render_cite_eb1911_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);
    let title = template_param(&named, &["wstitle", "title"])
        .or_else(|| positional.first().map(String::as_str))
        .map(|s| s.trim())
        .unwrap_or("");

    if title.is_empty() {
        return "''Encyclopædia Britannica'' (11th ed., 1911)".to_string();
    }

    format!(
        "\"{}\" in ''[[src:1911 Encyclopædia Britannica/{}|Encyclopædia Britannica]]'' (11th ed., 1911)",
        render_templates(title),
        title
    )
}

/// [cite dictionary](https://en.wikipedia.org/wiki/Template:Cite_dictionary)
fn render_cite_dictionary_template(params: &str) -> String {
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

    if let Some(dictionary) = template_param(&named, &["dictionary", "work"]) {
        parts.push(format!("''{}''", render_templates(dictionary)));
    }

    if let Some(edition) = template_param(&named, &["edition"]) {
        parts.push(format!("{} ed", render_templates(edition)));
    }

    if let Some(publisher) = template_param(&named, &["publisher"]) {
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

/// [cite press release](https://en.wikipedia.org/wiki/Template:Cite_press_release)
fn render_cite_press_release_template(params: &str) -> String {
    let named = template_named_params(params);
    let mut parts = Vec::new();

    let authors = citation_people(&named, PersonRole::Author);
    if !authors.is_empty() {
        parts.push(authors);
    }

    if let Some(title) = template_param(&named, &["title"]) {
        let title_text = format!("\"{}\" (Press release)", render_templates(title));
        let title_link = match template_param(&named, &["url"]) {
            Some(url) => format!("[{} {}]", render_templates(url), title_text),
            None => title_text,
        };
        parts.push(title_link);
    }

    if let Some(publisher) = template_param(&named, &["publisher"]) {
        parts.push(render_templates(publisher));
    }

    if let Some(date) = template_param(&named, &["date", "year"]) {
        parts.push(render_templates(date));
    }

    parts.join(". ")
}

fn parse_date_to_yymmdd(date_str: &str) -> Option<String> {
    let s = date_str.trim().to_lowercase();
    if s.is_empty() {
        return None;
    }

    let cleaned = s.replace([',', '-', '/'], " ");
    let parts: Vec<&str> = cleaned.split_whitespace().collect();

    if parts.len() != 3 {
        return None;
    }

    let months = [
        ("jan", "01"),
        ("feb", "02"),
        ("mar", "03"),
        ("apr", "04"),
        ("may", "05"),
        ("jun", "06"),
        ("jul", "07"),
        ("aug", "08"),
        ("sep", "09"),
        ("oct", "10"),
        ("nov", "11"),
        ("dec", "12"),
    ];

    let mut year = None;
    let mut month = None;
    let mut day = None;

    for part in &parts {
        if let Some(m_idx) = months.iter().position(|&(name, _)| part.starts_with(name)) {
            month = Some(months[m_idx].1.to_string());
        } else if let Ok(num) = part.parse::<u32>() {
            if num > 1000 {
                year = Some(num);
            } else if day.is_none() {
                day = Some(num);
            } else {
                if month.is_none() && (1..=12).contains(&num) {
                    month = Some(format!("{:02}", num));
                } else if year.is_none() {
                    year = Some(num);
                }
            }
        }
    }

    if (year.is_none() || month.is_none() || day.is_none()) && parts[0].len() == 4 {
        let parsed = (
            parts[0].parse::<u32>(),
            parts[1].parse::<u32>(),
            parts[2].parse::<u32>(),
        );
        if let (Ok(y), Ok(m), Ok(d)) = parsed {
            year = Some(y);
            month = Some(format!("{:02}", m));
            day = Some(d);
        }
    }

    if let (Some(y), Some(m), Some(d)) = (year, month, day) {
        let yy = format!("{:02}", y % 100);
        let dd = format!("{:02}", d);
        Some(format!("{}{}{}", yy, m, dd))
    } else {
        None
    }
}

/// [Cite APOD](https://en.wikipedia.org/wiki/Template:Cite_APOD)
fn render_cite_apod_template(params: &str) -> String {
    let named = template_named_params(params);
    let mut parts = Vec::new();

    let mut header = "R. Nemiroff & J. Bonnell, eds.".to_string();
    if let Some(date) = template_param(&named, &["date"]) {
        header.push_str(&format!(" ({})", render_templates(date)));
    }
    parts.push(header);

    if let Some(title) = template_param(&named, &["title"]) {
        let date_param = template_param(&named, &["date"]).unwrap_or("");
        let url = if let Some(yymmdd) = parse_date_to_yymmdd(date_param) {
            format!("https://apod.nasa.gov/apod/ap{}.html", yymmdd)
        } else {
            "https://apod.nasa.gov/apod/astropix.html".to_string()
        };
        parts.push(format!("[{} \"{}\"]", url, render_templates(title)));
    }

    parts.push("''Astronomy Picture of the Day''".to_string());
    parts.push("NASA".to_string());

    if let Some(access_date) = template_param(&named, &["access-date", "accessdate"]) {
        parts.push(format!("Retrieved {}", render_templates(access_date)));
    }

    parts.join(". ")
}

/// [Cite OED](https://en.wikipedia.org/wiki/Template:Cite_OED)
fn render_cite_oed_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let entry = template_param(&named, &["term", "entry", "1"])
        .map(|s| s.to_string())
        .or_else(|| positional.first().cloned())
        .unwrap_or_default();
    let entry = entry.trim();

    let id = template_param(&named, &["id", "2"])
        .map(|s| s.to_string())
        .or_else(|| positional.get(1).cloned())
        .unwrap_or_default();
    let id = id.trim();

    let mut parts = Vec::new();

    if !entry.is_empty() {
        let url = if !id.is_empty() {
            if let Ok(id_num) = id.parse::<u64>() {
                if id_num > 999999999 {
                    format!("https://doi.org/10.1093/OED/{}", id)
                } else {
                    format!("https://www.oed.com/view/Entry/{}", id)
                }
            } else {
                format!("https://www.oed.com/view/Entry/{}", id)
            }
        } else {
            format!(
                "https://www.oed.com/search/dictionary/?q={}",
                entry.replace(' ', "%20")
            )
        };

        parts.push(format!("[{} \"{}\"]", url, render_templates(entry)));
    }

    parts.push("''Oxford English Dictionary'' (Online ed.)".to_string());
    parts.push("Oxford University Press".to_string());

    if let Some(date) = template_param(&named, &["date", "year"]) {
        parts.push(render_templates(date));
    }

    if let Some(access_date) = template_param(&named, &["access-date", "accessdate"]) {
        parts.push(format!("Retrieved {}", render_templates(access_date)));
    }

    parts.join(". ")
}

/// [Cite AV media](https://en.wikipedia.org/wiki/Template:Cite_AV_media)
fn render_cite_av_media_template(params: &str) -> String {
    let named = template_named_params(params);
    let mut parts = Vec::new();

    let authors = citation_people(&named, PersonRole::Author);
    if !authors.is_empty() {
        parts.push(authors);
    }

    if let Some(title) = template_param(&named, &["title", "script-title"]) {
        let title_link = match template_param(&named, &["url"]) {
            Some(url) => format!(
                "[{} \"{}\"]",
                render_templates(url),
                render_templates(title)
            ),
            None => format!("\"{}\"", render_templates(title)),
        };
        parts.push(title_link);
    }

    if let Some(format_val) = template_param(&named, &["format"]) {
        parts.push(format!("({})", render_templates(format_val)));
    }

    let publisher = template_param(&named, &["publisher"]);
    let via = template_param(&named, &["via"]);
    if let Some(pub_val) = publisher {
        parts.push(render_templates(pub_val));
    }
    if let Some(via_val) = via
        && publisher.is_none_or(|p| !p.eq_ignore_ascii_case(via_val))
    {
        parts.push(render_templates(via_val));
    }

    if let Some(date) = template_param(&named, &["date", "year"]) {
        parts.push(render_templates(date));
    }

    if let Some(access_date) = template_param(&named, &["access-date", "accessdate"]) {
        parts.push(format!("Retrieved {}", render_templates(access_date)));
    }

    parts.join(". ")
}

/// [Cite American Heritage Dictionary](https://en.wikipedia.org/wiki/Template:Cite_American_Heritage_Dictionary)
fn render_cite_american_heritage_dictionary_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let entry = template_param(&named, &["1"])
        .map(|s| s.to_string())
        .or_else(|| positional.first().cloned())
        .unwrap_or_default();
    let entry = entry.trim();

    if entry.is_empty() {
        return String::new();
    }

    let mut parts = Vec::new();
    let url = format!(
        "https://www.ahdictionary.com/word/search.html?q={}",
        entry.replace(' ', "%20")
    );

    parts.push(format!("[{} \"{}\"]", url, render_templates(entry)));
    parts.push("''The American Heritage Dictionary of the English Language''".to_string());

    if let Some(date) = template_param(&named, &["date", "year"]) {
        parts.push(render_templates(date));
    }

    if let Some(access_date) = template_param(&named, &["access-date", "accessdate"]) {
        parts.push(format!("Retrieved {}", render_templates(access_date)));
    }

    parts.join(". ")
}

/// [Cite wikisource](https://en.wikipedia.org/wiki/Template:Cite_wikisource)
fn render_cite_wikisource_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let mut parts = Vec::new();

    let authors = citation_people(&named, PersonRole::Author);
    if !authors.is_empty() {
        parts.push(authors);
    }

    let title = template_param(&named, &["title", "1"])
        .map(|s| s.to_string())
        .or_else(|| positional.first().cloned())
        .unwrap_or_default();
    let title = title.trim();

    let wslink = template_param(&named, &["wslink"])
        .map(|s| s.to_string())
        .unwrap_or_else(|| title.to_string());
    let wslink = wslink.trim();

    let wslanguage = template_param(&named, &["wslanguage", "3"])
        .map(|s| s.to_string())
        .or_else(|| positional.get(2).cloned())
        .unwrap_or_default();
    let wslanguage = wslanguage.trim();

    if !title.is_empty() {
        let link_target = if !wslanguage.is_empty() && wslanguage != "en" {
            format!("{}:{}", wslanguage, wslink)
        } else {
            wslink.to_string()
        };
        parts.push(format!("''[[src:{}|{}]]''", link_target, title));
    }

    let mut publication = String::new();
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

    parts.push("[[Wikisource]]".to_string());

    parts.join(". ")
}

/// [Cite CIA World Factbook](https://en.wikipedia.org/wiki/Template:Cite_CIA_World_Factbook)
fn render_cite_cia_world_factbook_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let country = template_param(&named, &["country", "1"])
        .map(|s| s.to_string())
        .or_else(|| positional.first().cloned())
        .unwrap_or_default();
    let country = country.trim();

    let section = template_param(&named, &["section"]).unwrap_or("").trim();
    let year = template_param(&named, &["year"]).unwrap_or("").trim();

    let mut parts = Vec::new();

    if !country.is_empty() {
        let title = if !section.is_empty() {
            format!("{} § {}", country, section)
        } else {
            country.to_string()
        };

        let kebab = country.to_lowercase().replace(' ', "-");
        let url = format!(
            "https://www.cia.gov/the-world-factbook/countries/{}/",
            kebab
        );

        parts.push(format!("[{} \"{}\"]", url, render_templates(&title)));
    }

    let work = if !year.is_empty() {
        format!("''The World Factbook'' ({} ed.)", render_templates(year))
    } else {
        "''The World Factbook''".to_string()
    };
    parts.push(work);

    parts.push("Central Intelligence Agency".to_string());

    if let Some(access_date) = template_param(&named, &["access-date", "accessdate"]) {
        parts.push(format!("Retrieved {}", render_templates(access_date)));
    }

    parts.join(". ")
}

/// [Cite letter](https://en.wikipedia.org/wiki/Template:Cite_letter)
fn render_cite_letter_template(params: &str) -> String {
    let named = template_named_params(params);
    let mut parts = Vec::new();

    let authors = citation_people(&named, PersonRole::Author);
    if !authors.is_empty() {
        parts.push(authors);
    }

    if let Some(subject) = template_param(&named, &["subject", "title"]) {
        let recipient = template_param(&named, &["recipient"]);
        let type_text = match recipient {
            Some(rec) => format!("Letter to {}", render_templates(rec)),
            None => "Letter".to_string(),
        };

        let title_text = format!("\"{}\" ({})", render_templates(subject), type_text);
        let title_link = match template_param(&named, &["url"]) {
            Some(url) => format!("[{} {}]", render_templates(url), title_text),
            None => title_text,
        };
        parts.push(title_link);
    }

    if let Some(publisher) = template_param(&named, &["publisher"]) {
        parts.push(render_templates(publisher));
    }

    if let Some(date) = template_param(&named, &["date", "year"]) {
        parts.push(render_templates(date));
    }

    if let Some(access_date) = template_param(&named, &["access-date", "accessdate"]) {
        parts.push(format!("Retrieved {}", render_templates(access_date)));
    }

    parts.join(". ")
}

/// [Cite arXiv](https://en.wikipedia.org/wiki/Template:Cite_arXiv)
fn render_cite_arxiv_template(params: &str) -> String {
    let named = template_named_params(params);
    let mut parts = Vec::new();

    let authors = citation_people(&named, PersonRole::Author);
    if !authors.is_empty() {
        parts.push(authors);
    }

    if let Some(title) = template_param(&named, &["title"]) {
        let title_link = match template_param(&named, &["url"]) {
            Some(url) => format!(
                "[{} \"{}\"]",
                render_templates(url),
                render_templates(title)
            ),
            None => format!("\"{}\"", render_templates(title)),
        };
        parts.push(title_link);
    }

    if let Some(date) = template_param(&named, &["date", "year"]) {
        parts.push(render_templates(date));
    }

    if let Some(eprint) = template_param(&named, &["eprint", "arxiv"]) {
        let eprint = eprint.trim();
        let class_str = if let Some(class_val) = template_param(&named, &["class"]) {
            format!(" [{}]", render_templates(class_val.trim()))
        } else {
            String::new()
        };

        parts.push(format!(
            "arXiv:[https://arxiv.org/abs/{} {}]{}",
            eprint, eprint, class_str
        ));
    }

    parts.join(". ")
}

/// [Cite Q](https://en.wikipedia.org/wiki/Template:Cite_Q)
fn render_cite_q_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let qid_raw = template_param(&named, &["1"])
        .map(|s| s.to_string())
        .or_else(|| positional.first().cloned())
        .unwrap_or_default();
    let qid_raw = qid_raw.trim();

    if qid_raw.is_empty() {
        return String::new();
    }

    let qid = if let Some(last_slash_idx) = qid_raw.rfind('/') {
        &qid_raw[last_slash_idx + 1..]
    } else {
        qid_raw
    };

    format!(
        "Wikidata item [https://www.wikidata.org/wiki/{} {}]",
        qid, qid
    )
}

/// [cite merriam-webster](https://en.wikipedia.org/wiki/Template:Cite_Merriam-Webster)
fn render_cite_merriam_webster_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let word = positional
        .first()
        .or_else(|| named.get("word"))
        .or_else(|| named.get("entry"))
        .cloned()
        .unwrap_or_default();
    let word_trimmed = word.trim();
    if word_trimmed.is_empty() {
        return String::new();
    }

    let dict_type = positional
        .get(1)
        .map(|s| s.trim().to_lowercase())
        .unwrap_or_default();

    let (url, work) = if dict_type == "learners" {
        (
            format!(
                "https://www.learnersdictionary.com/definition/{}",
                word_trimmed
            ),
            "''Merriam-Webster's Learner's Dictionary''".to_string(),
        )
    } else if dict_type == "medical" {
        (
            format!("https://www.merriam-webster.com/medical/{}", word_trimmed),
            "''Merriam-Webster's Medical Dictionary''".to_string(),
        )
    } else {
        (
            format!(
                "https://www.merriam-webster.com/dictionary/{}",
                word_trimmed
            ),
            "''Merriam-Webster.com Dictionary''".to_string(),
        )
    };

    let mut parts = Vec::new();
    parts.push(format!("[[official-url:{}|\"{}\"]]", url, word_trimmed));
    parts.push(work);
    parts.push("Merriam-Webster".to_string());

    if let Some(access_date) = named.get("access-date").or_else(|| named.get("accessdate")) {
        parts.push(format!("Retrieved {}", render_templates(access_date)));
    }

    parts.join(". ")
}

fn render_multiref_template(params: &str) -> String {
    let parts = split_template_params(params);
    let mut rendered = Vec::new();
    for part in parts {
        let part_trimmed = part.trim();
        if part_trimmed.is_empty() {
            continue;
        }

        // Find top-level '=' (not nested inside braces or brackets)
        let mut eq_index = None;
        let mut brace_depth = 0;
        let mut bracket_depth = 0;
        let bytes = part_trimmed.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            if b == b'{' {
                brace_depth += 1;
            } else if b == b'}' {
                if brace_depth > 0 {
                    brace_depth -= 1;
                }
            } else if b == b'[' {
                bracket_depth += 1;
            } else if b == b']' {
                if bracket_depth > 0 {
                    bracket_depth -= 1;
                }
            } else if b == b'=' && brace_depth == 0 && bracket_depth == 0 {
                eq_index = Some(i);
                break;
            }
        }

        let (val_to_render, is_valid) = if let Some(idx) = eq_index {
            let key = part_trimmed[..idx].trim();
            let val = part_trimmed[idx + 1..].trim();
            if !key.is_empty()
                && key.chars().all(|c| c.is_ascii_digit())
                && key.parse::<u32>().is_ok()
            {
                (val, true)
            } else {
                ("", false)
            }
        } else {
            (part_trimmed, true)
        };

        if is_valid {
            let val = render_templates(val_to_render);
            if !val.trim().is_empty() {
                rendered.push(val);
            }
        }
    }
    rendered.join("; ")
}

fn render_hosking_jfood_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let page = named
        .get("page")
        .or_else(|| named.get("1"))
        .or_else(|| positional.first())
        .map(|s| s.as_str());

    let pages = named.get("pages").map(|s| s.as_str());

    let mut parts = Vec::new();
    parts.push("Hosking, Richard (1996). ''A Dictionary of Japanese Food: Ingredients & Culture''. Tuttle Publishing".to_string());

    if let Some(pgs) = pages {
        parts.push(format!("pp. {}", render_templates(pgs)));
    } else if let Some(pg) = page {
        parts.push(format!("p. {}", render_templates(pg)));
    }

    parts.push("ISBN 978-0-8048-2042-4".to_string());

    parts.join(". ")
}

fn render_e28_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let code = named
        .get("1")
        .or_else(|| positional.first())
        .map(|s| s.trim())
        .unwrap_or("");
    let name = named
        .get("2")
        .or_else(|| positional.get(1))
        .map(|s| s.trim())
        .unwrap_or("");

    let url = if !code.is_empty() {
        format!("https://www.ethnologue.com/language/{}", code)
    } else {
        "https://www.ethnologue.com/".to_string()
    };

    let title = if !name.is_empty() {
        name.to_string()
    } else if code == "kor" {
        "Korean".to_string()
    } else {
        code.to_string()
    };

    let link = format!("\"[[official-url:{}|{}]]\"", url, title);
    format!(
        "Eberhard, David M.; Simons, Gary F.; Fennig, Charles D., eds. (2025). {}. ''Ethnologue: Languages of the World'' (28th ed.). Dallas, Texas: SIL International",
        link
    )
}

fn render_citation_attribution_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let text = named
        .get("1")
        .or_else(|| positional.first())
        .map(|s| s.trim())
        .unwrap_or("");

    format!(
        "One or more of the preceding sentences incorporates text from a work now in the public domain: {}",
        render_templates(text)
    )
}

fn render_cite_opentopomap_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let named = template_named_params(params);

    let name = template_param(&named, &["name"])
        .or_else(|| positional.first().map(String::as_str))
        .map(str::trim)
        .unwrap_or("");
    let lat = template_param(&named, &["lat"])
        .or_else(|| positional.get(1).map(String::as_str))
        .map(str::trim)
        .unwrap_or("");
    let long = template_param(&named, &["long"])
        .or_else(|| positional.get(2).map(String::as_str))
        .map(str::trim)
        .unwrap_or("");
    let access_date = template_param(&named, &["access-date", "accessdate"])
        .or_else(|| positional.get(3).map(String::as_str))
        .map(str::trim)
        .unwrap_or("");
    let zoom = template_param(&named, &["zoom"])
        .or_else(|| positional.get(4).map(String::as_str))
        .map(str::trim)
        .unwrap_or("14");

    if lat.is_empty() || long.is_empty() {
        return String::new();
    }

    let url = format!("https://opentopomap.org/#marker={}/{}/{}", zoom, lat, long);
    let title = if name.is_empty() {
        "Topographic map".to_string()
    } else {
        format!("Topographic map of {}", render_templates(name))
    };

    let mut parts = Vec::new();
    parts.push(format!("\"[[official-url:{}|{}]]\"", url, title));
    parts.push("''opentopomap.org''".to_string());

    if !access_date.is_empty() {
        parts.push(format!("Retrieved {}", render_templates(access_date)));
    }

    parts.join(". ")
}

pub(crate) fn get_dispatch_table() -> DispatchTable {
    HashMap::from([
        (
            "citation needed span",
            render_citation_needed_span_template as TemplateHandler,
        ),
        ("cite web", render_cite_web_template as TemplateHandler),
        ("cite book", render_cite_book_template as TemplateHandler),
        (
            "cite dictionary",
            render_cite_dictionary_template as TemplateHandler,
        ),
        (
            "cite press release",
            render_cite_press_release_template as TemplateHandler,
        ),
        ("cite apod", render_cite_apod_template as TemplateHandler),
        ("cite oed", render_cite_oed_template as TemplateHandler),
        ("oed", render_cite_oed_template as TemplateHandler),
        (
            "cite av media",
            render_cite_av_media_template as TemplateHandler,
        ),
        (
            "cite american heritage dictionary",
            render_cite_american_heritage_dictionary_template as TemplateHandler,
        ),
        (
            "cite wikisource",
            render_cite_wikisource_template as TemplateHandler,
        ),
        (
            "cite cia world factbook",
            render_cite_cia_world_factbook_template as TemplateHandler,
        ),
        (
            "cite letter",
            render_cite_letter_template as TemplateHandler,
        ),
        ("cite arxiv", render_cite_arxiv_template as TemplateHandler),
        ("cite q", render_cite_q_template as TemplateHandler),
        (
            "cite journal",
            render_cite_journal_template as TemplateHandler,
        ),
        (
            "cite magazine",
            render_cite_journal_template as TemplateHandler,
        ),
        ("cite news", render_cite_journal_template as TemplateHandler),
        (
            "cite encyclopedia",
            render_cite_journal_template as TemplateHandler,
        ),
        (
            "cite report",
            render_cite_report_template as TemplateHandler,
        ),
        ("cite eccp", render_cite_eccp_template as TemplateHandler),
        ("cite gvp", render_cite_gvp_template as TemplateHandler),
        (
            "cite conference",
            render_citation_template as TemplateHandler,
        ),
        ("citation", render_citation_template as TemplateHandler),
        ("harvc", render_harvc_template as TemplateHandler),
        (
            "mw",
            render_cite_merriam_webster_template as TemplateHandler,
        ),
        (
            "cite merriam-webster",
            render_cite_merriam_webster_template as TemplateHandler,
        ),
        (
            "cite eb1911",
            render_cite_eb1911_template as TemplateHandler,
        ),
        ("harvp", render_harvp_template as TemplateHandler),
        ("harv", render_harvp_template as TemplateHandler),
        ("harvnb", render_harvnb_template as TemplateHandler),
        ("harvtxt", render_harvtxt_template as TemplateHandler),
        ("cite nsrw", render_cite_nsrw_template as TemplateHandler),
        ("cite thesis", render_citation_template as TemplateHandler),
        (
            "hosking-jfood",
            render_hosking_jfood_template as TemplateHandler,
        ),
        ("e28", render_e28_template as TemplateHandler),
        (
            "citation-attribution",
            render_citation_attribution_template as TemplateHandler,
        ),
        ("multiref", render_multiref_template as TemplateHandler),
        ("multiref2", render_multiref_template as TemplateHandler),
        (
            "cite opentopomap",
            render_cite_opentopomap_template as TemplateHandler,
        ),
    ])
}
