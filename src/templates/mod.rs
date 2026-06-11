pub mod citation;
pub mod convert;
pub mod formatting;
pub mod lang;

use std::collections::HashMap;

use cached::macros::cached;

use crate::types::{DispatchTable, EmptyDispatchTable, EmptyHandler, TemplateHandler};
pub(crate) use formatting::{
    PersonRole, citation_people, format_number_with_commas, join_plain_items,
    template_named_params, template_param, template_param_owned, template_positional_params,
};
pub(crate) use lang::format_interlanguage_link;

use citation::{
    render_citation_needed_span_template, render_citation_template,
    render_cite_american_heritage_dictionary_template, render_cite_apod_template,
    render_cite_arxiv_template, render_cite_av_media_template, render_cite_book_template,
    render_cite_cia_world_factbook_template, render_cite_dictionary_template,
    render_cite_eb1911_template, render_cite_eccp_template, render_cite_gvp_template,
    render_cite_journal_template, render_cite_letter_template,
    render_cite_merriam_webster_template, render_cite_nsrw_template, render_cite_oed_template,
    render_cite_press_release_template, render_cite_q_template, render_cite_report_template,
    render_cite_web_template, render_cite_wikisource_template, render_harvc_template,
    render_harvnb_template, render_harvp_template, render_harvtxt_template,
};
use convert::{
    render_convert_template, render_fx_convert_template, render_inflation_template,
    render_inflation_year_template, render_jpy_template, render_percentage_template,
    render_un_population_template, render_val_template,
};
use formatting::{
    render_abbr_template, render_age_template, render_annotated_link_template,
    render_arrow_template, render_article_link_template, render_as_of_template,
    render_asin_template, render_assassinated_template, render_awol_template, render_ayd_template,
    render_birth_date_and_age_template, render_blockquote_template, render_break_template,
    render_britannica_template, render_chem_template, render_chem2_template, render_circa_template,
    render_climate_chart_template, render_collapsible_list_template, render_color_box_template,
    render_color_template, render_columns_list_template, render_coord_template,
    render_died_in_template, render_died_of_wounds_template, render_doi_template,
    render_dp_template, render_dts_template, render_e_template,
    render_easy_css_image_crop_template, render_excerpt_template, render_executed_template,
    render_five_nonbreaking_spaces_template, render_flagdeco_template, render_floruit_template,
    render_for_multi_template, render_for_template, render_for_timeline_template,
    render_formatnum_template, render_frac_template, render_fs_interlinear_template,
    render_further_template, render_gburl_template, render_generic_ship_template,
    render_google_books_template, render_historical_populations_template, render_hlist_template,
    render_hyphen_template, render_infobox_country_template, render_infobox_generic_template,
    render_infobox_military_conflict_template, render_infobox_mountain_template,
    render_infobox_planet_template, render_infobox_settlement_template,
    render_interlanguage_link_template, render_internet_archive_short_film_template,
    render_isbn_template, render_issn_template, render_jaanus_template, render_jct_template,
    render_jpn_template, render_kia_template, render_kia2_template, render_lagrange_template,
    render_largest_cities_template, render_legend_template, render_lnl_template,
    render_main_list_template, render_main_template, render_mia_template, render_mpl_dash_template,
    render_mpl_template, render_multiple_images_template, render_native_name_list_template,
    render_natural_causes_template, render_ndldc_template, render_note_template,
    render_numero_template, render_oclc_template, render_official_website_template,
    render_open_access_template, render_openstreetmap_relation_template,
    render_openstreetmap_way_template, render_passthrough_template, render_pkia_template,
    render_plainlist_template, render_plus_minus_template, render_poem_quote_template,
    render_pow_template, render_pprime_template, render_proto_template, render_ra_template,
    render_reference_page_template, render_reign_template, render_republic_of_korea_ship_template,
    render_ric_template, render_route_box_template, render_section_link_template,
    render_see_also_template, render_ship_class_template, render_ship_template,
    render_sic_template, render_smaller_template, render_soft_hyphen_template,
    render_solar_radius_template, render_spaces_template, render_station_template,
    render_stn_template, render_su_template, render_sub_template, render_suicide_template,
    render_sup_template, render_surrendered_template, render_tooltip_template,
    render_track_gauge_template, render_turncoat_template, render_unbulleted_list_template,
    render_url_template, render_usurped_template, render_verse_translation_template,
    render_verse_transliteration_translation_template, render_visible_anchor_template,
    render_webarchive_template, render_wia_template, render_wikibooks_template,
    render_wikisource_template, render_wikivoyage_template, render_wiktionary_template,
    render_worldhistory_template,
};
use lang::{
    render_angbr_ipa_template, render_angbr_template, render_chinese_lang_template,
    render_english_ipa_template, render_gloss_template, render_in_lang_template,
    render_ipa_link_template, render_ipa_template, render_ja_platform_template,
    render_ja_rail_color_template, render_ja_rail_linem_template, render_japanese_template,
    render_korean_template, render_korean_transliteration_template, render_lang_template,
    render_langnf_template, render_langx_template, render_linktext_template,
    render_literal_template, render_na_template, render_native_name_template,
    render_nihongo_foot_template, render_nihongo_krt_template, render_nihongo2_template,
    render_nihongo3_template, render_respell_template, render_script_template,
    render_translation_template, render_transliteration_like_template,
    render_transliteration_template, render_unichar_template,
};

