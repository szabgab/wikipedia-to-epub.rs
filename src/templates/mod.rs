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
        ("spaced en dash", " – "),
        ("spaced ndash", " – "),
        ("spnd", " – "),
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
        ("emdash", "—"),
        ("eunum", "27"),
        ("hair space", "\u{200a}"),
        ("hairspace", "\u{200a}"),
        ("hsp", "\u{200a}"),
        ("asterisk", "*"),
        ("km2", "km²"),
        ("mi2", "mi²"),
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

#[cfg(test)]
mod tests {
    use super::{is_silent_template_name, render_templates};

    macro_rules! render_case {
        ($name:ident, $input:literal, $expected:literal) => {
            #[test]
            fn $name() {
                assert_eq!(render_templates($input), $expected);
            }
        };
    }

    macro_rules! silent_case {
        ($name:ident, $template:literal) => {
            #[test]
            fn $name() {
                assert!(is_silent_template_name($template));
                assert_eq!(
                    render_templates(concat!("{{", "{", $template, "}", "}")),
                    ""
                );
            }
        };
    }

    render_case!(
        renders_iast_template,
        "{{IAST|karma}}",
        "__WIKIPEDIA_TO_EPUB_LANG_START__sa-Latn__WIKIPEDIA_TO_EPUB_LANG_VALUE__karma__WIKIPEDIA_TO_EPUB_LANG_END__"
    );
    render_case!(
        renders_ibdb_name_template,
        "{{IBDB name|12345|Jane Doe}}",
        "[[official-url:https://www.ibdb.com/broadway-cast-staff/12345|Jane Doe]] at the Internet Broadway Database"
    );
    render_case!(renders_idn_template, "{{IDN}}", "[[Indonesia|Indonesia]]");
    render_case!(renders_ina_template, "{{INA}}", "[[Indonesia|Indonesia]]");
    render_case!(renders_ind_template, "{{IND}}", "[[India|India]]");
    render_case!(
        renders_ih_template,
        "{{ih|Canada}}",
        "[[Canada men's national ice hockey team|Canada]]"
    );
    render_case!(
        renders_imdb_event_template,
        "{{IMDb event|123|Awards|year=2020}}",
        "[[official-url:https://www.imdb.com/event/ev123/2020|Awards]] at IMDb"
    );
    render_case!(
        renders_imo_results_template,
        "{{IMO results|123|Ada}}",
        "[[official-url:https://www.imo-official.org/participant_r.aspx?id=123|Ada's results]] at International Mathematical Olympiad"
    );
    render_case!(
        renders_imslp_template,
        "{{IMSLP|Mozart|Wolfgang Mozart}}",
        "[[official-url:https://imslp.org/wiki/Category:Mozart|Free scores by Wolfgang Mozart]] at the International Music Score Library Project"
    );
    render_case!(renders_increase_template, "{{increase}}", "▲");
    render_case!(renders_increase_uppercase_template, "{{Increase}}", "▲");
    render_case!(renders_indent_template, "a{{indent|3}}b", "a   b");
    render_case!(
        renders_inrconvert_template,
        "{{INRConvert|5|c}}",
        "₹5 crore"
    );
    render_case!(
        renders_insee_template,
        "{{INSEE|ignored|Profile}}",
        "[[official-url:https://www.insee.fr/en/accueil|Profile]]"
    );
    render_case!(
        renders_instagram_template,
        "{{Instagram|example|Example}}",
        "[[official-url:https://www.instagram.com/example/|Example]] on Instagram"
    );
    render_case!(
        renders_in_our_time_template,
        "{{In Our Time|Topic|b0000000}}",
        "[[official-url:https://www.bbc.co.uk/programmes/b0000000|Topic]] on ''In Our Time'' at the BBC"
    );
    render_case!(
        renders_internet_archive_template,
        "{{Internet Archive|bookid|Book}}",
        "[[official-url:https://archive.org/details/bookid|Book]] at the Internet Archive"
    );
    render_case!(
        renders_internet_archive_author_template,
        "{{Internet Archive author|marktwain|Mark Twain}}",
        "[[official-url:https://archive.org/search?query=creator%3A%22marktwain%22|Mark Twain]] at the Internet Archive"
    );
    render_case!(
        renders_internet_archive_film_template,
        "{{Internet Archive film|filmid|Film}}",
        "[[official-url:https://archive.org/details/filmid|Film]] is available at the Internet Archive"
    );
    render_case!(renders_interp_template, "{{interp|added}}", "[added]");
    render_case!(
        renders_interlinear_template,
        "{{interlinear|one|two}}",
        "one two"
    );
    render_case!(
        renders_ipablink_template,
        "{{IPAblink|a}}",
        "__WIKIPEDIA_TO_EPUB_IPA_START__a__WIKIPEDIA_TO_EPUB_IPA_END__"
    );
    render_case!(
        renders_ipac_cmn_template,
        "{{IPAc-cmn|t|a}}",
        "__WIKIPEDIA_TO_EPUB_IPA_START__ta__WIKIPEDIA_TO_EPUB_IPA_END__"
    );
    render_case!(
        renders_ipac_hu_template,
        "{{IPAc-hu|t|a}}",
        "__WIKIPEDIA_TO_EPUB_IPA_START__ta__WIKIPEDIA_TO_EPUB_IPA_END__"
    );
    render_case!(
        renders_ipa_link_template,
        "{{IPA link|a}}",
        "__WIKIPEDIA_TO_EPUB_IPA_START__a__WIKIPEDIA_TO_EPUB_IPA_END__"
    );
    render_case!(
        renders_ipalink_template,
        "{{IPAlink|a}}",
        "__WIKIPEDIA_TO_EPUB_IPA_START__a__WIKIPEDIA_TO_EPUB_IPA_END__"
    );
    render_case!(renders_iri_template, "{{IRI}}", "[[Iran|Iran]]");
    render_case!(renders_irl_template, "{{IRL}}", "[[Ireland|Ireland]]");
    render_case!(renders_irn_template, "{{IRN}}", "[[Iran|Iran]]");
    render_case!(renders_irq_template, "{{IRQ}}", "[[Iraq|Iraq]]");
    render_case!(
        renders_isbnt_template,
        "{{ISBNT|9780000000000}}",
        "ISBN 9780000000000"
    );
    render_case!(renders_isl_template, "{{ISL}}", "[[Iceland|Iceland]]");
    render_case!(renders_isr_template, "{{ISR}}", "[[Israel|Israel]]");
    render_case!(
        renders_isu_short_track_skater_template,
        "{{ISU short track skater|new_id=jane-doe|name=Jane Doe}}",
        "[[official-url:https://isu-skating.com/short-track/skaters/jane-doe/|Jane Doe]] at the International Skating Union"
    );
    render_case!(renders_ita_template, "{{ITA}}", "[[Italy|Italy]]");

