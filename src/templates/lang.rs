use crate::split_template_params;
use crate::templates::{
    join_plain_items, render_templates, template_named_params, template_param,
    template_positional_params,
};
use std::collections::HashMap;

/// [Korean](https://en.wikipedia.org/wiki/Template:Korean)
/// [Korean/auto](https://en.wikipedia.org/wiki/Template:Korean/auto)
/// [ko](https://en.wikipedia.org/wiki/Template:Ko)
pub(crate) fn render_korean_template(params: &str) -> String {
    let mut hangul = None;
    let mut hanja = None;
    let mut ko_ipa = None;
    let mut positional = Vec::new();

    for part in split_template_params(params)
        .into_iter()
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
    {
        if let Some((key, value)) = part.split_once('=') {
            match key.trim().to_lowercase().as_str() {
                "hangul" => hangul = Some(clean_korean_auto_value(value)),
                "hanja" => hanja = Some(clean_korean_auto_value(value)),
                "ko_ipa" => ko_ipa = Some(value.trim().to_string()),
                _ => {}
            }
        } else {
            positional.push(clean_korean_auto_value(&part));
        }
    }

    let hangul = hangul.or_else(|| positional.first().cloned());
    let hanja = hanja.or_else(|| positional.get(1).cloned());
    let mut values = Vec::new();

    if let Some(hangul) = hangul.as_deref()
        && !hangul.trim().is_empty()
    {
        values.push(format!(
            "Korean: __WIKIPEDIA_TO_EPUB_KOREAN_HANGUL_START__{hangul}__WIKIPEDIA_TO_EPUB_KOREAN_SCRIPT_END__"
        ));
    }

    if let Some(hanja) = hanja.as_deref()
        && !hanja.trim().is_empty()
    {
        values.push(format!(
            "Hanja: __WIKIPEDIA_TO_EPUB_KOREAN_HANJA_START__{hanja}__WIKIPEDIA_TO_EPUB_KOREAN_SCRIPT_END__"
        ));
    }

    if let Some(ko_ipa) = ko_ipa.as_deref()
        && !ko_ipa.trim().is_empty()
    {
        values.push(format!("pronounced [{}]", render_templates(ko_ipa.trim())));
    }

    if values.is_empty() {
        return String::new();
    }

    format!(
        "__WIKIPEDIA_TO_EPUB_KOREAN_TEXT_START__{}__WIKIPEDIA_TO_EPUB_KOREAN_TEXT_END__",
        values.join(" / ")
    )
}

pub(crate) fn clean_korean_auto_value(value: &str) -> String {
    value
        .trim()
        .chars()
        .filter(|ch| !matches!(ch, '^' | '%' | '_'))
        .collect()
}

/// [Nihongo](https://en.wikipedia.org/wiki/Template:Nihongo)
pub(crate) fn render_japanese_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let english = positional
        .first()
        .cloned()
        .or_else(|| named.get("1").cloned())
        .or_else(|| named.get("english").cloned())
        .unwrap_or_default();
    let kanji = positional
        .get(1)
        .cloned()
        .or_else(|| named.get("2").cloned())
        .or_else(|| named.get("japanese").cloned())
        .unwrap_or_default();
    let romaji = positional
        .get(2)
        .cloned()
        .or_else(|| named.get("3").cloned())
        .or_else(|| named.get("romaji").cloned())
        .unwrap_or_default();
    let extra = positional
        .get(3)
        .cloned()
        .or_else(|| named.get("4").cloned())
        .or_else(|| named.get("extra").cloned())
        .unwrap_or_default();
    let extra2 = positional
        .get(4)
        .cloned()
        .or_else(|| named.get("5").cloned())
        .or_else(|| named.get("extra2").cloned())
        .unwrap_or_default();
    let lead = named.get("lead").cloned().unwrap_or_default();

    let english = render_templates(&english);
    let kanji = render_templates(&kanji);
    let romaji = render_templates(&romaji);
    let extra = render_templates(&extra);
    let extra2 = render_templates(&extra2);

    let mut parts = Vec::new();
    if !kanji.trim().is_empty() {
        let kanji_trimmed = kanji.trim();
        if lead.eq_ignore_ascii_case("yes") || lead.eq_ignore_ascii_case("y") {
            parts.push(format!(
                "Japanese: __WIKIPEDIA_TO_EPUB_JAPANESE_TEXT_START__{kanji_trimmed}__WIKIPEDIA_TO_EPUB_JAPANESE_TEXT_END__"
            ));
        } else {
            parts.push(format!(
                "__WIKIPEDIA_TO_EPUB_JAPANESE_TEXT_START__{kanji_trimmed}__WIKIPEDIA_TO_EPUB_JAPANESE_TEXT_END__"
            ));
        }
    }

    if !romaji.trim().is_empty() {
        let romaji_trimmed = romaji.trim();
        if lead.eq_ignore_ascii_case("yes") || lead.eq_ignore_ascii_case("y") {
            parts.push(format!("Hepburn: ''{romaji_trimmed}''"));
        } else {
            parts.push(format!("''{romaji_trimmed}''"));
        }
    }

    let mut inside = parts.join(", ");
    if !extra.trim().is_empty() {
        let extra_trimmed = extra.trim();
        if inside.is_empty() {
            inside = extra_trimmed.to_string();
        } else {
            inside = format!("{inside}; {extra_trimmed}");
        }
    }

    let paren_part = if inside.is_empty() {
        String::new()
    } else {
        format!(
            "__WIKIPEDIA_TO_EPUB_JAPANESE_NORMAL_START__ ({inside})__WIKIPEDIA_TO_EPUB_JAPANESE_NORMAL_END__"
        )
    };

    let mut result = english.trim().to_string();
    if !paren_part.is_empty() {
        if !result.is_empty() {
            result.push_str(&paren_part);
        } else {
            result = paren_part;
        }
    }
    if !extra2.trim().is_empty() {
        let extra2_trimmed = extra2.trim();
        if !result.is_empty() {
            result = format!("{result} {extra2_trimmed}");
        } else {
            result = extra2_trimmed.to_string();
        }
    }

    result
}

