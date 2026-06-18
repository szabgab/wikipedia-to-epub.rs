use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, RETRY_AFTER};
use tracing::Level;

use crate::DownloadCache;
use crate::DownloadStats;
use crate::FixturePageSource;
use crate::ImageRegistry;
use crate::InternalLinks;
use crate::PageResponse;
use crate::TemplateSkipCounts;
use crate::USER_AGENT;
use crate::article_file_candidates;
use crate::cache::CacheSource;
use crate::cache::FileDownloadSnapshot;
use crate::cache::FileDownloadStats;
use crate::cache::PageSource;
use crate::cache::http_failure_detail;
use crate::cache::read_or_fetch_bytes_with_stats;
use crate::cache::read_or_fetch_text_with_stats;
use crate::cache::wikipedia_parse_api_url;
use crate::cache::write_cache_text;
use crate::config::parse_args_from;
use crate::config::parse_config_str;
use crate::config::{ArticleConfig, ArticleType, BookConfig, CachingMode, LinksToExcludedPages};
use crate::error::{AppError, AppResult};
use crate::html_language_attributes;
use crate::internal_links;
use crate::normalized_wikipedia_language;
use crate::render_templates;
use crate::render_wikitext_tables;
use crate::render_wikitext_with_template_counts;
use crate::render_wikitext_with_template_counts_and_excluded_links;
use crate::strip_file_links;
use crate::templates::template_log_content;
use crate::templates::template_name_is_in_csv;
use crate::wikipedia_article_url;

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
fn render_wikitext_formats_reflist_template() {
    let rendered = render_wikitext(
        "Sample",
        r#"Intro text.<ref>{{cite web|title=Example reference|url=https://example.com|website=Example}}</ref>

== References ==
{{Reflist}}"#,
        &InternalLinks::new(),
        "en",
    );

    assert!(rendered.contains("<h2>References</h2>"), "{rendered}");
    assert!(
        rendered.contains(r#"<ol class="references">"#),
        "{rendered}"
    );
    assert!(rendered.contains("Example reference"), "{rendered}");
    assert!(rendered.contains(r#"https://example.com"#), "{rendered}");
}

#[test]
fn render_wikitext_formats_reflist_with_named_refs_param() {
    let rendered = render_wikitext(
        "Sample",
        r#"Named note text.<ref group="n" name="alpha" />

== Notes ==
{{Reflist|group=n|refs=
<ref group="n" name="alpha">{{cite web|title=Named note|url=https://example.com/note}}</ref>
}}"#,
        &InternalLinks::new(),
        "en",
    );

    assert!(rendered.contains("<h2>Notes</h2>"), "{rendered}");
    assert!(
        rendered.contains(r#"<ol class="references">"#),
        "{rendered}"
    );
    assert!(rendered.contains("Named note"), "{rendered}");
    assert!(
        rendered.contains(r#"https://example.com/note"#),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_handles_sections_links_and_lists() {
    let internal_links = internal_links(&["Sample".to_string(), "Seoul".to_string()]);
    let (rendered, counts) = render_wikitext_with_template_counts(
        "Sample",
        r#"Intro with [[Link target|visible text]] and '''bold''' text. See [[Seoul]].

== History ==
* First item
* Second [https://example.com link]
[[Category:Hidden]]
{{Infobox road|ignored=yes}}
<ref>omit this</ref>
"#,
        &internal_links,
        "en",
        None,
    );

    assert!(
            rendered.contains(
                r#"<p>Intro with <a href="https://en.wikipedia.org/wiki/Link_target">visible text</a><span class="external-link">↗</span> and <strong>bold</strong> text. See <a href="Seoul.xhtml">Seoul</a>.</p>"#
            )
        );
    assert!(rendered.contains("<h2>History</h2>"));
    assert!(rendered.contains("<ul>"));
    assert!(rendered.contains("<li>First item</li>"));
    assert!(rendered.contains("<li>Second link</li>"));
    assert!(!rendered.contains("Category:Hidden"));
    assert!(!rendered.contains("Infobox"));
    assert!(!rendered.contains("omit this"));

    assert_eq!(
        counts,
        TemplateSkipCounts {
            recognized: 1,
            unknown: 0
        }
    );
}

#[test]
fn excluded_wikipedia_links_can_be_displayed_without_emphasis() {
    let rendered = render_wikitext_with_excluded_links(
        "Sample",
        "See [[Busan]].",
        &InternalLinks::new(),
        "en",
        LinksToExcludedPages::Display,
    );

    assert!(
        rendered.contains(r#"<p>See <a href="https://en.wikipedia.org/wiki/Busan">Busan</a>.</p>"#),
        "{rendered}"
    );
    assert!(!rendered.contains(r#"class="external-link""#), "{rendered}");
}

#[test]
fn excluded_wikipedia_links_can_be_disregarded() {
    let rendered = render_wikitext_with_excluded_links(
        "Sample",
        "See [[Busan]].",
        &InternalLinks::new(),
        "en",
        LinksToExcludedPages::Disregard,
    );

    assert!(rendered.contains("<p>See Busan.</p>"), "{rendered}");
    assert!(!rendered.contains("<a href="), "{rendered}");
}

#[test]
fn render_wikitext_formats_for_templates() {
    let cases = [
        (
            "{{For|histories of the modern Korean countries|History of North Korea|History of South Korea}}",
            r#"For histories of the modern Korean countries, see: <a href="https://en.wikipedia.org/wiki/History_of_North_Korea">History of North Korea</a><span class="external-link">↗</span> and <a href="https://en.wikipedia.org/wiki/History_of_South_Korea">History of South Korea</a><span class="external-link">↗</span>"#,
        ),
        (
            "{{for|other uses|Korea (disambiguation)}}",
            r#"For other uses, see: <a href="https://en.wikipedia.org/wiki/Korea_(disambiguation)">Korea (disambiguation)</a><span class="external-link">↗</span>"#,
        ),
    ];

    for (template, expected) in cases {
        let (rendered, counts) = render_wikitext_with_template_counts(
            "Sample",
            template,
            &InternalLinks::new(),
            "en",
            None,
        );
        assert!(
            rendered.contains(expected),
            "For template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("For|"));
        assert_eq!(
            counts,
            TemplateSkipCounts {
                recognized: 0,
                unknown: 0
            }
        );
    }
}

#[test]
fn render_wikitext_formats_for_timeline_templates() {
    let (rendered, counts) = render_wikitext_with_template_counts(
        "Sample",
        "{{For timeline|Timeline of Sample}}",
        &InternalLinks::new(),
        "en",
        None,
    );

    assert!(rendered.contains("For a timeline, see: <a href=\"https://en.wikipedia.org/wiki/Timeline_of_Sample\">Timeline of Sample</a>"));
    assert_eq!(
        counts,
        TemplateSkipCounts {
            recognized: 0,
            unknown: 0
        }
    );
}

#[test]
fn render_wikitext_formats_excerpt_templates() {
    let rendered = render_wikitext(
        "Sample",
        "{{Excerpt|Korean literature|templates=no}}",
        &InternalLinks::new(),
        "en",
    );

    assert!(rendered.contains("Excerpt from: <a href=\"https://en.wikipedia.org/wiki/Korean_literature\">Korean literature</a>"));
}

#[test]
fn render_wikitext_formats_coord_templates() {
    let rendered = render_wikitext(
        "Sample",
        "{{Coord|37|33|36|N|126|59|24|E|region:KR-11_type:adm1st|display=inline,title}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("<p>37°33′36″N 126°59′24″E</p>"),
        "{rendered}"
    );

    let rendered = render_wikitext(
        "Sample",
        "{{Coord|43.65107|-79.347015|type:city|display=inline}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("<p>43.65107, -79.347015</p>"),
        "{rendered}"
    );

    let rendered = render_wikitext(
        "Sample",
        "{{Coord|37|33|36|N|126|59|24|E|display=title}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("37°33′36″N 126°59′24″E"), "{rendered}");

    let rendered = render_wikitext("Sample", "{{Coord|qid=Q884}}", &InternalLinks::new(), "en");
    assert!(!rendered.contains("Q884"), "{rendered}");
    assert!(!rendered.contains("Coord"), "{rendered}");
}

#[test]
fn render_wikitext_formats_frac_templates() {
    let rendered = render_wikitext("Sample", "{{frac|2|3}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("<p>2/3</p>"), "{rendered}");

    let rendered = render_wikitext("Sample", "{{frac|1|1|2}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("<p>1 1/2</p>"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{frac|{{linktext|2}}|3}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("<p>2/3</p>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_sfrac_templates() {
    let rendered = render_wikitext("Sample", "{{sfrac|6}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("<sup>1</sup>⁄<sub>6</sub>"), "{rendered}");

    let rendered = render_wikitext("Sample", "{{sfrac|1|6}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("<sup>1</sup>⁄<sub>6</sub>"), "{rendered}");

    let rendered = render_wikitext("Sample", "{{sfrac|2|1|3}}", &InternalLinks::new(), "en");
    assert!(
        rendered.contains("2 <sup>1</sup>⁄<sub>3</sub>"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_mvar_template() {
    let rendered = render_wikitext("Sample", "{{mvar|k}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("<em>k</em>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_math_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{math|''y'' {{=}} 2}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("<em>y</em> = 2"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{math|1=2 + 2 = 4}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("2 + 2 = 4"), "{rendered}");
}

#[test]
fn render_wikitext_formats_han_dynasty_templates() {
    let rendered = render_wikitext(
        "Sample",
        "{{floruit|6th century&nbsp;BC}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("fl. 6th century BC"), "{rendered}");

    let rendered = render_wikitext("Sample", "{{floruit}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("fl."), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{fraction|365|385|1539}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("<p>365 385/1539</p>"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{fraction|29|43|81}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("<p>29 43/81</p>"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{Library resources box|onlinebooks=yes}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(!rendered.contains("Library resources box"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{Spoken Wikipedia|EN-Han_dynasty-article.ogg|date=2016-04-27}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(!rendered.contains("Spoken Wikipedia"), "{rendered}");
}

#[test]
fn render_wikitext_formats_okina_template() {
    let rendered = render_wikitext("Sample", "{{okina}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("ʻ"), "{rendered}");
}

#[test]
fn render_wikitext_formats_possessive_s_template() {
    let rendered = render_wikitext("Sample", "''Han''{{'s}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("<em>Han</em>'s"), "{rendered}");
}

#[test]
fn render_wikitext_silently_skips_contains_special_characters_template() {
    let (rendered, counts) = render_wikitext_with_template_counts(
        "Sample",
        "{{Contains special characters|Old Hangul|section=section}}",
        &InternalLinks::new(),
        "en",
        None,
    );
    assert!(
        !rendered.contains("Contains special characters"),
        "{rendered}"
    );
    assert_eq!(
        counts,
        TemplateSkipCounts {
            recognized: 1,
            unknown: 0
        }
    );
}

#[test]
fn render_wikitext_formats_cite_conference_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{cite conference|author=Smith|title=Ancient Borders|book-title=Proceedings of Archaeology|year=2010}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("Smith. <em>Ancient Borders</em>. 2010"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_worldhistory_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{worldhistory|section=378|quote=the state of Parhae (or Bohai in Chinese)}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("\"the state of Parhae (or Bohai in Chinese)\". <em>The Encyclopedia of World History</em> (6th ed.)"), "{rendered}");
}

#[test]
fn render_wikitext_silently_skips_tree_chart_and_hyphen_templates() {
    let (rendered, counts) = render_wikitext_with_template_counts(
        "Sample",
        "{{tree chart/start}}\n{{tree chart|KNG}}\n{{-}}\n{{tree chart/end}}",
        &InternalLinks::new(),
        "en",
        None,
    );
    assert!(!rendered.contains("tree chart"), "{rendered}");
    assert_eq!(
        counts,
        TemplateSkipCounts {
            recognized: 4,
            unknown: 0
        }
    );
}

#[test]
fn render_wikitext_formats_nihongo2_template() {
    let rendered = render_wikitext("Sample", "{{nihongo2|日本}}", &InternalLinks::new(), "en");
    assert!(
        rendered.contains("<span lang=\"ja\">日本</span>"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_gloss_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{gloss|His Majesty's Reign}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("'His Majesty's Reign'"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{gloss|mode=def|ensemble drumming}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("(ensemble drumming)"), "{rendered}");
}

#[test]
fn render_wikitext_formats_xref_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{xref|(see [[Nanban trade]])}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("(see <a href=\"https://en.wikipedia.org/wiki/Nanban_trade\">Nanban trade</a><span class=\"external-link\">↗</span>)"), "{rendered}");
}

#[test]
fn render_wikitext_formats_shy_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Shy|Pre|fec|tures}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Pre\u{ad}fec\u{ad}tures"), "{rendered}");
}

#[test]
fn render_wikitext_formats_color_box_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{color box|#EF7979}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("<span style=\"color: #EF7979;\">■</span>"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_color_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{color|#EF7979|Colored text}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("<span style=\"color: #EF7979;\">Colored text</span>"),
        "{rendered}"
    );

    let rendered_british = render_wikitext(
        "Sample",
        "{{colour|red|British spelling}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered_british.contains("<span style=\"color: red;\">British spelling</span>"),
        "{rendered_british}"
    );

    let rendered_named = render_wikitext(
        "Sample",
        "{{color|color=blue|text=Named parameter}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered_named.contains("<span style=\"color: blue;\">Named parameter</span>"),
        "{rendered_named}"
    );
}

#[test]
fn render_wikitext_formats_pb_template() {
    let rendered = render_wikitext("Sample", "first{{pb}}second", &InternalLinks::new(), "en");
    assert!(rendered.contains("first<br /><br />second"), "{rendered}");
}

#[test]
fn render_wikitext_formats_osm_relation_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{OSM relation|382313}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("OpenStreetMap relation 382313"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_harvp_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{harvp|Martin|1966}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("(Martin 1966)"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{harvp|Whitman|1985|p=232}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("(Whitman 1985, p. 232)"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{harvp|Sohn|2001|loc=Section 1.5.3, pp. 12–13}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("(Sohn 2001, Section 1.5.3, pp. 12–13)"),
        "{rendered}"
    );

    let rendered = render_wikitext(
        "Sample",
        "{{harvp|Kang Yoonjung|Han Sungwoo|2013}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("(Kang Yoonjung &amp; Han Sungwoo 2013)"),
        "{rendered}"
    );

    let rendered = render_wikitext(
        "Sample",
        "{{harvp|Choi Jiyoun|Kim Sahyang|Cho Taehong|2020}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("(Choi Jiyoun, Kim Sahyang, &amp; Cho Taehong 2020)"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_harv_and_harvnb_templates() {
    let rendered_harv =
        render_wikitext("Sample", "{{harv|Davis|1999}}", &InternalLinks::new(), "en");
    assert!(rendered_harv.contains("(Davis 1999)"), "{rendered_harv}");

    let rendered_harvnb = render_wikitext(
        "Sample",
        "{{harvnb|Davis|1999|p=10}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered_harvnb.contains("Davis 1999, p. 10"),
        "{rendered_harvnb}"
    );
    assert!(!rendered_harvnb.contains("("), "{rendered_harvnb}");
}

#[test]
fn render_wikitext_formats_harvtxt_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{harvtxt|Martin|1966}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Martin (1966)"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{harvtxt|Whitman|1985|p=232}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Whitman (1985, p. 232)"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{harvtxt|Sohn|2001|loc=Section 1.5.3}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("Sohn (2001, Section 1.5.3)"),
        "{rendered}"
    );

    let rendered = render_wikitext(
        "Sample",
        "{{harvtxt|Kang Yoonjung|Han Sungwoo|2013}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("Kang Yoonjung &amp; Han Sungwoo (2013)"),
        "{rendered}"
    );

    let rendered = render_wikitext(
        "Sample",
        "{{harvtxt|Choi Jiyoun|Kim Sahyang|Cho Taehong|2020}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("Choi Jiyoun, Kim Sahyang, &amp; Cho Taehong (2020)"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_ndldc_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{NDLDC|782854/146}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("https://dl.ndl.go.jp/info:ndljp/pid/782854/146"),
        "{rendered}"
    );

    let rendered = render_wikitext(
        "Sample",
        "{{NDLDC|782854/146|format=url}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("https://dl.ndl.go.jp/en/pid/782854/146"),
        "{rendered}"
    );

    let rendered = render_wikitext(
        "Sample",
        "{{NDLDC|782854/146|format=pid}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("ndlpid"), "{rendered}");
    assert!(rendered.contains("782854/146"), "{rendered}");
    assert!(
        !rendered.contains("https://dl.ndl.go.jp/en/pid/782854/146"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_plainlist_templates() {
    let rendered = render_wikitext(
        "Sample",
        "{{plainlist|1=*[[Tokugawa Ieyasu]]\n*[[Maeda Toshiie]]}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Tokugawa Ieyasu"), "{rendered}");
    assert!(rendered.contains("Maeda Toshiie"), "{rendered}");

    let rendered_positional = render_wikitext(
        "Sample",
        "{{plainlist|*[[Tokugawa Ieyasu]]}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered_positional.contains("Tokugawa Ieyasu"),
        "{rendered_positional}"
    );
}

#[test]
fn render_wikitext_formats_interlanguage_link_alias_templates() {
    let rendered = render_wikitext(
        "Sample",
        "{{Interlanguage link|Sekigahara (1981 miniseries)|lt=''Sekigahara''|ja|関ヶ原 (テレビドラマ)}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Sekigahara"), "{rendered}");
}

#[test]
fn render_wikitext_formats_interlanguage_link_multi_alias_templates() {
    let rendered = render_wikitext(
        "Sample",
        "{{Interlanguage link multi|List of governors of Tokyo|ja|東京都知事一覧}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("List of governors of Tokyo"),
        "{rendered}"
    );
    assert!(rendered.contains("[ja]"), "{rendered}");
}

#[test]
fn render_wikitext_formats_illm_alias_templates() {
    let rendered = render_wikitext(
        "Sample",
        "{{illm|In-no-chō|ja|院庁|lt=''In-no-chō''}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("In-no-chō"), "{rendered}");
    assert!(rendered.contains("[ja]"), "{rendered}");
}

#[test]
fn render_wikitext_formats_jaanus_templates() {
    let rendered = render_wikitext(
        "Sample",
        "{{Jaanus|w/washi|Washi}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("Washi</a><span class=\"external-link\">↗</span> at JAANUS"),
        "{rendered}"
    );

    let rendered_no_label =
        render_wikitext("Sample", "{{Jaanus|w/washi}}", &InternalLinks::new(), "en");
    assert!(
        rendered_no_label.contains("w/washi</a><span class=\"external-link\">↗</span> at JAANUS"),
        "{rendered_no_label}"
    );
}

#[test]
fn render_wikitext_formats_translit_templates() {
    let rendered = render_wikitext(
        "Sample",
        "{{translit|ja|[[Genkō yōshi]]}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Genkō yōshi"), "{rendered}");
}

#[test]
fn render_wikitext_formats_cite_gvp_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{cite gvp|name=Norikuradake|vn=283060|access-date=2021-06-24}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains(r#""Norikuradake". <em>Global Volcanism Program</em>. Smithsonian Institution. Retrieved 2021-06-24"#),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_ipa_link_template() {
    let rendered = render_wikitext("Sample", "{{IPAslink|m}}", &InternalLinks::new(), "en");
    assert!(
        rendered.contains(r#"<span title="International Phonetic Alphabet">[m]</span>"#),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_angbr_templates() {
    let rendered = render_wikitext("Sample", "{{angbr|a}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("⟨a⟩"), "{rendered}");

    let rendered = render_wikitext("Sample", "{{angbr IPA|◌̧}}", &InternalLinks::new(), "en");
    assert!(
        rendered.contains("⟨<span lang=\"und-fonipa\">◌̧</span>⟩"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_unichar_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{unichar|0348|cwith=◌}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("◌͈ (U+0348)"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{unichar|0348|COMBINING DOUBLE VERTICAL LINE BELOW|cwith=◌}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("◌͈ (U+0348 COMBINING DOUBLE VERTICAL LINE BELOW)"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_xlit_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{xlit|ko|'''r'''odong}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("<span lang=\"ko-Latn\"><strong>r</strong>odong</span>"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_note_template() {
    let rendered = render_wikitext("Sample", "{{note|ㅏ|[A]}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("<strong>[A]</strong>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_fs_interlinear_template() {
    let wikitext = "{{fs interlinear|lang=ko\n| 가게에 가셨어요?\n| gage-e ga-syeoss-eo-yo\n| store-LOC go-HON.PAST-CONJ-POL\n| 'Did [you] go to the store?'\n}}";
    let rendered = render_wikitext("Sample", wikitext, &InternalLinks::new(), "en");
    assert!(rendered.contains("<blockquote>"), "{rendered}");
    assert!(
        rendered.contains("<strong><span lang=\"ko\">가게에 가셨어요?</span></strong>"),
        "{rendered}"
    );
    assert!(
        rendered.contains("<em>gage-e ga-syeoss-eo-yo</em>"),
        "{rendered}"
    );
    assert!(
        rendered.contains("<p>store-LOC go-HON.PAST-CONJ-POL</p>"),
        "{rendered}"
    );
    assert!(
        rendered.contains("<em>'Did [you] go to the store?'</em>"),
        "{rendered}"
    );
    assert!(rendered.contains("</blockquote>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_tooltip_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Tooltip|RR|Revised Romanization}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains(r#"<abbr title="Revised Romanization">RR</abbr>"#),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_nihongo_krt_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Nihongo krt||\u{5927}\u{962a}|\u{14c}saka}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("<span lang=\"ja\">\u{5927}\u{962a}</span> (<em>\u{14c}saka</em>)"),
        "{rendered}"
    );

    let rendered = render_wikitext(
        "Sample",
        "{{Nihongo krt|'knight'|\u{58eb}}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("<span lang=\"ja\">\u{58eb}</span> ('knight')"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_easy_css_image_crop_template() {
    let rendered = render_templates(
        "{{Easy CSS image crop|Image=Osaka Urban Railway network.svg|desired_width=300|caption=The rail network.}}",
    );
    assert_eq!(
        rendered,
        "[[File:Osaka Urban Railway network.svg|thumb|The rail network.]]"
    );
}

#[test]
fn render_wikitext_formats_south_korea_provincial_level_labelled_map_template() {
    let rendered = render_templates("{{South Korea Provincial level Labelled Map}}");
    assert_eq!(
        rendered,
        "[[File:Provinces of Korea (ROK point of view)+Inter-Korean border.svg|thumb|South Korea Provincial level Labelled Map]]"
    );
}

#[test]
fn render_wikitext_formats_multiple_images_template() {
    let wikitext = "{{Multiple images\n\
        | align = right\n\
        | direction = vertical\n\
        | header = Some Header\n\
        | footer = Some Footer\n\
        | image1 = Yoshiwara M.jpg\n\
        | caption1 = First Caption\n\
        | image2 = Hokusai.jpg\n\
        | caption2 = Second Caption\n\
    }}";
    let rendered = render_templates(wikitext);
    assert!(
        rendered.contains("<p><strong>Some Header</strong></p>"),
        "{rendered}"
    );
    assert!(
        rendered.contains("[[File:Yoshiwara M.jpg|thumb|First Caption]]"),
        "{rendered}"
    );
    assert!(
        rendered.contains("[[File:Hokusai.jpg|thumb|Second Caption]]"),
        "{rendered}"
    );
    assert!(
        rendered.contains("<p><em>Some Footer</em></p>"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_issn_template() {
    let rendered = render_wikitext("Sample", "{{ISSN|0268-4160}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("ISSN 0268-4160"), "{rendered}");
}

#[test]
fn render_wikitext_formats_doi_template() {
    let rendered1 = render_wikitext(
        "Sample",
        "{{doi|10.1080/02757206.2013.726990}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered1.contains("doi:10.1080/02757206.2013.726990"),
        "{rendered1}"
    );

    let rendered2 = render_wikitext(
        "Sample",
        "{{doi|1=10.2307/20033332}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered2.contains("doi:10.2307/20033332"), "{rendered2}");
}

#[test]
fn render_wikitext_formats_age_template() {
    let rendered1 = render_wikitext(
        "Sample",
        "{{age|1989|11|9|2019|11|9}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered1.contains("30"), "{rendered1}");

    let rendered2 = render_wikitext(
        "Sample",
        "{{age|-660|2|11|2026|6|3}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered2.contains("2685"), "{rendered2}");

    let rendered3 = render_wikitext("Sample", "{{age|1989|11|9}}", &InternalLinks::new(), "en");
    assert!(
        !rendered3.trim().is_empty(),
        "age output should not be empty"
    );
}

#[test]
fn render_wikitext_formats_ayd_template() {
    let rendered1 = render_wikitext(
        "Sample",
        "{{ayd|April 26, 2001|September 26, 2006}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered1.contains("5 years, 153 days"), "{rendered1}");

    let rendered2 = render_wikitext(
        "Sample",
        "{{ayd|1 October 2024}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        !rendered2.trim().is_empty(),
        "ayd output should not be empty"
    );

    let rendered3 = render_wikitext(
        "Sample",
        "{{ayd|2001|4|26|2006|9|26}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered3.contains("5 years, 153 days"), "{rendered3}");

    let rendered4 = render_wikitext("Sample", "{{ayd|2024|10|1}}", &InternalLinks::new(), "en");
    assert!(
        !rendered4.trim().is_empty(),
        "ayd output should not be empty"
    );
}

#[test]
fn render_wikitext_formats_cite_nsrw_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Cite NSRW|short=x|wstitle=Osaka}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("\"Osaka\""), "{rendered}");
    assert!(
        rendered.contains("The New Student's Reference Work"),
        "{rendered}"
    );
    assert!(
        rendered.contains("https://en.wikisource.org/wiki/"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_silently_skips_osaka_metadata_templates() {
    let (rendered, counts) = render_wikitext_with_template_counts(
        "Sample",
        "{{Div end}}\n{{Sister bar|auto=y}}\n{{Osaka}}\n{{Osaka Prefecture}}",
        &InternalLinks::new(),
        "en",
        None,
    );
    assert!(!rendered.contains("Div end"), "{rendered}");
    assert!(!rendered.contains("Sister bar"), "{rendered}");
    assert!(!rendered.contains("Osaka"), "{rendered}");
    assert_eq!(
        counts,
        TemplateSkipCounts {
            recognized: 4,
            unknown: 0
        }
    );
}

#[test]
fn render_wikitext_silently_skips_japan_metadata_templates() {
    let (rendered, counts) = render_wikitext_with_template_counts(
        "Sample",
        "{{redirect-several|Japan|Nihon}}\n{{bots|deny=OAbot}}\n{{TOClimit|3}}",
        &InternalLinks::new(),
        "en",
        None,
    );
    assert!(!rendered.contains("redirect-several"), "{rendered}");
    assert!(!rendered.contains("bots"), "{rendered}");
    assert_eq!(
        counts,
        TemplateSkipCounts {
            recognized: 3,
            unknown: 0
        }
    );
}

#[test]
fn render_wikitext_formats_korean_war_templates() {
    // 1. For-multi
    let rendered = render_wikitext(
        "Sample",
        "{{For-multi|other conflicts and wars involving Korea|List of Korean battles|the conflict from 1945 to the present|Korean conflict}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("For other conflicts and wars involving Korea, see <a href=\"https://en.wikipedia.org/wiki/List_of_Korean_battles\">List of Korean battles</a><span class=\"external-link\">↗</span>; for the conflict from 1945 to the present, see <a href=\"https://en.wikipedia.org/wiki/Korean_conflict\">Korean conflict</a><span class=\"external-link\">↗</span>."), "{rendered}");

    // 2. Inflation & Inflation/year
    let rendered = render_wikitext(
        "Sample",
        "equivalent to ${{Inflation|US|12|1950|fmt=c}}&nbsp;billion in {{Inflation/year|US}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("equivalent to $152 billion in 2023"),
        "{rendered}"
    );

    // 3. stack (passthrough)
    let rendered = render_wikitext(
        "Sample",
        "{{stack|Some stacked text}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("<p>Some stacked text</p>"), "{rendered}");

    // 4. USS & HMS
    let rendered = render_wikitext(
        "Sample",
        "{{USS|Missouri|BB-63|6}} and {{HMS|Jamaica|44|6}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("<a href=\"https://en.wikipedia.org/wiki/USS_Missouri_(BB-63)\"><em>Missouri</em></a><span class=\"external-link\">↗</span> and <a href=\"https://en.wikipedia.org/wiki/HMS_Jamaica_(44)\"><em>Jamaica</em></a><span class=\"external-link\">↗</span>"), "{rendered}");

    // 5. Collapsible list
    let rendered = render_wikitext(
        "Sample",
        "{{Collapsible list|bullets=yes|title=Breakdown of UN casualties|Item A|Item B}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("Breakdown of UN casualties"),
        "{rendered}"
    );
    assert!(rendered.contains("<li>Item A</li>"), "{rendered}");
    assert!(rendered.contains("<li>Item B</li>"), "{rendered}");

    // 6. Internet Archive short film
    let rendered = render_wikitext(
        "Sample",
        "{{Internet Archive short film|id=gov.archives.li.263.927|name=Film No. 927}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("<a href=\"https://archive.org/details/gov.archives.li.263.927\"><em>Film No. 927</em></a><span class=\"external-link\">↗</span> at the Internet Archive"), "{rendered}");

    // 7. Silent templates
    for t in &[
        "{{very long|date=December 2024|words=15,400}}",
        "{{additional citations needed|date=September 2025}}",
        "{{long|section|words=7,000|date=September 2025}}",
        "{{who|date=January 2026}}",
        "{{R|jstor2538736}}",
        "{{Explain|date=February 2026}}",
        "{{Ref|25|map}}",
        "{{PD-notice}}",
    ] {
        let rendered = render_wikitext("Sample", t, &InternalLinks::new(), "en");
        assert!(
            !rendered.contains("very long")
                && !rendered.contains("additional citations")
                && !rendered.contains("Explain"),
            "Failed on silent {}: {}",
            t,
            rendered
        );
    }
}

#[test]
fn render_wikitext_formats_historical_populations_templates() {
    let rendered = render_wikitext(
        "Sample",
        "{{Historical populations|5=1949|6=1437670|7=1960|8=2445402|align=right|source=ignored}}",
        &InternalLinks::new(),
        "en",
    );

    assert!(
        rendered.contains("<p>Historical populations:</p>"),
        "{rendered}"
    );
    assert!(rendered.contains("<li>1949: 1,437,670</li>"), "{rendered}");
    assert!(rendered.contains("<li>1960: 2,445,402</li>"), "{rendered}");
    assert!(!rendered.contains("align=right"), "{rendered}");
    assert!(!rendered.contains("source=ignored"), "{rendered}");
}

#[test]
fn render_wikitext_formats_climate_chart_templates() {
    let rendered = render_wikitext(
        "Sample",
        "{{climate chart|Busan|−0.1|8.2|34.5|1.5|10.2|49.6|5.3|13.8|89.7|10.1|18.2|140.9|14.6|22.0|155.9|18.3|24.6|188.4|22.1|27.5|326.8|23.7|29.5|266.5|19.8|26.4|160.6|14.5|22.5|79.6|8.3|16.6|50.4|2.0|10.4|33.8|float=Right}}",
        &InternalLinks::new(),
        "en",
    );

    assert!(
        rendered.contains("<p>Climate chart for Busan:</p>"),
        "{rendered}"
    );
    assert!(
        rendered.contains("<li>Jan: −0.1 to 8.2 °C, 34.5 mm</li>"),
        "{rendered}"
    );
    assert!(
        rendered.contains("<li>Dec: 2.0 to 10.4 °C, 33.8 mm</li>"),
        "{rendered}"
    );
    assert!(!rendered.contains("{{"));
    assert!(!rendered.contains("climate chart"));
    assert!(!rendered.contains("float=Right"));
}

#[test]
fn render_wikitext_silently_skips_unknown_templates() {
    let (rendered, counts) = render_wikitext_with_template_counts(
        "Sample",
        r#"Before
{{Some template name}}
{{Some template|with|parameters}}
Visible text."#,
        &InternalLinks::new(),
        "en",
        None,
    );
    assert_eq!(
        rendered,
        r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en">
  <head>
    <title>Sample</title>
    <link rel="stylesheet" type="text/css" href="style.css" />
  </head>
  <body>
    <h1>Sample</h1>
    <p>Before</p>
    <p>Visible text.</p>
  </body>
</html>
"#
    );
    assert_eq!(
        counts,
        TemplateSkipCounts {
            recognized: 0,
            unknown: 2
        }
    );
}

#[test]
fn render_wikitext_silently_skips_metadata_templates() {
    let (rendered, counts) = render_wikitext_with_template_counts(
        "Sample",
        r#"{{Short description|Sample page}}
{{About|the sample|other uses|Sample (disambiguation)}}
{{Distinguish|Example}}
{{ISBN?}}
{{Pp-move}}
{{Pp-pc|small=yes}}
{{Protection padlock|small=yes}}
{{Redirect|Sample}}
{{redirect-multi|3|Pusan|Fusan|Busan Metropolitan City|other uses|Pu San (disambiguation)}}
{{pp-semi-indef}}
{{Sfn|Author|2024|p=1}}
{{sfnm|1a1=Author|1y=2024|1p=1}}
{{efn|Footnote text}}
{{efn-ua|Footnote text}}
{{refn|Reference note text}}
{{Reflist|30em}}
{{notelist}}
{{notelist-ua}}
{{Refbegin|30em}}
{{refend}}
{{NoteFoot}}
{{flagicon|US}}
{{unreferenced section|date=November 2022}}
{{more citations needed section|date=May 2024}}
{{unbalanced opinion|date=April 2021}}
{{disputed section|date=April 2021}}
{{Overly detailed|section|details=not clear why this list is exclusive to the end of the article|date=April 2021}}
{{Excessive citations inline|date=November 2022}}
{{More citations needed|date=June 2022}}
{{Refimprove|date=December 2025}}
{{FACT|date=December 2025}}
{{citation needed|date=May 2023}}
{{cn|date=May 2025}}
{{anchor|Modern}}
{{huh|date=August 2025}}
{{when|date=August 2025}}
{{more cn section|date=August 2025|find=Korea|find2=1951 to present}}
{{cbignore|bot=medic}}
{{prose|section|date=August 2019}}
{{Unreliable source?|date=May 2026}}
{{Better source needed|date=May 2026}}
{{Dead link|date=May 2026}}
{{Page needed|date=May 2026}}
{{New archival link needed|date=April 2026}}
{{clear}}
{{div}}
{{div col|colwidth=20em}}
{{div col end}}
{{col-begin|width=auto}}
{{col-break|gap=1em}}
{{col-end}}
{{Portal bar|North Korea|South Korea|Asia|History|Linguistics|Monarchy|Biography}}
{{TOC limit|4}}
{{DEFAULTSORT:Sample, Page}}
{{Self-published|date=May 2026}}
{{self-published inline|date=May 2026}}
{{Use British English|date=March 2022}}
{{Use dmy dates|date=April 2022}}
{{Infobox road|name=Sample}}
{{History of Korea}}
{{Korea topics}}
{{East Asian topics}}
{{History of Asia}}
{{Seoul}}
{{Seoul weatherbox}}
{{Seoul landmarks}}
{{Busan}}
{{Other uses}}
{{Busan weatherbox}}
{{Navboxes|title=Articles related to Seoul}}
{{Authority control}}
{{Portal|Geography|Asia|North Korea|South Korea}}
{{Sister project links|Busan|voy=Busan|d=Q16520}}
{{Commons category|Sample page}}
{{Commons and category|Sample page}}
{{Commons-inline|Sample page}}
{{columns-list|colwidth=23em|
* Hidden first column item
* Hidden second column item
}}
{{location map+|Korea|places={{location map~|Korea|lat=37|long=127|label=Sample marker}}}}
{{Wikisource-inline|list={{Cite EB1911|wstitle=Sample|short=1}}}}
{{Wide image|Sample panorama.jpg|800px|Sample panorama}}
{{Pie chart|value1=40|label1=Hidden slice}}
{{ahnentafel|1=Hidden ancestor}}
{{S-start}}
{{s-hou|[[House of Yi]]|10 April|1397|17 February|1450}}
{{s-reg}}
{{s-bef|before=[[Taejong of Joseon|Taejong]]}}
{{s-ttl|title=[[List of monarchs of Korea|King of Joseon]]|years=1418–1450}}
{{s-aft|after=[[Munjong of Joseon|Munjong]]}}
{{s-end}}
{{Succession box|title=[[Capital of Korea|Capital of Baekje]]|before=|after=[[Gongju|Ungjin]]|years=18 BC – 475 AD}}
{{Busan weatherbox}}
{{Seoul weatherbox}}
{{Jeju City weatherbox}}
{{Seogwipo weatherbox}}
{{disambiguation|geo}}
{{in title|Mercury}}
{{look from|Mercury}}
{{tocright}}
{{CS1 config|mode=cs1}}
{{unsolved|astronomy| corona }}
{{discuss|talk page section}}
{{Italic title|reason=Category:Japanese words and phrases}}
{{Expand Japanese|和紙|topic=cult|date=August 2021}}
{{Tone inline|date=April 2026}}
{{j-railservice start}}
{{j-route|route=Takayama Main Line|col=#f77321|f=w}}
{{j-rserv|service=Limited Express}}
{{ja-rail-line|pfn=1-3}}
{{clarification needed|date=April 2022}}
{{Multiple issues}}
{{Advert}}
{{Original research}}
{{Unreferenced}}
Visible text."#,
        &InternalLinks::new(),
        "en",
        None,
    );

    assert_eq!(
        rendered,
        r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en">
  <head>
    <title>Sample</title>
    <link rel="stylesheet" type="text/css" href="style.css" />
  </head>
  <body>
    <h1>Sample</h1>
    <p>Visible text.</p>
  </body>
</html>
"#
    );
    assert_eq!(
        counts,
        TemplateSkipCounts {
            recognized: 112,
            unknown: 0
        }
    );
}

#[test]
fn render_wikitext_reports_template_skip_counts() {
    let (_rendered, counts) = render_wikitext_with_template_counts(
        "Sample",
        r#"{{Short description|Sample}}
{{Unknown template|{{Nested unknown|value}}|{{Dead link|date=May 2026}}}}
Visible text."#,
        &InternalLinks::new(),
        "en",
        None,
    );

    assert_eq!(
        counts,
        TemplateSkipCounts {
            recognized: 2,
            unknown: 2
        }
    );
}

#[test]
fn render_wikitext_formats_ship_class_templates() {
    let cases = [
        (
            "{{sclass|Valiant|harbor tug}}",
            r#"<p><a href="https://en.wikipedia.org/wiki/Valiant-class_harbor_tug"><em>Valiant</em>-class</a><span class="external-link">↗</span> <a href="https://en.wikipedia.org/wiki/harbor_tug">harbor tug</a><span class="external-link">↗</span></p>"#,
        ),
        (
            "{{sclass|Valiant|harbor tug|1}}",
            r#"<p><a href="https://en.wikipedia.org/wiki/Valiant-class_harbor_tug"><em>Valiant</em>-class harbor tug</a><span class="external-link">↗</span></p>"#,
        ),
        (
            "{{sclass|Valiant|harbor tug|2}}",
            r#"<p><a href="https://en.wikipedia.org/wiki/Valiant-class_harbor_tug"><em>Valiant</em>-class</a><span class="external-link">↗</span> harbor tug</p>"#,
        ),
        (
            "{{sclass|Valiant|harbor tug|4}}",
            r#"<p><a href="https://en.wikipedia.org/wiki/Valiant-class_harbor_tug"><em>Valiant</em> class</a><span class="external-link">↗</span></p>"#,
        ),
        (
            "{{sclass|Königsberg|cruiser|||1905}}",
            r#"<p><a href="https://en.wikipedia.org/wiki/Königsberg-class_cruiser_(1905)"><em>Königsberg</em>-class</a><span class="external-link">↗</span> <a href="https://en.wikipedia.org/wiki/cruiser">cruiser</a><span class="external-link">↗</span></p>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "sclass template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("sclass|"));
    }
}

#[test]
fn template_log_content_is_limited_to_twenty_characters() {
    let res = template_log_content(
        "Unhandled template with a long body that is more than 80 characters long. I think.",
    );
    assert_eq!(
        res,
        "Unhandled template with a long body that is more than 80 characters long. I thin"
    );
    assert_eq!(res.len(), 80);
    assert_eq!(
        template_log_content("短いtemplate content"),
        "短いtemplate content"
    );
}

#[test]
fn render_wikitext_formats_italic_markup() {
    let rendered = render_wikitext(
        "Sample",
        "Intro with ''italic text'' and [[Fortune Global 500|''Fortune'' Global 500]].",
        &InternalLinks::new(),
        "en",
    );

    assert!(rendered.contains(
            r#"<p>Intro with <em>italic text</em> and <a href="https://en.wikipedia.org/wiki/Fortune_Global_500"><em>Fortune</em> Global 500</a><span class="external-link">↗</span>.</p>"#
        ));
}

#[test]
fn render_wikitext_parses_example_file() {
    let rendered = render_wikitext(
        "Sample",
        r#"''{{ill|Hyangyakchips\u014fngbang|ko|\ud5a5\uc57d\uc9d1\uc131\ubc29}}''"#,
        &InternalLinks::new(),
        "en",
    );

    assert!(rendered.contains("<h1>Sample</h1>"));
    assert!(rendered.contains(
            r#"<p><em><a href="https://en.wikipedia.org/wiki/Hyangyakchips\u014fngbang">Hyangyakchips\u014fngbang</a><span class="external-link">↗</span> [ko]</em></p>"#
        ));
    assert!(!rendered.contains("{{"));
    assert!(!rendered.contains("ill|"));
}

#[test]
fn render_wikitext_parses_empty_template_inside_italics() {
    let rendered = render_wikitext("Sample", "''{{  }}''", &InternalLinks::new(), "en");

    assert!(rendered.contains("<p><em></em></p>"));
    assert!(!rendered.contains("{{"));
    assert!(!rendered.contains("}}"));
}

#[test]
fn render_wikitext_formats_korean_templates() {
    let rendered = render_wikitext(
        "Sample",
        "Traditionally, ''seoul'' ({{Korean|hangul=서울|labels=no}}) meant capital. Earlier {{Korean|labels=no|위례성|慰禮城}} was nearby. He was called {{Korean/auto|hangul=^해동_^요순|hanja=海東堯舜|mr=yes|labels=no}}. Busan is {{Korean/auto|hangul=부산|hanja=釜山|ko_ipa=pusʰa̠n}}.",
        &InternalLinks::new(),
        "en",
    );

    assert!(rendered.contains(
            r#"<p>Traditionally, <em>seoul</em> (<span title="Korean-language text">Korean: <span lang="ko-Hang">서울</span></span>) meant capital. Earlier <span title="Korean-language text">Korean: <span lang="ko-Hang">위례성</span> / Hanja: <span lang="ko-Hani">慰禮城</span></span> was nearby. He was called <span title="Korean-language text">Korean: <span lang="ko-Hang">해동요순</span> / Hanja: <span lang="ko-Hani">海東堯舜</span></span>. Busan is <span title="Korean-language text">Korean: <span lang="ko-Hang">부산</span> / Hanja: <span lang="ko-Hani">釜山</span> / pronounced [pusʰa̠n]</span>.</p>"#
        ));
}

#[test]
fn render_wikitext_formats_japanese_nihongo4_templates() {
    let rendered = render_wikitext(
        "Sample",
        "The city was formerly {{Nihongo4|''[[Edo (Tokyo)|Edo]]''|[[wikt:江戸|江戸]]}}.",
        &InternalLinks::new(),
        "en",
    );

    assert!(
            rendered.contains(
                r#"<p>The city was formerly <em><a href="https://en.wikipedia.org/wiki/Edo_(Tokyo)">Edo</a><span class="external-link">↗</span></em><span> (<span title="Japanese-language text"><span lang="ja"><a href="https://en.wiktionary.org/wiki/%E6%B1%9F%E6%88%B8">江戸</a><span class="external-link">↗</span></span></span>)</span>.</p>"#
            ),
            "{rendered}"
        );
}

#[test]
fn render_wikitext_formats_japanese_nihongo_templates() {
    let rendered = render_wikitext(
        "Sample",
        "*{{Nihongo|[[Busan Japanese School]]|[[:ja:釜山日本人学校|釜山日本人学校]] |extra={{lang|ko|부산일본인학교}}}}",
        &InternalLinks::new(),
        "en",
    );

    assert!(
        rendered.contains(
            r#"<li><a href="https://en.wikipedia.org/wiki/Busan_Japanese_School">Busan Japanese School</a><span class="external-link">↗</span><span> (<span title="Japanese-language text"><span lang="ja"><a href="https://ja.wikipedia.org/wiki/釜山日本人学校">釜山日本人学校</a><span class="external-link">↗</span></span></span>; <span lang="ko">부산일본인학교</span>)</span></li>"#
        ),
        "{rendered}"
    );
    assert!(!rendered.contains("{{"));
    assert!(!rendered.contains("Nihongo|"));
}

#[test]
fn render_wikitext_formats_japanese_nihongo_extended_templates() {
    // 1. Test positional romaji
    let rendered1 = render_wikitext(
        "Sample",
        "{{nihongo|'''Kiso Mountains'''|木曽山脈|Kiso Sanmyaku}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered1.contains(
            r#"<strong>Kiso Mountains</strong><span> (<span title="Japanese-language text"><span lang="ja">木曽山脈</span></span>, <em>Kiso Sanmyaku</em>)</span>"#
        ),
        "rendered1: {rendered1}"
    );

    // 2. Test positional extra and extra2
    let rendered2 = render_wikitext(
        "Sample",
        "{{nihongo|komusō|虚無僧|komusō|extra text|extra2 text}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered2.contains(
            r#"komusō<span> (<span title="Japanese-language text"><span lang="ja">虚無僧</span></span>, <em>komusō</em>; extra text)</span> extra2 text"#
        ),
        "rendered2: {rendered2}"
    );

    // 3. Test lead=yes parameter
    let rendered3 = render_wikitext(
        "Sample",
        "{{nihongo|Tokyo Tower|東京タワー|Tōkyō tawā|lead=yes}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered3.contains(
            r#"Tokyo Tower<span> (Japanese: <span title="Japanese-language text"><span lang="ja">東京タワー</span></span>, Hepburn: <em>Tōkyō tawā</em>)</span>"#
        ),
        "rendered3: {rendered3}"
    );
}

#[test]
fn render_wikitext_formats_lang_templates() {
    let cases = [
        ("{{lang|ko|서울}}", r#"<p><span lang="ko">서울</span></p>"#),
        (
            "{{lang|ja|''Edo''}}",
            r#"<p><span lang="ja"><em>Edo</em></span></p>"#,
        ),
        (
            "{{lang|ko-Hang|[[Seoul|서울]]}}",
            r#"<p><span lang="ko-Hang"><a href="https://en.wikipedia.org/wiki/Seoul">서울</a><span class="external-link">↗</span></span></p>"#,
        ),
        ("{{lang|ko}}", "<h1>Sample</h1>"),
        ("{{lang|!|서울}}", "<p>서울</p>"),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "lang template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("lang|"));
    }
}

#[test]
fn render_wikitext_formats_hangul_inline_templates() {
    let cases = [
        (
            "{{tlit|ko|mr|Chosŏn'gŭl}}",
            r#"<p><span lang="ko-Latn">Chosŏn'gŭl</span></p>"#,
        ),
        (
            "{{tlit|ko|Hangul}}",
            r#"<p><span lang="ko-Latn">Hangul</span></p>"#,
        ),
        (
            "{{crossreference|See {{slink|#Letter counts}}.}}",
            r#"<p>See <a href="https://en.wikipedia.org/wiki/#Letter_counts">§ Letter counts</a><span class="external-link">↗</span>.</p>"#,
        ),
        (
            "{{crossreference|(see {{slink|Hangul orthography|Buncheol vs. yeoncheol debate}})}}",
            r#"<p>(see <a href="https://en.wikipedia.org/wiki/Hangul_orthography#Buncheol_vs._yeoncheol_debate">Hangul orthography § Buncheol vs. yeoncheol debate</a><span class="external-link">↗</span>)</p>"#,
        ),
        ("{{nobold|{{cn|date=November 2025}}}}", "<h1>Sample</h1>"),
        ("{{Arrow|r}}", "<p>→</p>"),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "Hangul inline template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
    }
}

#[test]
fn render_wikitext_formats_in_lang_templates() {
    let cases = [
        ("{{in lang|ko}}", "<p>(in Korean)</p>"),
        ("{{in lang|ko|en}}", "<p>(in Korean and English)</p>"),
        (
            "{{in lang|ko|ja|zh-hant}}",
            "<p>(in Korean, Japanese, and Chinese)</p>",
        ),
        ("{{in lang|abc}}", "<p>(in abc)</p>"),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "in lang template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("in lang|"));
    }
}

#[test]
fn render_wikitext_formats_linktext_templates() {
    let cases = [
        (
            "{{lang|zh-hant|{{linktext|漢}}}}",
            r#"<p><span lang="zh-hant">漢</span></p>"#,
        ),
        ("{{linktext|漢|字}}", "<p>漢字</p>"),
        (
            "{{linktext|''[[Seoul]]''}}",
            r#"<p><em><a href="https://en.wikipedia.org/wiki/Seoul">Seoul</a><span class="external-link">↗</span></em></p>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "linktext template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("linktext|"));
    }
}

#[test]
fn render_wikitext_formats_langx_templates() {
    let cases = [
        (
            "{{langx|ko|溝樓|lit=Walled City|label=none}}",
            r#"<p><span lang="ko">溝樓</span>, lit. Walled City</p>"#,
        ),
        (
            "{{langx|ko|가우리|lit=Center|label=none}}",
            r#"<p><span lang="ko">가우리</span>, lit. Center</p>"#,
        ),
        (
            "{{Langx|ja|朝鮮|translit=Chōsen|label=none}}",
            r#"<p><span lang="ja">朝鮮</span> (Chōsen)</p>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "langx template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("langx|"));
    }
}

#[test]
fn render_wikitext_formats_chinese_lang_templates() {
    let cases = [
        (
            "{{Lang-zh|t=朝鮮|p=Cháoxiǎn|labels=no}}",
            r#"<p><span lang="zh">朝鮮</span> (Cháoxiǎn)</p>"#,
        ),
        (
            "{{zh|t=西漢|s=西汉|p=Xīhàn|first=t}}",
            r#"<p><span lang="zh">西漢</span> (Xīhàn)</p>"#,
        ),
        ("{{zhi|c=比}}", r#"<p><span lang="zh">比</span></p>"#),
        (
            "{{lang-zh|s=汉字|p=Hànzì}}",
            r#"<p><span lang="zh">汉字</span> (Hànzì)</p>"#,
        ),
        ("{{Lang-zh|中國}}", r#"<p><span lang="zh">中國</span></p>"#),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "Lang-zh template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("Lang-zh"));
        assert!(!rendered.contains("lang-zh"));
    }
}

#[test]
fn render_wikitext_formats_transliteration_templates() {
    let cases = [
        (
            "{{Transliteration|zh|pinyin|Zhuāngxiàn}}",
            r#"<p><span lang="zh-Latn">Zhuāngxiàn</span></p>"#,
        ),
        (
            "{{transliteration|ko|Han'guk}}",
            r#"<p><span lang="ko-Latn">Han'guk</span></p>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "transliteration template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("Transliteration|"));
    }
}

#[test]
fn render_wikitext_formats_korean_transliteration_templates() {
    let cases = [
        ("{{Ko-translit|rr|^한국}}", "Hanguk"),
        ("{{Ko-translit|mr|^한국}}", "Han'guk"),
        ("{{ko-translit|rr|^조선}}", "Joseon"),
        ("{{ko-translit|mr|^조선}}", "Chosŏn"),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(&format!("<p>{expected}</p>")),
            "Ko-translit template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("translit|"));
    }
}

#[test]
fn render_wikitext_formats_literal_templates() {
    let cases = [
        (
            "{{lit|Vernacular Script Commission}}",
            "<p>lit. Vernacular Script Commission</p>",
        ),
        (
            "{{lit|''vernacular'' [[script]]}}",
            r#"<p>lit. <em>vernacular</em> <a href="https://en.wikipedia.org/wiki/script">script</a><span class="external-link">↗</span></p>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "lit template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("lit|"));
    }
}

#[test]
fn render_wikitext_formats_isbn_templates() {
    let cases = [
        ("{{ISBN|0-8248-0673-5}}", "<p>ISBN 0-8248-0673-5</p>"),
        (
            "{{isbn|978-0-674-61576-2}}",
            "<p>ISBN 978-0-674-61576-2</p>",
        ),
        (
            "{{ISBN|''978-0-674-61576-2''}}",
            "<p>ISBN <em>978-0-674-61576-2</em></p>",
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "ISBN template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("ISBN|"));
    }
}

#[test]
fn render_wikitext_formats_ipa_templates() {
    let cases = [
        (
            "{{IPA|ko|haːnɡuk|}}",
            r#"<p><span title="International Phonetic Alphabet">[haːnɡuk]</span></p>"#,
        ),
        (
            "{{IPA|ko|sʰʌ.uɭ|IPA|ko-Seoul.ogg}}",
            r#"<p><span title="International Phonetic Alphabet">[sʰʌ.uɭ]</span></p>"#,
        ),
        (
            "{{IPAc-en|lang|ˈ|tʃ|oʊ|s|ʌ|n}}",
            r#"<p><span title="International Phonetic Alphabet">[ˈtʃoʊsʌn]</span></p>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "IPA template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("IPA|"));
    }
}

#[test]
fn render_wikitext_formats_respell_templates() {
    let rendered = render_wikitext(
        "Sample",
        "{{Respell|CHOH|sun}}",
        &InternalLinks::new(),
        "en",
    );

    assert!(rendered.contains("<p>CHOH-sun</p>"), "{rendered}");
    assert!(!rendered.contains("{{"));
    assert!(!rendered.contains("Respell"));
}

#[test]
fn render_wikitext_formats_abbr_templates() {
    let cases = [
        (
            "{{Abbr|c.|circa}}",
            r#"<p><abbr title="circa">c.</abbr></p>"#,
        ),
        (
            "{{abbr|HTML|HyperText Markup Language}}",
            r#"<p><abbr title="HyperText Markup Language">HTML</abbr></p>"#,
        ),
        (
            "{{abbr|''r.''|reigned}}",
            r#"<p><abbr title="reigned"><em>r.</em></abbr></p>"#,
        ),
        ("{{abbr|kg}}", "<p>kg</p>"),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "abbr template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("abbr|"));
    }
}

#[test]
fn render_wikitext_formats_reference_page_templates() {
    let cases = [
        ("A claim.{{rp|12}}", "<p>A claim. p. 12</p>"),
        ("A claim.{{Rp|12|15}}", "<p>A claim. pp. 12, 15</p>"),
        (
            "A claim.{{rp|{{convert|5|km|abbr=on}}}}",
            "<p>A claim. p. 5 km (3.11 mi)</p>",
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "rp template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("rp|"));
    }
}

#[test]
fn render_wikitext_formats_reference_page_alias_template() {
    let rendered = render_wikitext(
        "Sample",
        "A claim.{{Reference page|page=90}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("p. 90"), "{rendered}");
}

#[test]
fn render_wikitext_formats_cite_book_templates() {
    let cases = [
        (
            "{{Cite book | last = Oberdorfer | first = Don | title=The Two Koreas: a Contemporary History | year =2001| publisher =Basic Books| isbn =978-0465051625|oclc=47831650}}",
            r#"<p>Don Oberdorfer. <em>The Two Koreas: a Contemporary History</em>. Basic Books, 2001. ISBN 978-0465051625. OCLC 47831650</p>"#,
        ),
        (
            "{{cite book|last =Pratt| first = Keith L| title = Everlasting Flower: A History of Korea| year = 2006| publisher =Reaktion| location = London| isbn = 9781861892737 |oclc=63137295}}",
            r#"<p>Keith L Pratt. <em>Everlasting Flower: A History of Korea</em>. London: Reaktion, 2006. ISBN 9781861892737. OCLC 63137295</p>"#,
        ),
        (
            "{{cite book | last = Jager | first = Sheila Miyoshi |author-link= Sheila Miyoshi Jager | title = Brothers at War | url = https://example.com/book | year = 2013 | publisher = Profile Books | location = London}}",
            r#"<p><a href="https://en.wikipedia.org/wiki/Sheila_Miyoshi_Jager">Sheila Miyoshi Jager</a><span class="external-link">↗</span>. <em>Brothers at War</em>. London: Profile Books, 2013</p>"#,
        ),
        (
            "{{cite book | first1 = Ian | last1 = Castello-Cortes | first2 = Bruce | last2 = Cumings | title = Korea | edition = 2nd American | pages = 12–14}}",
            r#"<p>Ian Castello-Cortes and Bruce Cumings. <em>Korea</em>. 2nd American ed. p. 12–14</p>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "cite book template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("cite book"));
    }
}

#[test]
fn render_wikitext_formats_citation_templates() {
    let cases = [
        (
            "{{Citation | editor-first = Ian | editor-last = Castello-Cortes | title = World Reference Atlas | contribution = North Korea | edition = 2nd American | year = 1996 | publisher = Dorling Kindersley | location = New York | isbn = 978-0-7894-1085-6}}",
            r#"<p>Ian Castello-Cortes, ed. North Korea. <em>World Reference Atlas</em>. New York: Dorling Kindersley, 1996. 2nd American ed. ISBN 978-0-7894-1085-6</p>"#,
        ),
        (
            "{{Citation | last = Cumings | first = Bruce | title = Korea's Place in the Sun | publisher = Norton | year = 1997 | isbn = 978-0-393-31681-0 | url = https://archive.org/details/koreasplaceinsun00bruc }}",
            r#"<p>Bruce Cumings. <em>Korea's Place in the Sun</em>. Norton, 1997. ISBN 978-0-393-31681-0</p>"#,
        ),
        (
            "{{Citation | url = http://www.asianinfo.org/asianinfo/korea/history.htm | publisher = Asian Info | title = Korea | contribution = History | access-date = 11 July 2006}}",
            r#"<p>History. <em>Korea</em>. Asian Info</p>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "citation template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("Citation"));
    }
}

#[test]
fn render_wikitext_formats_cite_journal_templates() {
    let cases = [
        (
            "{{Cite journal |last=Kim |first=Chin W. |date=2000 |title=The Legacy of King Sejong the Great |url=https://www.ideals.illinois.edu/items/9673 |journal=Studies in the Linguistic Sciences |volume=30 |issue=1 |pages=3–12 |issn=0049-2388}}",
            r#"<p>Chin W. Kim. "The Legacy of King Sejong the Great". <em>Studies in the Linguistic Sciences</em>. 2000, vol. 30, no. 1, pp. 3–12. ISSN 0049-2388</p>"#,
        ),
        (
            "{{Cite journal |last1=Lee |first1=Sang-Hyun |last2=Baik |first2=Jong-Jin |date=1 March 2010 |title=Statistical and dynamical characteristics of the urban heat island intensity in Seoul |journal=Theoretical and Applied Climatology |volume=100 |issue=1–2 |pages=227–237 |doi=10.1007/s00704-009-0247-1}}",
            r#"<p>Sang-Hyun Lee and Jong-Jin Baik. "Statistical and dynamical characteristics of the urban heat island intensity in Seoul". <em>Theoretical and Applied Climatology</em>. 1 March 2010, vol. 100, no. 1–2, pp. 227–237. doi:10.1007/s00704-009-0247-1</p>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "cite journal template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("Cite journal"));
    }
}

#[test]
fn render_wikitext_formats_cite_report_templates() {
    let cases = [
        (
            "{{Cite report|last=Ledyard|first=Gari Keith|title=The Cultural Work of Sejong the Great|publication-date=November 2002|pages=7–18}} {{Open access}}",
            r#"<p>Gari Keith Ledyard. <em>The Cultural Work of Sejong the Great</em>. November 2002. p. 7–18 <span title="open access">&#128275;</span></p>"#,
        ),
        (
            "{{cite report|author=The Example Institute|title=Sample Report|year=1999|page=4}}",
            r#"<p>The Example Institute. <em>Sample Report</em>. 1999. p. 4</p>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "cite report template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("Cite report"));
    }
}

#[test]
fn render_wikitext_formats_cite_eccp_templates() {
    let rendered = render_wikitext(
        "Sample",
        "{{cite ECCP |last=Kennedy |first=George A. |title=Amin |pages=8–9 |date=1943}}",
        &InternalLinks::new(),
        "en",
    );

    assert!(
        rendered.contains(
            r#"<p>George A. Kennedy. "Amin". Eminent Chinese of the Ch'ing Period. 1943. pp. 8–9</p>"#
        ),
        "{rendered}"
    );
    assert!(!rendered.contains("{{"));
    assert!(!rendered.contains("cite ECCP"));
}

#[test]
fn render_wikitext_formats_harvc_templates() {
    let cases = [
        (
            "{{harvc|last=Peterson|first=Mark|year=1992|in=Kim-Renaud|c=The Sejong Sillok|author-link=Mark A. Peterson}}",
            r#"<p><a href="https://en.wikipedia.org/wiki/Mark_A._Peterson">Mark Peterson</a><span class="external-link">↗</span>. "The Sejong Sillok". In Kim-Renaud 1992</p>"#,
        ),
        (
            "{{Harvc|last=Yi|first=Tae-jin|year=1992|in=Kim-Renaud|c=The Arts under King Sejong|first2=Sang-Woon|last2=Jeon|first3=Don|last3=Baker|pp=45–67}}",
            r#"<p>Tae-jin Yi, Sang-Woon Jeon, and Don Baker. "The Arts under King Sejong". In Kim-Renaud 1992. pp. 45–67</p>"#,
        ),
        (
            "{{harvc|last=Benson|first=Ezra Taft|year=1957|chapter=Foreword|chapter-url=https://archive.org/example|in=Stefferud|p=vi|loc=§2}}",
            r#"<p>Ezra Taft Benson. "Foreword". In Stefferud 1957. p. vi. §2</p>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "harvc template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("harvc"));
    }
}

#[test]
fn render_wikitext_formats_as_of_templates() {
    let cases = [
        ("{{As of|2023}}", "<p>As of 2023</p>"),
        ("{{As of|2009|lc=y}}", "<p>as of 2009</p>"),
        ("{{As of|2024|5}}", "<p>As of May 2024</p>"),
        ("{{As of|2024|5|15}}", "<p>As of May 15, 2024</p>"),
        ("{{As of|2024|5|15|df=dmy}}", "<p>As of 15 May 2024</p>"),
        ("{{As of|2024|alt=currently}}", "<p>currently</p>"),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "as of template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("As of|"));
    }
}

#[test]
fn render_wikitext_formats_blockquote_templates() {
    let cases = [
        (
            "{{Blockquote|text=The sounds of our country's language are different from those of the [[Names of China|Middle Kingdom]].|source=''Hunminjeongeum''}}",
            r#"<blockquote>
    <p>The sounds of our country's language are different from those of the <a href="https://en.wikipedia.org/wiki/Names_of_China">Middle Kingdom</a><span class="external-link">↗</span>.</p>
    <p class="blockquote-source"><em>Hunminjeongeum</em></p>
    </blockquote>"#,
        ),
        (
            "{{blockquote|A short quoted passage.|Example source}}",
            r#"<blockquote>
    <p>A short quoted passage.</p>
    <p class="blockquote-source">Example source</p>
    </blockquote>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "blockquote template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("Blockquote|"));
    }
}

#[test]
fn render_wikitext_formats_percentage_templates() {
    let cases = [
        ("{{Percentage|1|4}}", "25%"),
        ("{{Percentage|1280000|26100000|1}}", "4.9%"),
        (
            "{{Percentage|7769000|{{UN_Population|Dem. People's Republic of Korea}}}}",
            "30%",
        ),
        (
            "{{Percentage|1280000|{{UN_Population|Dem. People's Republic of Korea}}|1}}",
            "4.9%",
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(&format!("<p>{expected}</p>")),
            "percentage template {template:?} rendered unexpectedly:\n{rendered}"
        );
    }
}

#[test]
fn render_wikitext_formats_un_population_templates() {
    let cases = [
        (
            "{{UN_Population|Dem. People's Republic of Korea}}",
            "<p>26,100,000</p>",
        ),
        ("{{UN_Population|ref}}", "<h1>Sample</h1>"),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "UN_Population template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("UN_Population|"));
    }
}

#[test]
fn render_wikitext_formats_convert_templates() {
    let cases = [
        ("{{convert|1100|km|abbr=on}}", "1,100 km (684 mi)"),
        ("{{cvt|314|km|0}}", "314 km (195 mi)"),
        ("{{Cvt|49.5|km}}", "49.5 km (30.8 mi)"),
        ("{{convert|30|°C|°F}}", "30 °C (86 °F)"),
        ("{{Convert|24|ug/m3||sp=us}}", "24 ug/m³"),
        ("{{convert|&minus;3|°C|1|disp=or}}", "−3 °C (26.6 °F)"),
        (
            "{{convert|10|to|47|km2|disp=or|abbr=on}}",
            "10 to 47 km² (3.86 to 18.1 mi²)",
        ),
        ("{{convert|15|km|0|abbr=on}}", "15 km (9 mi)"),
        (
            "{{convert|2.1|and|−5.5|C|F|1}}",
            "2.1 °C and −5.5 °C (35.8 °F and 22.1 °F)",
        ),
        ("{{convert|250|km|0|abbr=on}}", "250 km (155 mi)"),
        ("{{convert|268|km2|mi2|sp=us|abbr=on}}", "268 km² (103 mi²)"),
        (
            "{{convert|30.0|and|22.9|C|F|0}}",
            "30.0 °C and 22.9 °C (86 °F and 73 °F)",
        ),
        ("{{convert|300|km/h|0|abbr=on}}", "300 km/h (186 mph)"),
        ("{{convert|40|C|F|1}}", "40 °C (104.0 °F)"),
        ("{{convert|4|km|mile|sp=us|abbr=on}}", "4 km (2.49 mi)"),
        (
            "{{convert|605.25|km2|sqmi|abbr=unit}}",
            "605.25 km² (234 mi²)",
        ),
        ("{{convert|613|km2|mi2|sp=us|abbr=on}}", "613 km² (237 mi²)"),
        ("{{convert|940|km|abbr=on}}", "940 km (584 mi)"),
        ("{{convert|−10|C}}", "−10 °C (14 °F)"),
        ("{{convert|−15|C}}", "−15 °C (5 °F)"),
        ("{{convert|−20|C}}", "−20 °C (-4 °F)"),
        ("{{convert|42-56|km}}", "42-56 km (26.1-34.8 mi)"),
        ("{{convert|20-25|km|mi|abbr=on}}", "20-25 km (12.4-15.5 mi)"),
        ("{{convert|60|cm}}", "60 cm (23.6 in)"),
        ("{{cvt|45730|m2}}", "45,730 m² (492,000 ft²)"),
        (
            "{{convert|75|mm/year|in/year|abbr=on}}",
            "75 mm/year (2.95 in/year)",
        ),
        (
            "{{convert|360|GPa|e6psi|abbr=unit|lk=on}}",
            "360 GPa (52.2 million psi)",
        ),
        ("{{convert|384400|km}}", "384,400 km (239,000 mi)"),
        ("{{convert|737|K|C F|abbr=on}}", "737 K (464 °C, 867 °F)"),
        ("{{convert|20|C|K C F|0|order=out}}", "20 °C (293 K, 68 °F)"),
        (
            "{{convert|1|AU|e6km e6mi|lk=in|abbr=unit}}",
            "1 AU (150 million km, 93 million mi)",
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(&format!("<p>{expected}</p>")),
            "convert template {template:?} rendered unexpectedly:\n{rendered}"
        );
    }
}

#[test]
fn render_wikitext_shows_secondary_convert_values_for_supported_page_cases() {
    let template_pattern =
        Regex::new(r"\{\{(?:convert|cvt)[^{}]*\}\}").expect("valid convert regex");
    let mut failures = Vec::new();

    for entry in fs::read_dir("pages").expect("pages directory should exist") {
        let entry = entry.expect("page entry should be readable");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let source = fs::read_to_string(&path).expect("page fixture should be readable");
        let page: PageResponse = serde_json::from_str(&source).expect("page fixture should parse");

        for template_match in template_pattern.find_iter(&page.parse.wikitext.text) {
            let template = template_match.as_str();
            if !convert_template_should_show_secondary_value(template) {
                continue;
            }

            let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
            if !rendered.contains('(') || !rendered.contains(')') {
                failures.push(format!(
                    "{}: {} => {}",
                    path.display(),
                    template,
                    rendered.replace('\n', " ")
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "supported convert templates should render secondary values:\n{}",
        failures.join("\n")
    );
}

fn convert_template_should_show_secondary_value(template: &str) -> bool {
    let template = template
        .trim()
        .trim_start_matches("{{")
        .trim_end_matches("}}");
    let mut parts = template.split('|');
    let _name = parts.next();
    let positional = parts
        .map(str::trim)
        .filter(|part| !part.contains('='))
        .collect::<Vec<_>>();

    if positional.len() < 2 {
        return false;
    }

    let (source_unit, explicit_target) = if matches!(positional[1], "to" | "and" | "-" | "–" | "by")
    {
        (positional.get(3).copied(), positional.get(4).copied())
    } else {
        (positional.get(1).copied(), positional.get(2).copied())
    };

    if let Some(target) = explicit_target
        && convert_template_explicit_target(target)
    {
        return true;
    }

    source_unit.is_some_and(convert_template_has_default_target)
}

fn convert_template_explicit_target(target: &str) -> bool {
    let trimmed = target.trim();
    !trimmed.is_empty()
        && trimmed.parse::<f64>().is_err()
        && !trimmed.contains('<')
        && !trimmed.contains('(')
}

fn convert_template_has_default_target(unit: &str) -> bool {
    matches!(
        unit.trim(),
        "km" | "mi"
            | "m"
            | "meter"
            | "km2"
            | "km²"
            | "C"
            | "°C"
            | "C-change"
            | "cm"
            | "mm"
            | "km/h"
            | "km/s"
            | "m/s2"
            | "e6km"
            | "km3"
            | "m2"
            | "acres"
    )
}

#[test]
fn render_wikitext_formats_nbsp_templates() {
    let rendered = render_wikitext(
        "Sample",
        "Mt{{nbsp}}Hwangnyeong and 180&nbsp;km away.",
        &InternalLinks::new(),
        "en",
    );

    assert!(rendered.contains("<p>Mt Hwangnyeong and 180 km away.</p>"));
    assert!(!rendered.contains("{{"));
    assert!(!rendered.contains("nbsp"));
}

#[test]
fn render_wikitext_formats_simple_inline_templates() {
    let cases = [
        ("healthcare{{mdash}}such as", "<p>healthcare—such as</p>"),
        ("202 BC{{snd}}9 AD", "<p>202 BC – 9 AD</p>"),
        ("{{circa}} 10 million", "<p>c. 10 million</p>"),
        ("{{circa|1950}}", "<p>c. 1950</p>"),
        ("{{c.|115 BC}}", "<p>c. 115 BC</p>"),
        ("{{cx|150 AD}}", "<p>c. 150 AD</p>"),
        ("{{died-in|202 BC}}", "<p>d. 202 BC</p>"),
        ("Mawangdui tomb {{numero|3}}", "<p>Mawangdui tomb No. 3</p>"),
        (
            "{{anl|Battle of Jushi}}",
            r#"<p><a href="https://en.wikipedia.org/wiki/Battle_of_Jushi">Battle of Jushi</a><span class="external-link">↗</span></p>"#,
        ),
        (
            "{{legend|#EF767E|North Korean, Chinese, and Soviet forces}}",
            "<p>North Korean, Chinese, and Soviet forces</p>",
        ),
        ("{{sic|was}}", "<p>was [sic]</p>"),
        ("{{sic}}", "<p>[sic]</p>"),
        ("{{Nowrap|June 10}}", "<p>June 10</p>"),
        ("some{{!}}text", "<p>some|text</p>"),
        (
            "{{Nowrap|[[Seoul]] and [[Busan]]}}",
            r#"<p><a href="https://en.wikipedia.org/wiki/Seoul">Seoul</a><span class="external-link">↗</span> and <a href="https://en.wikipedia.org/wiki/Busan">Busan</a><span class="external-link">↗</span></p>"#,
        ),
        (
            "{{Smaller|<sup>a</sup> [[Revised Romanisation of Korean|Revised Romanisation]]}}",
            r#"<p><small>a <a href="https://en.wikipedia.org/wiki/Revised_Romanisation_of_Korean">Revised Romanisation</a><span class="external-link">↗</span></small></p>"#,
        ),
        (
            "{{ROKS|Sejong the Great||2}}",
            r#"<p><a href="https://en.wikipedia.org/wiki/ROKS_Sejong_the_Great">ROKS <em>Sejong the Great</em></a><span class="external-link">↗</span></p>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "inline template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
    }
}

#[test]
fn render_wikitext_formats_small_template() {
    let cases = [
        ("{{small|(2014)}}", "<p><small>(2014)</small></p>"),
        ("{{small|(specific)}}", "<p><small>(specific)</small></p>"),
        (
            "{{small|[[Seoul]]}}",
            r#"<p><small><a href="https://en.wikipedia.org/wiki/Seoul">Seoul</a><span class="external-link">↗</span></small></p>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "small template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
    }

    // Empty small template produces no output
    let empty_rendered = render_wikitext("Sample", "{{small|}}", &InternalLinks::new(), "en");
    assert!(
        !empty_rendered.contains("<small>"),
        "empty small template should produce no small tags, got:\n{empty_rendered}"
    );
}

#[test]
fn render_wikitext_formats_web_source_templates() {
    let cases = [
        (
            "{{cite web |last=Demick|first=Barbara|date=16 July 2010|title=North Korea's giant leap backwards|url=http://www.theguardian.com/world/2010/jul/17/north-korea-famine-fears|website=[[The Guardian]]}}",
            r#"<p>Barbara Demick. <a href="http://www.theguardian.com/world/2010/jul/17/north-korea-famine-fears">"North Korea's giant leap backwards"</a><span class="external-link">↗</span>. <em><a href="https://en.wikipedia.org/wiki/The_Guardian">The Guardian</a><span class="external-link">↗</span></em>. 16 July 2010</p>"#,
        ),
        (
            "{{cite web |script-title=zh:Korea原名Corea 美國改的名 |trans-title=Is Korea's original name Corea? |url=http://city.udn.com/54543/2933925 |website=[[United Daily News]]|date=5 July 2008|ref={{SfnRef|Country Profile|2007}}}}{{source-attribution}}",
            r#"<p><a href="http://city.udn.com/54543/2933925">"Is Korea's original name Corea?"</a><span class="external-link">↗</span>. <em><a href="https://en.wikipedia.org/wiki/United_Daily_News">United Daily News</a><span class="external-link">↗</span></em>. 5 July 2008</p>"#,
        ),
        (
            "{{Britannica|322222}}",
            r#"<p>Britannica: <a href="https://www.britannica.com/EBchecked/topic/322222">Encyclopaedia Britannica</a><span class="external-link">↗</span></p>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "web source template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
    }
}

#[test]
fn render_wikitext_formats_main_templates() {
    let mut internal_links = InternalLinks::new();
    internal_links.insert("namesofkorea".to_string(), "chapter-2.xhtml".to_string());

    let rendered = render_wikitext(
        "Sample",
        "{{Main|Names of Korea}}\n{{Main|Korean cuisine|Korean tea ceremony}}",
        &internal_links,
        "en",
    );

    assert!(
        rendered.contains(r#"Main article: <a href="chapter-2.xhtml">Names of Korea</a>"#),
        "{rendered}"
    );
    assert!(rendered.contains(
            r#"Main articles: <a href="https://en.wikipedia.org/wiki/Korean_cuisine">Korean cuisine</a><span class="external-link">↗</span> and <a href="https://en.wikipedia.org/wiki/Korean_tea_ceremony">Korean tea ceremony</a><span class="external-link">↗</span>"#
        ));
}

#[test]
fn render_wikitext_formats_see_also_templates() {
    let mut internal_links = InternalLinks::new();
    internal_links.insert("seoul".to_string(), "chapter-2.xhtml".to_string());

    let rendered = render_wikitext(
        "Sample",
        "{{See also|Seoul}}\n{{See also|Korean tea ceremony|Korean royal court cuisine}}",
        &internal_links,
        "en",
    );

    assert!(
        rendered.contains(r#"See also: <a href="chapter-2.xhtml">Seoul</a>"#),
        "{rendered}"
    );
    assert!(rendered.contains(
            r#"See also: <a href="https://en.wikipedia.org/wiki/Korean_tea_ceremony">Korean tea ceremony</a><span class="external-link">↗</span> and <a href="https://en.wikipedia.org/wiki/Korean_royal_court_cuisine">Korean royal court cuisine</a><span class="external-link">↗</span>"#
        ));
}

#[test]
fn render_wikitext_formats_further_templates() {
    let mut internal_links = InternalLinks::new();
    internal_links.insert("joseondynasty".to_string(), "chapter-3.xhtml".to_string());

    let rendered = render_wikitext(
        "Sample",
        "{{Further|Joseon dynasty|Downtown Seoul|Seongjeosimni}}\n{{Further|topic=the logistics and shipping company|Ilyang Logistics}}",
        &internal_links,
        "en",
    );

    assert!(
        rendered.contains(r#"Further information: <a href="chapter-3.xhtml">Joseon dynasty</a>, <a href="https://en.wikipedia.org/wiki/Downtown_Seoul">Downtown Seoul</a><span class="external-link">↗</span>, and <a href="https://en.wikipedia.org/wiki/Seongjeosimni">Seongjeosimni</a><span class="external-link">↗</span>"#),
        "{rendered}"
    );
    assert!(
        rendered.contains(r#"Further information about the logistics and shipping company: <a href="https://en.wikipedia.org/wiki/Ilyang_Logistics">Ilyang Logistics</a><span class="external-link">↗</span>"#),
        "{rendered}"
    );
    assert!(!rendered.contains("{{"));
    assert!(!rendered.contains("Further|"));
}

#[test]
fn render_wikitext_formats_wiktionary_templates() {
    let cases = [
        (
            "{{Wiktionary|Korea}}",
            r#"<p>Wiktionary: <a href="https://en.wiktionary.org/wiki/Korea">Korea</a><span class="external-link">↗</span></p>"#,
        ),
        (
            "{{wiktionary|Korean language|Korean}}",
            r#"<p>Wiktionary: <a href="https://en.wiktionary.org/wiki/Korean_language">Korean</a><span class="external-link">↗</span></p>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "wiktionary template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("Wiktionary|"));
    }
}

#[test]
fn render_wikitext_formats_wikivoyage_templates() {
    let cases = [
        (
            "{{Wikivoyage|Korea}}",
            r#"<p>Wikivoyage: <a href="https://en.wikivoyage.org/wiki/Korea">Korea</a><span class="external-link">↗</span></p>"#,
        ),
        (
            "{{wikivoyage|South Korea|travel guide}}",
            r#"<p>Wikivoyage: <a href="https://en.wikivoyage.org/wiki/South_Korea">travel guide</a><span class="external-link">↗</span></p>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "wikivoyage template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("Wikivoyage|"));
    }
}

#[test]
fn render_wikitext_formats_wikisource_templates() {
    let cases = [
        (
            "{{Wikisource|Littell's Living Age/Volume 129/Issue 1662/A Glimpse of the Korea|a description of a visit to Korea by a British ship in 1876}}",
            r#"<p>Wikisource: <a href="https://en.wikisource.org/wiki/Littell's_Living_Age/Volume_129/Issue_1662/A_Glimpse_of_the_Korea">a description of a visit to Korea by a British ship in 1876</a><span class="external-link">↗</span></p>"#,
        ),
        (
            "{{wikisource|Korea}}",
            r#"<p>Wikisource: <a href="https://en.wikisource.org/wiki/Korea">Korea</a><span class="external-link">↗</span></p>"#,
        ),
        (
            "{{Wikibooks|1=Saylor.org's Ancient Civilizations of the World|2=The Han dynasty and China's Classical Age|3=the Han Dynasty}}",
            r#"<p>Wikibooks: <a href="https://en.wikibooks.org/wiki/Saylor.org's_Ancient_Civilizations_of_the_World/The_Han_dynasty_and_China's_Classical_Age">the Han Dynasty</a><span class="external-link">↗</span></p>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "Wikisource template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("Wikisource|"));
    }
}

#[test]
fn render_wikitext_formats_official_website_templates() {
    let cases = [
        (
            "{{Official website|https://example.com}}",
            r#"<p><a href="https://example.com">Official website</a><span class="external-link">↗</span></p>"#,
        ),
        (
            "{{official website|http://www.korea.net/||name=The Republic of Korea}}",
            r#"<p><a href="http://www.korea.net/">The Republic of Korea</a><span class="external-link">↗</span></p>"#,
        ),
        (
            "{{Official website|url=example.org|title=''Example'' site}}",
            r#"<p><a href="https://example.org"><em>Example</em> site</a><span class="external-link">↗</span></p>"#,
        ),
        (
            "{{official|https://example.org|name=Official portal}}",
            r#"<p><a href="https://example.org">Official portal</a><span class="external-link">↗</span></p>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "Official website template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("Official website|"));
    }
}

#[test]
fn render_wikitext_formats_url_templates() {
    let cases = [
        (
            "{{URL|https://english.seoul.go.kr/|seoul.go.kr}}",
            r#"<p><a href="https://english.seoul.go.kr/">seoul.go.kr</a><span class="external-link">↗</span></p>"#,
        ),
        (
            "{{URL|1=https://english.seoul.go.kr/|2=Official website}}",
            r#"<p><a href="https://english.seoul.go.kr/">Official website</a><span class="external-link">↗</span></p>"#,
        ),
        (
            "{{url|example.org}}",
            r#"<p><a href="https://example.org">example.org</a><span class="external-link">↗</span></p>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "URL template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("URL|"));
    }
}

#[test]
fn render_wikitext_formats_openstreetmap_relation_templates() {
    let rendered = render_wikitext(
        "Sample",
        "*{{osmrelation-inline|2396450}}",
        &InternalLinks::new(),
        "en",
    );

    assert!(
        rendered.contains(
            r#"<li><a href="https://www.openstreetmap.org/relation/2396450">OpenStreetMap relation 2396450</a><span class="external-link">↗</span></li>"#
        ),
        "{rendered}"
    );
    assert!(!rendered.contains("{{"));
    assert!(!rendered.contains("osmrelation-inline"));
}

#[test]
fn render_wikitext_formats_webarchive_templates() {
    let cases = [
        (
            "{{Webarchive|url=https://web.archive.org/web/20140703095242/http://example.com/report.pdf|date=3 July 2014}}",
            r#"<p><a href="https://web.archive.org/web/20140703095242/http://example.com/report.pdf">Archived on 3 July 2014</a><span class="external-link">↗</span></p>"#,
        ),
        (
            "{{webarchive|url=https://web.archive.org/web/20170427003611/http://example.com/report.pdf}}",
            r#"<p><a href="https://web.archive.org/web/20170427003611/http://example.com/report.pdf">Archived copy</a><span class="external-link">↗</span></p>"#,
        ),
        (
            "{{webarchive|[http://example.com Korean Map]|http://example.com}}",
            r#"<p><a href="http://example.com">Archived copy</a><span class="external-link">↗</span></p>"#,
        ),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "Webarchive template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("Webarchive|"));
    }
}

#[test]
fn render_wikitext_formats_largest_cities_templates() {
    let rendered = render_wikitext(
        "Sample",
        "{{Largest cities|country=Korea|city_1=Seoul|div_1=Seoul|pop_1=9,904,312|city_2=[[Busan]]|div_2=Busan|pop_2=3,448,737}}",
        &InternalLinks::new(),
        "en",
    );

    assert!(
        rendered.contains("<p>Largest cities in Korea:</p>"),
        "{rendered}"
    );
    assert!(
        rendered.contains(r#"<li><a href="https://en.wikipedia.org/wiki/Seoul">Seoul</a><span class="external-link">↗</span> (Seoul, population 9,904,312)</li>"#),
        "{rendered}"
    );
    assert!(
        rendered.contains(r#"<li><a href="https://en.wikipedia.org/wiki/Busan">Busan</a><span class="external-link">↗</span> (Busan, population 3,448,737)</li>"#),
        "{rendered}"
    );
    assert!(!rendered.contains("{{"));
    assert!(!rendered.contains("Largest cities|"));
}

#[test]
fn render_wikitext_formats_interlanguage_link_templates() {
    let rendered = render_wikitext(
        "Sample",
        "Known as ''{{ill|Hyangyakchips\u{014f}ngbang|ko|향약집성방}}'' and {{ill|Seoul|ko|서울|lt=the capital}}.",
        &InternalLinks::new(),
        "en",
    );

    assert!(
            rendered.contains(
                r#"<p>Known as <em><a href="https://en.wikipedia.org/wiki/Hyangyakchipsŏngbang">Hyangyakchipsŏngbang</a><span class="external-link">↗</span> [ko]</em> and <a href="https://en.wikipedia.org/wiki/Seoul">the capital</a><span class="external-link">↗</span> [ko].</p>"#
            ),
            "{rendered}"
        );
}

#[test]
fn render_wikitext_formats_reign_templates() {
    let cases = [
        ("{{Reign}}", "r."),
        ("{{Reign|1207|1272}}", "r. 1207–1272"),
        (
            "{{Reign |1 October 1207 |1272}}",
            "r. 1 October 1207 – 1272",
        ),
        ("{{Reign|1207|present}}", "r. 1207–present"),
        ("{{Reign||940}}", "r. ?–940"),
        ("{{Reign|89|67|era=BCE}}", "r. 89–67 BCE"),
        ("{{Reign|single=1872}}", "r. 1872"),
        ("{{Reign|1962|present|show=word}}", "reigned 1962–present"),
        ("{{Reign|1962|present|show=colon}}", "reign: 1962–present"),
        ("{{Reign|1962|present|show=blank}}", "1962–present"),
        ("{{Reign|label=ruled|1967|1969}}", "ruled 1967–1969"),
        ("{{Reign|1267|1272|post-date=1275}}", "r. 1267–1272, 1275"),
    ];

    for (template, expected) in cases {
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(&format!("<p>{expected}</p>")),
            "reign template {template:?} rendered unexpectedly:\n{rendered}"
        );
    }
}

#[test]
fn render_wikitext_formats_open_access_templates() {
    let rendered = render_wikitext(
        "Sample",
        "A citation. {{Open access}}\nA second citation. {{open access}}\nA third citation. {{Free access}}",
        &InternalLinks::new(),
        "en",
    );

    assert!(rendered.contains(
        r#"<p>A citation. <span title="open access">&#128275;</span> A second citation. <span title="open access">&#128275;</span> A third citation. <span title="open access">&#128275;</span></p>"#
    ));
    assert!(!rendered.contains("{{"));
    assert!(!rendered.contains("Open access"));
    assert!(!rendered.contains("Free access"));
}

#[test]
fn strip_balanced_sections_removes_nested_templates() {
    let cleaned = strip_balanced_sections("before {{a {{nested}} value}} after", "{{", "}}");
    assert_eq!(cleaned, "before  after");
}

#[test]
fn strip_wikitext_tables_removes_table_sections() {
    // Simple table with class attribute is removed, surrounding text preserved.
    // The surrounding newlines remain after the table block is stripped (a blank
    // line is the natural result of removing multiple full lines).
    let cleaned = strip_wikitext_tables("before\n{| class=\"wikitable\"\n|-\n| cell\n|}\nafter");
    assert!(cleaned.contains("before"));
    assert!(cleaned.contains("after"));
    assert!(!cleaned.contains("{|"));

    // Multiple sequential tables are all removed
    let cleaned = strip_wikitext_tables(
        "intro\n{| class=\"a\"\n| row\n|}\nmiddle\n{| class=\"b\"\n| row\n|}\nend",
    );
    assert!(cleaned.contains("intro"));
    assert!(cleaned.contains("middle"));
    assert!(cleaned.contains("end"));
    assert!(!cleaned.contains("{|"));
    assert!(!cleaned.contains("|}"));

    // Nested tables are handled by depth tracking
    let cleaned = strip_wikitext_tables("start\n{| outer\n| {| inner\n|-\n|}\n|}\nfinish");
    assert!(cleaned.contains("start"));
    assert!(cleaned.contains("finish"));
    assert!(!cleaned.contains("{|"));

    // Text with no tables passes through unchanged
    let no_tables = "plain text without any tables";
    assert_eq!(strip_wikitext_tables(no_tables), no_tables);
}

#[test]
fn render_wikitable_preserves_various_classes_and_skips_unrecognized() {
    let internal_links = InternalLinks::new();

    // 1. A table with class="wikitable sortable" is rendered with its classes preserved
    let wikitext_sortable =
        "before\n{| class=\"wikitable sortable\"\n|-\n! Header\n|-\n| Cell\n|}\nafter";
    let rendered_sortable = render_wikitext("Sample", wikitext_sortable, &internal_links, "en");
    assert!(rendered_sortable.contains("<table class=\"wikitable sortable\">"));
    assert!(rendered_sortable.contains("<th>Header</th>"));
    assert!(rendered_sortable.contains("<td>Cell</td>"));

    // 2. A table with class="wikitable plainrowheaders" is rendered with its classes preserved
    let wikitext_plain =
        "before\n{| class=\"wikitable plainrowheaders\"\n|-\n! Header\n|-\n| Cell\n|}\nafter";
    let rendered_plain = render_wikitext("Sample", wikitext_plain, &internal_links, "en");
    assert!(rendered_plain.contains("<table class=\"wikitable plainrowheaders\">"));

    // 3. A table with an unrecognized class is skipped (stripped) entirely
    let wikitext_unrecognized = "before\n{| class=\"infobox\"\n|-\n| Cell\n|}\nafter";
    let rendered_unrecognized =
        render_wikitext("Sample", wikitext_unrecognized, &internal_links, "en");
    assert!(!rendered_unrecognized.contains("<table"));
    assert!(!rendered_unrecognized.contains("Cell"));
    assert!(rendered_unrecognized.contains("before"));
    assert!(rendered_unrecognized.contains("after"));

    // 4. A table with no class is rendered with the default "wikitable" class
    let wikitext_no_class = "before\n{|\n|-\n| Cell\n|}\nafter";
    let rendered_no_class = render_wikitext("Sample", wikitext_no_class, &internal_links, "en");
    assert!(rendered_no_class.contains("<table class=\"wikitable\">"));
    assert!(rendered_no_class.contains("<td>Cell</td>"));
}

#[test]
fn strip_file_links_removes_nested_caption_links() {
    let cleaned = strip_file_links(
        "before [[File:Hangul.svg|thumb|[[Hangul]], afterwards called [[Korean alphabet]]]] after",
    );

    assert_eq!(cleaned, "before  after");
}

#[test]
fn render_wikitext_omits_file_links_without_leaking_closing_markup() {
    let internal_links = InternalLinks::new();
    let rendered = render_wikitext(
        "Sample",
        "[[File:Gimjang.jpg|thumb|[[Gimjang]], the process for making [[kimchi]]]] Koreans traditionally believe in spices.",
        &internal_links,
        "en",
    );

    assert!(rendered.contains("<p>Koreans traditionally believe in spices.</p>"));
    assert!(!rendered.contains("[[File:"));
    assert!(!rendered.contains("]]"));
    assert!(!rendered.contains("Gimjang"));
}

#[test]
fn render_wikitext_embeds_resolved_file_links_when_images_are_enabled() {
    let internal_links = InternalLinks::new();
    let mut image_registry =
        ImageRegistry::new(Some(std::path::Path::new("pages"))).expect("image registry loads");
    let rendered = render_wikitext_with_template_counts(
        "Sample",
        "[[File:Ships in Busan.jpg|thumb|alt=Shipyard view|[[HJ Shipbuilding & Construction|Hanjin Heavy Industries]] shipyard]] Text.",
        &internal_links,
        "en",
        Some(&mut image_registry),
    )
    .0;

    assert!(
        rendered.contains(
            r#"<div class="image"><img src="images/image-1.svg" alt="Shipyard view" /><p class="caption"><a href="https://en.wikipedia.org/wiki/HJ_Shipbuilding_&amp;_Construction">Hanjin Heavy Industries</a><span class="external-link">↗</span> shipyard</p></div>"#
        ),
        "{rendered}"
    );
    assert!(rendered.contains("<p>Text.</p>"));
    assert_eq!(image_registry.images.len(), 1);
    assert_eq!(image_registry.occurrences.len(), 1);
}

#[test]
fn render_wikitext_embeds_japan_file_link() {
    let internal_links = InternalLinks::new();
    let mut image_registry =
        ImageRegistry::new(Some(std::path::Path::new("pages"))).expect("image registry loads");
    let (rendered, _) = render_wikitext_with_template_counts(
        "Sample",
        "[[File:Regions and Prefectures of Japan 2.svg|thumb|center|upright=1.3|{{Shy|Pre|fec|tures}} of Japan with colored regions]]",
        &internal_links,
        "en",
        Some(&mut image_registry),
    );

    println!("RENDERED:\n{}", rendered);
    assert_eq!(image_registry.images.len(), 1);
    assert_eq!(image_registry.occurrences.len(), 1);
}

#[test]
fn test_render_table_with_image() {
    let internal_links = InternalLinks::new();
    let mut image_registry =
        ImageRegistry::new(Some(std::path::Path::new("pages"))).expect("image registry loads");
    let wikitext = r#"
{|
|rowspan="2"|[[File:Regions and Prefectures of Japan 2.svg|thumb|center|upright=1.3|{{Shy|Pre|fec|tures}} of Japan with colored regions]]
|}
"#;
    let mut tables = Vec::new();
    let text = crate::render_wikitext_tables_with_excluded_links(
        wikitext,
        &mut tables,
        &internal_links,
        "en",
        LinksToExcludedPages::Emphasize,
        Some(&mut image_registry),
        "Japan",
    );
    println!("TEXT: {}", text);
    println!("TABLES: {:?}", tables);
    println!("REGISTRY: {:?}", image_registry);
    assert_eq!(image_registry.images.len(), 1);
}

#[test]
fn book_config_defaults_images_to_false() {
    let config = serde_yaml::from_str::<BookConfig>(
        r#"chapters: title
metadata:
  title: Sample
  author: Wikipedia contributors
  language: en
  edition: First edition
output-file: sample.epub
cover: "None"
links_to_pages: false
links_to_excluded_pages: emphasize
caching: none
depth: 0
articles:
  - Sample
"#,
    )
    .expect("config parses");

    assert!(!config.images);
}

#[test]
fn book_config_accepts_images_true() {
    let config = serde_yaml::from_str::<BookConfig>(
        r#"chapters: title
metadata:
  title: Sample
  author: Wikipedia contributors
  language: en
  edition: First edition
output-file: sample.epub
images: true
cover: "None"
links_to_pages: false
links_to_excluded_pages: emphasize
caching: none
depth: 0
articles:
  - Sample
"#,
    )
    .expect("config parses");

    assert!(config.images);
}

#[test]
fn book_config_parses_links_to_excluded_pages() {
    let config = serde_yaml::from_str::<BookConfig>(
        r#"chapters: title
metadata:
  title: Sample
  author: Wikipedia contributors
  language: en
  edition: First edition
output-file: sample.epub
images: true
cover: "None"
links_to_pages: false
links_to_excluded_pages: display
caching: none
depth: 0
articles:
  - Sample
"#,
    )
    .expect("config parses");

    assert_eq!(
        config.links_to_excluded_pages,
        LinksToExcludedPages::Display
    );
}

#[test]
fn book_config_requires_links_to_excluded_pages() {
    let error = serde_yaml::from_str::<BookConfig>(
        r#"chapters: title
metadata:
  title: Sample
  author: Wikipedia contributors
  language: en
  edition: First edition
output-file: sample.epub
cover: "None"
links_to_pages: false
caching: none
depth: 0
articles:
  - Sample
"#,
    )
    .expect_err("config should reject missing links_to_excluded_pages");

    assert!(
        error.to_string().contains("links_to_excluded_pages"),
        "{error}"
    );
}

#[test]
fn read_config_rejects_unknown_fields_with_clear_error() {
    let error = parse_config_str(
        Path::new("sample.yaml"),
        r#"chapters: title
metadata:
  title: Sample
  author: Wikipedia contributors
  language: en
  edition: First edition
output-file: sample.epub
cover: "None"
links_to_pages: false
links_to_excluded_pages: emphasize
caching: none
depth: 0
unexpected: true
articles:
  - Sample
"#,
    )
    .expect_err("config should reject unknown fields");

    let message = error.to_string();
    assert!(
        message.contains("invalid configuration in sample.yaml"),
        "{message}"
    );
    assert!(message.contains("unknown field `unexpected`"), "{message}");
}

#[test]
fn read_config_rejects_invalid_values_with_clear_error() {
    let error = parse_config_str(
        Path::new("sample.yaml"),
        r#"chapters: title
metadata:
  title: Sample
  author: Wikipedia contributors
  language: en
  edition: First edition
output-file: sample.epub
cover: "None"
links_to_pages: false
links_to_excluded_pages: emphasize
caching: remote
depth: 0
articles:
  - Sample
"#,
    )
    .expect_err("config should reject invalid values");

    let message = error.to_string();
    assert!(
        message.contains("invalid configuration in sample.yaml"),
        "{message}"
    );
    assert!(message.contains("caching"), "{message}");
    assert!(message.contains("unknown variant `remote`"), "{message}");
}

#[test]
fn read_config_rejects_missing_fields_with_clear_error() {
    let error = parse_config_str(
        Path::new("sample.yaml"),
        r#"chapters: title
metadata:
  title: Sample
  author: Wikipedia contributors
  language: en
  edition: First edition
cover: "None"
links_to_pages: false
links_to_excluded_pages: emphasize
caching: none
depth: 0
articles:
  - Sample
"#,
    )
    .expect_err("config should reject missing required fields");

    let message = error.to_string();
    assert!(
        message.contains("invalid configuration in sample.yaml"),
        "{message}"
    );
    assert!(message.contains("missing field `output-file`"), "{message}");
}

#[test]
fn read_config_rejects_duplicate_pages_with_clear_error() {
    let error = parse_config_str(
        Path::new("sample.yaml"),
        r#"chapters: title
metadata:
  title: Sample
  author: Wikipedia contributors
  language: en
  edition: First edition
output-file: sample.epub
cover: "None"
links_to_pages: false
links_to_excluded_pages: emphasize
caching: none
depth: 0
articles:
  - Korea
  - title: History
    type: section
    articles:
      - Korea
"#,
    )
    .expect_err("config should reject duplicate pages");

    let message = error.to_string();
    assert!(
        message.contains("invalid configuration in sample.yaml"),
        "{message}"
    );
    assert!(message.contains("duplicate page `Korea`"), "{message}");
}

#[test]
fn read_config_rejects_duplicate_section_title_and_article() {
    let error = parse_config_str(
        Path::new("sample.yaml"),
        r#"chapters: title
metadata:
  title: Sample
  author: Wikipedia contributors
  language: en
  edition: First edition
output-file: sample.epub
cover: "None"
links_to_pages: false
links_to_excluded_pages: emphasize
caching: none
depth: 0
articles:
  - title: Jōmon period
    type: section
    articles:
      - Jōmon period
"#,
    )
    .expect_err("config should reject duplicate page when section title and article are the same");

    let message = error.to_string();
    assert!(
        message.contains("invalid configuration in sample.yaml"),
        "{message}"
    );
    assert!(
        message.contains("duplicate page `Jōmon period`"),
        "{message}"
    );
}

#[test]
fn read_config_rejects_duplicate_section_titles() {
    let error = parse_config_str(
        Path::new("sample.yaml"),
        r#"chapters: title
metadata:
  title: Sample
  author: Wikipedia contributors
  language: en
  edition: First edition
output-file: sample.epub
cover: "None"
links_to_pages: false
links_to_excluded_pages: emphasize
caching: none
depth: 0
articles:
  - title: History
    type: section
    articles:
      - Korea
  - title: History
    type: section
    articles:
      - Japan
"#,
    )
    .expect_err("config should reject duplicate page when section titles are duplicate");

    let message = error.to_string();
    assert!(
        message.contains("invalid configuration in sample.yaml"),
        "{message}"
    );
    assert!(message.contains("duplicate page `History`"), "{message}");
}

#[test]
fn fixture_page_source_uses_local_page_dumps() {
    let source = FixturePageSource::new("pages");
    let page = source.load_page("Korea").expect("fixture page should load");

    assert_eq!(page.parse.title, "Korea");
    assert!(page.parse.wikitext.text.contains("East Asia"));
}

#[test]
fn wikipedia_urls_use_configured_language() {
    assert_eq!(
        wikipedia_parse_api_url("es")
            .expect("Spanish API URL should build")
            .as_str(),
        "https://es.wikipedia.org/w/api.php"
    );
    assert_eq!(
        wikipedia_article_url("Corea del Sur", "es"),
        "https://es.wikipedia.org/wiki/Corea_del_Sur"
    );
}

#[test]
fn wikipedia_language_rejects_invalid_hostname_labels() {
    let err =
        normalized_wikipedia_language("en.example.com").expect_err("invalid language should fail");

    assert!(err.to_string().contains("invalid Wikipedia language code"));
}

#[test]
fn hebrew_html_uses_right_to_left_direction() {
    assert_eq!(html_language_attributes("he"), r#"xml:lang="he" dir="rtl""#);
    assert_eq!(html_language_attributes("en"), r#"xml:lang="en""#);
}

#[test]
fn parse_args_accepts_local_pages_dir() {
    let args = parse_args_from(["wikipedia-to-epub", "books/korea.yaml", "--local", "pages"])
        .expect("args should parse");

    assert_eq!(args.config_path, PathBuf::from("books/korea.yaml"));
    assert_eq!(args.local_pages_dir, Some(PathBuf::from("pages")));
    assert!(!args.refresh_cache);
    assert_eq!(args.log_level, Level::INFO);
}

#[test]
fn parse_args_accepts_refresh_cache() {
    let args = parse_args_from(["wikipedia-to-epub", "books/korea.yaml", "--refresh-cache"])
        .expect("args should parse");

    assert!(args.refresh_cache);
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
fn parse_args_accepts_images() {
    let args = parse_args_from(["wikipedia-to-epub", "books/korea.yaml", "--images"])
        .expect("args should parse");

    assert!(args.images);
    assert!(!args.no_images);
}

#[test]
fn parse_args_accepts_no_images() {
    let args = parse_args_from(["wikipedia-to-epub", "books/korea.yaml", "--no-images"])
        .expect("args should parse");

    assert!(!args.images);
    assert!(args.no_images);
}

#[test]
fn parse_args_accepts_logfile() {
    let args = parse_args_from([
        "wikipedia-to-epub",
        "books/korea.yaml",
        "--logfile",
        "custom.log",
    ])
    .expect("args should parse");

    assert_eq!(args.logfile, Some(PathBuf::from("custom.log")));
}

#[test]
fn parse_args_accepts_caching() {
    let args = parse_args_from([
        "wikipedia-to-epub",
        "books/korea.yaml",
        "--caching",
        "local",
    ])
    .expect("args should parse");
    assert_eq!(args.caching, Some(CachingMode::Local));

    let args = parse_args_from(["wikipedia-to-epub", "books/korea.yaml", "--caching", "none"])
        .expect("args should parse");
    assert_eq!(args.caching, Some(CachingMode::None));

    let args = parse_args_from([
        "wikipedia-to-epub",
        "books/korea.yaml",
        "--caching",
        "central",
    ])
    .expect("args should parse");
    assert_eq!(args.caching, Some(CachingMode::Central));
}

#[test]
fn parse_args_rejects_both_images_and_no_images() {
    let err = parse_args_from([
        "wikipedia-to-epub",
        "books/korea.yaml",
        "--images",
        "--no-images",
    ])
    .expect_err("mutually exclusive flags should fail");

    let err_message = err.to_string();
    assert!(err_message.contains("cannot be used with"));
}

#[test]
fn read_or_fetch_text_writes_cache_on_miss() {
    let cache_path = test_cache_path("text-miss").join("value.txt");
    let calls = std::cell::Cell::new(0);

    let (content, source) = read_or_fetch_text(&cache_path, false, || {
        calls.set(calls.get() + 1);
        Ok("fresh text".to_string())
    })
    .expect("cache miss fetches");

    assert_eq!(content, "fresh text");
    assert_eq!(source, CacheSource::Refreshed);
    assert_eq!(calls.get(), 1);
    assert_eq!(fs::read_to_string(cache_path).unwrap(), "fresh text");
}

#[test]
fn read_or_fetch_text_uses_cache_hit_without_fetching() {
    let cache_path = test_cache_path("text-hit").join("value.txt");
    write_cache_text(&cache_path, "cached text").expect("cache writes");
    let calls = std::cell::Cell::new(0);

    let (content, source) = read_or_fetch_text(&cache_path, false, || {
        calls.set(calls.get() + 1);
        Ok("fresh text".to_string())
    })
    .expect("cache hit reads");

    assert_eq!(content, "cached text");
    assert_eq!(source, CacheSource::Hit);
    assert_eq!(calls.get(), 0);
}

#[test]
fn read_or_fetch_text_refreshes_existing_cache() {
    let cache_path = test_cache_path("text-refresh").join("value.txt");
    write_cache_text(&cache_path, "cached text").expect("cache writes");
    let calls = std::cell::Cell::new(0);

    let (content, source) = read_or_fetch_text(&cache_path, true, || {
        calls.set(calls.get() + 1);
        Ok("fresh text".to_string())
    })
    .expect("cache refresh fetches");

    assert_eq!(content, "fresh text");
    assert_eq!(source, CacheSource::Refreshed);
    assert_eq!(calls.get(), 1);
    assert_eq!(fs::read_to_string(cache_path).unwrap(), "fresh text");
}

#[test]
fn read_or_fetch_bytes_uses_cache_and_refreshes() {
    let cache_path = test_cache_path("bytes").join("value.bin");
    let miss_calls = std::cell::Cell::new(0);
    let (bytes, source) = read_or_fetch_bytes(&cache_path, false, || {
        miss_calls.set(miss_calls.get() + 1);
        Ok(vec![1, 2, 3])
    })
    .expect("cache miss fetches bytes");
    assert_eq!(bytes, vec![1, 2, 3]);
    assert_eq!(source, CacheSource::Refreshed);
    assert_eq!(miss_calls.get(), 1);

    let hit_calls = std::cell::Cell::new(0);
    let (bytes, source) = read_or_fetch_bytes(&cache_path, false, || {
        hit_calls.set(hit_calls.get() + 1);
        Ok(vec![4, 5, 6])
    })
    .expect("cache hit reads bytes");
    assert_eq!(bytes, vec![1, 2, 3]);
    assert_eq!(source, CacheSource::Hit);
    assert_eq!(hit_calls.get(), 0);

    let refresh_calls = std::cell::Cell::new(0);
    let (bytes, source) = read_or_fetch_bytes(&cache_path, true, || {
        refresh_calls.set(refresh_calls.get() + 1);
        Ok(vec![7, 8, 9])
    })
    .expect("cache refresh fetches bytes");
    assert_eq!(bytes, vec![7, 8, 9]);
    assert_eq!(source, CacheSource::Refreshed);
    assert_eq!(refresh_calls.get(), 1);
    assert_eq!(fs::read(cache_path).unwrap(), vec![7, 8, 9]);
}

#[test]
fn read_or_fetch_helpers_update_download_stats() {
    let text_stats = FileDownloadStats::default();
    let text_path = test_cache_path("text-stats").join("value.txt");
    let (_content, source) =
        read_or_fetch_text_with_stats(&text_path, false, Some(&text_stats), true, || {
            Ok("fresh text".to_string())
        })
        .expect("text cache miss fetches");
    assert_eq!(source, CacheSource::Refreshed);
    assert_eq!(
        text_stats.snapshot(),
        FileDownloadSnapshot {
            needed: 1,
            from_cache: 0,
            downloaded: 1,
            failed: 0,
        }
    );

    let (_content, source) =
        read_or_fetch_text_with_stats(&text_path, false, Some(&text_stats), true, || {
            Ok("unused".to_string())
        })
        .expect("text cache hit reads");
    assert_eq!(source, CacheSource::Hit);
    assert_eq!(
        text_stats.snapshot(),
        FileDownloadSnapshot {
            needed: 2,
            from_cache: 1,
            downloaded: 1,
            failed: 0,
        }
    );

    let bytes_stats = FileDownloadStats::default();
    let bytes_path = test_cache_path("bytes-stats").join("value.bin");
    let err = read_or_fetch_bytes_with_stats(&bytes_path, false, Some(&bytes_stats), true, || {
        Err(AppError::Message("download failed".to_string()))
    })
    .expect_err("byte download failure is returned");
    assert!(err.to_string().contains("download failed"));
    assert_eq!(
        bytes_stats.snapshot(),
        FileDownloadSnapshot {
            needed: 1,
            from_cache: 0,
            downloaded: 0,
            failed: 1,
        }
    );
}

#[test]
fn download_cache_paths_are_safe_for_non_ascii_titles() {
    let cache = DownloadCache::new(
        PathBuf::from("/tmp/cache-root"),
        false,
        DownloadStats::default(),
        true,
    );
    let long_url = format!(
        "https://upload.wikimedia.org/wikipedia/commons/thumb/{}",
        "very-long-path-segment/".repeat(40)
    );

    assert_eq!(
        cache.page_json_path("ko", "서울").file_name().unwrap(),
        "2761d049a3924bf7.json"
    );
    assert_eq!(
        cache
            .image_metadata_path("en", "Busan Port (1).jpg")
            .file_name()
            .unwrap(),
        "1ebe8870dbb0cdee.json"
    );
    assert_eq!(
        cache
            .image_file_path("https://upload.wikimedia.org/example image.jpg", "jpg")
            .file_name()
            .unwrap(),
        "77b8aaaf434fea44.jpg"
    );
    assert_eq!(
        cache
            .image_file_path(&long_url, "jpg")
            .file_name()
            .unwrap()
            .len(),
        "77b8aaaf434fea44.jpg".len()
    );
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

#[test]
fn render_wikitext_formats_formatnum_template() {
    assert_eq!(render_templates("{{formatnum:5324}}"), "5,324");
    assert_eq!(render_templates("{{formatnum:20413}}"), "20,413");
    assert_eq!(render_templates("{{formatnum|9523}}"), "9,523");
    assert_eq!(
        render_templates("{{formatnum:-1234567.89}}"),
        "-1,234,567.89"
    );
    assert_eq!(
        render_templates("{{formatnum:+987654.321}}"),
        "+987,654.321"
    );
    assert_eq!(render_templates("{{formatnum:abc}}"), "abc");
}

#[test]
fn render_wikitext_formats_stn_template() {
    assert_eq!(render_templates("{{STN|Ginza}}"), "[[Ginza Station|Ginza]]");
    assert_eq!(
        render_templates("{{STN|Hamaōtsu|x}}"),
        "[[Hamaōtsu Station|Hamaōtsu]]"
    );
    assert_eq!(
        render_templates("{{STN|Jiyūgaoka|Tokyo}}"),
        "[[Jiyūgaoka Station (Tokyo)|Jiyūgaoka]]"
    );
    assert_eq!(
        render_templates("{{STN|Tokyo||Tōkyō}}"),
        "[[Tokyo Station|Tōkyō]]"
    );
}

#[test]
fn render_wikitext_silently_skips_kyoto_metadata_templates() {
    let (rendered, counts) = render_wikitext_with_template_counts(
        "Sample",
        "{{Expand section|date=July 2012}}\n{{Unreferencedsect|date=July 2012}}\n{{Clear left}}\n{{Kyoto}}\n{{Kyoto Prefecture}}",
        &InternalLinks::new(),
        "en",
        None,
    );
    assert!(!rendered.contains("Expand section"), "{rendered}");
    assert!(!rendered.contains("Unreferencedsect"), "{rendered}");
    assert!(!rendered.contains("Clear left"), "{rendered}");
    assert!(!rendered.contains("Kyoto"), "{rendered}");
    assert_eq!(
        counts,
        TemplateSkipCounts {
            recognized: 5,
            unknown: 0
        }
    );
}

#[test]
fn render_wikitext_formats_nihongo3_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{nihongo3|shrine temple|神宮寺|[[jingū-ji]]}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains(r#"<span><em><a href="https://en.wikipedia.org/wiki/jingū-ji">jingū-ji</a><span class="external-link">↗</span></em> (<span title="Japanese-language text"><span lang="ja">神宮寺</span></span>, "shrine temple")</span>"#),
        "{rendered}"
    );

    let rendered = render_wikitext(
        "Sample",
        "{{nihongo3|mountain name|山号|sangō|extra text}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains(r#"<span><em>sangō</em> (<span title="Japanese-language text"><span lang="ja">山号</span></span>, "mountain name", extra text)</span>"#),
        "{rendered}"
    );

    let rendered = render_wikitext(
        "Sample",
        "{{nihongo3||宮寺|miya-ji}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains(r#"<span><em>miya-ji</em> (<span title="Japanese-language text"><span lang="ja">宮寺</span></span>)</span>"#),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_gburl_template() {
    assert_eq!(
        render_templates("{{GBurl|id=VXj_AQAAQBAJ}}"),
        "https://books.google.com/books?id=VXj_AQAAQBAJ"
    );
    assert_eq!(
        render_templates("{{GBurl|id=zfMYBQAAQBAJ|p=22}}"),
        "https://books.google.com/books?id=zfMYBQAAQBAJ&pg=PA22"
    );
    assert_eq!(
        render_templates("{{GBurl|id=zfMYBQAAQBAJ|pg=RA1-PA243}}"),
        "https://books.google.com/books?id=zfMYBQAAQBAJ&pg=RA1-PA243"
    );
    assert_eq!(
        render_templates("{{GBurl|id=zfMYBQAAQBAJ|q=koguryo powerful em}}"),
        "https://books.google.com/books?id=zfMYBQAAQBAJ&q=koguryo+powerful+em"
    );
}

#[test]
fn render_wikitext_formats_google_books_template() {
    assert_eq!(
        render_templates("{{Google books|0syC6L77dpAC|page=|keywords=|text=|plainurl=yes}}"),
        "https://books.google.com/books?id=0syC6L77dpAC"
    );

    let rendered = render_wikitext(
        "Sample",
        "{{Google books|_mh4Qv4lAkQC|''The Koreas,'' p. 57-58.|page=57}}",
        &InternalLinks::new(),
        "en",
    );

    assert!(
        rendered.contains(
            r#"<p><a href="https://books.google.com/books?id=_mh4Qv4lAkQC&amp;pg=PA57"><em>The Koreas,</em> p. 57-58.</a><span class="external-link">↗</span></p>"#
        ),
        "{rendered}"
    );
    assert!(!rendered.contains("{{"), "{rendered}");
}

#[test]
fn render_wikitext_formats_cite_thesis_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{cite thesis |last=Byington |first=Mark |title=A History |publisher=Harvard University |year=2003}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("Mark Byington. <em>A History</em>. Harvard University, 2003"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_usurped_template() {
    assert_eq!(
        render_templates("{{usurped|1=[https://web.archive.org/web/20050403...]}}"),
        "[https://web.archive.org/web/20050403...]"
    );
}

#[test]
fn render_wikitext_formats_awol_template() {
    assert_eq!(
        render_templates("{{AWOL}}"),
        "&nbsp;([[Absent without leave|__WIKIPEDIA_TO_EPUB_ABBR_START__Desertion__WIKIPEDIA_TO_EPUB_ABBR_VALUE__AWOL__WIKIPEDIA_TO_EPUB_ABBR_END__]])"
    );
}

#[test]
fn render_wikitext_formats_assassinated_template() {
    assert_eq!(
        render_templates("{{Assassinated}}"),
        "&nbsp;[[Assassination|'''X''']]"
    );
    assert_eq!(
        render_templates("{{Assassinated|bold=no}}"),
        "&nbsp;[[Assassination|X]]"
    );
    assert_eq!(
        render_templates("{{Assassinated|alt=yes}}"),
        "&nbsp;[[Assassination|(Assassinated)]]"
    );
    assert_eq!(
        render_templates("{{Assassinated|Some custom link|bold=no}}"),
        "&nbsp;[[Some custom link|X]]"
    );
}

#[test]
fn render_wikitext_formats_died_of_wounds_template() {
    assert_eq!(
        render_templates("{{Died of wounds}}"),
        "&nbsp;([[Killed in action|__WIKIPEDIA_TO_EPUB_ABBR_START__Died of wounds__WIKIPEDIA_TO_EPUB_ABBR_VALUE__DOW__WIKIPEDIA_TO_EPUB_ABBR_END__]])"
    );
}

#[test]
fn render_wikitext_formats_dow_template() {
    assert_eq!(
        render_templates("{{DOW}}"),
        "&nbsp;([[Killed in action|__WIKIPEDIA_TO_EPUB_ABBR_START__Died of wounds__WIKIPEDIA_TO_EPUB_ABBR_VALUE__DOW__WIKIPEDIA_TO_EPUB_ABBR_END__]])"
    );
}

#[test]
fn render_wikitext_formats_executed_template() {
    assert_eq!(
        render_templates("{{Executed}}"),
        "&nbsp;[[File:Skull and Crossbones.svg|14px|Executed|link=Capital punishment]]"
    );
    assert_eq!(
        render_templates("{{Executed|Some link}}"),
        "&nbsp;[[File:Skull and Crossbones.svg|14px|Executed|link=Some link]]"
    );
}

#[test]
fn render_wikitext_formats_kia_template() {
    assert_eq!(
        render_templates("{{KIA}}"),
        "&nbsp;[[Killed in action|'''†''']]"
    );
    assert_eq!(
        render_templates("{{KIA|bold=no}}"),
        "&nbsp;[[Killed in action|†]]"
    );
    assert_eq!(
        render_templates("{{KIA|alt=yes}}"),
        "&nbsp;[[Killed in action|(KIA)]]"
    );
    assert_eq!(
        render_templates("{{KIA|Some custom link|alt=yes}}"),
        "&nbsp;[[Some custom link|(KIA)]]"
    );
}

#[test]
fn render_wikitext_formats_kia2_template() {
    assert_eq!(
        render_templates("{{KIA2}}"),
        "&nbsp;[[Killed in action|(KIA)]]"
    );
}

#[test]
fn render_wikitext_formats_mia_template() {
    assert_eq!(
        render_templates("{{MIA}}"),
        "&nbsp;([[Missing in action|__WIKIPEDIA_TO_EPUB_ABBR_START__Missing in action__WIKIPEDIA_TO_EPUB_ABBR_VALUE__MIA__WIKIPEDIA_TO_EPUB_ABBR_END__]])"
    );
}

#[test]
fn render_wikitext_formats_natural_causes_template() {
    assert_eq!(
        render_templates("{{Natural Causes}}"),
        "&nbsp;[[Manner of death#Natural causes of death|__WIKIPEDIA_TO_EPUB_ABBR_START__Natural causes__WIKIPEDIA_TO_EPUB_ABBR_VALUE__'''#'''__WIKIPEDIA_TO_EPUB_ABBR_END__]]"
    );
    assert_eq!(
        render_templates("{{Natural Causes|bold=no}}"),
        "&nbsp;[[Manner of death#Natural causes of death|__WIKIPEDIA_TO_EPUB_ABBR_START__Natural causes__WIKIPEDIA_TO_EPUB_ABBR_VALUE__#__WIKIPEDIA_TO_EPUB_ABBR_END__]]"
    );
    assert_eq!(
        render_templates("{{Natural Causes|alt=yes}}"),
        "&nbsp;[[Manner of death#Natural causes of death|__WIKIPEDIA_TO_EPUB_ABBR_START__Natural causes__WIKIPEDIA_TO_EPUB_ABBR_VALUE__(Natural causes)__WIKIPEDIA_TO_EPUB_ABBR_END__]]"
    );
    assert_eq!(
        render_templates("{{Natural Causes|Some link|alt=yes}}"),
        "&nbsp;[[Some link|__WIKIPEDIA_TO_EPUB_ABBR_START__Natural causes__WIKIPEDIA_TO_EPUB_ABBR_VALUE__(Natural causes)__WIKIPEDIA_TO_EPUB_ABBR_END__]]"
    );
}

#[test]
fn render_wikitext_formats_pkia_template() {
    assert_eq!(
        render_templates("{{PKIA}}"),
        "&nbsp;([[Killed in action|__WIKIPEDIA_TO_EPUB_ABBR_START__Presumed killed in action__WIKIPEDIA_TO_EPUB_ABBR_VALUE__PKIA__WIKIPEDIA_TO_EPUB_ABBR_END__]])"
    );
}

#[test]
fn render_wikitext_formats_pow_template() {
    assert_eq!(
        render_templates("{{POW}}"),
        "&#x20;<span style=\"white-space:nowrap\">([[Prisoner of war|__WIKIPEDIA_TO_EPUB_ABBR_START__Prisoner of war__WIKIPEDIA_TO_EPUB_ABBR_VALUE__POW__WIKIPEDIA_TO_EPUB_ABBR_END__]])</span>"
    );
}

#[test]
fn render_wikitext_formats_suicide_template() {
    assert_eq!(
        render_templates("{{Suicide}}"),
        "&nbsp;[[Suicide|'''‡‡''']]"
    );
    assert_eq!(
        render_templates("{{Suicide|bold=no}}"),
        "&nbsp;[[Suicide|‡‡]]"
    );
    assert_eq!(
        render_templates("{{Suicide|alt=yes}}"),
        "&nbsp;[[Suicide|(__WIKIPEDIA_TO_EPUB_ABBR_START__[[Suicide]]__WIKIPEDIA_TO_EPUB_ABBR_VALUE__Suicide__WIKIPEDIA_TO_EPUB_ABBR_END__)]]"
    );
    assert_eq!(
        render_templates("{{Suicide|Some custom link|bold=no}}"),
        "&nbsp;[[Some custom link|‡‡]]"
    );
}

#[test]
fn render_wikitext_formats_surrendered_template() {
    assert_eq!(
        render_templates("{{Surrendered}}"),
        "&nbsp;[[File:White flag icon.svg|14px|Surrendered|link=Surrender (military)]]"
    );
    assert_eq!(
        render_templates("{{Surrendered|Some link}}"),
        "&nbsp;[[File:White flag icon.svg|14px|Surrendered|link=Some link]]"
    );
}

#[test]
fn render_wikitext_formats_turncoat_template() {
    assert_eq!(
        render_templates("{{Turncoat}}"),
        "&nbsp;[[File:Black flag icon.svg|14px|Turncoat|link=Turncoat]]"
    );
    assert_eq!(
        render_templates("{{Turncoat|Some link}}"),
        "&nbsp;[[File:Black flag icon.svg|14px|Turncoat|link=Some link]]"
    );
}

#[test]
fn render_wikitext_formats_wia_template() {
    assert_eq!(
        render_templates("{{WIA}}"),
        "&nbsp;([[Wounded in action|__WIKIPEDIA_TO_EPUB_ABBR_START__Wounded in action__WIKIPEDIA_TO_EPUB_ABBR_VALUE__WIA__WIKIPEDIA_TO_EPUB_ABBR_END__]])"
    );
}

#[test]
fn render_wikitext_formats_translation_template() {
    assert_eq!(
        render_templates("{{Translation|word}}"),
        "__WIKIPEDIA_TO_EPUB_ABBR_START__translation__WIKIPEDIA_TO_EPUB_ABBR_VALUE__transl.__WIKIPEDIA_TO_EPUB_ABBR_END__\u{2009}word"
    );
    assert_eq!(
        render_templates("{{Translation|word|literal=yes}}"),
        "__WIKIPEDIA_TO_EPUB_ABBR_START__literal translation__WIKIPEDIA_TO_EPUB_ABBR_VALUE__lit. transl.__WIKIPEDIA_TO_EPUB_ABBR_END__\u{2009}word"
    );
    assert_eq!(
        render_templates("{{Translation|word|literal=no|i=yes}}"),
        "''transl.''\u{2009}word"
    );
    assert_eq!(
        render_templates("{{Translation|word1|word2}}"),
        "__WIKIPEDIA_TO_EPUB_ABBR_START__translation__WIKIPEDIA_TO_EPUB_ABBR_VALUE__transl.__WIKIPEDIA_TO_EPUB_ABBR_END__\u{2009}word1 – transl.\u{2009}word2"
    );
}

#[test]
fn render_wikitext_formats_station_template() {
    assert_eq!(
        render_templates("{{Station|Shibuya}}"),
        "[[Shibuya station|Shibuya]]"
    );
    assert_eq!(
        render_templates("{{Station|Shibuya|1}}"),
        "[[Shibuya Station|Shibuya]]"
    );
    assert_eq!(
        render_templates("{{Station|Shibuya|1|Tokyo}}"),
        "[[Shibuya Station (Tokyo)|Shibuya]]"
    );
    assert_eq!(
        render_templates("{{Station|Shibuya|1|Tokyo|Shibuya Stn}}"),
        "[[Shibuya Station (Tokyo)|Shibuya Stn]]"
    );
}

#[test]
fn render_wikitext_formats_ja_rail_linem_template() {
    assert_eq!(
        render_templates("{{ja-rail-linem|linename=Yamanote Line}}"),
        "|-\n| <span style=\"color:white\">■</span>&nbsp;[[Yamanote Line]]\n| \n"
    );
    assert_eq!(
        render_templates("{{ja-rail-linem|linename=Yamanote Line|lineindex=JY}}"),
        "|-\n| <span style=\"color:white\">■</span>&nbsp;[[Yamanote Line|JY]]\n| \n"
    );
    assert_eq!(
        render_templates(
            "{{ja-rail-linem|linename=Yamanote Line|span=2|pfn=Platform 1|dir=For Tokyo}}"
        ),
        "|-\n| rowspan=2 | '''Platform 1'''\n| <span style=\"color:white\">■</span>&nbsp;[[Yamanote Line]]\n| For Tokyo\n"
    );
    assert_eq!(
        render_templates(
            "{{ja-rail-linem|linename=Yamanote Line|linecol=green|dir=For Tokyo|next=Shinagawa}}"
        ),
        "|-\n| <span style=\"color:green\">■</span>&nbsp;[[Yamanote Line]]\n| For Tokyo <small>(Shinagawa)</small>\n"
    );
    assert_eq!(
        render_templates("{{ja-rail-linem|m|linename=Tokyo Metro|linecol=blue}}"),
        "|-\n| <span style=\"color:blue\">'''○'''</span>&nbsp;[[Tokyo Metro]]\n| \n"
    );
}

#[test]
fn render_wikitext_formats_jpn_template() {
    assert_eq!(render_templates("{{JPN}}"), "🇯🇵 [[Japan]]");
    assert_eq!(
        render_templates("{{JPN|name=Japan Team}}"),
        "🇯🇵 [[Japan|Japan Team]]"
    );
}

#[test]
fn render_wikitext_formats_langnf_template() {
    assert_eq!(
        render_templates("{{Language with name/for|es|Casa|house}}"),
        "__WIKIPEDIA_TO_EPUB_LANG_START__es__WIKIPEDIA_TO_EPUB_LANG_VALUE__''Casa''__WIKIPEDIA_TO_EPUB_LANG_END__ ([[Spanish language|Spanish]] for 'house')"
    );
    assert_eq!(
        render_templates("{{langnf|es|Casa|house|break=yes}}"),
        "__WIKIPEDIA_TO_EPUB_LANG_START__es__WIKIPEDIA_TO_EPUB_LANG_VALUE__''Casa''__WIKIPEDIA_TO_EPUB_LANG_END__<br />([[Spanish language|Spanish]] for 'house')"
    );
    assert_eq!(
        render_templates("{{langnf|es|Casa|house|paren=none}}"),
        "__WIKIPEDIA_TO_EPUB_LANG_START__es__WIKIPEDIA_TO_EPUB_LANG_VALUE__''Casa''__WIKIPEDIA_TO_EPUB_LANG_END__ [[Spanish language|Spanish]] for 'house'"
    );
    assert_eq!(
        render_templates("{{langnf|es|Casa|term1=house|term2=hut|term3=mansion|italic-term=yes}}"),
        "__WIKIPEDIA_TO_EPUB_LANG_START__es__WIKIPEDIA_TO_EPUB_LANG_VALUE__''Casa''__WIKIPEDIA_TO_EPUB_LANG_END__ ([[Spanish language|Spanish]] for '<em>house</em>' / '<em>hut</em>' / '<em>mansion</em>')"
    );
    assert_eq!(
        render_templates("{{langnf||kuncannowet|breast|lang-name=Massachusett}}"),
        "__WIKIPEDIA_TO_EPUB_LANG_START__mis__WIKIPEDIA_TO_EPUB_LANG_VALUE__''kuncannowet''__WIKIPEDIA_TO_EPUB_LANG_END__ ([[Massachusett language|Massachusett]] for 'breast')"
    );
}

#[test]
fn render_wikitext_formats_track_gauge_template() {
    assert_eq!(
        render_templates("{{Track gauge|1435 mm}}"),
        "1,435 mm (4 ft 8+1\u{2044}2 in)"
    );
    assert_eq!(
        render_templates("{{RailGauge|Cape gauge|al=on}}"),
        "1,067 mm (3 ft 6 in) Cape gauge"
    );
    assert_eq!(
        render_templates("{{Track gauge|1520 mm|allk=on}}"),
        "1,520 mm (4 ft 11+27\u{2044}32 in) [[5 ft and 1520 mm track gauge|Russian gauge]]"
    );
    assert_eq!(render_templates("{{Track gauge|1234 mm}}"), "1234 mm");
}

#[test]
fn render_wikitext_silently_skips_goguryeo_metadata_templates() {
    let (rendered, counts) = render_wikitext_with_template_counts(
        "Sample",
        "{{Cleanup|reason=idiosyncratic}}\n{{tone|section|date=October 2014}}",
        &InternalLinks::new(),
        "en",
        None,
    );
    assert!(!rendered.contains("Cleanup"), "{rendered}");
    assert!(!rendered.contains("tone"), "{rendered}");
    assert_eq!(
        counts,
        TemplateSkipCounts {
            recognized: 2,
            unknown: 0
        }
    );
}

#[test]
fn render_wikitext_silently_skips_hungary_metadata_templates() {
    let (rendered, counts) = render_wikitext_with_template_counts(
        "Sample",
        "{{Wikiatlas|Hungary}}\n{{Hungary articles}}",
        &InternalLinks::new(),
        "en",
        None,
    );
    assert!(!rendered.contains("Wikiatlas"), "{rendered}");
    assert!(!rendered.contains("Hungary articles"), "{rendered}");
    assert_eq!(
        counts,
        TemplateSkipCounts {
            recognized: 2,
            unknown: 0
        }
    );
}

#[test]
fn render_wikitext_formats_citation_needed_span_template() {
    // Positional parameter
    let rendered = render_wikitext(
        "Sample",
        "{{citation needed span|unoccupied western part}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("unoccupied western part"), "{rendered}");

    // Named parameter
    let rendered = render_wikitext(
        "Sample",
        "{{citation needed span|1=which ended in 1991|date=March 2026}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("which ended in 1991"), "{rendered}");
}

#[test]
fn render_wikitext_formats_ndash_template() {
    let rendered = render_wikitext("Sample", "Buda{{ndash}}Pest", &InternalLinks::new(), "en");
    assert!(rendered.contains("Buda–Pest"), "{rendered}");
}

#[test]
fn render_wikitext_formats_endash_template() {
    let rendered = render_wikitext("Sample", "Buda{{endash}}Pest", &InternalLinks::new(), "en");
    assert!(rendered.contains("Buda–Pest"), "{rendered}");
}

#[test]
fn render_wikitext_formats_jpy_template() {
    let rendered1 = render_wikitext("Sample", "Cost is {{JPY}}.", &InternalLinks::new(), "en");
    assert!(rendered1.contains("Cost is ¥."), "{rendered1}");

    let rendered2 = render_wikitext(
        "Sample",
        "Cost is {{JPY|1234.56}}.",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered2.contains("Cost is ¥1,234.56."), "{rendered2}");

    let rendered3 = render_wikitext(
        "Sample",
        "Cost is {{JPY|amount=8.5}}.",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered3.contains("Cost is ¥8.5."), "{rendered3}");

    let rendered4 = render_wikitext(
        "Sample",
        "Cost is {{JPY|1=9876}}.",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered4.contains("Cost is ¥9,876."), "{rendered4}");
}

#[test]
fn render_wikitext_formats_quote_box_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Quote box|width=30%|quote=It is outstanding|source=UNESCO}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("It is outstanding"), "{rendered}");
    assert!(rendered.contains("UNESCO"), "{rendered}");
}

#[test]
fn render_wikitext_formats_center_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{center|'''City of Budapest'''}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("<strong>City of Budapest</strong>"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_singular_template() {
    let rendered = render_wikitext("Sample", "{{singular}}", &InternalLinks::new(), "en");
    assert!(
        rendered.contains("<abbr title=\"singular form\">sg.</abbr>"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_silently_skips_budapest_metadata_templates() {
    let (rendered, counts) = render_wikitext_with_template_counts(
        "Sample",
        "{{update section|date=2018}}\n{{party color|Respect}}\n{{category see also|Parks}}\n{{clarify|reason=explain}}\n{{colbegin}}\n{{colend}}\n{{Geographic location|Centre=Budapest}}\n{{Budapest}}",
        &InternalLinks::new(),
        "en",
        None,
    );
    assert!(!rendered.contains("update section"), "{rendered}");
    assert!(!rendered.contains("party color"), "{rendered}");
    assert!(!rendered.contains("category see also"), "{rendered}");
    assert!(!rendered.contains("clarify"), "{rendered}");
    assert!(!rendered.contains("colbegin"), "{rendered}");
    assert!(!rendered.contains("colend"), "{rendered}");
    assert!(!rendered.contains("Geographic location"), "{rendered}");
    assert!(!rendered.contains("Budapest"), "{rendered}");
    assert_eq!(
        counts,
        TemplateSkipCounts {
            recognized: 8,
            unknown: 0
        }
    );
}

#[test]
fn render_wikitext_silently_skips_old_choson_metadata_templates() {
    let (rendered, counts) = render_wikitext_with_template_counts(
        "Sample",
        "{{POV|date=January 2023}}\n{{dubious|reason=discuss}}",
        &InternalLinks::new(),
        "en",
        None,
    );
    assert!(!rendered.contains("POV"), "{rendered}");
    assert!(!rendered.contains("dubious"), "{rendered}");
    assert_eq!(
        counts,
        TemplateSkipCounts {
            recognized: 2,
            unknown: 0
        }
    );
}

#[test]
fn test_template_name_is_in_csv_disregards_comments_after_comma() {
    let mock_csv = "Template1,this is a comment\nTemplate2\nTemplate3, another comment with spaces\n\"Template, with comma\",comment\n\"Another, comma template\"";
    assert!(template_name_is_in_csv("Template1", mock_csv));
    assert!(template_name_is_in_csv("template2", mock_csv));
    assert!(template_name_is_in_csv("Template3", mock_csv));
    assert!(template_name_is_in_csv("Template, with comma", mock_csv));
    assert!(template_name_is_in_csv("Another, comma template", mock_csv));
    assert!(!template_name_is_in_csv("this is a comment", mock_csv));
    assert!(!template_name_is_in_csv(
        "another comment with spaces",
        mock_csv
    ));
}

#[test]
fn render_wikitext_formats_main_article_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Main article|Miura Gorō}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains(
            "Main article: <a href=\"https://en.wikipedia.org/wiki/Miura_Gorō\">Miura Gorō</a><span class=\"external-link\">↗</span>"
        ),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_quote_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Quote|text=This was a matter I decided|author=Miura}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("This was a matter I decided"),
        "{rendered}"
    );
    assert!(rendered.contains("Miura"), "{rendered}");
}

#[test]
fn render_wikitext_formats_break_template() {
    let rendered = render_wikitext(
        "Sample",
        "Line 1{{Break}}Line 2{{Break|2}}Line 3",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("Line 1<br />Line 2<br /><br />Line 3"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_fx_convert_template() {
    let rendered = render_wikitext(
        "Sample",
        "Cost: {{FXConvert|KOR|293.823|b|cursign=[[₩]]|year=2020|showdate=no}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("Cost: ₩293.82 billion (US$248.95 million)"),
        "{rendered}"
    );

    let rendered_simple = render_wikitext(
        "Sample",
        "Cost: {{FXConvert|EUR|100}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered_simple.contains("Cost: €100"), "{rendered_simple}");
}

#[test]
fn render_wikitext_formats_osm_way_template() {
    let rendered = render_wikitext(
        "Sample",
        "Way: {{Osmway|131922091}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("Way: <a href=\"https://www.openstreetmap.org/way/131922091\">OpenStreetMap way 131922091</a><span class=\"external-link\">↗</span>"),
        "{rendered}"
    );
}

fn test_cache_path(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock is after Unix epoch")
        .as_nanos();
    env::temp_dir().join(format!(
        "wikipedia-to-epub-cache-test-{name}-{}-{nanos}",
        std::process::id()
    ))
}

#[test]
fn caching_mode_none_bypasses_cache_writes() {
    let cache_path = test_cache_path("none-bypass").join("value.txt");
    let calls = std::cell::Cell::new(0);

    let (content, source) = read_or_fetch_text_with_stats(&cache_path, false, None, false, || {
        calls.set(calls.get() + 1);
        Ok("fresh text".to_string())
    })
    .expect("fetch succeeds");

    assert_eq!(content, "fresh text");
    assert_eq!(source, CacheSource::Refreshed);
    assert_eq!(calls.get(), 1);
    assert!(
        !cache_path.exists(),
        "Cache file should not be written when caching is none!"
    );
}

#[test]
fn caching_mode_local_resolves_path() {
    let config_none = serde_yaml::from_str::<BookConfig>(
        r#"chapters: title
metadata:
  title: Sample
  author: Wikipedia contributors
  language: en
  edition: First edition
output-file: sample.epub
cover: "None"
links_to_pages: false
links_to_excluded_pages: emphasize
caching: none
depth: 0
articles:
  - Sample
"#,
    )
    .expect("config parses");
    assert_eq!(config_none.caching, CachingMode::None);

    let config_local = serde_yaml::from_str::<BookConfig>(
        r#"chapters: title
metadata:
  title: Sample
  author: Wikipedia contributors
  language: en
  edition: First edition
output-file: sample.epub
cover: "None"
links_to_pages: false
links_to_excluded_pages: emphasize
caching: local
depth: 0
articles:
  - Sample
"#,
    )
    .expect("config parses");
    assert_eq!(config_local.caching, CachingMode::Local);

    let config_central = serde_yaml::from_str::<BookConfig>(
        r#"chapters: title
metadata:
  title: Sample
  author: Wikipedia contributors
  language: en
  edition: First edition
output-file: sample.epub
cover: "None"
links_to_pages: false
links_to_excluded_pages: emphasize
caching: central
depth: 0
articles:
  - Sample
"#,
    )
    .expect("config parses");
    assert_eq!(config_central.caching, CachingMode::Central);
}

#[test]
fn render_wikitext_skips_silent_templates_with_underscores() {
    let (_rendered, counts) = render_wikitext_with_template_counts(
        "Sample",
        "{{History_of_Korea}}",
        &InternalLinks::new(),
        "en",
        None,
    );
    assert_eq!(
        counts,
        TemplateSkipCounts {
            recognized: 1,
            unknown: 0
        }
    );
}

#[test]
fn render_wikitext_formats_ko_alias_template() {
    let rendered = render_wikitext(
        "Sample",
        "It is called {{ko|hangul=십제|hanja=十濟|lit=Ten Vassals}}.",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains(
            r#"<p>It is called <span title="Korean-language text">Korean: <span lang="ko-Hang">십제</span> / Hanja: <span lang="ko-Hani">十濟</span></span>.</p>"#
        ),
        "rendered output was: {rendered}"
    );
}

#[test]
fn render_wikitext_formats_jct_template() {
    let rendered = render_wikitext(
        "Sample",
        "Take {{jct|country=JPN|Route|41}} to Takayama.",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains(
            r#"<p>Take <a href="https://en.wikipedia.org/wiki/Japan_National_Route_41">National Route 41</a><span class="external-link">↗</span> to Takayama.</p>"#
        ),
        "rendered output was: {rendered}"
    );
}

#[test]
fn render_wikitext_formats_cite_magazine_template() {
    let rendered = render_wikitext(
        "Sample",
        "See {{cite magazine|title=The Japan Alps|magazine=National Geographic|date=1910}}.",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains(r#"<p>See "The Japan Alps". <em>National Geographic</em>. 1910.</p>"#),
        "rendered output was: {rendered}"
    );
}

#[test]
fn render_wikitext_formats_cite_news_template() {
    let rendered = render_wikitext(
        "Sample",
        "See {{cite news|title=Alpine Explorer|newspaper=The Times|date=1920}}.",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains(r#"<p>See "Alpine Explorer". <em>The Times</em>. 1920.</p>"#),
        "rendered output was: {rendered}"
    );
}

#[test]
fn render_wikitext_silently_skips_mount_ena_metadata_templates() {
    let (rendered, counts) = render_wikitext_with_template_counts(
        "Sample",
        "See {{commonscat|position=left}}.",
        &InternalLinks::new(),
        "en",
        None,
    );
    assert!(!rendered.contains("commonscat"), "{rendered}");
    assert_eq!(
        counts,
        TemplateSkipCounts {
            recognized: 1,
            unknown: 0
        }
    );
}

#[test]
fn render_wikitext_formats_legend0_template() {
    let rendered = render_wikitext(
        "Sample",
        "See {{legend0|#EAB|City}}.",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("<p>See City.</p>"),
        "rendered output was: {rendered}"
    );
}

#[test]
fn render_wikitext_formats_oclc_template() {
    let rendered = render_wikitext(
        "Sample",
        "See {{oclc|58053128}}.",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("<p>See OCLC 58053128.</p>"),
        "rendered output was: {rendered}"
    );
}

#[test]
fn render_wikitext_formats_asin_template() {
    let rendered = render_wikitext("Sample", "{{ASIN|B00086U61Y}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("ASIN B00086U61Y"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{ASIN|B00086U61Y|title=Item's Title}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("ASIN B00086U61Y, <em>Item's Title</em>"),
        "{rendered}"
    );

    let rendered = render_wikitext(
        "Sample",
        "{{ASIN|B00086U61Y|date=2000-12-24}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("ASIN B00086U61Y (2000-12-24)"),
        "{rendered}"
    );

    let rendered = render_wikitext(
        "Sample",
        "{{ASIN|B00086U61Y|title=Item's Title|date=2000-12-24}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("ASIN B00086U61Y, <em>Item's Title</em> (2000-12-24)"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_script_template() {
    let rendered = render_wikitext("Sample", "{{Script|Hani|神}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("神"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{Script|Hani|神道}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("神道"), "{rendered}");
}

#[test]
fn render_wikitext_silently_skips_tree_list_templates() {
    let wikitext = r#"
{{Tree list}}
* First level
** Second level
*** {{Tree list/final branch}} Final branch text
{{Tree list/end}}
"#;
    let rendered = render_wikitext("Sample", wikitext, &InternalLinks::new(), "en");
    assert!(!rendered.contains("Tree list"), "{rendered}");
    assert!(rendered.contains("First level"), "{rendered}");
    assert!(rendered.contains("Second level"), "{rendered}");
    assert!(rendered.contains("Final branch text"), "{rendered}");
}

#[test]
fn render_wikitext_formats_dash_template() {
    let rendered = render_wikitext("Sample", "202 BC{{dash}}9 AD", &InternalLinks::new(), "en");
    assert!(rendered.contains("202 BC – 9 AD"), "{rendered}");
}

#[test]
fn render_wikitext_formats_snds_template() {
    let rendered = render_wikitext("Sample", "202 BC{{snds}}9 AD", &InternalLinks::new(), "en");
    assert!(rendered.contains("202 BC – 9 AD"), "{rendered}");
}

#[test]
fn render_wikitext_formats_birth_date_and_age_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Birth date and age|1931|3|7|df=yes}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("7 March 1931 (age"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{birth date and age|1931|3|7}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("March 7, 1931 (age"), "{rendered}");
}

#[test]
fn render_wikitext_formats_unbulleted_list_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{unbulleted list|Prince Nobuhiko Higashikuni|Princess Fumiko Higashikuni}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("<li>Prince Nobuhiko Higashikuni</li>"),
        "{rendered}"
    );
    assert!(
        rendered.contains("<li>Princess Fumiko Higashikuni</li>"),
        "{rendered}"
    );

    let rendered = render_wikitext(
        "Sample",
        "{{ubli|Prince Nobuhiko Higashikuni|Princess Fumiko Higashikuni}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("<li>Prince Nobuhiko Higashikuni</li>"),
        "{rendered}"
    );
    assert!(
        rendered.contains("<li>Princess Fumiko Higashikuni</li>"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_ublist_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{ublist|Prince Nobuhiko Higashikuni|Princess Fumiko Higashikuni}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("<li>Prince Nobuhiko Higashikuni</li>"),
        "{rendered}"
    );
    assert!(
        rendered.contains("<li>Princess Fumiko Higashikuni</li>"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_silently_skips_end_plainlist_template() {
    assert_eq!(render_templates("{{end plainlist}}"), "");
}

#[test]
fn render_wikitext_formats_multiref_template() {
    assert_eq!(render_templates("{{multiref|Ref 1|Ref 2}}"), "Ref 1; Ref 2");
    assert_eq!(
        render_templates("{{multiref|1=Ref 1|2=Ref 2|group=n}}"),
        "Ref 1; Ref 2"
    );
    assert_eq!(
        render_templates("{{multiref| {{est.|1990}} | {{est.|2000}} }}"),
        "__WIKIPEDIA_TO_EPUB_ABBR_START__estimate__WIKIPEDIA_TO_EPUB_ABBR_VALUE__est.__WIKIPEDIA_TO_EPUB_ABBR_END__ 1990; __WIKIPEDIA_TO_EPUB_ABBR_START__estimate__WIKIPEDIA_TO_EPUB_ABBR_VALUE__est.__WIKIPEDIA_TO_EPUB_ABBR_END__ 2000"
    );
}

#[test]
fn render_wikitext_formats_hosking_jfood_template() {
    assert_eq!(
        render_templates("{{hosking-jfood|page=123}}"),
        "Hosking, Richard (1996). ''A Dictionary of Japanese Food: Ingredients & Culture''. Tuttle Publishing. p. 123. ISBN 978-0-8048-2042-4"
    );
}

#[test]
fn render_wikitext_formats_parabr_template() {
    assert_eq!(
        render_templates("{{parabr}}"),
        "__WIKIPEDIA_TO_EPUB_BR____WIKIPEDIA_TO_EPUB_BR__"
    );
}

#[test]
fn render_wikitext_formats_multiref2_template() {
    assert_eq!(
        render_templates("{{Multiref2|Ref A|Ref B}}"),
        "Ref A; Ref B"
    );
}

#[test]
fn render_wikitext_formats_age_in_years_months_weeks_days_template() {
    assert_eq!(
        render_templates("{{Age in years, months, weeks and days|2020|1|15|2021|3|20}}"),
        "1 year, 2 months and 5 days"
    );
}

#[test]
fn render_wikitext_formats_est_template() {
    assert_eq!(
        render_templates("{{est.|1990}}"),
        "__WIKIPEDIA_TO_EPUB_ABBR_START__estimate__WIKIPEDIA_TO_EPUB_ABBR_VALUE__est.__WIKIPEDIA_TO_EPUB_ABBR_END__ 1990"
    );
}

#[test]
fn render_wikitext_formats_e28_template() {
    assert_eq!(
        render_templates("{{e28|kor|Korean}}"),
        "Eberhard, David M.; Simons, Gary F.; Fennig, Charles D., eds. (2025). \"[[official-url:https://www.ethnologue.com/language/kor|Korean]]\". ''Ethnologue: Languages of the World'' (28th ed.). Dallas, Texas: SIL International"
    );
}

#[test]
fn render_wikitext_formats_britannica_url_template() {
    assert_eq!(
        render_templates(
            "{{Britannica URL|url=https://www.britannica.com/topic/test|title=Test Topic|author=Author Name}}"
        ),
        "\"[[official-url:https://www.britannica.com/topic/test|Test Topic]]\" by Author Name at ''Encyclopædia Britannica''"
    );
}

#[test]
fn render_wikitext_silently_skips_surname_template() {
    assert_eq!(render_templates("{{Surname}}"), "");
}

#[test]
fn render_wikitext_formats_citation_attribution_template() {
    assert_eq!(
        render_templates("{{citation-attribution|text from public domain}}"),
        "One or more of the preceding sentences incorporates text from a work now in the public domain: text from public domain"
    );
}

#[test]
fn render_wikitext_formats_ordered_list_template() {
    assert_eq!(
        render_templates("{{olist|Item A|Item B}}"),
        "\n# Item A\n# Item B"
    );
    assert_eq!(
        render_templates("{{ordered list|Item A|Item B}}"),
        "\n# Item A\n# Item B"
    );
}

#[test]
fn render_wikitext_formats_webtrans_template() {
    assert_eq!(
        render_templates("{{webtrans|http://example.com/test.pdf|Merger proposal|ja}}"),
        "[[official-url:http://example.com/test.pdf|Merger proposal]] (in Japanese)"
    );
    assert_eq!(
        render_templates("{{webtrans|url=http://example.com|title=Example|lang=de}}"),
        "[[official-url:http://example.com|Example]] (in German)"
    );
}

#[test]
fn render_wikitext_formats_osm_template() {
    assert_eq!(
        render_templates("{{OSM|n|7530096619|Glen Blair}}"),
        "[[official-url:https://www.openstreetmap.org/node/7530096619|7530096619 Glen Blair on OpenStreetMap]]"
    );
    assert_eq!(
        render_templates("{{OSM|w|10273762|Sherwood Rd.}}"),
        "[[osmway:10273762|10273762 Sherwood Rd. on OpenStreetMap]]"
    );
    assert_eq!(
        render_templates("{{OSM|r|9942914}}"),
        "[[osmrelation:9942914|9942914 on OpenStreetMap]]"
    );
    assert_eq!(
        render_templates("{{OSM|relation=9942914}}"),
        "[[osmrelation:9942914|9942914]]"
    );
}

#[test]
fn render_wikitext_formats_wiktionary_inline_template() {
    assert_eq!(
        render_templates("{{Wiktionary-inline|word}}"),
        "The dictionary definition of [[wikt:word|word]] at Wiktionary"
    );
    assert_eq!(
        render_templates("{{wti|word|Word|extratext=sense #2}}"),
        "The dictionary definition of [[wikt:word|Word]] at Wiktionary, sense #2"
    );
}

#[test]
fn render_wikitext_formats_cite_opentopomap_template() {
    assert_eq!(
        render_templates(
            "{{cite opentopomap|name=Mount Everest|lat=27.988056|long=86.925278|access-date=2020-06-08}}"
        ),
        "\"[[official-url:https://opentopomap.org/#marker=14/27.988056/86.925278|Topographic map of Mount Everest]]\". ''opentopomap.org''. Retrieved 2020-06-08"
    );
}

#[test]
fn render_wikitext_silently_skips_engvarb_template() {
    assert_eq!(render_templates("{{EngvarB}}"), "");
}

#[test]
fn render_wikitext_formats_colorbull_template() {
    assert_eq!(
        render_templates("{{colorbull|red|circle|Mount Everest}}"),
        "[[Mount Everest|__WIKIPEDIA_TO_EPUB_COLOR_START__red__WIKIPEDIA_TO_EPUB_COLOR_MID__○__WIKIPEDIA_TO_EPUB_COLOR_END__]]"
    );
    assert_eq!(
        render_templates("{{colorbull|blue}}"),
        "__WIKIPEDIA_TO_EPUB_COLOR_START__blue__WIKIPEDIA_TO_EPUB_COLOR_MID__■__WIKIPEDIA_TO_EPUB_COLOR_END__"
    );
}

#[test]
fn render_wikitext_silently_skips_how_to_template() {
    assert_eq!(render_templates("{{how-to}}"), "");
}

#[test]
fn render_wikitext_silently_skips_stub_templates() {
    assert_eq!(render_templates("{{Busan-geo-stub}}"), "");
    assert_eq!(render_templates("{{some-other-stub}}"), "");
    assert_eq!(render_templates("{{STUB}}"), "");
}

#[test]
fn render_wikitext_formats_airport_codes_template() {
    assert_eq!(
        render_templates("{{airport codes|MMJ|RJAF}}"),
        "(IATA: MMJ, ICAO: RJAF)"
    );
    assert_eq!(
        render_templates("{{airport codes|MMJ|RJAF|p=n}}"),
        "IATA: MMJ, ICAO: RJAF"
    );
    assert_eq!(render_templates("{{airport codes|||1G4}}"), "(FAA: 1G4)");
}

#[test]
fn render_wikitext_formats_airport_dest_list_template() {
    assert_eq!(
        render_templates(
            "{{Airport-dest-list| [[Fuji Dream Airlines]] | [[Fukuoka Airport|Fukuoka]], [[Kobe Airport|Kobe]] | [[Japan Airlines]] | '''Seasonal:''' [[Osaka Itami Airport|Osaka–Itami]]}}"
        ),
        "{| class=\"wikitable\"\n|-\n! Airlines\n! Destinations\n|-\n| [[Fuji Dream Airlines]]\n| [[Fukuoka Airport|Fukuoka]], [[Kobe Airport|Kobe]]\n|-\n| [[Japan Airlines]]\n| '''Seasonal:''' [[Osaka Itami Airport|Osaka–Itami]]\n|}"
    );
}

#[test]
fn render_wikitext_formats_nws_current_template() {
    assert_eq!(
        render_templates("{{NWS-current|RJAF}}"),
        "[http://tgftp.nws.noaa.gov/weather/current/RJAF.html Current weather for RJAF] at NOAA/NWS"
    );
    assert_eq!(
        render_templates("{{NWS-current|LSZH|Zurich Airport}}"),
        "[http://tgftp.nws.noaa.gov/weather/current/LSZH.html Current weather for Zurich Airport] at NOAA/NWS"
    );
}

#[test]
fn render_wikitext_formats_portal_inline_template() {
    assert_eq!(
        render_templates("{{Portal-inline|Canada}}"),
        "[[Portal:Canada|Canada portal]]"
    );
    assert_eq!(
        render_templates("{{portal inline|Canada|short=yes}}"),
        "[[Portal:Canada|Canada]]"
    );
    assert_eq!(
        render_templates("{{Portal-inline|Canada|text=Canadian portal}}"),
        "[[Portal:Canada|Canadian portal]]"
    );
}

#[test]
fn render_wikitext_silently_skips_end_box_template() {
    assert_eq!(render_templates("{{end box}}"), "");
}

#[test]
fn render_wikitext_formats_mp_template() {
    assert_eq!(
        render_templates("{{Mp|2004 MN|4}}"),
        "2004 MN__WIKIPEDIA_TO_EPUB_SUB_START__4__WIKIPEDIA_TO_EPUB_SUB_END__"
    );
    assert_eq!(
        render_templates("{{minor planet|15788|1993 SB}}"),
        "(15788) 1993 SB"
    );
    assert_eq!(
        render_templates("{{Mp|15760|1992 QB|1}}"),
        "(15760) 1992 QB__WIKIPEDIA_TO_EPUB_SUB_START__1__WIKIPEDIA_TO_EPUB_SUB_END__"
    );
    assert_eq!(
        render_templates("{{minor planet|S|2000|1998 WW|31|1}}"),
        "S/2000 (1998 WW__WIKIPEDIA_TO_EPUB_SUB_START__31__WIKIPEDIA_TO_EPUB_SUB_END__) 1"
    );
    assert_eq!(
        render_templates("{{Mp|S|2005|1994 XD|1}}"),
        "S/2005 (1994 XD) 1"
    );
}

#[test]
fn render_wikitext_formats_poem_quote_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Poem quote|\nold pond\nfrog leaps in\nwater's sound\n}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("<blockquote>"), "{rendered}");
    assert!(rendered.contains("<p>old pond</p>"), "{rendered}");
    assert!(rendered.contains("<p>frog leaps in</p>"), "{rendered}");
    assert!(rendered.contains("<p>water's sound</p>"), "{rendered}");
    assert!(rendered.contains("</blockquote>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_verse_translation_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Verse translation|lang1=it|L'autunno giovane|The young autumn}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("<blockquote>"), "{rendered}");
    assert!(
        rendered.contains("<em>L'autunno giovane</em>"),
        "{rendered}"
    );
    assert!(rendered.contains("<p>The young autumn</p>"), "{rendered}");
    assert!(rendered.contains("</blockquote>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_verse_transliteration_translation_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Verse transliteration-translation|稲妻の|inazuma no|the flash}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("<blockquote>"), "{rendered}");
    assert!(rendered.contains("<p>稲妻の</p>"), "{rendered}");
    assert!(rendered.contains("<em>inazuma no</em>"), "{rendered}");
    assert!(rendered.contains("<p>the flash</p>"), "{rendered}");
    assert!(rendered.contains("</blockquote>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_main_list_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Main list|List of members of the Diet of Japan}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("For a more comprehensive list, see <a href=\"https://en.wikipedia.org/wiki/List_of_members_of_the_Diet_of_Japan\">List of members of the Diet of Japan</a>"),
        "{rendered}"
    );

    let rendered2 = render_wikitext(
        "Sample",
        "{{Main list|more=no|List of members of the Diet of Japan}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered2.contains("For a comprehensive list, see <a href=\"https://en.wikipedia.org/wiki/List_of_members_of_the_Diet_of_Japan\">List of members of the Diet of Japan</a>"),
        "{rendered2}"
    );
}

#[test]
fn render_wikitext_formats_dts_template() {
    let rendered = render_wikitext("Sample", "{{dts|1947-5-20}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("May 20, 1947"), "{rendered}");

    let rendered_dmy = render_wikitext(
        "Sample",
        "{{dts|1947-05-20|format=dmy}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered_dmy.contains("20 May 1947"), "{rendered_dmy}");

    let rendered_parts =
        render_wikitext("Sample", "{{dts|1947|May|20}}", &InternalLinks::new(), "en");
    assert!(rendered_parts.contains("May 20, 1947"), "{rendered_parts}");

    let rendered_bc = render_wikitext(
        "Sample",
        "{{dts|0100-05-20|bc=yes}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered_bc.contains("May 20, 100 BC"), "{rendered_bc}");
}

#[test]
fn render_wikitext_formats_wikivoyage_inline_template() {
    let rendered = render_wikitext(
        "Sample",
        "See {{Wikivoyage-inline|Gifu (prefecture)}}.",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("Wikivoyage: <a href=\"https://en.wikivoyage.org/wiki/Gifu_(prefecture)\">Gifu (prefecture)</a>"),
        "rendered output was: {rendered}"
    );
}

#[test]
fn render_wikitext_formats_wikivoyage_inline_space_separated_template() {
    let rendered = render_wikitext(
        "Sample",
        "See {{Wikivoyage inline|Honshu}}.",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered
            .contains("Wikivoyage: <a href=\"https://en.wikivoyage.org/wiki/Honshu\">Honshu</a>"),
        "rendered output was: {rendered}"
    );
}

#[test]
fn render_wikitext_formats_nb5_template() {
    let rendered = render_wikitext("Sample", "A{{Nb5}}B", &InternalLinks::new(), "en");
    assert!(
        rendered.contains("<p>A B</p>"),
        "rendered output was: {rendered}"
    );
}

#[test]
fn render_wikitext_formats_generic_ship_template() {
    let rendered = render_wikitext(
        "Sample",
        "See {{ship|Japanese cruiser|Kiso}}.",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains(r#"<a href="https://en.wikipedia.org/wiki/Japanese_cruiser_Kiso">Japanese cruiser <em>Kiso</em></a>"#),
        "rendered output was: {rendered}"
    );
}

#[test]
fn test_hierarchical_book_config_parsing() {
    let yaml = r#"chapters: title
metadata:
  title: "The Solar System"
  author: "Wikipedia contributors"
  language: en
  edition: First edition
output-file: planets.epub
cover: "None"
links_to_pages: false
links_to_excluded_pages: emphasize
caching: none
depth: 0
articles:
  - "Earth"
  - title: "Solar System"
    articles:
      - "Sun"
      - "Mercury"
  - title: "Planets Info"
    type: "section"
    articles:
      - "Venus"
"#;
    let config =
        serde_yaml::from_str::<BookConfig>(yaml).expect("should parse hierarchical config");
    assert_eq!(config.articles.len(), 3);

    match &config.articles[0] {
        ArticleConfig::Simple(title) => assert_eq!(title, "Earth"),
        _ => panic!("Expected simple article entry"),
    }

    match &config.articles[1] {
        ArticleConfig::Detailed(detailed) => {
            assert_eq!(detailed.title, "Solar System");
            assert_eq!(detailed.r#type, None);
            assert_eq!(detailed.articles.len(), 2);
        }
        _ => panic!("Expected detailed article entry"),
    }

    match &config.articles[2] {
        ArticleConfig::Detailed(detailed) => {
            assert_eq!(detailed.title, "Planets Info");
            assert_eq!(detailed.r#type, Some(ArticleType::Section));
            assert_eq!(detailed.articles.len(), 1);
        }
        _ => panic!("Expected detailed article entry"),
    }
}

#[test]
fn render_wikitext_formats_proto_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Proto|germanic|erþō}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Proto-Germanic *erþō"), "{rendered}");
}

#[test]
fn render_wikitext_formats_wktl_and_langr_templates() {
    let rendered = render_wikitext("Sample", "{{wktl|grc|γῆ|gē}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("lang=\"grc\""), "{rendered}");
    assert!(rendered.contains("γῆ"), "{rendered}");

    let rendered = render_wikitext("Sample", "{{langr|la|Terra}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("lang=\"la\""), "{rendered}");
    assert!(rendered.contains("Terra"), "{rendered}");
}

#[test]
fn render_wikitext_formats_val_and_value_templates() {
    let rendered = render_wikitext(
        "Sample",
        "{{val|4.5682|0.0002|0.0004}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("4.5682 (+0.0004/-0.0002)"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{val|4.54|0.04|u=Ga}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("4.54 ± 0.04 Ga"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{val|600|–|540|u=Ma}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("600 – 540 Ma"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{val|5.97|e=24|ul=kg}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("5.97 × 10<sup>24</sup> kg"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{Value|5.97|u=[[Ronnagram|Rg]]}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("5.97 <a href=\"https://en.wikipedia.org/wiki/Ronnagram\">Rg</a>"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_chem2_template() {
    let rendered = render_wikitext("Sample", "{{chem2|O2}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("O<sub>2</sub>"), "{rendered}");

    let rendered = render_wikitext("Sample", "{{chem2|CO2}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("CO<sub>2</sub>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_e_template() {
    let rendered = render_wikitext("Sample", "{{e|-5}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("× 10<sup>-5</sup>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_sup_and_sub_templates() {
    let rendered = render_wikitext("Sample", "{{sup|2}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("<sup>2</sup>"), "{rendered}");

    let rendered = render_wikitext("Sample", "{{sub|x}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("<sub>x</sub>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_su_template() {
    let rendered = render_wikitext("Sample", "{{su|p=2}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("<sup>2</sup>"), "{rendered}");

    let rendered = render_wikitext("Sample", "{{su|b=x}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("<sub>x</sub>"), "{rendered}");

    let rendered = render_wikitext("Sample", "{{su|p=2|b=x}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("<sup>2</sup><sub>x</sub>"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{su|p={{sup|2}}|b={{sub|x}}}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("<sup><sup>2</sup></sup><sub><sub>x</sub></sub>"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_mpl_template() {
    let rendered = render_wikitext("Sample", "{{mpl|2010 TK|7}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("2010 TK7"), "{rendered}");
}

#[test]
fn render_wikitext_formats_columns_list_template() {
    let wikitext = "{{columns list|colwidth=22em|\n* Item 1\n* Item 2}}";
    let rendered = render_wikitext("Sample", wikitext, &InternalLinks::new(), "en");
    assert!(rendered.contains("<li>Item 1</li>"), "{rendered}");
    assert!(rendered.contains("<li>Item 2</li>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_annotated_link_template() {
    let mut links = InternalLinks::new();
    links.insert(
        "celestialsphere".to_string(),
        "Celestial_sphere.xhtml".to_string(),
    );
    let rendered = render_wikitext(
        "Sample",
        "{{annotated link|Celestial sphere}}",
        &links,
        "en",
    );
    assert!(
        rendered.contains("href=\"Celestial_sphere.xhtml\""),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_dp_template() {
    let rendered = render_wikitext("Sample", "{{Dp|Ceres}}", &InternalLinks::new(), "en");
    assert!(
        rendered.contains("href=\"https://en.wikipedia.org/wiki/Ceres_(dwarf_planet)\""),
        "{rendered}"
    );
    assert!(rendered.contains("Ceres"), "{rendered}");

    let rendered = render_wikitext("Sample", "{{dp|makemake}}", &InternalLinks::new(), "en");
    assert!(
        rendered.contains("href=\"https://en.wikipedia.org/wiki/Makemake\""),
        "{rendered}"
    );
    assert!(rendered.contains("makemake"), "{rendered}");
}

#[test]
fn render_wikitext_formats_visible_anchor_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Visible anchor|Mercury|text=[[Mercury]]}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Mercury"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{visible anchor|Earth}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Earth"), "{rendered}");
}

#[test]
fn render_wikitext_formats_lagrange_templates() {
    let rendered = render_wikitext("Sample", "{{L4}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("L<sub>4</sub>"), "{rendered}");

    let rendered = render_wikitext("Sample", "{{L5}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("L<sub>5</sub>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_cite_eb1911_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Cite EB1911|wstitle=Solar System}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Solar System"), "{rendered}");
    assert!(
        rendered.contains(
            "href=\"https://en.wikisource.org/wiki/1911_Encyclop%C3%A6dia_Britannica/Solar_System\""
        ),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_longitem_template() {
    let rendered = render_wikitext(
        "Sample",
        "A{{longitem|B <br /> C}}D",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("<p>AB CD</p>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_flagdeco_template() {
    let rendered = render_wikitext(
        "Sample",
        "A{{flagdeco|United Nations}}B",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("<p>AB</p>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_pprime_template() {
    let rendered = render_wikitext("Sample", "A{{pprime|9.7}}B", &InternalLinks::new(), "en");
    assert!(rendered.contains("<p>A9.7″B</p>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_ra_template() {
    let rendered = render_wikitext("Sample", "A{{RA|18|11|2}}B", &InternalLinks::new(), "en");
    assert!(
        rendered.contains("<p>A18<sup>h</sup> 11<sup>m</sup> 2<sup>s</sup>B</p>"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_mw_template() {
    let rendered = render_wikitext(
        "Sample",
        "A{{MW|Venusian|access-date=2026-06-11}}B",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("href=\"https://www.merriam-webster.com/dictionary/Venusian\""),
        "{rendered}"
    );
    assert!(
        rendered.contains("<em>Merriam-Webster.com Dictionary</em>"),
        "{rendered}"
    );
    assert!(rendered.contains("Retrieved 2026-06-11"), "{rendered}");
}

#[test]
fn render_wikitext_formats_indented_plainlist_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{indented plainlist|* Item 1\n* Item 2}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("<li>Item 1</li>"), "{rendered}");
    assert!(rendered.contains("<li>Item 2</li>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_bulleted_list_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{bulleted list|Item 1|Item 2}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("<li>Item 1</li>"), "{rendered}");
    assert!(rendered.contains("<li>Item 2</li>"), "{rendered}");

    let rendered2 = render_wikitext(
        "Sample",
        "{{blist|Item X|Item Y}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered2.contains("<li>Item X</li>"), "{rendered2}");
    assert!(rendered2.contains("<li>Item Y</li>"), "{rendered2}");
}

#[test]
fn render_wikitext_formats_hyphen_template() {
    let rendered = render_wikitext("Sample", "A{{Hyphen}}B", &InternalLinks::new(), "en");
    assert!(rendered.contains("<p>A-B</p>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_native_phrase_template() {
    let rendered = render_wikitext(
        "Sample",
        "A{{native phrase|ko|渤海}}B",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("<em>渤海</em> (Korean)"), "{rendered}");

    let rendered2 = render_wikitext(
        "Sample",
        "A{{native name|ko|高句麗|paren=omit}}B",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered2.contains("<em>高句麗</em>"), "{rendered2}");
    assert!(!rendered2.contains("(Korean)"), "{rendered2}");
}

#[test]
fn render_wikitext_formats_spaces_template() {
    let rendered = render_wikitext("Sample", "A{{spaces|3}}B", &InternalLinks::new(), "en");
    assert!(rendered.contains("<p>A B</p>"), "{rendered}");
    assert!(!rendered.contains("{{"));

    let rendered = render_wikitext("Sample", "A{{spaces}}B", &InternalLinks::new(), "en");
    assert!(rendered.contains("<p>A B</p>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_mpl_dash_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{mpl-|322756|2001 CK|32}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("href=\"https://en.wikipedia.org/wiki/(322756)_2001_CK32\""),
        "{rendered}"
    );
    assert!(rendered.contains("(322756) 2001 CK32"), "{rendered}");

    let rendered = render_wikitext(
        "Sample",
        "{{mpl-|322756|2001 CK}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("href=\"https://en.wikipedia.org/wiki/(322756)_2001_CK\""),
        "{rendered}"
    );
    assert!(rendered.contains("(322756) 2001 CK"), "{rendered}");
}

#[test]
fn render_wikitext_formats_chem_template() {
    let rendered = render_wikitext("Sample", "{{chem|H|2|O}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("H<sub>2</sub>O"), "{rendered}");

    let rendered = render_wikitext("Sample", "{{chem|CO|3|2-}}", &InternalLinks::new(), "en");
    assert!(
        rendered.contains("CO<sub>3</sub><sup>2-</sup>"),
        "{rendered}"
    );

    let rendered = render_wikitext("Sample", "{{chem|H|3|O|+}}", &InternalLinks::new(), "en");
    assert!(
        rendered.contains("H<sub>3</sub>O<sup>+</sup>"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_also_template() {
    let rendered = render_wikitext(
        "Sample",
        "See {{also|Standard solar model}}.",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("See also: <a href=\"https://en.wikipedia.org/wiki/Standard_solar_model\">Standard solar model</a>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_solar_radius_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{solar radius|1.2}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("1.2 R<sub>☉</sub>"), "{rendered}");

    let rendered = render_wikitext("Sample", "{{solar radius}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("R<sub>☉</sub>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_plus_minus_template() {
    let rendered = render_wikitext("Sample", "{{±|10|2}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("± 10 2"), "{rendered}");

    let rendered = render_wikitext("Sample", "{{±}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("±"), "{rendered}");
}

#[test]
fn render_wikitext_formats_cite_encyclopedia_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{cite encyclopedia|title=Solar activity|encyclopedia=Scholarpedia}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Solar activity"), "{rendered}");
    assert!(rendered.contains("Scholarpedia"), "{rendered}");
}

#[test]
fn render_wikitext_formats_ja_rail_color_template() {
    let rendered1 = render_wikitext(
        "Sample",
        "color:{{Ja-rail-color|JY}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered1.contains("color:#80c241"), "{rendered1}");

    let rendered2 = render_wikitext(
        "Sample",
        "color:{{ja-rail-color|jk}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered2.contains("color:#00b2e5"), "{rendered2}");

    let rendered3 = render_wikitext(
        "Sample",
        "color:{{Ja-rail-color|invalid}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered3.contains("color:#333333"), "{rendered3}");
}

#[test]
fn render_wikitext_formats_route_box_template() {
    let rendered1 = render_wikitext(
        "Sample",
        "{{RouteBox|Yamanote Line|Yamanote Line|#80c241|white}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered1.contains("background-color: #80c241"),
        "{rendered1}"
    );
    assert!(rendered1.contains("color: white"), "{rendered1}");
    assert!(rendered1.contains("Yamanote Line"), "{rendered1}");
    assert!(
        rendered1
            .contains("<a href=\"https://en.wikipedia.org/wiki/Yamanote_Line\">Yamanote Line</a>"),
        "{rendered1}"
    );

    // Test with nested Ja-rail-color
    let rendered2 = render_wikitext(
        "Sample",
        "{{RouteBox|JY|Yamanote Line|{{Ja-rail-color|JY}}|white}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered2.contains("background-color: #80c241"),
        "{rendered2}"
    );
    assert!(rendered2.contains("color: white"), "{rendered2}");
    assert!(
        rendered2.contains("<a href=\"https://en.wikipedia.org/wiki/Yamanote_Line\">JY</a>"),
        "{rendered2}"
    );
}

#[test]
fn render_wikitext_formats_nihongo_foot_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Nihongo foot|'''Tokyo Metropolis'''|東京都|Tōkyō-to|{{IPA|ja|toː.kʲoꜜː.to}}|post=,}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("<strong>Tokyo Metropolis</strong>"),
        "{rendered}"
    );
    assert!(rendered.contains("東京都"), "{rendered}");
    assert!(rendered.contains("Tōkyō-to"), "{rendered}");
    assert!(rendered.contains("[toː.kʲoꜜː.to]"), "{rendered}");
    assert!(rendered.contains(","), "{rendered}");
}

#[test]
fn render_wikitext_formats_literal_translation_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Literal translation|[[Capital of Japan|Eastern Capital]]}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains(
            "lit. <a href=\"https://en.wikipedia.org/wiki/Capital_of_Japan\">Eastern Capital</a>"
        ),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_na_template() {
    let rendered1 = render_wikitext("Sample", "{{N/A}}", &InternalLinks::new(), "en");
    assert!(rendered1.contains("<p>N/A</p>"), "{rendered1}");

    let rendered2 = render_wikitext("Sample", "{{NA|custom text}}", &InternalLinks::new(), "en");
    assert!(rendered2.contains("<p>custom text</p>"), "{rendered2}");

    let rendered3 = render_wikitext("Sample", "{{Not applicable|}}", &InternalLinks::new(), "en");
    assert!(rendered3.contains("<p>N/A</p>"), "{rendered3}");
}

#[test]
fn render_wikitext_formats_quote_escaping_templates() {
    let rendered1 = render_wikitext("Sample", "a{{'\"}}b", &InternalLinks::new(), "en");
    assert!(rendered1.contains("a'\"b"), "{rendered1}");

    let rendered2 = render_wikitext("Sample", "a{{\"'}}b", &InternalLinks::new(), "en");
    assert!(rendered2.contains("a\"'b"), "{rendered2}");
}

#[test]
fn render_wikitext_formats_nbndash_template() {
    let rendered = render_wikitext("Sample", "2020{{nbndash}}2022", &InternalLinks::new(), "en");
    assert!(rendered.contains("2020–2022"), "{rendered}");
}

#[test]
fn render_wikitext_formats_ric_template() {
    let rendered1 = render_wikitext("Sample", "{{ric|JR East|JT}}", &InternalLinks::new(), "en");
    assert!(rendered1.contains("[JT]"), "{rendered1}");

    let rendered2 = render_wikitext(
        "Sample",
        "{{rint|london|underground}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered2.contains("[underground]"), "{rendered2}");
}

#[test]
fn render_wikitext_formats_ja_platform_template() {
    let rendered = render_wikitext(
        "Sample",
        "{| class=\"wikitable\"\n{{jpf|pfn=1|name=Yamanote Line|dir=for Tokyo}}\n|}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("<td><strong>1</strong></td>"),
        "{rendered}"
    );
    assert!(rendered.contains("<td>Yamanote Line</td>"), "{rendered}");
    assert!(rendered.contains("<td>for Tokyo</td>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_lnl_template() {
    let rendered_jy = render_wikitext("Sample", "{{lnl|JR East|JY}}", &InternalLinks::new(), "en");
    assert!(
        rendered_jy.contains("href=\"https://en.wikipedia.org/wiki/Yamanote_Line\""),
        "{rendered_jy}"
    );
    assert!(rendered_jy.contains("Yamanote Line"), "{rendered_jy}");

    let rendered_jc = render_wikitext("Sample", "{{lnl|JR East|JC}}", &InternalLinks::new(), "en");
    assert!(
        rendered_jc.contains("href=\"https://en.wikipedia.org/wiki/Chūō_Line_(Rapid)\""),
        "{rendered_jc}"
    );
    assert!(rendered_jc.contains("Chūō Line"), "{rendered_jc}");

    let rendered_fallback = render_wikitext(
        "Sample",
        "{{lnl|Tokyo Metro|M}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered_fallback.contains("href=\"https://en.wikipedia.org/wiki/Tokyo_Metro_M_Line\""),
        "{rendered_fallback}"
    );
    assert!(rendered_fallback.contains("M Line"), "{rendered_fallback}");
}

#[test]
fn test_load_markdown_chapter() {
    use std::fs::File;
    use std::io::Write;
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join("test_chapter.md");
    let mut file = File::create(&file_path).unwrap();
    let md_content = "\
# My Test Chapter

Hello world! This is a **markdown** paragraph.

- List item 1
- List item 2
";
    file.write_all(md_content.as_bytes()).unwrap();

    let chapter = crate::load_markdown_chapter(&file_path, "en").unwrap();
    assert_eq!(chapter.title, "My Test Chapter");
    assert_eq!(chapter.file_name, "test_chapter.xhtml");

    // Check that the content is valid XHTML and contains translated Markdown elements
    assert!(chapter.content.contains("xml:lang=\"en\""));
    assert!(chapter.content.contains("<title>My Test Chapter</title>"));
    assert!(chapter.content.contains("<h1>My Test Chapter</h1>"));
    assert!(
        chapter
            .content
            .contains("<p>Hello world! This is a <strong>markdown</strong> paragraph.</p>")
    );
    assert!(chapter.content.contains("<ul>"));
    assert!(chapter.content.contains("<li>List item 1</li>"));

    // Clean up
    std::fs::remove_file(file_path).ok();
}

#[test]
fn render_wikitext_formats_hlist_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{hlist|[[Gifu Prefecture|Gifu]]|[[Nagano Prefecture|Nagano]]}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("Gifu</a>")
            && rendered.contains(", <a")
            && rendered.contains("Nagano</a>"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_native_name_list_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{native name list |tag1=ja|name1=木曽山脈 |tag2=ja|name2=中央アルプス}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("木曽山脈 (Japanese), 中央アルプス (Japanese)"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_infobox_mountain_template() {
    let rendered = render_wikitext(
        "Sample",
        r#"{{Infobox mountain
| name = Kiso Mountains
| native_name = {{native name list |tag1=ja|name1=木曽山脈}}
| country = [[Japan]]
| elevation_m = 2956
}}"#,
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Name"), "{rendered}");
    assert!(rendered.contains("Kiso Mountains"), "{rendered}");
    assert!(rendered.contains("Native name"), "{rendered}");
    assert!(rendered.contains("木曽山脈 (Japanese)"), "{rendered}");
    assert!(rendered.contains("Country"), "{rendered}");
    assert!(rendered.contains("Japan</a>"), "{rendered}");
    assert!(rendered.contains("Elevation"), "{rendered}");
    assert!(rendered.contains("2956"), "{rendered}");
}

#[test]
fn render_wikitext_formats_infobox_country_template() {
    let rendered = render_wikitext(
        "Sample",
        r#"{{Infobox country
| conventional_long_name = Republic of Korea
| common_name = South Korea
| native_name = {{lang|ko|대한민국}}
| image_flag = Flag of South Korea.svg
| image_coat = Emblem of South Korea.svg
| symbol_type = Emblem
| anthem = "[[Aegukga|Patriotic Song]]"
| capital = [[Seoul]]
| official_languages = [[Korean language|Korean]]
| demonym = [[South Koreans|South Korean]]
| religion = [[Buddhism]]
| currency = [[South Korean won|Won]]
| year_start = 1948
}}"#,
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Name"), "{rendered}");
    assert!(rendered.contains("Republic of Korea"), "{rendered}");
    assert!(rendered.contains("Common name"), "{rendered}");
    assert!(rendered.contains("South Korea"), "{rendered}");
    assert!(rendered.contains("Native name"), "{rendered}");
    assert!(rendered.contains("대한민국"), "{rendered}");
    assert!(rendered.contains("Flag"), "{rendered}");
    assert!(rendered.contains("Flag of South Korea.svg"), "{rendered}");
    assert!(rendered.contains("Emblem"), "{rendered}");
    assert!(rendered.contains("Emblem of South Korea.svg"), "{rendered}");
    assert!(rendered.contains("Anthem"), "{rendered}");
    assert!(rendered.contains("Patriotic Song"), "{rendered}");
    assert!(rendered.contains("Capital"), "{rendered}");
    assert!(rendered.contains("Seoul</a>"), "{rendered}");
    assert!(rendered.contains("Official languages"), "{rendered}");
    assert!(rendered.contains("Korean</a>"), "{rendered}");
    assert!(rendered.contains("Demonym"), "{rendered}");
    assert!(rendered.contains("South Korean"), "{rendered}");
    assert!(rendered.contains("Religion"), "{rendered}");
    assert!(rendered.contains("Buddhism"), "{rendered}");
    assert!(rendered.contains("Currency"), "{rendered}");
    assert!(rendered.contains("Won"), "{rendered}");
    assert!(rendered.contains("Year established"), "{rendered}");
    assert!(rendered.contains("1948"), "{rendered}");
}

#[test]
fn render_wikitext_formats_infobox_military_conflict_template() {
    let rendered = render_wikitext(
        "Sample",
        r#"{{Infobox military conflict
| conflict = Korean War
| partof = the [[Cold War]]
| image = [[File:Battle.jpg]]
| footer = Clockwise from top left
| date = 25 June 1950 – 27 July 1953
| place = [[Korean Peninsula]]
| territory = [[Korean Demilitarized Zone]] established
| result = Inconclusive
| combatant1 = {{Plainlist|* [[South Korea]]|* [[United Nations]]}}
| combatant2 = {{Plainlist|* [[North Korea]]|* [[China]]}}
| commander1 = [[Douglas MacArthur]]
| commander2 = [[Kim Il Sung]]
| strength1 = 968,302
| strength2 = 1,642,600
| casualties1 = 178,236 dead
| casualties2 = 600,000 dead
}}"#,
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Conflict"), "{rendered}");
    assert!(rendered.contains("Korean War"), "{rendered}");
    assert!(rendered.contains("Part of"), "{rendered}");
    assert!(rendered.contains("Cold War"), "{rendered}");
    assert!(rendered.contains("Image"), "{rendered}");
    assert!(rendered.contains("Clockwise from top left"), "{rendered}");
    assert!(rendered.contains("Date"), "{rendered}");
    assert!(rendered.contains("25 June 1950"), "{rendered}");
    assert!(rendered.contains("Place"), "{rendered}");
    assert!(rendered.contains("Korean Peninsula"), "{rendered}");
    assert!(rendered.contains("Territorial changes"), "{rendered}");
    assert!(rendered.contains("Korean Demilitarized Zone"), "{rendered}");
    assert!(rendered.contains("Result"), "{rendered}");
    assert!(rendered.contains("Inconclusive"), "{rendered}");
    assert!(rendered.contains("Combatant 1"), "{rendered}");
    assert!(rendered.contains("South Korea"), "{rendered}");
    assert!(rendered.contains("Combatant 2"), "{rendered}");
    assert!(rendered.contains("North Korea"), "{rendered}");
    assert!(rendered.contains("Commander 1"), "{rendered}");
    assert!(rendered.contains("Douglas MacArthur"), "{rendered}");
    assert!(rendered.contains("Commander 2"), "{rendered}");
    assert!(rendered.contains("Kim Il Sung"), "{rendered}");
    assert!(rendered.contains("Strength 1"), "{rendered}");
    assert!(rendered.contains("968,302"), "{rendered}");
    assert!(rendered.contains("Casualties 2"), "{rendered}");
    assert!(rendered.contains("600,000 dead"), "{rendered}");
}

#[test]
fn render_wikitext_formats_infobox_military_conflict_template_with_plain_image() {
    let internal_links = InternalLinks::new();
    let mut image_registry =
        ImageRegistry::new(Some(std::path::Path::new("pages"))).expect("image registry loads");
    let rendered = render_wikitext_with_template_counts(
        "Sample",
        r#"{{Infobox military conflict
| conflict = Battle of Sekigahara
| image = Sekigaharascreen.jpg
| caption = Screen of the Battle of Sekigahara
}}"#,
        &internal_links,
        "en",
        Some(&mut image_registry),
    )
    .0;

    assert!(rendered.contains("Battle of Sekigahara"), "{rendered}");
    assert!(
        rendered.contains("Sekigaharascreen.jpg") || rendered.contains("images/"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_infobox_settlement_template() {
    let rendered = render_wikitext(
        "Sample",
        r#"{{Infobox settlement
| name = Osaka
| official_name = Osaka City
| native_name = {{nobold|大阪市}}
| subdivision_type = Country
| subdivision_name = Japan
| population_total = 2,816,247
| area_total_km2 = 225.21
}}"#,
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Name"), "{rendered}");
    assert!(rendered.contains("Osaka"), "{rendered}");
    assert!(rendered.contains("Official name"), "{rendered}");
    assert!(rendered.contains("Osaka City"), "{rendered}");
    assert!(rendered.contains("Native name"), "{rendered}");
    assert!(rendered.contains("大阪市"), "{rendered}");
    assert!(rendered.contains("Country"), "{rendered}");
    assert!(rendered.contains("Japan"), "{rendered}");
    assert!(rendered.contains("Population"), "{rendered}");
    assert!(rendered.contains("2,816,247"), "{rendered}");
    assert!(rendered.contains("Area"), "{rendered}");
    assert!(rendered.contains("225.21"), "{rendered}");
}

#[test]
fn render_wikitext_formats_infobox_planet_template() {
    let rendered = render_wikitext(
        "Sample",
        r#"{{Infobox planet
| name = Mars
| symbol = [[File:Mars symbol (bold).svg|24px|♂|class=skin-invert]]
| image = Mars.png
| caption = Mars in true color
| aphelion = {{convert|249261000|km|AU|abbr=on}}
| mean_radius = {{val|3389.5|u=km}}
| satellites = [[Moons of Mars|2]]
| temp_name2 = Surface
| min_temp_2 = −110&nbsp;°C
| mean_temp_2 = −24&nbsp;°C
| max_temp_2 = 35&nbsp;°C
| atmosphere_composition = [[Carbon dioxide]]
}}"#,
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Name"), "{rendered}");
    assert!(rendered.contains("Mars"), "{rendered}");
    assert!(rendered.contains("Symbol"), "{rendered}");
    assert!(rendered.contains("♂"), "{rendered}");
    assert!(rendered.contains("Image"), "{rendered}");
    assert!(rendered.contains("Mars.png"), "{rendered}");
    assert!(rendered.contains("Mars in true color"), "{rendered}");
    assert!(rendered.contains("Aphelion"), "{rendered}");
    assert!(rendered.contains("249,261,000 km"), "{rendered}");
    assert!(rendered.contains("Mean radius"), "{rendered}");
    assert!(rendered.contains("3389.5 km"), "{rendered}");
    assert!(rendered.contains("Satellites"), "{rendered}");
    assert!(
        rendered.contains("https://en.wikipedia.org/wiki/Moons_of_Mars"),
        "{rendered}"
    );
    assert!(rendered.contains(">2</a>"), "{rendered}");
    assert!(rendered.contains("Surface"), "{rendered}");
    assert!(rendered.contains("min −110"), "{rendered}");
    assert!(rendered.contains("mean −24"), "{rendered}");
    assert!(rendered.contains("max 35"), "{rendered}");
    assert!(rendered.contains("Atmosphere composition"), "{rendered}");
    assert!(rendered.contains("Carbon dioxide"), "{rendered}");
}

#[test]
fn render_wikitext_formats_infobox_generic_template() {
    let rendered = render_wikitext(
        "Sample",
        r#"{{infobox
| title = Sun
| label1 = Names
| data1 = Sun, Sol
| label2 = Adjectives
| data2 = Solar
}}"#,
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Sun"), "{rendered}");
    assert!(rendered.contains("Names"), "{rendered}");
    assert!(rendered.contains("Sun, Sol"), "{rendered}");
    assert!(rendered.contains("Adjectives"), "{rendered}");
    assert!(rendered.contains("Solar"), "{rendered}");
}

#[test]
fn render_wikitext_formats_cite_dictionary_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{cite dictionary |last=Smith |first=John |title=Apple |dictionary=English Dictionary |edition=2nd |publisher=Oxford |date=2020 |page=15}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("John Smith"), "{rendered}");
    assert!(rendered.contains("\"Apple\""), "{rendered}");
    assert!(
        rendered.contains("<em>English Dictionary</em>"),
        "{rendered}"
    );
    assert!(rendered.contains("2nd ed"), "{rendered}");
    assert!(rendered.contains("Oxford"), "{rendered}");
    assert!(rendered.contains("2020"), "{rendered}");
    assert!(rendered.contains("p. 15"), "{rendered}");
}

#[test]
fn render_wikitext_formats_cite_press_release_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{cite press release |last=Doe |first=Jane |title=New Release |publisher=Company |date=2021 |url=https://press.com}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Jane Doe"), "{rendered}");
    assert!(
        rendered.contains("\"New Release\" (Press release)"),
        "{rendered}"
    );
    assert!(rendered.contains("Company"), "{rendered}");
    assert!(rendered.contains("2021"), "{rendered}");
    assert!(!rendered.contains("href=\"http"), "{rendered}");
}

#[test]
fn render_wikitext_formats_cite_apod_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{cite apod |title=Nebula |date=2020-04-15 |access-date=2020-05-01}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("R. Nemiroff"), "{rendered}");
    assert!(rendered.contains("J. Bonnell"), "{rendered}");
    assert!(rendered.contains("\"Nebula\""), "{rendered}");
    assert!(
        rendered.contains("<em>Astronomy Picture of the Day</em>"),
        "{rendered}"
    );
    assert!(rendered.contains("NASA"), "{rendered}");
    assert!(rendered.contains("Retrieved 2020-05-01"), "{rendered}");
    assert!(!rendered.contains("href=\"http"), "{rendered}");
}

#[test]
fn render_wikitext_formats_cite_oed_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{cite OED |entry=Word |id=12345 |date=2015 |access-date=2016}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("\"Word\""), "{rendered}");
    assert!(
        rendered.contains("<em>Oxford English Dictionary</em> (Online ed.)"),
        "{rendered}"
    );
    assert!(rendered.contains("Oxford University Press"), "{rendered}");
    assert!(rendered.contains("2015"), "{rendered}");
    assert!(rendered.contains("Retrieved 2016"), "{rendered}");
    assert!(!rendered.contains("href=\"http"), "{rendered}");

    let rendered2 = render_wikitext(
        "Sample",
        "{{cite OED |entry=Big Word}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered2.contains("\"Big Word\""), "{rendered2}");
    assert!(!rendered2.contains("href=\"http"), "{rendered2}");
}

#[test]
fn render_wikitext_formats_oed_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{OED |entry=Word |id=12345 |date=2015 |access-date=2016}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("\"Word\""), "{rendered}");
    assert!(
        rendered.contains("<em>Oxford English Dictionary</em> (Online ed.)"),
        "{rendered}"
    );
    assert!(rendered.contains("Oxford University Press"), "{rendered}");
    assert!(rendered.contains("2015"), "{rendered}");
    assert!(rendered.contains("Retrieved 2016"), "{rendered}");
    assert!(!rendered.contains("href=\"http"), "{rendered}");
}

#[test]
fn render_wikitext_formats_cite_av_media_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{cite AV media |last=Director |first=A. |title=Movie |format=Film |publisher=Studio |via=YouTube |date=2010 |access-date=2012}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("A. Director"), "{rendered}");
    assert!(rendered.contains("\"Movie\""), "{rendered}");
    assert!(rendered.contains("(Film)"), "{rendered}");
    assert!(rendered.contains("Studio"), "{rendered}");
    assert!(rendered.contains("YouTube"), "{rendered}");
    assert!(rendered.contains("2010"), "{rendered}");
    assert!(rendered.contains("Retrieved 2012"), "{rendered}");
}

#[test]
fn render_wikitext_formats_cite_american_heritage_dictionary_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{cite American Heritage Dictionary |1=Lexicon |date=2018 |access-date=2019}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("\"Lexicon\""), "{rendered}");
    assert!(
        rendered.contains("<em>The American Heritage Dictionary of the English Language</em>"),
        "{rendered}"
    );
    assert!(rendered.contains("2018"), "{rendered}");
    assert!(rendered.contains("Retrieved 2019"), "{rendered}");
    assert!(!rendered.contains("href=\"http"), "{rendered}");
}

#[test]
fn render_wikitext_formats_cite_wikisource_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{cite wikisource |last=Author |first=B. |title=Book Title |wslink=Book Title |wslanguage=fr |publisher=Paris |year=1800}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("B. Author"), "{rendered}");
    assert!(
        rendered.contains("href=\"https://en.wikisource.org/wiki/fr:Book_Title\""),
        "{rendered}"
    );
    assert!(rendered.contains("Book Title"), "{rendered}");
    assert!(rendered.contains("Paris, 1800"), "{rendered}");
    assert!(rendered.contains("Wikisource"), "{rendered}");

    let rendered2 = render_wikitext(
        "Sample",
        "{{cite wikisource |title=English Book |wslink=English Book |publisher=London |year=1750}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered2.contains("href=\"https://en.wikisource.org/wiki/English_Book\""),
        "{rendered2}"
    );
}

#[test]
fn render_wikitext_formats_cite_cia_world_factbook_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{cite CIA World Factbook |country=North Korea |section=Geography |year=2021 |access-date=2022-03-01}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("\"North Korea § Geography\""),
        "{rendered}"
    );
    assert!(
        rendered.contains("<em>The World Factbook</em> (2021 ed.)"),
        "{rendered}"
    );
    assert!(
        rendered.contains("Central Intelligence Agency"),
        "{rendered}"
    );
    assert!(rendered.contains("Retrieved 2022-03-01"), "{rendered}");
    assert!(!rendered.contains("href=\"http"), "{rendered}");
}

#[test]
fn render_wikitext_formats_cite_letter_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{cite letter |last=Sender |first=A. |recipient=Recipient |subject=Important Matters |publisher=Archive |date=1900 |access-date=1950}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("A. Sender"), "{rendered}");
    assert!(
        rendered.contains("\"Important Matters\" (Letter to Recipient)"),
        "{rendered}"
    );
    assert!(rendered.contains("Archive"), "{rendered}");
    assert!(rendered.contains("1900"), "{rendered}");
    assert!(rendered.contains("Retrieved 1950"), "{rendered}");
}

#[test]
fn render_wikitext_formats_cite_arxiv_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{cite arXiv |last=Physicist |first=A. |title=Quantum Theory |date=2019 |eprint=1901.00001 |class=hep-th}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("A. Physicist"), "{rendered}");
    assert!(rendered.contains("\"Quantum Theory\""), "{rendered}");
    assert!(rendered.contains("2019"), "{rendered}");
    assert!(rendered.contains("1901.00001"), "{rendered}");
    assert!(rendered.contains("[hep-th]"), "{rendered}");
    assert!(!rendered.contains("href=\"http"), "{rendered}");
}

#[test]
fn render_wikitext_formats_cite_q_template() {
    let rendered = render_wikitext("Sample", "{{cite q |Q123456}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("Wikidata item Q123456"), "{rendered}");
    assert!(!rendered.contains("href=\"http"), "{rendered}");

    let rendered2 = render_wikitext(
        "Sample",
        "{{cite q |https://www.wikidata.org/wiki/Q654321}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered2.contains("Wikidata item Q654321"), "{rendered2}");
    assert!(!rendered2.contains("href=\"http"), "{rendered2}");
}

fn read_or_fetch_text(
    cache_path: &Path,
    refresh: bool,
    fetch: impl FnOnce() -> AppResult<String>,
) -> AppResult<(String, CacheSource)> {
    read_or_fetch_text_with_stats(cache_path, refresh, None, true, fetch)
}

fn read_or_fetch_bytes(
    cache_path: &Path,
    refresh: bool,
    fetch: impl FnOnce() -> AppResult<Vec<u8>>,
) -> AppResult<(Vec<u8>, CacheSource)> {
    read_or_fetch_bytes_with_stats(cache_path, refresh, None, true, fetch)
}

fn render_wikitext(
    title: &str,
    wikitext: &str,
    internal_links: &InternalLinks,
    language: &str,
) -> String {
    render_wikitext_with_template_counts(title, wikitext, internal_links, language, None).0
}

fn render_wikitext_with_excluded_links(
    title: &str,
    wikitext: &str,
    internal_links: &InternalLinks,
    language: &str,
    links_to_excluded_pages: LinksToExcludedPages,
) -> String {
    render_wikitext_with_template_counts_and_excluded_links(
        title,
        wikitext,
        internal_links,
        language,
        links_to_excluded_pages,
        None,
    )
    .0
}

fn strip_wikitext_tables(text: &str) -> String {
    let mut tables = Vec::new();
    let internal_links = InternalLinks::new();
    let mut text_with_placeholders =
        render_wikitext_tables(text, &mut tables, &internal_links, "en");
    for i in 0..tables.len() {
        text_with_placeholders =
            text_with_placeholders.replace(&format!("__WIKIPEDIA_TO_EPUB_TABLE_{}__", i), "");
    }
    text_with_placeholders
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

fn extract_main_image(wikitext: &str) -> Option<String> {
    let re = Regex::new(r"(?i)\b(?:image|map|basemap)\s*=\s*([^|}\n]+\.(?:svg|png|jpg|jpeg|gif))")
        .unwrap();
    for cap in re.captures_iter(wikitext) {
        let img = cap[1].trim().to_string();
        let lower = img.to_lowercase();
        if !lower.contains("pog")
            && !lower.contains("dot")
            && !lower.contains("pointer")
            && !lower.contains("marker")
        {
            return Some(img);
        }
    }

    let re_any = Regex::new(r"(?i)\b([^|}\n\s]+\.(?:svg|png|jpg|jpeg|gif))").unwrap();
    for cap in re_any.captures_iter(wikitext) {
        let img = cap[1].trim().to_string();
        let lower = img.to_lowercase();
        if !lower.contains("pog")
            && !lower.contains("dot")
            && !lower.contains("pointer")
            && !lower.contains("marker")
        {
            return Some(img);
        }
    }
    None
}

#[test]
fn render_wikitext_formats_mathworld_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{MathWorld|urlname=NormalDistributionFunction|title=Normal Distribution Function}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("Weisstein, Eric W. \"Normal Distribution Function\".")
            && rendered.contains("MathWorld"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_as_ref_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{AS ref|26, eqn 26.2.12|932}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("Abramowitz and Stegun")
            && rendered.contains("p. 932, § 26, eqn 26.2.12"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_oeis2c_template() {
    let rendered = render_wikitext("Sample", "{{OEIS2C|A178647}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("A178647"), "{rendered}");
}

#[test]
fn render_wikitext_formats_thinsp_template() {
    let rendered = render_wikitext("Sample", "{{thinsp|a|b}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("a b"), "{rendered}");
}

#[test]
fn render_wikitext_formats_dfn_template() {
    let rendered = render_wikitext("Sample", "{{dfn|variance}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("<dfn>variance</dfn>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_subsup_template() {
    let rendered = render_wikitext("Sample", "{{subsup|x|1|2}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("x<sub>1</sub><sup>2</sup>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_abs_template() {
    let rendered = render_wikitext("Sample", "{{abs|x}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("|x|"), "{rendered}");
}

#[test]
fn render_wikitext_formats_mono_template() {
    let rendered = render_wikitext("Sample", "{{mono|erfc()}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("<code>erfc()</code>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_pi_template() {
    let rendered = render_wikitext("Sample", "{{pi}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("π"), "{rendered}");
}

#[test]
fn render_wikitext_formats_springer_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Springer|title=Normal Distribution|id=p/n067460}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("\"Normal Distribution\"")
            && rendered.contains("Encyclopedia of Mathematics")
            && rendered.contains("Springer"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_skips_probability_fundamentals_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Probability fundamentals}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(!rendered.contains("Probability fundamentals"), "{rendered}");
}

#[test]
fn render_wikitext_skips_prob_distributions_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{ProbDistributions|continuous-infinite}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(!rendered.contains("ProbDistributions"), "{rendered}");
}

#[test]
fn render_wikitext_skips_divcol_templates() {
    let rendered = render_wikitext(
        "Sample",
        "{{divcol}}content{{divcol end}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(!rendered.contains("divcol"), "{rendered}");
}

#[test]
fn render_wikitext_formats_jstor_template() {
    let rendered = render_wikitext("Sample", "{{JSTOR|1400906}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("JSTOR 1400906"), "{rendered}");
}

#[test]
fn render_wikitext_formats_wspsm_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{wsPSM|Quetelet on the Science of Man|1|May 1872|first=Edward Burnett|last=Tylor|authorlink=Edward Burnett Tylor}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Tylor, Edward Burnett"), "{rendered}");
    assert!(rendered.contains("May 1872"), "{rendered}");
    assert!(
        rendered.contains("Quetelet on the Science of Man"),
        "{rendered}"
    );
    assert!(rendered.contains("Popular Science Monthly"), "{rendered}");
    assert!(rendered.contains("Vol. 1"), "{rendered}");
}

#[test]
fn render_wikitext_formats_em_template() {
    let rendered = render_wikitext("Sample", "{{em|some text}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("<em>some text</em>"), "{rendered}");
}

#[test]
fn render_wikitext_skips_stats_topic_toc_template() {
    let rendered = render_wikitext("Sample", "{{StatsTopicTOC}}", &InternalLinks::new(), "en");
    assert!(!rendered.contains("StatsTopicTOC"), "{rendered}");
}

#[test]
fn render_wikitext_skips_math_topics_toc_template() {
    let rendered = render_wikitext("Sample", "{{Math topics TOC}}", &InternalLinks::new(), "en");
    assert!(!rendered.contains("Math topics TOC"), "{rendered}");
}

#[test]
fn render_wikitext_skips_areas_of_mathematics_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Areas of mathematics |collapsed}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(!rendered.contains("Areas of mathematics"), "{rendered}");
}

#[test]
fn render_wikitext_skips_glossaries_of_science_and_engineering_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Glossaries of science and engineering}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        !rendered.contains("Glossaries of science and engineering"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_tmath_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{tmath|1=E = mc^2}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("E = mc^2"), "{rendered}");

    let rendered2 = render_wikitext("Sample", "{{tmath|\\sigma^2}}", &InternalLinks::new(), "en");
    assert!(rendered2.contains("\\sigma^2"), "{rendered2}");
}

#[test]
fn render_wikitext_formats_closed_open_template() {
    let rendered1 = render_wikitext("Sample", "{{closed-open|a|b}}", &InternalLinks::new(), "en");
    assert!(rendered1.contains("[a, b)"), "{rendered1}");

    let rendered2 = render_wikitext(
        "Sample",
        "{{closed-open|a, b}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered2.contains("[a, b)"), "{rendered2}");
}

#[test]
fn render_wikitext_formats_sqrt_template() {
    let rendered = render_wikitext("Sample", "{{sqrt|x}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("√x"), "{rendered}");
}

#[test]
fn render_wikitext_formats_section_link_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Section link|Page|Sec1|Sec2}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Page § Sec1 § Sec2"), "{rendered}");
}

#[test]
fn render_wikitext_formats_section_link_lowercase_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{section link|Page#Sec1}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Page § Sec1"), "{rendered}");
}

#[test]
fn render_wikitext_formats_mset_template() {
    let rendered = render_wikitext("Sample", "{{mset|1|2|3}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("{1, 2, 3}"), "{rendered}");
}

#[test]
fn render_wikitext_formats_hidden_begin_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{hidden begin|title=Proof}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("<strong>Proof</strong>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_hidden_end_template() {
    let rendered = render_wikitext(
        "Sample",
        "some content{{hidden end}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("some content"), "{rendered}");
}

#[test]
#[ignore]
fn scrape_map_templates() {
    let client = reqwest::blocking::Client::builder()
        .user_agent("wikipedia-to-epub scraper (contact: github.com/szabgab/wikipedia-to-epub.rs)")
        .build()
        .unwrap();

    let mut members = Vec::new();
    let mut cmcontinue: Option<String> = None;

    loop {
        let mut request = client.get("https://en.wikipedia.org/w/api.php").query(&[
            ("action", "query"),
            ("list", "categorymembers"),
            ("cmtitle", "Category:Labelled_map_templates"),
            ("cmlimit", "500"),
            ("format", "json"),
        ]);
        if let Some(ref cont) = cmcontinue {
            request = request.query(&[("cmcontinue", cont)]);
        }

        let resp: serde_json::Value = request.send().unwrap().json().unwrap();
        if let Some(arr) = resp
            .get("query")
            .and_then(|q| q.get("categorymembers"))
            .and_then(|cm| cm.as_array())
        {
            for item in arr {
                let title = item
                    .get("title")
                    .and_then(|t| t.as_str())
                    .unwrap()
                    .to_string();
                let ns = item.get("ns").and_then(|n| n.as_i64()).unwrap();
                if ns == 10 {
                    // Namespace 10 is Template
                    members.push(title);
                }
            }
        }

        if let Some(cont) = resp
            .get("continue")
            .and_then(|c| c.get("cmcontinue"))
            .and_then(|c| c.as_str())
        {
            cmcontinue = Some(cont.to_string());
        } else {
            break;
        }
    }

    println!("Found {} templates in category", members.len());

    let mut map_entries = Vec::new();

    // Fetch wikitext in chunks of 50
    for chunk in members.chunks(50) {
        let titles = chunk.join("|");
        let resp: serde_json::Value = client
            .get("https://en.wikipedia.org/w/api.php")
            .query(&[
                ("action", "query"),
                ("prop", "revisions"),
                ("rvprop", "content"),
                ("rvslots", "main"),
                ("titles", &titles),
                ("format", "json"),
            ])
            .send()
            .unwrap()
            .json()
            .unwrap();
        if let Some(pages) = resp
            .get("query")
            .and_then(|q| q.get("pages"))
            .and_then(|p| p.as_object())
        {
            for (_, page) in pages {
                let title = page
                    .get("title")
                    .and_then(|t| t.as_str())
                    .unwrap()
                    .to_string();
                if let Some(rev) = page
                    .get("revisions")
                    .and_then(|r| r.as_array())
                    .and_then(|revs| revs.first())
                {
                    let text = if let Some(slots) = rev.get("slots").and_then(|s| s.get("main")) {
                        slots.get("*").and_then(|t| t.as_str()).unwrap_or("")
                    } else {
                        rev.get("*").and_then(|t| t.as_str()).unwrap_or("")
                    };
                    if let Some(img) = extract_main_image(text) {
                        let clean_title = title.strip_prefix("Template:").unwrap_or(&title);
                        let clean_title = clean_title.replace('_', " ").trim().to_string();
                        map_entries.push((clean_title, img));
                    }
                }
            }
        }
    }

    map_entries.sort_by_key(|a| a.0.to_lowercase());

    let mut csv_content = String::new();
    for (template, img) in map_entries {
        csv_content.push_str(&format!("\"{}\",\"{}\"\n", template, img));
    }

    fs::write("src/maps.csv", csv_content).unwrap();
    println!("Successfully wrote src/maps.csv");
}

#[test]
fn render_wikitext_formats_cite_paper_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{cite paper |title=Paper Title |last=Doe |first=John |journal=Journal Name |date=2023}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("John Doe")
            && rendered.contains("\"Paper Title\"")
            && rendered.contains("Journal Name")
            && rendered.contains("2023"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_cite_court_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{cite court |litigants=Parker v. D.C. |vol=478 |reporter=F.3d |opinion=370 |pinpoint=401 |court=D.C. Cir. |date=2007}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("Parker v. D.C.")
            && rendered.contains("478 F.3d 370")
            && rendered.contains("401")
            && rendered.contains("D.C. Cir. 2007"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_cite_dictionary_com_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Cite Dictionary.com |spoon |access-date=2026-06-13}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("spoon")
            && rendered.contains("Dictionary.com Unabridged")
            && rendered.contains("Retrieved 2026-06-13"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_cite_speech_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{cite speech |title=Economic Isolationism |first=Mike |last=Eskew |event=Executive Speeches |location=Washington, D.C. |date=2004}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("Mike Eskew")
            && rendered.contains("Economic Isolationism")
            && rendered.contains("Speech")
            && rendered.contains("Executive Speeches")
            && rendered.contains("Washington, D.C.")
            && rendered.contains("2004"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_cite_ssrn_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{cite SSRN |ssrn=1900856 |title=Example Paper |last=Doe |first=John |date=2023}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("John Doe")
            && rendered.contains("Example Paper")
            && rendered.contains("SSRN")
            && rendered.contains("1900856"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_cite_tech_report_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{cite tech report |title=Technical Report |last=Smith |first=Jane |publisher=Company |date=2022}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("Jane Smith")
            && rendered.contains("Technical Report")
            && rendered.contains("2022"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_cite_citeseerx_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Cite CiteSeerX |citeseerx=10.1.1.239.1803 |title=Paper Title |last=Doe |first=John |date=2024}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("John Doe")
            && rendered.contains("Paper Title")
            && rendered.contains("CiteSeerX")
            && rendered.contains("10.1.1.239.1803"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_nobr_template() {
    let rendered = render_wikitext("Sample", "{{nobr|content}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("content"), "{rendered}");
}

#[test]
fn render_wikitext_skips_which_template() {
    let rendered = render_wikitext("Sample", "{{which?}}", &InternalLinks::new(), "en");
    assert!(!rendered.contains("which?"), "{rendered}");
}

#[test]
fn render_wikitext_skips_redirect_distinguish_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Redirect-distinguish|A|B}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(!rendered.contains("Redirect-distinguish"), "{rendered}");
}

#[test]
fn render_wikitext_formats_collapse_top_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{Collapse top|title=My Title}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("<strong>My Title</strong>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_collapse_bottom_template() {
    let rendered = render_wikitext("Sample", "{{Collapse bottom}}", &InternalLinks::new(), "en");
    assert!(!rendered.contains("Collapse bottom"), "{rendered}");
}

#[test]
fn render_wikitext_formats_var_template() {
    let rendered = render_wikitext("Sample", "{{var|x}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("<var>x</var>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_gaps_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{gaps|1|2|3|e=-4|u=kg|lhs=y}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("y = 1 2 3")
            && rendered.contains("×10<sup>-4</sup>")
            && rendered.contains("kg"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_skips_example_needed_template() {
    let rendered = render_wikitext("Sample", "{{example needed}}", &InternalLinks::new(), "en");
    assert!(!rendered.contains("example needed"), "{rendered}");
}

#[test]
fn render_wikitext_formats_right_template() {
    let rendered_with_param = render_wikitext(
        "Sample",
        "{{right|My Content}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered_with_param.contains("My Content"),
        "{}",
        rendered_with_param
    );

    let rendered_without_param = render_templates("{{right}}");
    assert_eq!(rendered_without_param, "style=\"text-align:right\"|");
}

#[test]
fn render_wikitext_formats_cite_peakbagger_template() {
    let rendered_named =
        render_templates("{{cite peakbagger|pid=2829|name=Mount Whitney|access-date=2008-10-31}}");
    assert_eq!(
        rendered_named,
        "[[official-url:http://www.peakbagger.com/peak.aspx?pid=2829|\"Mount Whitney\"]]. ''Peakbagger.com''. Retrieved 2008-10-31"
    );

    let rendered_positional = render_templates("{{cite peakbagger|2829|Mount Whitney|2008-10-31}}");
    assert_eq!(
        rendered_positional,
        "[[official-url:http://www.peakbagger.com/peak.aspx?pid=2829|\"Mount Whitney\"]]. ''Peakbagger.com''. Retrieved 2008-10-31"
    );

    let rendered_list = render_templates("{{cite peakbagger|lid=12003|name=State High Points}}");
    assert_eq!(
        rendered_list,
        "[[official-url:http://www.peakbagger.com/list.aspx?lid=12003|\"State High Points\"]]. ''Peakbagger.com''"
    );
}

#[test]
fn render_wikitext_formats_wikibooks_inline_template() {
    let rendered = render_templates("{{Wikibooks inline|Work|Custom Label}}");
    assert_eq!(rendered, "[[b:Work|Custom Label]] at Wikibooks");

    let rendered_links = render_templates("{{Wikibooks inline|links=[[b:Foo|]] and [[b:Bar|]]}}");
    assert_eq!(rendered_links, "[[b:Foo|]] and [[b:Bar|]] at Wikibooks");
}

#[test]
fn render_wikitext_formats_refh_template() {
    let rendered = render_templates("{{refh}}");
    assert_eq!(
        rendered,
        "__WIKIPEDIA_TO_EPUB_ABBR_START__References__WIKIPEDIA_TO_EPUB_ABBR_VALUE__Refs.__WIKIPEDIA_TO_EPUB_ABBR_END__"
    );

    let rendered_single = render_templates("{{refh|multi=no}}");
    assert_eq!(
        rendered_single,
        "__WIKIPEDIA_TO_EPUB_ABBR_START__Reference__WIKIPEDIA_TO_EPUB_ABBR_VALUE__Ref.__WIKIPEDIA_TO_EPUB_ABBR_END__"
    );
}

#[test]
fn render_wikitext_skips_by_whom_template() {
    let rendered = render_wikitext("Sample", "{{By whom}}", &InternalLinks::new(), "en");
    assert!(!rendered.contains("By whom"), "{}", rendered);
}

#[test]
fn render_wikitext_formats_earthquake_magnitude_template() {
    let rendered_w = render_templates("{{M|w|7.2}}");
    assert_eq!(rendered_w, "M<sub>w</sub>\u{2009}7.2");

    let rendered_b = render_templates("{{M|B|6.5|src=USGS}}");
    assert_eq!(rendered_b, "mB<sup>(USGS)</sup>\u{2009}6.5");

    let rendered_link = render_templates("{{M|w|7.2|link=y}}");
    assert_eq!(
        rendered_link,
        "[[Seismic magnitude scales#Mw|M<sub>w</sub>]]\u{2009}7.2"
    );
}

#[test]
fn render_wikitext_formats_cite_video_template() {
    let rendered = render_templates(
        "{{cite video|title=My Video|url=http://example.com/video|publisher=Publisher|date=2020-01-01}}",
    );
    assert_eq!(
        rendered,
        "[http://example.com/video \"My Video\"]. Publisher. 2020-01-01"
    );
}

#[test]
fn render_wikitext_formats_cite_tweet_template() {
    let rendered = render_templates(
        "{{cite tweet|user=jack|number=20|tweet=just setting up my twttr|date=2006-03-21}}",
    );
    assert_eq!(
        rendered,
        "@jack. [https://twitter.com/jack/status/20 \"just setting up my twttr\"]. (Tweet). 2006-03-21. via Twitter"
    );
}

#[test]
fn render_wikitext_formats_cite_constitution_template() {
    let rendered = render_templates(
        "{{cite constitution|country=United States|article=I|section=8|date=1787}}",
    );
    assert_eq!(
        rendered,
        "Constitution of United States, Art. I, Sec. 8, 1787"
    );
}

#[test]
fn render_wikitext_formats_cite_biorxiv_template() {
    let rendered =
        render_templates("{{cite bioRxiv|title=A paper|biorxiv=10.1101/123456|date=2020}}");
    assert_eq!(
        rendered,
        "\"A paper\". 2020. bioRxiv:[https://doi.org/10.1101/10.1101/123456 10.1101/123456]"
    );
}

#[test]
fn render_wikitext_formats_harvard_citation_text_template() {
    let rendered = render_templates("{{Harvard citation text|Smith|2020|p=15}}");
    assert_eq!(rendered, "Smith (2020, p. 15)");
}

#[test]
fn render_wikitext_formats_cite_mw_template() {
    let rendered = render_templates("{{cite MW|entry=dictionary}}");
    assert_eq!(
        rendered,
        "[[official-url:https://www.merriam-webster.com/dictionary/dictionary|\"dictionary\"]]. ''Merriam-Webster.com Dictionary''. Merriam-Webster"
    );
}

#[test]
fn render_wikitext_formats_term_and_defn_templates() {
    let rendered_term = render_templates("{{term|1=glossary term}}");
    assert_eq!(rendered_term, "'''glossary term'''");

    let rendered_defn = render_templates("{{defn|1=the definition of glossary term}}");
    assert_eq!(rendered_defn, "the definition of glossary term");
}

#[test]
fn render_wikitext_formats_cquote_template() {
    let rendered = render_templates("{{cquote|A fine quote|An Author|Some Book}}");
    assert!(rendered.contains("__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_START__"));
    assert!(rendered.contains("A fine quote"));
    assert!(rendered.contains("__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_SOURCE__An Author, Some Book"));
    assert!(rendered.contains("__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_END__"));
}

#[test]
fn render_wikitext_formats_london_gazette_template() {
    let rendered = render_templates("{{London Gazette|12345|page=12|date=1914-11-20}}");
    assert_eq!(
        rendered,
        "[https://www.thegazette.co.uk/London/issue/12345/page/12 \"No. 12345\"]. ''The London Gazette''. 1914-11-20. p. 12"
    );
}

#[test]
fn render_wikitext_formats_us_dollar_template() {
    let rendered = render_templates("{{US$|123.45}}");
    assert_eq!(rendered, "US$123.45");
}

#[test]
fn render_wikitext_formats_frac2_template() {
    let rendered = render_wikitext("Sample", "{{frac2|1|3}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("<sup>1</sup>⁄<sub>3</sub>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_vanchor_template() {
    let rendered_text_param = render_wikitext(
        "Sample",
        "{{vanchor|Mercury|text=[[Mercury]]}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered_text_param.contains("Mercury"),
        "{rendered_text_param}"
    );

    let rendered_pos =
        render_wikitext("Sample", "{{vanchor|Mercury}}", &InternalLinks::new(), "en");
    assert!(rendered_pos.contains("Mercury"), "{rendered_pos}");

    let rendered_named_one = render_wikitext(
        "Sample",
        "{{vanchor|1=Mercury}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered_named_one.contains("Mercury"),
        "{rendered_named_one}"
    );
}

#[test]
fn render_wikitext_formats_block_indent_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{block indent|1=Hello world}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("<blockquote>"), "{rendered}");
    assert!(rendered.contains("Hello world"), "{rendered}");
    assert!(rendered.contains("</blockquote>"), "{rendered}");
}

#[test]
fn render_wikitext_formats_dfni_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{dfni|1=technical term}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered.contains("<dfn><em>technical term</em></dfn>"),
        "{rendered}"
    );
}

#[test]
fn render_wikitext_formats_radic_template() {
    let rendered_simple = render_wikitext("Sample", "{{radic|9}}", &InternalLinks::new(), "en");
    assert!(rendered_simple.contains("√9"), "{rendered_simple}");

    let rendered_deg = render_wikitext("Sample", "{{radic|8|3}}", &InternalLinks::new(), "en");
    assert!(rendered_deg.contains("<sup>3</sup>√8"), "{rendered_deg}");
}

#[test]
fn render_wikitext_formats_diagonal_split_header_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{diagonal split header|Rows|Cols}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Rows \\ Cols"), "{rendered}");
}

#[test]
fn render_wikitext_formats_pipe_template() {
    let rendered = render_templates("{{pipe}}");
    assert_eq!(rendered, "|");
}

#[test]
fn render_wikitext_formats_legend_line_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{legend-line|black solid 2px|Label text}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("Label text"), "{rendered}");

    let rendered_named = render_wikitext(
        "Sample",
        "{{legend-line|black solid 2px|2=Label text}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered_named.contains("Label text"), "{rendered_named}");
}

#[test]
fn render_wikitext_formats_prime_template() {
    let rendered_empty = render_wikitext("Sample", "{{prime}}", &InternalLinks::new(), "en");
    assert!(rendered_empty.contains("′"), "{rendered_empty}");

    let rendered_text = render_wikitext("Sample", "{{prime|x}}", &InternalLinks::new(), "en");
    assert!(rendered_text.contains("x′"), "{rendered_text}");
}

#[test]
fn render_wikitext_formats_isup_template() {
    let rendered_one = render_wikitext("Sample", "{{isup|st}}", &InternalLinks::new(), "en");
    assert!(rendered_one.contains("<sup>st</sup>"), "{rendered_one}");

    let rendered_two = render_wikitext("Sample", "{{isup|2px|nd}}", &InternalLinks::new(), "en");
    assert!(rendered_two.contains("<sup>nd</sup>"), "{rendered_two}");

    let rendered_named = render_wikitext("Sample", "{{isup|2=nd}}", &InternalLinks::new(), "en");
    assert!(rendered_named.contains("<sup>nd</sup>"), "{rendered_named}");

    let rendered_named_one =
        render_wikitext("Sample", "{{isup|1=st}}", &InternalLinks::new(), "en");
    assert!(
        rendered_named_one.contains("<sup>st</sup>"),
        "{rendered_named_one}"
    );
}

#[test]
fn render_wikitext_formats_cjkv_template() {
    let rendered = render_wikitext(
        "Sample",
        "{{CJKV|t=繁|s=饰|p=pinyin|l=literal}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered.contains("traditional Chinese: 繁"), "{rendered}");
    assert!(rendered.contains("simplified Chinese: 饰"), "{rendered}");
    assert!(rendered.contains("pinyin: <em>pinyin</em>"), "{rendered}");
    assert!(rendered.contains("literal"), "{rendered}");
}

#[test]
fn render_wikitext_formats_udl_template() {
    let rendered_wrap = render_wikitext(
        "Sample",
        "{{udl|wrap=some content}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(rendered_wrap.contains("some content"), "{rendered_wrap}");

    let rendered_pos = render_wikitext(
        "Sample",
        "{{udl|some other content}}",
        &InternalLinks::new(),
        "en",
    );
    assert!(
        rendered_pos.contains("some other content"),
        "{rendered_pos}"
    );
}

#[test]
fn render_wikitext_formats_silent_templates_new() {
    assert_eq!(render_templates("{{sisterlinks}}"), "");
    assert_eq!(render_templates("{{sister links}}"), "");
    assert_eq!(render_templates("{{Wikinews}}"), "");
    assert_eq!(render_templates("{{wikiquote}}"), "");
    assert_eq!(render_templates("{{fv}}"), "");
    assert_eq!(render_templates("{{clear right}}"), "");
    assert_eq!(render_templates("{{clr}}"), "");
    assert_eq!(render_templates("{{empty section}}"), "");
    assert_eq!(render_templates("{{family name hatnote}}"), "");
    assert_eq!(render_templates("{{Bare URL inline}}"), "");
}
