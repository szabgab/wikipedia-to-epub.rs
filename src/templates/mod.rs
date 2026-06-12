pub mod citation;
pub mod convert;
pub mod formatting;
pub mod lang;

use std::collections::HashMap;

use cached::macros::cached;
use tracing::debug;

use crate::types::DispatchTable;

use crate::tools::split_template_name;

use crate::templates::formatting::{
    get_empty_dispatch_table, render_formatnum_template, render_lagrange_template,
    render_ship_template,
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

pub(crate) fn matching_template_end(text: &str, start: usize) -> Option<usize> {
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
        ("nbsp", " "),
        ("snd", " – "),
        ("dash", " – "),
        ("snds", " – "),
        ("mdash", "—"),
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
        ("'s", "'s"),
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

pub(crate) fn render_template(content: &str) -> String {
    let (template, params) = split_template_name(content);
    let template_normalized = template.trim().replace('_', " ");
    let template = template_normalized.as_str();

    let fixed = get_fixed();
    let lower = template.to_lowercase();
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

    if template.eq_ignore_ascii_case("USS") {
        render_ship_template("USS", params)
    } else if template.eq_ignore_ascii_case("HMS") {
        render_ship_template("HMS", params)
    } else if template.eq_ignore_ascii_case("L1") {
        render_lagrange_template("1")
    } else if template.eq_ignore_ascii_case("L2") {
        render_lagrange_template("2")
    } else if template.eq_ignore_ascii_case("L3") {
        render_lagrange_template("3")
    } else if template.eq_ignore_ascii_case("L4") {
        render_lagrange_template("4")
    } else if template.eq_ignore_ascii_case("L5") {
        render_lagrange_template("5")
    } else if template
        .get(.."formatnum:".len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("formatnum:"))
        || template.eq_ignore_ascii_case("formatnum")
    {
        render_formatnum_template(template, params)
    } else if is_silent_template_name(template) {
        increment_recognized_skipped_template_count();
        String::new()
    } else {
        increment_unknown_skipped_template_count();
        debug!(
            content = template_log_content(content),
            "removing unhandled wikitext template"
        );
        log_and_count_nested_skipped_templates(params);
        String::new()
    }
}

fn log_and_count_nested_skipped_templates(text: &str) {
    let mut offset = 0;

    while let Some(start) = text[offset..].find("{{").map(|index| offset + index) {
        if let Some(end) = matching_template_end(text, start) {
            let content = &text[start + 2..end];
            let (template, params) = split_template_name(content);
            let template_normalized = template.trim().replace('_', " ");
            let template = template_normalized.as_str();
            if is_silent_template_name(template) {
                increment_recognized_skipped_template_count();
            } else if !is_handled_template_name(template) {
                increment_unknown_skipped_template_count();
                debug!(
                    content = template_log_content(content),
                    "removing nested unhandled wikitext template"
                );
                log_and_count_nested_skipped_templates(params);
            }
            offset = end + 2;
        } else {
            break;
        }
    }
}

pub(crate) fn template_log_content(content: &str) -> String {
    content.chars().take(80).collect()
}

fn is_handled_template_name(template: &str) -> bool {
    template.eq_ignore_ascii_case("Korean")
        || template.eq_ignore_ascii_case("Korean/auto")
        || template.eq_ignore_ascii_case("ko")
        || template.eq_ignore_ascii_case("!")
        || template.eq_ignore_ascii_case("citation needed span")
        || template.eq_ignore_ascii_case("ndash")
        || template.eq_ignore_ascii_case("Quote box")
        || template.eq_ignore_ascii_case("Quote")
        || template.eq_ignore_ascii_case("Poem quote")
        || template.eq_ignore_ascii_case("poemquote")
        || template.eq_ignore_ascii_case("Verse translation")
        || template.eq_ignore_ascii_case("Verse transliteration-translation")
        || template.eq_ignore_ascii_case("center")
        || template.eq_ignore_ascii_case("singular")
        || template.eq_ignore_ascii_case("Nihongo4")
        || template.eq_ignore_ascii_case("Nihongo")
        || template.eq_ignore_ascii_case("nbsp")
        || template.eq_ignore_ascii_case("snd")
        || template.eq_ignore_ascii_case("dash")
        || template.eq_ignore_ascii_case("snds")
        || template.eq_ignore_ascii_case("mdash")
        || template.eq_ignore_ascii_case("nowrap")
        || template.eq_ignore_ascii_case("smaller")
        || template.eq_ignore_ascii_case("small")
        || template.eq_ignore_ascii_case("sic")
        || template.eq_ignore_ascii_case("circa")
        || template.eq_ignore_ascii_case("c.")
        || template.eq_ignore_ascii_case("cx")
        || template.eq_ignore_ascii_case("lang")
        || template.eq_ignore_ascii_case("in lang")
        || template.eq_ignore_ascii_case("langx")
        || template.eq_ignore_ascii_case("linktext")
        || template.eq_ignore_ascii_case("lang-zh")
        || template.eq_ignore_ascii_case("zh")
        || template.eq_ignore_ascii_case("zhi")
        || template.eq_ignore_ascii_case("transliteration")
        || template.eq_ignore_ascii_case("translit")
        || template.eq_ignore_ascii_case("tlit")
        || template.eq_ignore_ascii_case("ko-translit")
        || template.eq_ignore_ascii_case("lit")
        || template.eq_ignore_ascii_case("Literal translation")
        || template.eq_ignore_ascii_case("literal")
        || template.eq_ignore_ascii_case("translation")
        || template.eq_ignore_ascii_case("Language with name/for")
        || template.eq_ignore_ascii_case("langnf")
        || template.eq_ignore_ascii_case("isbn")
        || template.eq_ignore_ascii_case("asin")
        || template.eq_ignore_ascii_case("script")
        || template.eq_ignore_ascii_case("oclc")
        || template.eq_ignore_ascii_case("ipa")
        || template.eq_ignore_ascii_case("IPAc-en")
        || template.eq_ignore_ascii_case("Respell")
        || template.eq_ignore_ascii_case("abbr")
        || template.eq_ignore_ascii_case("AWOL")
        || template.eq_ignore_ascii_case("Assassinated")
        || template.eq_ignore_ascii_case("DOW")
        || template.eq_ignore_ascii_case("Died of wounds")
        || template.eq_ignore_ascii_case("Executed")
        || template.eq_ignore_ascii_case("KIA")
        || template.eq_ignore_ascii_case("KIA2")
        || template.eq_ignore_ascii_case("MIA")
        || template.eq_ignore_ascii_case("Natural Causes")
        || template.eq_ignore_ascii_case("PKIA")
        || template.eq_ignore_ascii_case("POW")
        || template.eq_ignore_ascii_case("Suicide")
        || template.eq_ignore_ascii_case("Surrendered")
        || template.eq_ignore_ascii_case("Turncoat")
        || template.eq_ignore_ascii_case("WIA")
        || template.eq_ignore_ascii_case("frac")
        || template.eq_ignore_ascii_case("fraction")
        || template.eq_ignore_ascii_case("longitem")
        || template.eq_ignore_ascii_case("flagdeco")
        || template.eq_ignore_ascii_case("pprime")
        || template.eq_ignore_ascii_case("ra")
        || template.eq_ignore_ascii_case("mw")
        || template.eq_ignore_ascii_case("cite merriam-webster")
        || template.eq_ignore_ascii_case("indented plainlist")
        || template.eq_ignore_ascii_case("bulleted list")
        || template.eq_ignore_ascii_case("blist")
        || template.eq_ignore_ascii_case("hyphen")
        || template.eq_ignore_ascii_case("native phrase")
        || template.eq_ignore_ascii_case("native name")
        || template.eq_ignore_ascii_case("floruit")
        || template.eq_ignore_ascii_case("coord")
        || template.eq_ignore_ascii_case("rp")
        || template.eq_ignore_ascii_case("Reference page")
        || template.eq_ignore_ascii_case("cite web")
        || template.eq_ignore_ascii_case("cite book")
        || template.eq_ignore_ascii_case("cite dictionary")
        || template.eq_ignore_ascii_case("cite press release")
        || template.eq_ignore_ascii_case("cite apod")
        || template.eq_ignore_ascii_case("cite oed")
        || template.eq_ignore_ascii_case("oed")
        || template.eq_ignore_ascii_case("cite av media")
        || template.eq_ignore_ascii_case("cite american heritage dictionary")
        || template.eq_ignore_ascii_case("cite wikisource")
        || template.eq_ignore_ascii_case("cite cia world factbook")
        || template.eq_ignore_ascii_case("cite letter")
        || template.eq_ignore_ascii_case("cite arxiv")
        || template.eq_ignore_ascii_case("cite q")
        || template.eq_ignore_ascii_case("cite journal")
        || template.eq_ignore_ascii_case("cite magazine")
        || template.eq_ignore_ascii_case("cite news")
        || template.eq_ignore_ascii_case("cite report")
        || template.eq_ignore_ascii_case("cite ECCP")
        || template.eq_ignore_ascii_case("cite gvp")
        || template.eq_ignore_ascii_case("cite conference")
        || template.eq_ignore_ascii_case("citation")
        || template.eq_ignore_ascii_case("cite encyclopedia")
        || template.eq_ignore_ascii_case("harvc")
        || template.eq_ignore_ascii_case("as of")
        || template.eq_ignore_ascii_case("died-in")
        || template.eq_ignore_ascii_case("blockquote")
        || template.eq_ignore_ascii_case("percentage")
        || template.eq_ignore_ascii_case("UN Population")
        || template.eq_ignore_ascii_case("convert")
        || template.eq_ignore_ascii_case("cvt")
        || template.eq_ignore_ascii_case("for")
        || template.eq_ignore_ascii_case("for timeline")
        || template.eq_ignore_ascii_case("crossreference")
        || template.eq_ignore_ascii_case("slink")
        || template.eq_ignore_ascii_case("legend")
        || template.eq_ignore_ascii_case("legend0")
        || template.eq_ignore_ascii_case("numero")
        || template.eq_ignore_ascii_case("anl")
        || template.eq_ignore_ascii_case("excerpt")
        || template.eq_ignore_ascii_case("main")
        || template.eq_ignore_ascii_case("Main article")
        || template.eq_ignore_ascii_case("Main list")
        || template.eq_ignore_ascii_case("see also")
        || template.eq_ignore_ascii_case("also")
        || template.eq_ignore_ascii_case("further")
        || template.eq_ignore_ascii_case("wiktionary")
        || template.eq_ignore_ascii_case("wikivoyage")
        || template.eq_ignore_ascii_case("wikivoyage-inline")
        || template.eq_ignore_ascii_case("wikivoyage inline")
        || template.eq_ignore_ascii_case("wikisource")
        || template.eq_ignore_ascii_case("wikibooks")
        || template.eq_ignore_ascii_case("britannica")
        || template.eq_ignore_ascii_case("official website")
        || template.eq_ignore_ascii_case("official")
        || template.eq_ignore_ascii_case("url")
        || template.eq_ignore_ascii_case("osmrelation-inline")
        || template.eq_ignore_ascii_case("osmway")
        || template.eq_ignore_ascii_case("webarchive")
        || template.eq_ignore_ascii_case("largest cities")
        || template.eq_ignore_ascii_case("historical populations")
        || template.eq_ignore_ascii_case("climate chart")
        || template.eq_ignore_ascii_case("sclass")
        || template.eq_ignore_ascii_case("nobold")
        || template.eq_ignore_ascii_case("Arrow")
        || template.eq_ignore_ascii_case("ROKS")
        || template.eq_ignore_ascii_case("ill")
        || template.eq_ignore_ascii_case("illm")
        || template.eq_ignore_ascii_case("Interlanguage link")
        || template.eq_ignore_ascii_case("Interlanguage link multi")
        || template.eq_ignore_ascii_case("reign")
        || template.eq_ignore_ascii_case("open access")
        || template.eq_ignore_ascii_case("free access")
        || template.eq_ignore_ascii_case("For-multi")
        || template.eq_ignore_ascii_case("Inflation")
        || template.eq_ignore_ascii_case("Inflation/year")
        || template.eq_ignore_ascii_case("stack")
        || template.eq_ignore_ascii_case("USS")
        || template.eq_ignore_ascii_case("HMS")
        || template.eq_ignore_ascii_case("ship")
        || template.eq_ignore_ascii_case("Nb5")
        || template.eq_ignore_ascii_case("Collapsible list")
        || template.eq_ignore_ascii_case("Internet Archive short film")
        || template.eq_ignore_ascii_case("worldhistory")
        || template.eq_ignore_ascii_case("nihongo2")
        || template.eq_ignore_ascii_case("gloss")
        || template.eq_ignore_ascii_case("xref")
        || template.eq_ignore_ascii_case("Shy")
        || template.eq_ignore_ascii_case("color box")
        || template.eq_ignore_ascii_case("pb")
        || template.eq_ignore_ascii_case("OSM relation")
        || template.eq_ignore_ascii_case("OSM way")
        || template.eq_ignore_ascii_case("okina")
        || template.eq_ignore_ascii_case("'s")
        || template.eq_ignore_ascii_case("harvp")
        || template.eq_ignore_ascii_case("harv")
        || template.eq_ignore_ascii_case("harvnb")
        || template.eq_ignore_ascii_case("harvtxt")
        || template.eq_ignore_ascii_case("NDLDC")
        || template.eq_ignore_ascii_case("plainlist")
        || template.eq_ignore_ascii_case("unbulleted list")
        || template.eq_ignore_ascii_case("ubl")
        || template.eq_ignore_ascii_case("ubli")
        || template.eq_ignore_ascii_case("unbulleted indent list")
        || template.eq_ignore_ascii_case("IPAslink")
        || template.eq_ignore_ascii_case("angbr")
        || template.eq_ignore_ascii_case("angbr IPA")
        || template.eq_ignore_ascii_case("unichar")
        || template.eq_ignore_ascii_case("xlit")
        || template.eq_ignore_ascii_case("note")
        || template.eq_ignore_ascii_case("fs interlinear")
        || template.eq_ignore_ascii_case("Tooltip")
        || template.eq_ignore_ascii_case("Nihongo krt")
        || template.eq_ignore_ascii_case("Jaanus")
        || template.eq_ignore_ascii_case("nihongo3")
        || template.eq_ignore_ascii_case("Easy CSS image crop")
        || template.eq_ignore_ascii_case("Multiple images")
        || template.eq_ignore_ascii_case("Multiple image")
        || template.eq_ignore_ascii_case("ISSN")
        || template.eq_ignore_ascii_case("Cite NSRW")
        || template
            .get(.."formatnum:".len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("formatnum:"))
        || template.eq_ignore_ascii_case("formatnum")
        || template.eq_ignore_ascii_case("STN")
        || template.eq_ignore_ascii_case("Station")
        || template.eq_ignore_ascii_case("JPN")
        || template.eq_ignore_ascii_case("Track gauge")
        || template.eq_ignore_ascii_case("RailGauge")
        || template.eq_ignore_ascii_case("GBurl")
        || template.eq_ignore_ascii_case("Google books")
        || template.eq_ignore_ascii_case("cite thesis")
        || template.eq_ignore_ascii_case("usurped")
        || template.eq_ignore_ascii_case("Break")
        || template.eq_ignore_ascii_case("br")
        || template.eq_ignore_ascii_case("brk")
        || template.eq_ignore_ascii_case("crlf")
        || template.eq_ignore_ascii_case("jct")
        || template.eq_ignore_ascii_case("FXConvert")
        || template.eq_ignore_ascii_case("JPY")
        || template.eq_ignore_ascii_case("doi")
        || template.eq_ignore_ascii_case("dts")
        || template.eq_ignore_ascii_case("age")
        || template.eq_ignore_ascii_case("Birth date and age")
        || template.eq_ignore_ascii_case("birth date and age")
        || template.eq_ignore_ascii_case("ayd")
        || template.eq_ignore_ascii_case("RouteBox")
        || template.eq_ignore_ascii_case("Ja-rail-color")
        || template.eq_ignore_ascii_case("age in years and days nts")
        || template.eq_ignore_ascii_case("Age in years and days nts")
        || template.eq_ignore_ascii_case("Proto")
        || template.eq_ignore_ascii_case("wktl")
        || template.eq_ignore_ascii_case("wikt-lang")
        || template.eq_ignore_ascii_case("langr")
        || template.eq_ignore_ascii_case("val")
        || template.eq_ignore_ascii_case("chem2")
        || template.eq_ignore_ascii_case("Value")
        || template.eq_ignore_ascii_case("value")
        || template.eq_ignore_ascii_case("e")
        || template.eq_ignore_ascii_case("sup")
        || template.eq_ignore_ascii_case("sub")
        || template.eq_ignore_ascii_case("su")
        || template.eq_ignore_ascii_case("mpl")
        || template.eq_ignore_ascii_case("en dash")
        || template.eq_ignore_ascii_case("En dash")
        || template.eq_ignore_ascii_case("endash")
        || template.eq_ignore_ascii_case("columns list")
        || template.eq_ignore_ascii_case("annotated link")
        || template.eq_ignore_ascii_case("Dp")
        || template.eq_ignore_ascii_case("dp")
        || template.eq_ignore_ascii_case("Visible anchor")
        || template.eq_ignore_ascii_case("visible anchor")
        || template.eq_ignore_ascii_case("L1")
        || template.eq_ignore_ascii_case("L2")
        || template.eq_ignore_ascii_case("L3")
        || template.eq_ignore_ascii_case("L4")
        || template.eq_ignore_ascii_case("L5")
        || template.eq_ignore_ascii_case("Cite EB1911")
        || template.eq_ignore_ascii_case("spaces")
        || template.eq_ignore_ascii_case("mpl-")
        || template.eq_ignore_ascii_case("chem")
        || template.eq_ignore_ascii_case("solar radius")
        || template.eq_ignore_ascii_case("±")
        || template.eq_ignore_ascii_case("Nihongo foot")
        || template.eq_ignore_ascii_case("Literal translation")
        || template.eq_ignore_ascii_case("N/A")
        || template.eq_ignore_ascii_case("NA")
        || template.eq_ignore_ascii_case("Not applicable")
        || template == "'\""
        || template == "\"'"
        || template.eq_ignore_ascii_case("nbndash")
        || template.eq_ignore_ascii_case("nbnd")
        || template.eq_ignore_ascii_case("Ja-platform")
        || template.eq_ignore_ascii_case("jpf")
        || template.eq_ignore_ascii_case("Ja-platform-m")
        || template.eq_ignore_ascii_case("jpfm")
        || template.eq_ignore_ascii_case("ja-rail-linem")
        || template.eq_ignore_ascii_case("rail-interchange")
        || template.eq_ignore_ascii_case("ric")
        || template.eq_ignore_ascii_case("rint")
        || template.eq_ignore_ascii_case("Line link")
        || template.eq_ignore_ascii_case("lnl")
        || template.eq_ignore_ascii_case("color")
        || template.eq_ignore_ascii_case("colour")
        || template.eq_ignore_ascii_case("Infobox mountain")
        || template.eq_ignore_ascii_case("Infobox country")
        || template.eq_ignore_ascii_case("Infobox military conflict")
        || template.eq_ignore_ascii_case("Infobox planet")
        || template.eq_ignore_ascii_case("Infobox settlement")
        || template.eq_ignore_ascii_case("Infobox")
        || template.eq_ignore_ascii_case("native name list")
        || template.eq_ignore_ascii_case("hlist")
        || template.eq_ignore_ascii_case("flatlist")
        || template.eq_ignore_ascii_case("ublist")
        || template.eq_ignore_ascii_case("multiref")
        || template.eq_ignore_ascii_case("hosking-jfood")
        || template.eq_ignore_ascii_case("parabr")
        || template.eq_ignore_ascii_case("Multiref2")
        || template.eq_ignore_ascii_case("Age in years, months, weeks and days")
        || template.eq_ignore_ascii_case("est.")
        || template.eq_ignore_ascii_case("e28")
        || template.eq_ignore_ascii_case("Britannica URL")
        || template.eq_ignore_ascii_case("citation-attribution")
        || template.eq_ignore_ascii_case("olist")
        || template.eq_ignore_ascii_case("ordered list")
        || template.eq_ignore_ascii_case("webtrans")
        || template.eq_ignore_ascii_case("osm")
        || template.eq_ignore_ascii_case("wiktionary-inline")
        || template.eq_ignore_ascii_case("wiktionary inline")
        || template.eq_ignore_ascii_case("wti")
        || template.eq_ignore_ascii_case("cite opentopomap")
        || template.eq_ignore_ascii_case("colorbull")
        || template.eq_ignore_ascii_case("portal-inline")
        || template.eq_ignore_ascii_case("portal inline")
        || template.eq_ignore_ascii_case("mp")
        || template.eq_ignore_ascii_case("minor planet")
        || is_silent_template_name(template)
}

fn is_silent_template_name(template: &str) -> bool {
    let template = template.trim();
    template.starts_with('#')
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