/// [Nihongo foot](https://en.wikipedia.org/wiki/Template:Nihongo_foot)
pub(crate) fn render_nihongo_foot_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let english = positional
        .first()
        .cloned()
        .or_else(|| named.get("1").cloned())
        .unwrap_or_default();
    let kanji = positional
        .get(1)
        .cloned()
        .or_else(|| named.get("2").cloned())
        .unwrap_or_default();
    let romaji = positional
        .get(2)
        .cloned()
        .or_else(|| named.get("3").cloned())
        .unwrap_or_default();
    let extra = positional
        .get(3)
        .cloned()
        .or_else(|| named.get("4").cloned())
        .unwrap_or_default();
    let post = named.get("post").cloned().unwrap_or_default();

    let english = render_templates(&english);
    let kanji = render_templates(&kanji);
    let romaji = render_templates(&romaji);
    let extra = render_templates(&extra);
    let post = render_templates(&post);

    let mut parts = Vec::new();
    if !kanji.trim().is_empty() {
        parts.push(format!(
            "__WIKIPEDIA_TO_EPUB_JAPANESE_TEXT_START__{}__WIKIPEDIA_TO_EPUB_JAPANESE_TEXT_END__",
            kanji.trim()
        ));
    }
    if !romaji.trim().is_empty() {
        parts.push(format!("<em>{}</em>", romaji.trim()));
    }
    if !extra.trim().is_empty() {
        parts.push(extra.trim().to_string());
    }

    if parts.is_empty() {
        format!("{english}{post}")
    } else {
        let inside = parts.join(", ");
        format!(
            "{english}__WIKIPEDIA_TO_EPUB_JAPANESE_NORMAL_START__ ({inside})__WIKIPEDIA_TO_EPUB_JAPANESE_NORMAL_END__{post}"
        )
    }
}

/// [N/A](https://en.wikipedia.org/wiki/Template:N/A)
/// [NA](https://en.wikipedia.org/wiki/Template:NA)
/// [Not applicable](https://en.wikipedia.org/wiki/Template:Not_applicable)
pub(crate) fn render_na_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let text = positional
        .first()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim())
        .unwrap_or("N/A");
    text.to_string()
}