    silent_case!(
        silences_ice_hockey_world_championships_template,
        "Ice Hockey World Championships"
    );
    silent_case!(
        silences_ieee_medal_of_honor_laureates_hyphen_template,
        "IEEE Medal of Honor Laureates 1951-1975"
    );
    silent_case!(
        silences_ieee_medal_of_honor_laureates_dash_template,
        "IEEE Medal of Honor Laureates 1951–1975"
    );
    silent_case!(silences_iihf_template, "IIHF");
    silent_case!(
        silences_iihf_world_championship_venues_template,
        "IIHF World Championship venues"
    );
    silent_case!(
        silences_ijf_world_tour_hungary_template,
        "IJF World Tour Hungary"
    );
    silent_case!(silences_imagefact_template, "imagefact");
    silent_case!(silences_image_frame_template, "Image frame");
    silent_case!(silences_image_key_lowercase_template, "image key");
    silent_case!(silences_image_key_titlecase_template, "Image key");
    silent_case!(silences_image_label_begin_template, "Image label begin");
    silent_case!(silences_image_label_end_template, "Image label end");
    silent_case!(silences_image_label_small_template, "Image label small");
    silent_case!(silences_incarceration_template, "Incarceration");
    silent_case!(silences_incomplete_template, "incomplete");
    silent_case!(silences_incubator_template, "Incubator");
    silent_case!(
        silences_independent_source_inline_template,
        "Independent source inline"
    );
    silent_case!(
        silences_indigenous_rights_footer_template,
        "Indigenous rights footer"
    );
    silent_case!(
        silences_indo_aryan_languages_template,
        "Indo-Aryan languages"
    );
    silent_case!(silences_indo_european_template, "Indo-European");
    silent_case!(
        silences_indo_european_languages_template,
        "Indo-European languages"
    );
    silent_case!(
        silences_indo_european_topics_template,
        "Indo-European topics"
    );
    silent_case!(silences_infantes_of_aragon_template, "Infantes of Aragon");
    silent_case!(silences_inflation_fn_slash_template, "inflation/fn");
    silent_case!(silences_inflation_fn_template, "Inflation-fn");
    silent_case!(silences_inflation_year_template, "Inflation-year");
    silent_case!(silences_informa_template, "Informa");
    silent_case!(
        silences_information_security_template,
        "Information security"
    );
    silent_case!(silences_inline_template, "inline");
    silent_case!(silences_inner_asia_template, "Inner Asia");
    silent_case!(silences_in_popular_culture_template, "in popular culture");
    silent_case!(silences_insurance_template, "Insurance");
    silent_case!(silences_interkosmos_template, "Interkosmos");
    silent_case!(silences_interwiki_lowercase_template, "interWiki");
    silent_case!(silences_interwiki_titlecase_template, "InterWiki");
    silent_case!(
        silences_international_athletics_template,
        "International athletics"
    );
    silent_case!(
        silences_international_criminal_law_template,
        "International Criminal Law"
    );
    silent_case!(
        silences_international_development_and_investment_banks_template,
        "International development and investment banks"
    );
    silent_case!(
        silences_international_football_template,
        "International Football"
    );
    silent_case!(
        silences_international_forum_of_public_universities_template,
        "International Forum of Public Universities"
    );
    silent_case!(
        silences_international_futsal_template,
        "International_futsal"
    );
    silent_case!(
        silences_international_human_rights_legal_instruments_template,
        "International human rights legal instruments"
    );
    silent_case!(
        silences_international_mathematical_activities_template,
        "International mathematical activities"
    );
    silent_case!(
        silences_international_monetary_fund_by_country_template,
        "International Monetary Fund by country"
    );
    silent_case!(
        silences_international_organisations_template,
        "International organisations"
    );
    silent_case!(
        silences_international_organizations_template,
        "International organizations"
    );
    silent_case!(silences_international_power_template, "International power");
    silent_case!(
        silences_international_relations_template,
        "International relations"
    );
    silent_case!(
        silences_international_science_council_template,
        "International Science Council"
    );
    silent_case!(
        silences_international_science_olympiad_template,
        "International Science Olympiad"
    );
    silent_case!(
        silences_international_sports_federations_template,
        "International Sports Federations"
    );
    silent_case!(
        silences_international_water_polo_template,
        "International water polo"
    );
    silent_case!(silences_iranian_languages_template, "Iranian languages");
    silent_case!(silences_iranian_peoples_template, "Iranian peoples");
    silent_case!(silences_irredentism_template, "Irredentism");
    silent_case!(silences_irrelevant_citation_template, "irrelevant citation");
    silent_case!(silences_irreligion_template, "Irreligion");
    silent_case!(silences_irreligion_sidebar_template, "Irreligion sidebar");
    silent_case!(silences_isbn_missing_template, "ISBN missing");
    silent_case!(silences_islam_template, "Islam");
    silent_case!(silences_islam_and_iman_template, "Islam and iman");
    silent_case!(silences_islamic_geography_template, "Islamic geography");
    silent_case!(silences_islam_topics_template, "Islam topics");
    silent_case!(silences_istanbul_template, "Istanbul");
    silent_case!(silences_italy_topics_template, "Italy topics");
    silent_case!(silences_ithaca_new_york_template, "Ithaca, New York");
    silent_case!(
        silences_itu_world_championships_template,
        "ITU World Championships"
    );
}