use crate::increment_recognized_skipped_template_count;
use crate::increment_unknown_skipped_template_count;
use tracing::debug;

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

pub(crate) fn render_template(content: &str) -> String {
    let (template, params) = split_template_name(content);
    let template_normalized = template.trim().replace('_', " ");
    let template = template_normalized.as_str();

    let fixed = get_fixed();
    let lower = template.to_lowercase();
    if fixed.contains_key(&lower.as_str()) {
        return fixed.get(lower.as_str()).unwrap().to_string();
    }

    let lookup: DispatchTable = HashMap::from([
        ("korean", render_korean_template as TemplateHandler),
        ("korean/auto", render_korean_template as TemplateHandler),
        ("ko", render_korean_template as TemplateHandler),
        ("nihongo4", render_japanese_template as TemplateHandler),
        ("nihongo", render_japanese_template as TemplateHandler),
        (
            "nihongo foot",
            render_nihongo_foot_template as TemplateHandler,
        ),
        ("nowrap", render_passthrough_template as TemplateHandler),
        (
            "citation needed span",
            render_citation_needed_span_template as TemplateHandler,
        ),
        ("quote box", render_blockquote_template as TemplateHandler),
        ("quote", render_blockquote_template as TemplateHandler),
        ("poem quote", render_poem_quote_template as TemplateHandler),
        ("poemquote", render_poem_quote_template as TemplateHandler),
        (
            "verse translation",
            render_verse_translation_template as TemplateHandler,
        ),
        (
            "verse transliteration-translation",
            render_verse_transliteration_translation_template as TemplateHandler,
        ),
        ("center", render_passthrough_template as TemplateHandler),
        ("smaller", render_smaller_template as TemplateHandler),
        ("small", render_smaller_template as TemplateHandler),
        ("sic", render_sic_template as TemplateHandler),
        ("circa", render_circa_template as TemplateHandler),
        ("c.", render_circa_template as TemplateHandler),
        ("cx", render_circa_template as TemplateHandler),
        ("lang", render_lang_template as TemplateHandler),
        ("langx", render_langx_template as TemplateHandler),
        ("in lang", render_in_lang_template as TemplateHandler),
        ("linktext", render_linktext_template as TemplateHandler),
        ("lang-zh", render_chinese_lang_template as TemplateHandler),
        ("zh", render_chinese_lang_template as TemplateHandler),
        ("zhi", render_chinese_lang_template as TemplateHandler),
        (
            "transliteration",
            render_transliteration_template as TemplateHandler,
        ),
        (
            "translit",
            render_transliteration_template as TemplateHandler,
        ),
        (
            "tlit",
            render_transliteration_like_template as TemplateHandler,
        ),
        (
            "ko-translit",
            render_korean_transliteration_template as TemplateHandler,
        ),
        ("lit", render_literal_template as TemplateHandler),
        (
            "literal translation",
            render_literal_template as TemplateHandler,
        ),
        ("literal", render_literal_template as TemplateHandler),
        (
            "translation",
            render_translation_template as TemplateHandler,
        ),
        (
            "language with name/for",
            render_langnf_template as TemplateHandler,
        ),
        ("langnf", render_langnf_template as TemplateHandler),
        ("isbn", render_isbn_template as TemplateHandler),
        ("asin", render_asin_template as TemplateHandler),
        ("script", render_script_template as TemplateHandler),
        ("oclc", render_oclc_template as TemplateHandler),
        ("ipa", render_ipa_template as TemplateHandler),
        ("ipac-en", render_english_ipa_template as TemplateHandler),
        ("respell", render_respell_template as TemplateHandler),
        ("abbr", render_abbr_template as TemplateHandler),
        (
            "assassinated",
            render_assassinated_template as TemplateHandler,
        ),
        ("executed", render_executed_template as TemplateHandler),
        ("kia", render_kia_template as TemplateHandler),
        ("kia2", render_kia2_template as TemplateHandler),
        (
            "natural causes",
            render_natural_causes_template as TemplateHandler,
        ),
        ("suicide", render_suicide_template as TemplateHandler),
        (
            "surrendered",
            render_surrendered_template as TemplateHandler,
        ),
        ("turncoat", render_turncoat_template as TemplateHandler),
        ("frac", render_frac_template as TemplateHandler),
        ("fraction", render_frac_template as TemplateHandler),
        ("floruit", render_floruit_template as TemplateHandler),
        ("coord", render_coord_template as TemplateHandler),
        ("rp", render_reference_page_template as TemplateHandler),
        (
            "reference page",
            render_reference_page_template as TemplateHandler,
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
        ("as of", render_as_of_template as TemplateHandler),
        ("died-in", render_died_in_template as TemplateHandler),
        ("blockquote", render_blockquote_template as TemplateHandler),
        ("percentage", render_percentage_template as TemplateHandler),
        (
            "un population",
            render_un_population_template as TemplateHandler,
        ),
        ("convert", render_convert_template as TemplateHandler),
        ("cvt", render_convert_template as TemplateHandler),
        ("for", render_for_template as TemplateHandler),
        (
            "for timeline",
            render_for_timeline_template as TemplateHandler,
        ),
        (
            "crossreference",
            render_passthrough_template as TemplateHandler,
        ),
        ("slink", render_section_link_template as TemplateHandler),
        ("legend", render_legend_template as TemplateHandler),
        ("legend0", render_legend_template as TemplateHandler),
        ("numero", render_numero_template as TemplateHandler),
        ("anl", render_article_link_template as TemplateHandler),
        ("excerpt", render_excerpt_template as TemplateHandler),
        ("main", render_main_template as TemplateHandler),
        ("main article", render_main_template as TemplateHandler),
        ("main list", render_main_list_template as TemplateHandler),
        ("see also", render_see_also_template as TemplateHandler),
        ("also", render_see_also_template as TemplateHandler),
        ("further", render_further_template as TemplateHandler),
        ("wiktionary", render_wiktionary_template as TemplateHandler),
        ("wikivoyage", render_wikivoyage_template as TemplateHandler),
        (
            "wikivoyage-inline",
            render_wikivoyage_template as TemplateHandler,
        ),
        (
            "wikivoyage inline",
            render_wikivoyage_template as TemplateHandler,
        ),
        ("wikisource", render_wikisource_template as TemplateHandler),
        ("wikibooks", render_wikibooks_template as TemplateHandler),
        ("britannica", render_britannica_template as TemplateHandler),
        (
            "official website",
            render_official_website_template as TemplateHandler,
        ),
        (
            "official",
            render_official_website_template as TemplateHandler,
        ),
        ("url", render_url_template as TemplateHandler),
        (
            "osmrelation-inline",
            render_openstreetmap_relation_template as TemplateHandler,
        ),
        (
            "osmway",
            render_openstreetmap_way_template as TemplateHandler,
        ),
        ("webarchive", render_webarchive_template as TemplateHandler),
        (
            "largest cities",
            render_largest_cities_template as TemplateHandler,
        ),
        (
            "historical populations",
            render_historical_populations_template as TemplateHandler,
        ),
        (
            "climate chart",
            render_climate_chart_template as TemplateHandler,
        ),
        ("sclass", render_ship_class_template as TemplateHandler),
        ("nobold", render_passthrough_template as TemplateHandler),
        ("arrow", render_arrow_template as TemplateHandler),
        (
            "roks",
            render_republic_of_korea_ship_template as TemplateHandler,
        ),
        ("ill", render_interlanguage_link_template as TemplateHandler),
        (
            "illm",
            render_interlanguage_link_template as TemplateHandler,
        ),
        (
            "interlanguage link",
            render_interlanguage_link_template as TemplateHandler,
        ),
        (
            "interlanguage link multi",
            render_interlanguage_link_template as TemplateHandler,
        ),
        ("reign", render_reign_template as TemplateHandler),
        ("for-multi", render_for_multi_template as TemplateHandler),
        ("inflation", render_inflation_template as TemplateHandler),
        (
            "inflation/year",
            render_inflation_year_template as TemplateHandler,
        ),
        ("stack", render_passthrough_template as TemplateHandler),
        ("longitem", render_passthrough_template as TemplateHandler),
        ("flagdeco", render_flagdeco_template as TemplateHandler),
        ("pprime", render_pprime_template as TemplateHandler),
        ("ra", render_ra_template as TemplateHandler),
        (
            "mw",
            render_cite_merriam_webster_template as TemplateHandler,
        ),
        (
            "cite merriam-webster",
            render_cite_merriam_webster_template as TemplateHandler,
        ),
        (
            "indented plainlist",
            render_plainlist_template as TemplateHandler,
        ),
        (
            "bulleted list",
            render_unbulleted_list_template as TemplateHandler,
        ),
        ("blist", render_unbulleted_list_template as TemplateHandler),
        ("hyphen", render_hyphen_template as TemplateHandler),
        (
            "native phrase",
            render_native_name_template as TemplateHandler,
        ),
        (
            "native name",
            render_native_name_template as TemplateHandler,
        ),
        ("ship", render_generic_ship_template as TemplateHandler),
        ("proto", render_proto_template as TemplateHandler),
        (
            "infobox",
            render_infobox_generic_template as TemplateHandler,
        ),
        ("wktl", render_lang_template as TemplateHandler),
        ("wikt-lang", render_lang_template as TemplateHandler),
        ("langr", render_lang_template as TemplateHandler),
        ("val", render_val_template as TemplateHandler),
        ("value", render_val_template as TemplateHandler),
        ("chem2", render_chem2_template as TemplateHandler),
        ("e", render_e_template as TemplateHandler),
        ("sup", render_sup_template as TemplateHandler),
        ("sub", render_sub_template as TemplateHandler),
        ("su", render_su_template as TemplateHandler),
        ("mpl", render_mpl_template as TemplateHandler),
        (
            "columns list",
            render_columns_list_template as TemplateHandler,
        ),
        (
            "annotated link",
            render_annotated_link_template as TemplateHandler,
        ),
        ("dp", render_dp_template as TemplateHandler),
        (
            "visible anchor",
            render_visible_anchor_template as TemplateHandler,
        ),
        (
            "cite eb1911",
            render_cite_eb1911_template as TemplateHandler,
        ),
        ("spaces", render_spaces_template as TemplateHandler),
        ("mpl-", render_mpl_dash_template as TemplateHandler),
        ("chem", render_chem_template as TemplateHandler),
        (
            "solar radius",
            render_solar_radius_template as TemplateHandler,
        ),
        ("±", render_plus_minus_template as TemplateHandler),
        (
            "collapsible list",
            render_collapsible_list_template as TemplateHandler,
        ),
        (
            "internet archive short film",
            render_internet_archive_short_film_template as TemplateHandler,
        ),
        (
            "worldhistory",
            render_worldhistory_template as TemplateHandler,
        ),
        ("nihongo2", render_nihongo2_template as TemplateHandler),
        ("gloss", render_gloss_template as TemplateHandler),
        ("xref", render_passthrough_template as TemplateHandler),
        ("shy", render_soft_hyphen_template as TemplateHandler),
        ("color box", render_color_box_template as TemplateHandler),
        ("color", render_color_template as TemplateHandler),
        ("colour", render_color_template as TemplateHandler),
        (
            "osm relation",
            render_openstreetmap_relation_template as TemplateHandler,
        ),
        (
            "osm way",
            render_openstreetmap_way_template as TemplateHandler,
        ),
        ("harvp", render_harvp_template as TemplateHandler),
        ("harv", render_harvp_template as TemplateHandler),
        ("harvnb", render_harvnb_template as TemplateHandler),
        ("harvtxt", render_harvtxt_template as TemplateHandler),
        ("ndldc", render_ndldc_template as TemplateHandler),
        ("plainlist", render_plainlist_template as TemplateHandler),
        (
            "unbulleted list",
            render_unbulleted_list_template as TemplateHandler,
        ),
        ("ubl", render_unbulleted_list_template as TemplateHandler),
        ("ubli", render_unbulleted_list_template as TemplateHandler),
        (
            "unbulleted indent list",
            render_unbulleted_list_template as TemplateHandler,
        ),
        ("ipaslink", render_ipa_link_template as TemplateHandler),
        ("angbr", render_angbr_template as TemplateHandler),
        ("angbr ipa", render_angbr_ipa_template as TemplateHandler),
        ("unichar", render_unichar_template as TemplateHandler),
        ("xlit", render_transliteration_template as TemplateHandler),
        ("note", render_note_template as TemplateHandler),
        (
            "fs interlinear",
            render_fs_interlinear_template as TemplateHandler,
        ),
        ("tooltip", render_tooltip_template as TemplateHandler),
        (
            "nihongo krt",
            render_nihongo_krt_template as TemplateHandler,
        ),
        ("jaanus", render_jaanus_template as TemplateHandler),
        ("nihongo3", render_nihongo3_template as TemplateHandler),
        (
            "easy css image crop",
            render_easy_css_image_crop_template as TemplateHandler,
        ),
        (
            "multiple images",
            render_multiple_images_template as TemplateHandler,
        ),
        (
            "multiple image",
            render_multiple_images_template as TemplateHandler,
        ),
        ("issn", render_issn_template as TemplateHandler),
        ("cite nsrw", render_cite_nsrw_template as TemplateHandler),
        ("stn", render_stn_template as TemplateHandler),
        ("station", render_station_template as TemplateHandler),
        ("jpn", render_jpn_template as TemplateHandler),
        (
            "track gauge",
            render_track_gauge_template as TemplateHandler,
        ),
        ("railgauge", render_track_gauge_template as TemplateHandler),
        ("gburl", render_gburl_template as TemplateHandler),
        (
            "google books",
            render_google_books_template as TemplateHandler,
        ),
        ("cite thesis", render_citation_template as TemplateHandler),
        ("usurped", render_usurped_template as TemplateHandler),
        ("break", render_break_template as TemplateHandler),
        ("br", render_break_template as TemplateHandler),
        ("brk", render_break_template as TemplateHandler),
        ("crlf", render_break_template as TemplateHandler),
        ("jct", render_jct_template as TemplateHandler),
        ("fxconvert", render_fx_convert_template as TemplateHandler),
        ("jpy", render_jpy_template as TemplateHandler),
        ("dts", render_dts_template as TemplateHandler),
        ("doi", render_doi_template as TemplateHandler),
        ("age", render_age_template as TemplateHandler),
        (
            "birth date and age",
            render_birth_date_and_age_template as TemplateHandler,
        ),
        ("ayd", render_ayd_template as TemplateHandler),
        (
            "age in years and days nts",
            render_ayd_template as TemplateHandler,
        ),
        ("routebox", render_route_box_template as TemplateHandler),
        (
            "ja-rail-color",
            render_ja_rail_color_template as TemplateHandler,
        ),
        ("n/a", render_na_template as TemplateHandler),
        ("na", render_na_template as TemplateHandler),
        ("not applicable", render_na_template as TemplateHandler),
        (
            "ja-platform",
            render_ja_platform_template as TemplateHandler,
        ),
        ("jpf", render_ja_platform_template as TemplateHandler),
        (
            "ja-platform-m",
            render_ja_platform_template as TemplateHandler,
        ),
        ("jpfm", render_ja_platform_template as TemplateHandler),
        (
            "ja-rail-linem",
            render_ja_rail_linem_template as TemplateHandler,
        ),
        ("rail-interchange", render_ric_template as TemplateHandler),
        ("ric", render_ric_template as TemplateHandler),
        ("rint", render_ric_template as TemplateHandler),
        ("line link", render_lnl_template as TemplateHandler),
        ("lnl", render_lnl_template as TemplateHandler),
        (
            "infobox mountain",
            render_infobox_mountain_template as TemplateHandler,
        ),
        (
            "infobox country",
            render_infobox_country_template as TemplateHandler,
        ),
        (
            "infobox military conflict",
            render_infobox_military_conflict_template as TemplateHandler,
        ),
        (
            "infobox planet",
            render_infobox_planet_template as TemplateHandler,
        ),
        (
            "infobox settlement",
            render_infobox_settlement_template as TemplateHandler,
        ),
        (
            "infobox",
            render_infobox_generic_template as TemplateHandler,
        ),
        (
            "native name list",
            render_native_name_list_template as TemplateHandler,
        ),
        ("hlist", render_hlist_template as TemplateHandler),
        ("flatlist", render_hlist_template as TemplateHandler),
    ]);

    if lookup.contains_key(&lower.as_str()) {
        return lookup.get(lower.as_str()).unwrap()(params);
    }

    let empty_lookup: EmptyDispatchTable = HashMap::from([
        ("awol", render_awol_template as EmptyHandler),
        (
            "died of wounds",
            render_died_of_wounds_template as EmptyHandler,
        ),
        ("dow", render_died_of_wounds_template as EmptyHandler),
        ("pkia", render_pkia_template as EmptyHandler),
        ("pow", render_pow_template as EmptyHandler),
        ("mia", render_mia_template as EmptyHandler),
        ("wia", render_wia_template as EmptyHandler),
        ("open access", render_open_access_template as EmptyHandler),
        ("free access", render_open_access_template as EmptyHandler),
        (
            "nb5",
            render_five_nonbreaking_spaces_template as EmptyHandler,
        ),
    ]);

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

pub(crate) fn log_and_count_nested_skipped_templates(text: &str) {
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

pub(crate) fn is_handled_template_name(template: &str) -> bool {
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
        || is_silent_template_name(template)
}

pub(crate) fn is_silent_template_name(template: &str) -> bool {
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

pub(crate) fn is_observed_navigation_template_name(template: &str) -> bool {
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

pub(crate) fn is_succession_template_name(template: &str) -> bool {
    template
        .get(.."s-".len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("s-"))
        || template.eq_ignore_ascii_case("Succession box")
}

pub(crate) fn split_template_name(content: &str) -> (&str, &str) {
    let mut template_depth = 0usize;
    let mut link_depth = 0usize;
    let mut chars = content.char_indices().peekable();

    while let Some((index, ch)) = chars.next() {
        if ch == '[' && chars.peek().is_some_and(|(_, next)| *next == '[') {
            chars.next();
            link_depth += 1;
        } else if ch == ']' && chars.peek().is_some_and(|(_, next)| *next == ']') {
            chars.next();
            link_depth = link_depth.saturating_sub(1);
        } else if ch == '{' && chars.peek().is_some_and(|(_, next)| *next == '{') {
            chars.next();
            template_depth += 1;
        } else if ch == '}' && chars.peek().is_some_and(|(_, next)| *next == '}') {
            chars.next();
            template_depth = template_depth.saturating_sub(1);
        } else if ch == '|' && template_depth == 0 && link_depth == 0 {
            return (&content[..index], &content[index + 1..]);
        }
    }

    (content, "")
}