/// [nihongo3](https://en.wikipedia.org/wiki/Template:Nihongo3)
pub(crate) fn render_nihongo3_template(params: &str) -> String {
    let positional = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.contains('='))
        .collect::<Vec<_>>();

    let english = positional.first().map(|s| s.as_str()).unwrap_or("");
    let kanji = positional.get(1).map(|s| s.as_str()).unwrap_or("");
    let romaji = positional.get(2).map(|s| s.as_str()).unwrap_or("");
    let extra1 = positional.get(3).map(|s| s.as_str()).unwrap_or("");
    let extra2 = positional.get(4).map(|s| s.as_str()).unwrap_or("");

    let english = render_templates(english);
    let kanji = render_templates(kanji);
    let romaji = render_templates(romaji);
    let extra1 = render_templates(extra1);
    let extra2 = render_templates(extra2);

    let romaji_part = if !romaji.is_empty() {
        format!("''{}''", romaji)
    } else {
        String::new()
    };

    let mut paren_elements = Vec::new();

    if !kanji.is_empty() {
        paren_elements.push(format!(
            "__WIKIPEDIA_TO_EPUB_JAPANESE_TEXT_START__{kanji}__WIKIPEDIA_TO_EPUB_JAPANESE_TEXT_END__"
        ));
    }

    if !english.is_empty() {
        paren_elements.push(format!("\"{}\"", english));
    }

    if !extra1.is_empty() {
        paren_elements.push(extra1);
    }
    if !extra2.is_empty() {
        paren_elements.push(extra2);
    }

    let paren_str = if !paren_elements.is_empty() {
        format!(" ({})", paren_elements.join(", "))
    } else {
        String::new()
    };

    format!(
        "__WIKIPEDIA_TO_EPUB_JAPANESE_NORMAL_START__{romaji_part}{paren_str}__WIKIPEDIA_TO_EPUB_JAPANESE_NORMAL_END__"
    )
}

/// [lang](https://en.wikipedia.org/wiki/Template:Lang)
/// [wktl](https://en.wikipedia.org/wiki/Template:Wktl)
/// [wikt-lang](https://en.wikipedia.org/wiki/Template:Wikt-lang)
/// [langr](https://en.wikipedia.org/wiki/Template:Langr)
pub(crate) fn render_lang_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    let Some(language) = params
        .first()
        .map(String::as_str)
        .filter(|value| !value.is_empty())
    else {
        return String::new();
    };
    let Some(text) = params
        .get(1)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
    else {
        return String::new();
    };

    let language = language
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
        .collect::<String>();

    if language.is_empty() {
        return text.to_string();
    }

    let text = render_templates(text);

    format!(
        "__WIKIPEDIA_TO_EPUB_LANG_START__{language}__WIKIPEDIA_TO_EPUB_LANG_VALUE__{text}__WIKIPEDIA_TO_EPUB_LANG_END__"
    )
}

/// [in lang](https://en.wikipedia.org/wiki/Template:In_lang)
pub(crate) fn render_in_lang_template(params: &str) -> String {
    let languages = template_positional_params(params)
        .into_iter()
        .map(|language| language_name_for_in_lang(&language).to_string())
        .filter(|language| !language.is_empty())
        .collect::<Vec<_>>();

    match languages.as_slice() {
        [] => String::new(),
        [language] => format!("(in {language})"),
        languages => format!("(in {})", join_plain_items(languages)),
    }
}

pub(crate) fn language_name_for_in_lang(language: &str) -> &str {
    match language.trim().to_ascii_lowercase().as_str() {
        "ar" => "Arabic",
        "de" => "German",
        "en" => "English",
        "es" => "Spanish",
        "fa" => "Persian",
        "fr" => "French",
        "he" => "Hebrew",
        "ja" => "Japanese",
        "ko" => "Korean",
        "ru" => "Russian",
        "zh" | "zh-cn" | "zh-hans" | "zh-hant" | "zh-tw" => "Chinese",
        _ => language.trim(),
    }
}

/// [linktext](https://en.wikipedia.org/wiki/Template:Linktext)
pub(crate) fn render_linktext_template(params: &str) -> String {
    template_positional_params(params)
        .into_iter()
        .map(|param| render_templates(&param))
        .collect::<Vec<_>>()
        .join("")
}

/// [langx](https://en.wikipedia.org/wiki/Template:Langx)
pub(crate) fn render_langx_template(params: &str) -> String {
    let mut positional = Vec::new();
    let mut named = HashMap::new();

    for param in split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty())
    {
        if let Some((key, value)) = param.split_once('=') {
            named.insert(key.trim().to_lowercase(), value.trim().to_string());
        } else {
            positional.push(param);
        }
    }

    let Some(language) = positional.first().map(String::as_str) else {
        return String::new();
    };
    let Some(text) = positional.get(1).map(String::as_str) else {
        return String::new();
    };

    let mut rendered = render_lang_template(&format!("{language}|{text}"));

    if let Some(translit) = named
        .get("translit")
        .filter(|value| !value.trim().is_empty())
    {
        rendered.push_str(" (");
        rendered.push_str(translit.trim());
        rendered.push(')');
    }

    if let Some(literal) = named.get("lit").filter(|value| !value.trim().is_empty()) {
        rendered.push_str(", lit. ");
        rendered.push_str(literal.trim());
    }

    rendered
}

