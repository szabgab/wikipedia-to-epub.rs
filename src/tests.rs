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
    let rendered = render_wikitext(
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
        let rendered = render_wikitext("Sample", template, &InternalLinks::new(), "en");
        assert!(
            rendered.contains(expected),
            "For template {template:?} rendered unexpectedly:\n{rendered}"
        );
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("For|"));
    }
}

#[test]
fn render_wikitext_formats_for_timeline_templates() {
    let rendered = render_wikitext(
        "Sample",
        "{{For timeline|Timeline of Sample}}",
        &InternalLinks::new(),
        "en",
    );

    assert!(rendered.contains("For a timeline, see: <a href=\"https://en.wikipedia.org/wiki/Timeline_of_Sample\">Timeline of Sample</a>"));
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
fn render_wikitext_silently_skips_metadata_templates() {
    let rendered = render_wikitext(
        "Sample",
        r#"{{Short description|Sample page}}
{{About|the sample|other uses|Sample (disambiguation)}}
{{Distinguish|Example}}
{{ISBN?}}
{{Pp-move}}
{{Protection padlock|small=yes}}
{{Redirect|Sample}}
{{redirect-multi|3|Pusan|Fusan|Busan Metropolitan City|other uses|Pu San (disambiguation)}}
{{pp-semi-indef}}
{{Sfn|Author|2024|p=1}}
{{sfnm|1a1=Author|1y=2024|1p=1}}
{{efn|Footnote text}}
{{refn|Reference note text}}
{{Reflist|30em}}
{{notelist}}
{{Refbegin|30em}}
{{refend}}
{{flagicon|US}}
{{unreferenced section|date=November 2022}}
{{Excessive citations inline|date=November 2022}}
{{More citations needed|date=June 2022}}
{{Refimprove|date=December 2025}}
{{FACT|date=December 2025}}
{{citation needed|date=May 2023}}
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
{{Portal bar|North Korea|South Korea|Asia|History|Linguistics|Monarchy|Biography}}
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
Visible text."#,
        &InternalLinks::new(),
        "en",
    );

    assert!(rendered.contains("<h1>Sample</h1>"));
    assert!(rendered.contains("<p>Visible text.</p>"));
    assert!(!rendered.contains("Short description"));
    assert!(!rendered.contains("About"));
    assert!(!rendered.contains("Distinguish"));
    assert!(!rendered.contains("sfnm"));
    assert!(!rendered.contains("Reflist"));
    assert!(!rendered.contains("notelist"));
    assert!(!rendered.contains("Refbegin"));
    assert!(!rendered.contains("refend"));
    assert!(!rendered.contains("flagicon"));
    assert!(!rendered.contains("unreferenced section"));
    assert!(!rendered.contains("Excessive citations inline"));
    assert!(!rendered.contains("More citations needed"));
    assert!(!rendered.contains("Refimprove"));
    assert!(!rendered.contains("FACT"));
    assert!(!rendered.contains("citation needed"));
    assert!(!rendered.contains("Modern"));
    assert!(!rendered.contains("huh"));
    assert!(!rendered.contains("when"));
    assert!(!rendered.contains("more cn section"));
    assert!(!rendered.contains("1951 to present"));
    assert!(!rendered.contains("cbignore"));
    assert!(!rendered.contains("medic"));
    assert!(!rendered.contains("prose"));
    assert!(!rendered.contains("Unreliable source?"));
    assert!(!rendered.contains("Better source needed"));
    assert!(!rendered.contains("Dead link"));
    assert!(!rendered.contains("Page needed"));
    assert!(!rendered.contains("New archival link needed"));
    assert!(!rendered.contains("clear"));
    assert!(!rendered.contains("div col"));
    assert!(!rendered.contains("Portal bar"));
    assert!(!rendered.contains("DEFAULTSORT"));
    assert!(!rendered.contains("Sample, Page"));
    assert!(!rendered.contains("Self-published"));
    assert!(!rendered.contains("self-published inline"));
    assert!(!rendered.contains("Use British English"));
    assert!(!rendered.contains("Use dmy dates"));
    assert!(!rendered.contains("Pp-move"));
    assert!(!rendered.contains("Protection padlock"));
    assert!(!rendered.contains("Redirect"));
    assert!(!rendered.contains("redirect-multi"));
    assert!(!rendered.contains("Pu San"));
    assert!(!rendered.contains("pp-semi-indef"));
    assert!(!rendered.contains("Author"));
    assert!(!rendered.contains("Footnote text"));
    assert!(!rendered.contains("Reference note text"));
    assert!(!rendered.contains("Infobox"));
    assert!(!rendered.contains("History of Korea"));
    assert!(!rendered.contains("Korea topics"));
    assert!(!rendered.contains("East Asian topics"));
    assert!(!rendered.contains("History of Asia"));
    assert!(!rendered.contains("Seoul"));
    assert!(!rendered.contains("Busan weatherbox"));
    assert!(!rendered.contains("Navboxes"));
    assert!(!rendered.contains("Sister project links"));
    assert!(!rendered.contains("Authority control"));
    assert!(!rendered.contains("Portal"));
    assert!(!rendered.contains("Commons category"));
    assert!(!rendered.contains("Commons and category"));
    assert!(!rendered.contains("columns-list"));
    assert!(!rendered.contains("Hidden first column item"));
    assert!(!rendered.contains("Hidden second column item"));
    assert!(!rendered.contains("location map+"));
    assert!(!rendered.contains("location map~"));
    assert!(!rendered.contains("Sample marker"));
    assert!(!rendered.contains("Wikisource-inline"));
    assert!(!rendered.contains("Cite EB1911"));
    assert!(!rendered.contains("Wide image"));
    assert!(!rendered.contains("Sample panorama"));
    assert!(!rendered.contains("Pie chart"));
    assert!(!rendered.contains("Hidden slice"));
    assert!(!rendered.contains("ahnentafel"));
    assert!(!rendered.contains("Hidden ancestor"));
    assert!(!rendered.contains("S-start"));
    assert!(!rendered.contains("House of Yi"));
    assert!(!rendered.contains("s-reg"));
    assert!(!rendered.contains("Taejong"));
    assert!(!rendered.contains("King of Joseon"));
    assert!(!rendered.contains("Munjong"));
    assert!(!rendered.contains("s-end"));
    assert!(!rendered.contains("Succession box"));
    assert!(!rendered.contains("Capital of Baekje"));
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
        "Traditionally, ''seoul'' ({{Korean|hangul=서울|labels=no}}) meant capital. Earlier {{Korean|labels=no|위례성|慰禮城}} was nearby. He was called {{Korean/auto|hangul=^해동_^요순|hanja=海東堯舜|mr=yes|labels=no}}.",
        &InternalLinks::new(),
        "en",
    );

    assert!(rendered.contains(
            r#"<p>Traditionally, <em>seoul</em> (<span title="Korean-language text"><span lang="ko-Hang">서울</span></span>) meant capital. Earlier <span title="Korean-language text"><span lang="ko-Hang">위례성</span> / <span lang="ko-Hani">慰禮城</span></span> was nearby. He was called <span title="Korean-language text"><span lang="ko-Hang">해동요순</span> / <span lang="ko-Hani">海東堯舜</span></span>.</p>"#
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
    assert_eq!(args.log_level, Level::INFO);
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
