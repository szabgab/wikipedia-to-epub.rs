use crate::PersonRole;
use crate::citation_people;
use crate::templates::{
    render_templates, template_named_params, template_param, template_param_owned,
    template_positional_params,
};
use std::collections::HashMap;

/// [citation needed span](https://en.wikipedia.org/wiki/Template:Citation_needed_span)
pub(crate) fn render_citation_needed_span_template(params: &str) -> String {
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
pub(crate) fn render_cite_web_template(params: &str) -> String {
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
pub(crate) fn render_cite_book_template(params: &str) -> String {
    render_citation_template(params)
}

/// [cite journal](https://en.wikipedia.org/wiki/Template:Cite_journal)
/// [cite magazine](https://en.wikipedia.org/wiki/Template:Cite_magazine)
/// [cite news](https://en.wikipedia.org/wiki/Template:Cite_news)
/// [cite encyclopedia](https://en.wikipedia.org/wiki/Template:Cite_encyclopedia)
pub(crate) fn render_cite_journal_template(params: &str) -> String {
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
pub(crate) fn render_cite_report_template(params: &str) -> String {
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
pub(crate) fn render_cite_eccp_template(params: &str) -> String {
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
pub(crate) fn render_cite_gvp_template(params: &str) -> String {
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

pub(crate) fn format_harvard_citation(params: &str) -> String {
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
pub(crate) fn render_harvp_template(params: &str) -> String {
    let formatted = format_harvard_citation(params);
    if formatted.is_empty() {
        String::new()
    } else {
        format!("({formatted})")
    }
}

/// [harvnb](https://en.wikipedia.org/wiki/Template:Harvnb)
pub(crate) fn render_harvnb_template(params: &str) -> String {
    format_harvard_citation(params)
}

/// [harvtxt](https://en.wikipedia.org/wiki/Template:Harvtxt)
pub(crate) fn render_harvtxt_template(params: &str) -> String {
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
pub(crate) fn render_cite_nsrw_template(params: &str) -> String {
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
pub(crate) fn render_citation_template(params: &str) -> String {
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
pub(crate) fn render_harvc_template(params: &str) -> String {
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

pub(crate) fn harvc_source(named: &HashMap<String, String>) -> String {
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
pub(crate) fn render_cite_eb1911_template(params: &str) -> String {
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