/// [lang-zh](https://en.wikipedia.org/wiki/Template:Lang-zh)
/// [zh](https://en.wikipedia.org/wiki/Template:Zh)
/// [zhi](https://en.wikipedia.org/wiki/Template:Zhi)
pub(crate) fn render_chinese_lang_template(params: &str) -> String {
    let mut positional = Vec::new();
    let mut named = HashMap::new();

    for param in split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty())
    {
        if let Some((key, value)) = param.split_once('=') {
            named.insert(key.trim().to_lowercase(), value.trim().to_string());
        } else {
            positional.push(param);
        }
    }

    let Some(text) = named
        .get("t")
        .or_else(|| named.get("s"))
        .or_else(|| named.get("c"))
        .or_else(|| named.get("text"))
        .or_else(|| positional.first())
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
    else {
        return String::new();
    };

    let mut rendered = render_lang_template(&format!("zh|{text}"));

    if let Some(pinyin) = named
        .get("p")
        .or_else(|| named.get("pinyin"))
        .filter(|value| !value.trim().is_empty())
    {
        rendered.push_str(" (");
        rendered.push_str(pinyin.trim());
        rendered.push(')');
    }

    rendered
}

/// [transliteration](https://en.wikipedia.org/wiki/Template:Transliteration)
/// [translit](https://en.wikipedia.org/wiki/Template:Translit)
/// [xlit](https://en.wikipedia.org/wiki/Template:Xlit)
pub(crate) fn render_transliteration_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    let Some(language) = params.first().map(String::as_str) else {
        return String::new();
    };
    let Some(text) = params
        .last()
        .map(String::as_str)
        .filter(|value| !value.is_empty() && params.len() > 1)
    else {
        return String::new();
    };

    let language = language
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
        .collect::<String>();

    if language.is_empty() {
        return render_templates(text);
    }

    format!(
        "__WIKIPEDIA_TO_EPUB_LANG_START__{language}-Latn__WIKIPEDIA_TO_EPUB_LANG_VALUE__{}__WIKIPEDIA_TO_EPUB_LANG_END__",
        render_templates(text)
    )
}

/// [tlit](https://en.wikipedia.org/wiki/Template:Tlit)
pub(crate) fn render_transliteration_like_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    let Some(language) = params.first().map(String::as_str) else {
        return String::new();
    };
    let Some(text) = params
        .last()
        .map(String::as_str)
        .filter(|_| params.len() > 1)
    else {
        return String::new();
    };

    let language = language
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
        .collect::<String>();

    if language.is_empty() {
        return render_templates(text);
    }

    format!(
        "__WIKIPEDIA_TO_EPUB_LANG_START__{language}-Latn__WIKIPEDIA_TO_EPUB_LANG_VALUE__{}__WIKIPEDIA_TO_EPUB_LANG_END__",
        render_templates(text)
    )
}

/// [ko-translit](https://en.wikipedia.org/wiki/Template:Ko-translit)
pub(crate) fn render_korean_transliteration_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    let Some(system) = params.first().map(String::as_str) else {
        return String::new();
    };
    let Some(korean) = params.get(1).map(|value| clean_korean_auto_value(value)) else {
        return String::new();
    };

    match (system.trim().to_ascii_lowercase().as_str(), korean.as_str()) {
        ("rr", "한국") => "Hanguk".to_string(),
        ("mr", "한국") => "Han'guk".to_string(),
        ("rr", "조선") => "Joseon".to_string(),
        ("mr", "조선") => "Chosŏn".to_string(),
        _ => korean,
    }
}

/// [lit](https://en.wikipedia.org/wiki/Template:Lit)
/// [Literal translation](https://en.wikipedia.org/wiki/Template:Literal_translation)
/// [literal](https://en.wikipedia.org/wiki/Template:Literal)
pub(crate) fn render_literal_template(params: &str) -> String {
    let Some(text) = template_positional_params(params)
        .into_iter()
        .find(|value| !value.trim().is_empty())
    else {
        return String::new();
    };

    format!("lit. {}", render_templates(&text))
}

/// [script](https://en.wikipedia.org/wiki/Template:Script)
pub(crate) fn render_script_template(params: &str) -> String {
    let positional = template_positional_params(params);
    if positional.len() >= 2 {
        render_templates(&positional[1])
    } else if let Some(first) = positional.first() {
        render_templates(first)
    } else {
        String::new()
    }
}

/// [ipa](https://en.wikipedia.org/wiki/Template:Ipa)
pub(crate) fn render_ipa_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    let Some(ipa) = params.get(1).map(String::as_str) else {
        return String::new();
    };

    format!(
        "__WIKIPEDIA_TO_EPUB_IPA_START__{}__WIKIPEDIA_TO_EPUB_IPA_END__",
        render_templates(ipa)
    )
}

