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
    let (rendered, counts) = render_wikitext_with_template_counts(
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
        "en",
        None,
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

    assert_eq!(
        counts,
        TemplateSkipCounts {
            recognized: 1,
            unknown: 0
        }
    );
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
fn render_wikitext_formats_issn_template() {
    let rendered = render_wikitext("Sample", "{{ISSN|0268-4160}}", &InternalLinks::new(), "en");
    assert!(rendered.contains("ISSN 0268-4160"), "{rendered}");
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
{{Infobox settlement|name=Sample}}
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
            recognized: 93,
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
            r#"<p>See <a href="https://en.wikipedia.org/wiki/#Letter_counts">Letter counts</a><span class="external-link">↗</span>.</p>"#,
        ),
        (
            "{{crossreference|(see {{slink|Hangul orthography|Buncheol vs. yeoncheol debate}})}}",
            r#"<p>(see <a href="https://en.wikipedia.org/wiki/Hangul_orthography#Buncheol_vs._yeoncheol_debate">Buncheol vs. yeoncheol debate</a><span class="external-link">↗</span>)</p>"#,
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
            "<p>A claim. p. 5 km</p>",
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
        ("{{convert|1100|km|abbr=on}}", "1100 km"),
        ("{{cvt|314|km|0}}", "314 km"),
        ("{{Cvt|49.5|km}}", "49.5 km"),
        ("{{convert|30|°C|°F}}", "30 °C"),
        ("{{Convert|24|ug/m3||sp=us}}", "24 ug/m³"),
        ("{{convert|&minus;3|°C|1|disp=or}}", "−3 °C"),
        ("{{convert|10|to|47|km2|disp=or|abbr=on}}", "10 to 47 km²"),
        ("{{convert|15|km|0|abbr=on}}", "15 km"),
        ("{{convert|2.1|and|−5.5|C|F|1}}", "2.1 °C and −5.5 °C"),
        ("{{convert|250|km|0|abbr=on}}", "250 km"),
        ("{{convert|268|km2|mi2|sp=us|abbr=on}}", "268 km²"),
        ("{{convert|30.0|and|22.9|C|F|0}}", "30.0 °C and 22.9 °C"),
        ("{{convert|300|km/h|0|abbr=on}}", "300 km/h"),
        ("{{convert|40|C|F|1}}", "40 °C"),
        ("{{convert|4|km|mile|sp=us|abbr=on}}", "4 km"),
        ("{{convert|605.25|km2|sqmi|abbr=unit}}", "605.25 km²"),
        ("{{convert|613|km2|mi2|sp=us|abbr=on}}", "613 km²"),
        ("{{convert|940|km|abbr=on}}", "940 km"),
        ("{{convert|−10|C}}", "−10 °C"),
        ("{{convert|−15|C}}", "−15 °C"),
        ("{{convert|−20|C}}", "−20 °C"),
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
fn book_config_defaults_images_to_false() {
    let config = serde_yaml::from_str::<BookConfig>(
        r#"metadata:
  title: Sample
  author: Wikipedia contributors
  language: en
  edition: First edition
output-file: sample.epub
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
        r#"metadata:
  title: Sample
  author: Wikipedia contributors
  language: en
  edition: First edition
output-file: sample.epub
images: true
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

fn test_cache_path(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
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
        r#"metadata:
  title: Sample
  author: Wikipedia contributors
  language: en
  edition: First edition
output-file: sample.epub
caching: none
depth: 0
articles:
  - Sample
"#,
    )
    .expect("config parses");
    assert_eq!(config_none.caching, CachingMode::None);

    let config_local = serde_yaml::from_str::<BookConfig>(
        r#"metadata:
  title: Sample
  author: Wikipedia contributors
  language: en
  edition: First edition
output-file: sample.epub
caching: local
depth: 0
articles:
  - Sample
"#,
    )
    .expect("config parses");
    assert_eq!(config_local.caching, CachingMode::Local);

    let config_central = serde_yaml::from_str::<BookConfig>(
        r#"metadata:
  title: Sample
  author: Wikipedia contributors
  language: en
  edition: First edition
output-file: sample.epub
caching: central
depth: 0
articles:
  - Sample
"#,
    )
    .expect("config parses");
    assert_eq!(config_central.caching, CachingMode::Central);
}
