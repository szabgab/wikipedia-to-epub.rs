mod citation;
mod convert;
mod formatting;
mod lang;

use std::collections::HashMap;

use cached::macros::cached;
use tracing::warn;

use crate::types::DispatchTable;

use crate::tools::{matching_template_end, split_template_name};

use crate::templates::formatting::{
    get_dispatch_template_params, get_empty_dispatch_table, render_formatnum_template,
};

use crate::increment_recognized_skipped_template_count;
use crate::increment_unknown_skipped_template_count;

pub(crate) fn render_templates(text: &str) -> String {
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

/// [nbsp](https://en.wikipedia.org/wiki/Template:Nbsp)
///
/// [snd](https://en.wikipedia.org/wiki/Template:Snd)
/// [dash](https://en.wikipedia.org/wiki/Template:Dash)
/// [snds](https://en.wikipedia.org/wiki/Template:Snds)
///
/// [mdash](https://en.wikipedia.org/wiki/Template:Mdash)
///
/// [ndash](https://en.wikipedia.org/wiki/Template:Ndash)
/// [endash](https://en.wikipedia.org/wiki/Template:Endash)
/// [nbndash](https://en.wikipedia.org/wiki/Template:Nbndash)
/// [nbnd](https://en.wikipedia.org/wiki/Template:Nbnd)
/// [en dash](https://en.wikipedia.org/wiki/Template:En_dash)
/// [En dash](https://en.wikipedia.org/wiki/Template:En_dash)
///
/// [singular](https://en.wikipedia.org/wiki/Template:Singular)
#[cached]
fn get_fixed() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("'\"", "'\""),
        ("\"'", "\"'"),
        ("!", "|"),
        ("pipe", "|"),
        ("tba", "TBA"),
        ("dagger", "†"),
        ("nbsp", " "),
        ("snd", " – "),
        ("dash", " – "),
        ("snds", " – "),
        ("mdash", "—"),
        ("em dash", "—"),
        ("ndash", "–"),
        ("endash", "–"),
        ("nbndash", "–"),
        ("nbnd", "–"),
        ("en dash", "–"),
        (
            "singular",
            "__WIKIPEDIA_TO_EPUB_ABBR_START__singular form__WIKIPEDIA_TO_EPUB_ABBR_VALUE__sg.__WIKIPEDIA_TO_EPUB_ABBR_END__",
        ),
        ("pb", "__WIKIPEDIA_TO_EPUB_PB__"),
        ("okina", "ʻ"),
        ("'s", "__WIKIPEDIA_TO_EPUB_LITERAL_QUOTE__s"),
        ("=", "="),
        ("1/2", "1/2"),
        ("hidden end", ""),
        ("collapse bottom", ""),
    ])
}

#[cached]
fn get_dispatch_table() -> DispatchTable {
    let mut table = HashMap::new();

    for (key, value) in crate::templates::lang::get_dispatch_table() {
        table.insert(key, value);
    }

    for (key, value) in crate::templates::formatting::get_dispatch_table() {
        table.insert(key, value);
    }

    for (key, value) in crate::templates::citation::get_dispatch_table() {
        table.insert(key, value);
    }

    for (key, value) in crate::templates::convert::get_dispatch_table() {
        table.insert(key, value);
    }
    table
}

fn render_template(content: &str) -> String {
    let (template, params) = split_template_name(content);
    let template_normalized = template.trim().replace('_', " ");
    let template = template_normalized.as_str();

    let lower = template.to_lowercase();

    let fixed = get_fixed();
    if fixed.contains_key(&lower.as_str()) {
        return fixed.get(lower.as_str()).unwrap().to_string();
    }

    let lookup = get_dispatch_table();
    if lookup.contains_key(&lower.as_str()) {
        return lookup.get(lower.as_str()).unwrap()(params);
    }

    let empty_lookup = get_empty_dispatch_table();
    if empty_lookup.contains_key(&lower.as_str()) {
        return empty_lookup.get(lower.as_str()).unwrap()();
    }

    let template_params_lookup = get_dispatch_template_params();
    if template_params_lookup.contains_key(&lower.as_str()) {
        return template_params_lookup.get(lower.as_str()).unwrap()(template, params);
    }

    if template
        .get(.."formatnum:".len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("formatnum:"))
        || template.eq_ignore_ascii_case("formatnum")
    {
        render_formatnum_template(template, params)
    } else if let Some(image_name) = find_map_image(template) {
        format!("[[File:{}|thumb|{}]]", image_name, template)
    } else if is_silent_template_name(template) {
        increment_recognized_skipped_template_count();
        String::new()
    } else {
        increment_unknown_skipped_template_count();
        warn!(
            content = template_log_content(content),
            "removing unhandled wikitext template"
        );
        String::new()
    }
}

pub(crate) fn template_log_content(content: &str) -> String {
    content.chars().take(80).collect()
}

fn is_silent_template_name(template: &str) -> bool {
    let template = template.trim();
    template.starts_with('#')
        || template.to_ascii_lowercase().ends_with("stub")
        || template_name_is_in_csv(template, include_str!("../silent.csv"))
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
        || (template
            .get(.."Infobox".len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("Infobox"))
            && !template.eq_ignore_ascii_case("Infobox mountain")
            && !template.eq_ignore_ascii_case("Infobox country")
            && !template.eq_ignore_ascii_case("Infobox military conflict")
            && !template.eq_ignore_ascii_case("Infobox planet")
            && !template.eq_ignore_ascii_case("Infobox settlement")
            && !template.eq_ignore_ascii_case("Infobox"))
        || template
            .get(.."Campaignbox".len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("Campaignbox"))
        || is_observed_navigation_template_name(template)
}

fn is_observed_navigation_template_name(template: &str) -> bool {
    template_name_is_in_csv(template.trim(), include_str!("../navigations.csv"))
}

pub(crate) fn template_name_is_in_csv(template: &str, csv: &str) -> bool {
    csv.lines().any(|line| {
        let name = if let Some(stripped) = line.strip_prefix('"') {
            if let Some(end_idx) = stripped.find('"') {
                &stripped[..end_idx]
            } else {
                line
            }
        } else {
            line.split_once(',').map_or(line, |(name, _)| name)
        };

        name.trim().eq_ignore_ascii_case(template)
    })
}

fn is_succession_template_name(template: &str) -> bool {
    template
        .get(.."s-".len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("s-"))
        || template.eq_ignore_ascii_case("Succession box")
}

fn find_map_image(template: &str) -> Option<&'static str> {
    let csv = include_str!("../maps.csv");
    for line in csv.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split("\",\"").collect();
        if parts.len() == 2 {
            let clean_t = parts[0].trim_start_matches('"');
            let clean_i = parts[1].trim_end_matches('"');
            if clean_t.eq_ignore_ascii_case(template) {
                return Some(clean_i);
            }
        }
    }
    None
}