/// [IPAc-en](https://en.wikipedia.org/wiki/Template:IPAc-en)
pub(crate) fn render_english_ipa_template(params: &str) -> String {
    let ipa = template_positional_params(params)
        .into_iter()
        .filter(|param| {
            !matches!(
                param.trim().to_ascii_lowercase().as_str(),
                "lang" | "pron" | "pronunciation"
            )
        })
        .map(|param| render_templates(&param))
        .collect::<Vec<_>>()
        .join("");

    if ipa.is_empty() {
        return String::new();
    }

    format!("__WIKIPEDIA_TO_EPUB_IPA_START__{ipa}__WIKIPEDIA_TO_EPUB_IPA_END__")
}

/// [Respell](https://en.wikipedia.org/wiki/Template:Respell)
pub(crate) fn render_respell_template(params: &str) -> String {
    template_positional_params(params)
        .into_iter()
        .map(|param| render_templates(&param))
        .filter(|param| !param.trim().is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// [nihongo2](https://en.wikipedia.org/wiki/Template:Nihongo2)
pub(crate) fn render_nihongo2_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(text) = positional.first().filter(|t| !t.trim().is_empty()) else {
        return String::new();
    };
    let text = render_templates(text);
    format!(
        "__WIKIPEDIA_TO_EPUB_LANG_START__ja__WIKIPEDIA_TO_EPUB_LANG_VALUE__{text}__WIKIPEDIA_TO_EPUB_LANG_END__"
    )
}

/// [gloss](https://en.wikipedia.org/wiki/Template:Gloss)
pub(crate) fn render_gloss_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);
    let Some(text) = positional.first().filter(|t| !t.trim().is_empty()) else {
        return String::new();
    };
    let text = render_templates(text);
    if template_param(&named, &["mode"]).is_some_and(|mode| mode.trim() == "def") {
        format!("({text})")
    } else {
        format!("'{text}'")
    }
}

/// [IPAslink](https://en.wikipedia.org/wiki/Template:IPAslink)
pub(crate) fn render_ipa_link_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(symbol) = positional.first().filter(|s| !s.trim().is_empty()) else {
        return String::new();
    };
    let label = positional
        .get(1)
        .filter(|l| !l.trim().is_empty())
        .unwrap_or(symbol);
    format!(
        "__WIKIPEDIA_TO_EPUB_IPA_START__{}__WIKIPEDIA_TO_EPUB_IPA_END__",
        render_templates(label.trim())
    )
}

/// [angbr](https://en.wikipedia.org/wiki/Template:Angbr)
pub(crate) fn render_angbr_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(text) = positional.first().filter(|t| !t.trim().is_empty()) else {
        return String::new();
    };
    format!("⟨{}⟩", render_templates(text.trim()))
}

/// [angbr IPA](https://en.wikipedia.org/wiki/Template:Angbr_IPA)
pub(crate) fn render_angbr_ipa_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(text) = positional.first().filter(|t| !t.trim().is_empty()) else {
        return String::new();
    };
    let text = render_templates(text.trim());
    format!(
        "⟨__WIKIPEDIA_TO_EPUB_LANG_START__und-fonipa__WIKIPEDIA_TO_EPUB_LANG_VALUE__{text}__WIKIPEDIA_TO_EPUB_LANG_END__⟩"
    )
}

/// [unichar](https://en.wikipedia.org/wiki/Template:Unichar)
pub(crate) fn render_unichar_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);
    let Some(hex_str) = positional.first().filter(|s| !s.trim().is_empty()) else {
        return String::new();
    };
    let hex_str = hex_str.trim();
    let ch = u32::from_str_radix(hex_str, 16)
        .ok()
        .and_then(char::from_u32);

    let ch_str = match ch {
        Some(c) => c.to_string(),
        None => String::new(),
    };

    let base = template_param(&named, &["cwith"])
        .map(|s| s.trim())
        .unwrap_or("");

    let name = positional
        .get(1)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    let glyph = format!("{base}{ch_str}");

    let details = match name {
        Some(n) => format!("U+{} {}", hex_str.to_uppercase(), n),
        None => format!("U+{}", hex_str.to_uppercase()),
    };

    format!("{glyph} ({details})")
}

/// [Nihongo krt](https://en.wikipedia.org/wiki/Template:Nihongo_krt)
pub(crate) fn render_nihongo_krt_template(params: &str) -> String {
    let positional = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.contains('='))
        .collect::<Vec<_>>();

    let english = positional.first().map(|s| s.as_str()).unwrap_or("");
    let kanji = positional.get(1).map(|s| s.as_str()).unwrap_or("");
    let romaji = positional.get(2).map(|s| s.as_str()).unwrap_or("");

    if kanji.is_empty() {
        return render_templates(english);
    }

    let mut inside = Vec::new();
    if !romaji.is_empty() {
        inside.push(format!("''{}''", render_templates(romaji)));
    }
    if !english.is_empty() {
        inside.push(render_templates(english).to_string());
    }

    let kanji_rendered = format!(
        "__WIKIPEDIA_TO_EPUB_LANG_START__ja__WIKIPEDIA_TO_EPUB_LANG_VALUE__{kanji}__WIKIPEDIA_TO_EPUB_LANG_END__"
    );

    if inside.is_empty() {
        kanji_rendered
    } else {
        format!("{kanji_rendered} ({})", inside.join(", "))
    }
}

/// [Ja-rail-color](https://en.wikipedia.org/wiki/Template:Ja-rail-color)
pub(crate) fn render_ja_rail_color_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(code) = positional.first().map(|s| s.to_ascii_uppercase()) else {
        return "#333333".to_string();
    };

    let color = match code.as_str() {
        "JY" => "#80c241", // Yamanote Line
        "JK" => "#00b2e5", // Keihin-Tohoku Line
        "JU" => "#f58220", // Utsunomiya/Takasaki Line
        "JC" => "#f15a22", // Chuo Line
        "JO" => "#007ac1", // Yokosuka/Sobu Rapid Line
        "JB" => "#ffd400", // Chuo-Sobu Line
        "JE" => "#c9242f", // Keiyo Line
        "JH" => "#80c241", // Yokohama Line
        "JT" => "#f58220", // Tokaido Line
        "JJ" => "#00b261", // Joban Line
        "JM" => "#f15a22", // Musashino Line
        "JN" => "#ffd400", // Nambu Line
        "JI" => "#ffd400", // Tsurumi Line
        "MO" => "#007ac1", // Tokyo Monorail
        "KK" => "#e60012", // Keikyu
        "U" => "#007ac1",  // Yurikamome
        "TR" => "#007ac1", // Toyo Rapid
        "SR" => "#007ac1", // Saitama Rapid / Shibayama Railway
        "N" => "#00ac9a",  // Tokyo Metro Namboku Line
        "HS" => "#007ac1", // Hokuso Line
        _ => "#333333",
    };
    color.to_string()
}

/// [Ja-platform](https://en.wikipedia.org/wiki/Template:Ja-platform)
/// [jpf](https://en.wikipedia.org/wiki/Template:Jpf)
/// [Ja-platform-m](https://en.wikipedia.org/wiki/Template:Ja-platform-m)
/// [jpfm](https://en.wikipedia.org/wiki/Template:Jpfm)
pub(crate) fn render_ja_platform_template(params: &str) -> String {
    let named = template_named_params(params);

    let pfn = template_param(&named, &["pfn"])
        .map(str::to_string)
        .unwrap_or_default();
    let name = template_param(&named, &["name"])
        .map(str::to_string)
        .unwrap_or_default();
    let symbol = template_param(&named, &["symbol", "imgfile"])
        .map(str::to_string)
        .unwrap_or_default();
    let dir = template_param(&named, &["dir"])
        .map(str::to_string)
        .unwrap_or_default();

    let pfn = render_templates(&pfn);
    let name = render_templates(&name);
    let symbol = render_templates(&symbol);
    let dir = render_templates(&dir);

    let cell_line = if symbol.is_empty() {
        name
    } else if name.is_empty() {
        symbol
    } else {
        format!("{} {}", symbol, name)
    };

    format!("|-\n| '''{}'''\n| {}\n| {}\n", pfn, cell_line, dir)
}

pub(crate) fn format_interlanguage_link(
    article: &str,
    label: Option<&str>,
    language: Option<&String>,
) -> String {
    let link = if let Some(label) = label {
        format!("[[{article}|{label}]]")
    } else {
        format!("[[{article}]]")
    };

    match language
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(language) => format!("{link} [{language}]"),
        None => link,
    }
}

/// [Translation](https://en.wikipedia.org/wiki/Template:Translation)
pub(crate) fn render_translation_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let val1 = positional
        .first()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("");
    let val2 = positional
        .get(1)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("");

    let sortable = template_param(&named, &["sortable"])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .is_some();
    let italic = template_param(&named, &["i"])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .is_some();
    let literal = template_param(&named, &["literal"])
        .map(|s| s.trim())
        .unwrap_or("");

    let mut result = String::new();

    if sortable && !val1.is_empty() {
        result.push_str(&format!(
            "<span style=\"display:none;\">{}</span>",
            render_templates(val1)
        ));
    }

    if italic {
        result.push_str("''");
    }

    match literal {
        "no" | "off" => result.push_str("transl."),
        "yes" | "on" => {
            result.push_str(&render_templates(
                "{{Abbr|lit. transl.|literal translation}}",
            ));
        }
        _ => {
            result.push_str(&render_templates("{{Abbr|transl.|translation}}"));
        }
    }

    if italic {
        result.push_str("''");
    }

    if !val1.is_empty() {
        result.push_str(&format!("\u{2009}{}", render_templates(val1)));
    }

    if !val2.is_empty() {
        result.push_str(&format!(" – transl.\u{2009}{}", render_templates(val2)));
    }

    result
}

/// [ja-rail-linem](https://en.wikipedia.org/wiki/Template:Ja-rail-linem)
pub(crate) fn render_ja_rail_linem_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let span = template_param(&named, &["span"])
        .map(|s| s.trim())
        .unwrap_or("");
    let pfn = template_param(&named, &["pfn"])
        .map(|s| s.trim())
        .unwrap_or("");
    let linecol = template_param(&named, &["linecol"])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("white");
    let nolinkindex = template_param(&named, &["nolinkindex"])
        .map(|s| s.trim())
        .unwrap_or("");
    let linename = template_param(&named, &["linename"])
        .map(|s| s.trim())
        .unwrap_or("");
    let lineindex = template_param(&named, &["lineindex"])
        .map(|s| s.trim())
        .unwrap_or("");
    let dir = template_param(&named, &["dir"])
        .map(|s| s.trim())
        .unwrap_or("");
    let next = template_param(&named, &["next"])
        .map(|s| s.trim())
        .unwrap_or("");
    let nextstop = template_param(&named, &["nextstop"])
        .map(|s| s.trim())
        .unwrap_or("");

    // Determine symbol type
    let pos1 = positional.first().map(String::as_str).unwrap_or("");
    let pos2 = positional.get(1).map(String::as_str).unwrap_or("");
    let is_m = pos1 == "m" || pos2 == "m";
    let symbol = if is_m { "'''○'''" } else { "■" };

    // Format line part
    let line_part = if !nolinkindex.is_empty() {
        nolinkindex.to_string()
    } else if !linename.is_empty() {
        if !lineindex.is_empty() {
            format!("[[{linename}|{lineindex}]]")
        } else {
            format!("[[{linename}]]")
        }
    } else {
        String::new()
    };

    let line_symbol_part = if line_part.is_empty() {
        String::new()
    } else {
        format!("<span style=\"color:{linecol}\">{symbol}</span>&nbsp;{line_part}")
    };

    let mut result = String::new();
    result.push_str("|-\n");

    if !span.is_empty() {
        let rowspan_attr = if span != "1" && !span.is_empty() {
            format!(" rowspan={span}")
        } else {
            String::new()
        };
        result.push_str(&format!(
            "|{rowspan_attr} | '''{}'''\n",
            render_templates(pfn)
        ));
        result.push_str(&format!("| {}\n", render_templates(&line_symbol_part)));

        let dir_formatted = if !nextstop.is_empty() {
            format!("{dir} <small>({nextstop})</small>")
        } else {
            dir.to_string()
        };
        result.push_str(&format!("| {}\n", render_templates(&dir_formatted)));
    } else {
        result.push_str(&format!("| {}\n", render_templates(&line_symbol_part)));

        let dir_formatted = if !next.is_empty() {
            format!("{dir} <small>({next})</small>")
        } else {
            dir.to_string()
        };
        result.push_str(&format!("| {}\n", render_templates(&dir_formatted)));
    }

    result
}

/// [Language with name/for](https://en.wikipedia.org/wiki/Template:Language_with_name/for)
/// [langnf](https://en.wikipedia.org/wiki/Template:Language_with_name/for)
pub(crate) fn render_langnf_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.contains('='))
        .collect::<Vec<_>>();

    let lang_tag = template_param(&named, &["lang"])
        .or_else(|| positional.first().map(String::as_str))
        .map(|s| s.trim())
        .unwrap_or("");
    let text = template_param(&named, &["text"])
        .or_else(|| positional.get(1).map(String::as_str))
        .map(|s| s.trim())
        .unwrap_or("");

    // If both lang tag and text are empty, return empty
    if lang_tag.is_empty() && text.is_empty() {
        return String::new();
    }

    let lang_name_opt = template_param(&named, &["lang-name"])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    let language_name = if let Some(custom) = lang_name_opt {
        custom.to_string()
    } else if !lang_tag.is_empty() {
        language_name_for_in_lang(lang_tag).to_string()
    } else {
        String::new()
    };

    let language_link = if language_name.is_empty() {
        String::new()
    } else if language_name.contains("[[") {
        language_name
    } else {
        format!("[[{language_name} language|{language_name}]]")
    };

    // Render foreign text
    let lang_attr = if !lang_tag.is_empty() {
        lang_tag.to_string()
    } else {
        "mis".to_string()
    };

    let is_cjk =
        lang_attr.starts_with("ja") || lang_attr.starts_with("ko") || lang_attr.starts_with("zh");
    let formatted_text = if is_cjk || (text.starts_with("''") && text.ends_with("''")) {
        text.to_string()
    } else {
        format!("''{text}''")
    };

    let foreign_span = format!(
        "__WIKIPEDIA_TO_EPUB_LANG_START__{lang_attr}__WIKIPEDIA_TO_EPUB_LANG_VALUE__{formatted_text}__WIKIPEDIA_TO_EPUB_LANG_END__"
    );

    // Collect translations (term1 or positional 3, term2, term3, term4, term5)
    let italic_term = template_param(&named, &["italic-term"])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .is_some_and(|s| s == "yes" || s == "on");

    let is_break = template_param(&named, &["break"])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .is_some_and(|s| s == "yes" || s == "on");

    let paren = template_param(&named, &["paren"])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("");

    let mut terms = Vec::new();
    if let Some(t1) = template_param(&named, &["term1"])
        .or_else(|| positional.get(2).map(String::as_str))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        terms.push(t1);
    }
    for i in 2..=5 {
        if let Some(ti) = template_param(&named, &[&format!("term{i}")])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            terms.push(ti);
        }
    }

    let terms_part = if terms.is_empty() {
        String::new()
    } else {
        let formatted_terms = terms
            .into_iter()
            .map(|term| {
                if italic_term {
                    format!("'<em>{}</em>'", render_templates(term))
                } else {
                    format!("'{}'", render_templates(term))
                }
            })
            .collect::<Vec<_>>();
        formatted_terms.join(" / ")
    };

    let parenthetical_inner = if terms_part.is_empty() {
        language_link
    } else if language_link.is_empty() {
        format!("for {terms_part}")
    } else {
        format!("{language_link} for {terms_part}")
    };

    let (left_paren, right_paren) = match paren.to_ascii_lowercase().as_str() {
        "none" => ("", ""),
        "left" => ("(", ""),
        _ => ("(", ")"),
    };

    let separator = if is_break { "<br />" } else { " " };

    if parenthetical_inner.is_empty() {
        foreign_span
    } else {
        format!("{foreign_span}{separator}{left_paren}{parenthetical_inner}{right_paren}")
    }
}

fn get_lang_name(tag: &str) -> &'static str {
    match tag.trim().to_lowercase().as_str() {
        "ja" => "Japanese",
        "ko" => "Korean",
        "zh" => "Chinese",
        "en" => "English",
        "fr" => "French",
        "de" => "German",
        "es" => "Spanish",
        "it" => "Italian",
        "ru" => "Russian",
        "ar" => "Arabic",
        "he" => "Hebrew",
        "la" => "Latin",
        "grc" => "Ancient Greek",
        "el" => "Greek",
        "vi" => "Vietnamese",
        _ => "",
    }
}

/// [native name](https://en.wikipedia.org/wiki/Template:Native_name)
pub(crate) fn render_native_name_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let lang_code = positional.first().cloned().unwrap_or_default();
    let name = positional.get(1).cloned().unwrap_or_default();
    let name_trimmed = name.trim();
    if name_trimmed.is_empty() {
        return String::new();
    }

    let italics = named
        .get("italics")
        .or_else(|| named.get("italic"))
        .map(|s| s.trim().to_lowercase())
        .unwrap_or_default();
    let use_italics = italics != "off" && italics != "no";

    let formatted_name = if use_italics {
        format!("''{}''", name_trimmed)
    } else {
        name_trimmed.to_string()
    };

    let paren = named
        .get("paren")
        .or_else(|| named.get("icon"))
        .map(|s| s.trim().to_lowercase())
        .unwrap_or_default();
    let omit_paren = paren == "omit" || paren == "off" || paren == "no";

    if omit_paren {
        formatted_name
    } else {
        let lang_name = get_lang_name(&lang_code);
        if lang_name.is_empty() {
            formatted_name
        } else {
            format!("{} ({})", formatted_name, lang_name)
        }
    }
}
