use std::collections::HashMap;

use regex::Regex;

use crate::types::{DispatchTable, EmptyDispatchTable, EmptyHandler, TemplateHandler};

use crate::config::current_utc_date;
use crate::config::parse_date_string;

use crate::tools::{
    split_parameter_by_equals, split_template_params, template_named_params, template_param,
    template_param_owned, template_positional_params,
};

use crate::templates::render_templates;

/// [jct](https://en.wikipedia.org/wiki/Template:Jct)
fn render_jct_template(params: &str) -> String {
    let mut country = None;
    let mut state = None;
    let mut positional = Vec::new();

    for part in split_template_params(params)
        .into_iter()
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
    {
        if let Some((key, value)) = part.split_once('=') {
            match key.trim().to_lowercase().as_str() {
                "country" => country = Some(value.trim().to_string()),
                "state" => state = Some(value.trim().to_string()),
                _ => {}
            }
        } else {
            positional.push(part);
        }
    }

    let country = country.as_deref().unwrap_or("");
    let state = state.as_deref().unwrap_or("");

    if country == "JPN" && positional.len() >= 2 && positional[0] == "Route" {
        let route_num = &positional[1];
        return format!("[[Japan National Route {route_num}|National Route {route_num}]]");
    }

    if !positional.is_empty() {
        let route_num = positional.last().unwrap();
        let prefix = if !state.is_empty() {
            state
        } else if !country.is_empty() {
            country
        } else {
            positional.first().unwrap().as_str()
        };
        return format!("{prefix} {route_num}");
    }

    String::new()
}

/// [sfrac](https://en.wikipedia.org/wiki/Template:Sfrac)
fn render_sfrac_template(params: &str) -> String {
    let params = template_positional_params(params)
        .into_iter()
        .map(|param| render_templates(&param))
        .collect::<Vec<_>>();

    match params.as_slice() {
        [] => String::new(),
        [denominator] => format!(
            "__WIKIPEDIA_TO_EPUB_SUP_START__1__WIKIPEDIA_TO_EPUB_SUP_END__⁄__WIKIPEDIA_TO_EPUB_SUB_START__{}__WIKIPEDIA_TO_EPUB_SUB_END__",
            denominator
        ),
        [numerator, denominator] => format!(
            "__WIKIPEDIA_TO_EPUB_SUP_START__{}__WIKIPEDIA_TO_EPUB_SUP_END__⁄__WIKIPEDIA_TO_EPUB_SUB_START__{}__WIKIPEDIA_TO_EPUB_SUB_END__",
            numerator, denominator
        ),
        [whole, numerator, denominator] => format!(
            "{} __WIKIPEDIA_TO_EPUB_SUP_START__{}__WIKIPEDIA_TO_EPUB_SUP_END__⁄__WIKIPEDIA_TO_EPUB_SUB_START__{}__WIKIPEDIA_TO_EPUB_SUB_END__",
            whole, numerator, denominator
        ),
        [first, rest @ ..] => {
            let num = rest.first().map(String::as_str).unwrap_or("");
            let den = rest.get(1).map(String::as_str).unwrap_or("");
            format!(
                "{} __WIKIPEDIA_TO_EPUB_SUP_START__{}__WIKIPEDIA_TO_EPUB_SUP_END__⁄__WIKIPEDIA_TO_EPUB_SUB_START__{}__WIKIPEDIA_TO_EPUB_SUB_END__",
                first, num, den
            )
        }
    }
}

/// [tmath](https://en.wikipedia.org/wiki/Template:Tmath)
fn render_tmath_template(params: &str) -> String {
    let named = template_named_params(params);
    if let Some(val) = template_param(&named, &["1"]) {
        val.to_string()
    } else {
        let positional = template_positional_params(params);
        if let Some(val) = positional.first() {
            val.to_string()
        } else {
            String::new()
        }
    }
}

/// [closed-open](https://en.wikipedia.org/wiki/Template:Closed-open)
fn render_closed_open_template(params: &str) -> String {
    let positional = template_positional_params(params);
    match positional.as_slice() {
        [] => String::new(),
        [single] => {
            if let Some((a, b)) = single.split_once(',') {
                format!("[{}, {})", a.trim(), b.trim())
            } else {
                format!("[{}, )", single.trim())
            }
        }
        [a, b, ..] => {
            format!("[{}, {})", a.trim(), b.trim())
        }
    }
}

/// [sqrt](https://en.wikipedia.org/wiki/Template:Sqrt)
fn render_sqrt_template(params: &str) -> String {
    let positional = template_positional_params(params);
    if let Some(val) = positional.first() {
        format!("√{}", render_templates(val))
    } else {
        "√".to_string()
    }
}

/// [Section link](https://en.wikipedia.org/wiki/Template:Section_link)
fn render_section_link_template(params: &str) -> String {
    let positional = template_positional_params(params)
        .into_iter()
        .map(|param| render_templates(&param))
        .collect::<Vec<_>>();

    if positional.is_empty() {
        return String::new();
    }

    let first = &positional[0];
    let (page, first_section) = if let Some((p, s)) = first.split_once('#') {
        (p.trim(), Some(s.trim()))
    } else {
        (first.trim(), None)
    };

    let mut sections = Vec::new();
    if let Some(s) = first_section.filter(|s| !s.is_empty()) {
        sections.push(s.to_string());
    }
    for sec in positional.iter().skip(1) {
        let sec_trimmed = sec.trim();
        if !sec_trimmed.is_empty() {
            sections.push(sec_trimmed.to_string());
        }
    }

    if sections.is_empty() {
        format!("[[{page}]]")
    } else {
        let target_section = &sections[0];
        let target = if page.is_empty() {
            format!("#{target_section}")
        } else {
            format!("{page}#{target_section}")
        };

        let label = if page.is_empty() {
            format!("§ {}", sections.join(" § "))
        } else {
            format!("{page} § {}", sections.join(" § "))
        };

        format!("[[{target}|{label}]]")
    }
}

/// [mset](https://en.wikipedia.org/wiki/Template:Mset)
fn render_mset_template(params: &str) -> String {
    let positional = template_positional_params(params)
        .into_iter()
        .map(|param| render_templates(&param))
        .collect::<Vec<_>>();
    format!("{{{}}}", positional.join(", "))
}

/// [hidden begin](https://en.wikipedia.org/wiki/Template:Hidden_begin)
fn render_hidden_begin_template(params: &str) -> String {
    let named = template_named_params(params);
    let title = template_param(&named, &["title", "header", "1"])
        .map(str::to_string)
        .or_else(|| {
            let positional = template_positional_params(params);
            positional.first().cloned()
        })
        .unwrap_or_else(|| "Show".to_string());

    format!("\n'''{}'''\n", title.trim())
}

/// [Collapse top](https://en.wikipedia.org/wiki/Template:Collapse_top)
fn render_collapse_top_template(params: &str) -> String {
    let named = template_named_params(params);
    let title = template_param(&named, &["title", "header", "1"])
        .map(str::to_string)
        .or_else(|| {
            let positional = template_positional_params(params);
            positional.first().cloned()
        })
        .unwrap_or_else(|| "Extended content".to_string());

    format!("\n'''{}'''\n", title.trim())
}

/// [var](https://en.wikipedia.org/wiki/Template:Var)
fn render_var_template(params: &str) -> String {
    let named = template_named_params(params);
    let content = if let Some(val) = template_param(&named, &["1"]) {
        val.to_string()
    } else {
        let positional = template_positional_params(params);
        if let Some(val) = positional.first() {
            val.to_string()
        } else {
            String::new()
        }
    };

    if content.is_empty() {
        String::new()
    } else {
        format!(
            "__WIKIPEDIA_TO_EPUB_VAR_START__{}__WIKIPEDIA_TO_EPUB_VAR_END__",
            render_templates(&content)
        )
    }
}

/// [gaps](https://en.wikipedia.org/wiki/Template:Gaps)
fn render_gaps_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let mut parts = Vec::new();

    for p in positional {
        let trimmed = p.trim();
        if !trimmed.is_empty() {
            parts.push(render_templates(trimmed));
        }
    }

    let mut num_str = parts.join("__WIKIPEDIA_TO_EPUB_THINSP_TEMPLATE__");

    if let Some(e_val) = template_param(&named, &["e"]) {
        let base_val = template_param(&named, &["base"]).unwrap_or("10");
        let scientific = format!(
            "×{}__WIKIPEDIA_TO_EPUB_SUP_START__{}__WIKIPEDIA_TO_EPUB_SUP_END__",
            base_val,
            render_templates(e_val)
        );
        num_str.push_str(&scientific);
    }

    if let Some(unit) = template_param(&named, &["u"]) {
        num_str.push(' ');
        num_str.push_str(&render_templates(unit));
    }

    if let Some(lhs) = template_param(&named, &["lhs"]) {
        num_str = format!("{} = {}", render_templates(lhs), num_str);
    }

    num_str
}

/// [mvar](https://en.wikipedia.org/wiki/Template:Mvar)
fn render_mvar_template(params: &str) -> String {
    let positional = template_positional_params(params);
    if let Some(variable) = positional.first() {
        format!("''{}''", variable.trim())
    } else {
        String::new()
    }
}

/// [math](https://en.wikipedia.org/wiki/Template:Math)
fn render_math_template(params: &str) -> String {
    let named = template_named_params(params);
    let content = if let Some(val) = template_param(&named, &["1"]) {
        val.to_string()
    } else {
        let positional = template_positional_params(params);
        if let Some(val) = positional.first() {
            val.to_string()
        } else {
            String::new()
        }
    };

    if content.is_empty() {
        String::new()
    } else {
        render_templates(&content)
    }
}

/// [MathWorld](https://en.wikipedia.org/wiki/Template:MathWorld)
fn render_mathworld_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);
    let title = template_param(&named, &["title"])
        .or_else(|| positional.first().map(String::as_str))
        .unwrap_or("");
    if title.is_empty() {
        "Weisstein, Eric W. ''[[MathWorld]]''".to_string()
    } else {
        format!(
            "Weisstein, Eric W. \"{}\". ''[[MathWorld]]''",
            render_templates(title)
        )
    }
}

/// [AS ref](https://en.wikipedia.org/wiki/Template:AS_ref)
fn render_as_ref_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let section = positional.first().map(String::as_str).unwrap_or("");
    let page = positional.get(1).map(String::as_str).unwrap_or("");

    let mut parts = vec!["[[Abramowitz and Stegun]]".to_string()];
    if !page.is_empty() {
        parts.push(format!("p. {page}"));
    }
    if !section.is_empty() {
        parts.push(format!("§ {section}"));
    }
    parts.join(", ")
}

/// [OEIS2C](https://en.wikipedia.org/wiki/Template:OEIS2C)
fn render_oeis2c_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let id = positional.first().map(String::as_str).unwrap_or("");
    if id.is_empty() {
        String::new()
    } else {
        format!("[[oeis:{id}|{id}]]")
    }
}

/// [thinsp](https://en.wikipedia.org/wiki/Template:Thinsp)
fn render_thinsp_template(params: &str) -> String {
    let positional = template_positional_params(params)
        .into_iter()
        .map(|param| render_templates(&param))
        .collect::<Vec<_>>();

    if positional.is_empty() {
        "__WIKIPEDIA_TO_EPUB_THINSP_TEMPLATE__".to_string()
    } else {
        positional.join("__WIKIPEDIA_TO_EPUB_THINSP_TEMPLATE__")
    }
}

/// [dfn](https://en.wikipedia.org/wiki/Template:Dfn)
fn render_dfn_template(params: &str) -> String {
    let named = template_named_params(params);
    let content = if let Some(val) = template_param(&named, &["1"]) {
        val.to_string()
    } else {
        let positional = template_positional_params(params);
        if let Some(val) = positional.first() {
            val.to_string()
        } else {
            String::new()
        }
    };

    if content.is_empty() {
        String::new()
    } else {
        format!(
            "__WIKIPEDIA_TO_EPUB_DFN_START__{}__WIKIPEDIA_TO_EPUB_DFN_END__",
            render_templates(&content)
        )
    }
}

/// [subsup](https://en.wikipedia.org/wiki/Template:Subsup)
fn render_subsup_template(params: &str) -> String {
    let positional = template_positional_params(params);
    if positional.is_empty() {
        return String::new();
    }
    let base = render_templates(&positional[0]);
    let sub = positional
        .get(1)
        .map(|s| render_templates(s))
        .unwrap_or_default();
    let sup = positional
        .get(2)
        .map(|s| render_templates(s))
        .unwrap_or_default();

    let mut result = base;
    if !sub.is_empty() {
        result.push_str(&format!(
            "__WIKIPEDIA_TO_EPUB_SUB_START__{sub}__WIKIPEDIA_TO_EPUB_SUB_END__"
        ));
    }
    if !sup.is_empty() {
        result.push_str(&format!(
            "__WIKIPEDIA_TO_EPUB_SUP_START__{sup}__WIKIPEDIA_TO_EPUB_SUP_END__"
        ));
    }
    result
}

/// [abs](https://en.wikipedia.org/wiki/Template:Abs)
fn render_abs_template(params: &str) -> String {
    let named = template_named_params(params);
    let content = if let Some(val) = template_param(&named, &["1"]) {
        val.to_string()
    } else {
        let positional = template_positional_params(params);
        if let Some(val) = positional.first() {
            val.to_string()
        } else {
            String::new()
        }
    };

    if content.is_empty() {
        String::new()
    } else {
        format!("&#124;{}&#124;", render_templates(&content))
    }
}

/// [mono](https://en.wikipedia.org/wiki/Template:Mono)
fn render_mono_template(params: &str) -> String {
    let named = template_named_params(params);
    let content = if let Some(val) = template_param(&named, &["1"]) {
        val.to_string()
    } else {
        let positional = template_positional_params(params);
        if let Some(val) = positional.first() {
            val.to_string()
        } else {
            String::new()
        }
    };

    if content.is_empty() {
        String::new()
    } else {
        format!(
            "__WIKIPEDIA_TO_EPUB_CODE_START__{}__WIKIPEDIA_TO_EPUB_CODE_END__",
            render_templates(&content)
        )
    }
}

/// [pi](https://en.wikipedia.org/wiki/Template:Pi)
fn render_pi_template(_params: &str) -> String {
    "π".to_string()
}

/// [Springer](https://en.wikipedia.org/wiki/Template:Springer)
fn render_springer_template(params: &str) -> String {
    let named = template_named_params(params);
    let title = template_param(&named, &["title", "1"]).unwrap_or("");
    if title.is_empty() {
        "''[[Encyclopedia of Mathematics]]'', Springer".to_string()
    } else {
        format!(
            "\"{}\", ''[[Encyclopedia of Mathematics]]'', Springer",
            render_templates(title)
        )
    }
}

/// [JSTOR](https://en.wikipedia.org/wiki/Template:JSTOR)
fn render_jstor_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);
    let jstor_id = template_param(&named, &["1"])
        .or_else(|| positional.first().map(String::as_str))
        .unwrap_or("");
    if jstor_id.is_empty() {
        String::new()
    } else {
        format!("JSTOR {}", render_templates(jstor_id))
    }
}

/// [wsPSM](https://en.wikipedia.org/wiki/Template:WsPSM)
fn render_wspsm_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let title = positional.first().map(String::as_str).unwrap_or("");
    let volume = positional.get(1).map(String::as_str).unwrap_or("");
    let date = positional.get(2).map(String::as_str).unwrap_or("");

    let first = template_param(&named, &["first"]).unwrap_or("");
    let last = template_param(&named, &["last"]).unwrap_or("");

    let author = if !last.is_empty() && !first.is_empty() {
        format!("{last}, {first}")
    } else if !first.is_empty() {
        first.to_string()
    } else if !last.is_empty() {
        last.to_string()
    } else {
        String::new()
    };

    let mut parts = Vec::new();
    if !author.is_empty() {
        if !date.is_empty() {
            parts.push(format!("{author} ({date})"));
        } else {
            parts.push(author);
        }
    } else if !date.is_empty() {
        parts.push(format!("({date})"));
    }

    if !title.is_empty() {
        let ws_path = if !volume.is_empty() && !date.is_empty() {
            format!("Popular Science Monthly/Volume {volume}/{date}/{title}")
        } else {
            format!("Popular Science Monthly/{title}")
        };
        parts.push(format!("\"[[src:{ws_path}|{title}]]\""));
    }

    parts.push("''[[Popular Science Monthly]]''".to_string());

    if !volume.is_empty() {
        parts.push(format!("Vol. {volume}"));
    }

    parts.join(". ")
}

/// [em](https://en.wikipedia.org/wiki/Template:Em)
fn render_em_template(params: &str) -> String {
    let named = template_named_params(params);
    let content = if let Some(val) = template_param(&named, &["1"]) {
        val.to_string()
    } else {
        let positional = template_positional_params(params);
        if let Some(val) = positional.first() {
            val.to_string()
        } else {
            String::new()
        }
    };

    if content.is_empty() {
        String::new()
    } else {
        format!("''{}''", render_templates(&content))
    }
}

/// [nowrap](https://en.wikipedia.org/wiki/Template:Nowrap)
/// [center](https://en.wikipedia.org/wiki/Template:Center)
/// [crossreference](https://en.wikipedia.org/wiki/Template:Crossreference)
/// [nobold](https://en.wikipedia.org/wiki/Template:Nobold)
/// [stack](https://en.wikipedia.org/wiki/Template:Stack)
/// [xref](https://en.wikipedia.org/wiki/Template:Xref)
fn render_passthrough_template(params: &str) -> String {
    template_positional_params(params)
        .into_iter()
        .map(|param| render_templates(&param))
        .filter(|param| !param.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// [smaller](https://en.wikipedia.org/wiki/Template:Smaller)
/// [small](https://en.wikipedia.org/wiki/Template:Small)
fn render_smaller_template(params: &str) -> String {
    let text = render_passthrough_template(params);
    if text.is_empty() {
        return String::new();
    }

    format!("__WIKIPEDIA_TO_EPUB_SMALL_START__{text}__WIKIPEDIA_TO_EPUB_SMALL_END__")
}

/// [sic](https://en.wikipedia.org/wiki/Template:Sic)
fn render_sic_template(params: &str) -> String {
    let text = render_passthrough_template(params);
    if text.is_empty() {
        "[sic]".to_string()
    } else {
        format!("{text} [sic]")
    }
}

/// [circa](https://en.wikipedia.org/wiki/Template:Circa)
/// [c.](https://en.wikipedia.org/wiki/Template:C.)
/// [cx](https://en.wikipedia.org/wiki/Template:Cx)
fn render_circa_template(params: &str) -> String {
    let text = render_passthrough_template(params);
    if text.is_empty() {
        "c.".to_string()
    } else {
        format!("c. {text}")
    }
}

/// [isbn](https://en.wikipedia.org/wiki/Template:Isbn)
fn render_isbn_template(params: &str) -> String {
    let Some(isbn) = template_positional_params(params)
        .into_iter()
        .find(|value| !value.trim().is_empty())
    else {
        return String::new();
    };

    format!("ISBN {}", render_templates(&isbn))
}

/// [asin](https://en.wikipedia.org/wiki/Template:Asin)
fn render_asin_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let asin_id = template_param(&named, &["1"])
        .or_else(|| positional.first().map(String::as_str))
        .unwrap_or("");
    if asin_id.is_empty() {
        return String::new();
    }

    let mut parts = vec![format!("ASIN {}", asin_id)];

    if let Some(title) = template_param(&named, &["title"]) {
        parts.push(format!("''{}''", title));
    }

    let mut date_part = String::new();
    if let Some(date) = template_param(&named, &["date"]) {
        date_part = format!(" ({})", date);
    }

    let base = parts.join(", ");
    format!("{}{}", base, date_part)
}

/// [oclc](https://en.wikipedia.org/wiki/Template:Oclc)
fn render_oclc_template(params: &str) -> String {
    let Some(oclc) = template_positional_params(params)
        .into_iter()
        .find(|value| !value.trim().is_empty())
    else {
        return String::new();
    };

    format!("OCLC {}", render_templates(&oclc))
}

/// [abbr](https://en.wikipedia.org/wiki/Template:Abbr)
fn render_abbr_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .collect::<Vec<_>>();

    let Some(text) = params.first().filter(|value| !value.is_empty()) else {
        return String::new();
    };

    let Some(title) = params.get(1).filter(|value| !value.is_empty()) else {
        return render_templates(text);
    };

    format!(
        "__WIKIPEDIA_TO_EPUB_ABBR_START__{}__WIKIPEDIA_TO_EPUB_ABBR_VALUE__{}__WIKIPEDIA_TO_EPUB_ABBR_END__",
        render_templates(title),
        render_templates(text)
    )
}

/// [frac](https://en.wikipedia.org/wiki/Template:Frac)
/// [fraction](https://en.wikipedia.org/wiki/Template:Fraction)
fn render_frac_template(params: &str) -> String {
    let params = template_positional_params(params)
        .into_iter()
        .map(|param| render_templates(&param))
        .collect::<Vec<_>>();

    match params.as_slice() {
        [] => String::new(),
        [value] => value.clone(),
        [numerator, denominator] => format!("{numerator}/{denominator}"),
        [whole, numerator, denominator] => format!("{whole} {numerator}/{denominator}"),
        [first, rest @ ..] => format!("{first} {}", rest.join("/")),
    }
}

/// [floruit](https://en.wikipedia.org/wiki/Template:Floruit)
fn render_floruit_template(params: &str) -> String {
    let text = render_passthrough_template(params);
    if text.is_empty() {
        "fl.".to_string()
    } else {
        format!("fl. {text}")
    }
}

/// [coord](https://en.wikipedia.org/wiki/Template:Coord)
fn render_coord_template(params: &str) -> String {
    let named = template_named_params(params);
    // For now both inline and title will display inline
    if let Some(display) = template_param(&named, &["display"]) {
        let shows_inline = display.split([',', ';']).any(|value| {
            value.trim().eq_ignore_ascii_case("inline")
                || value.trim().eq_ignore_ascii_case("title")
        });
        if !shows_inline {
            return String::new();
        }
    }

    let positional = split_template_params(params)
        .into_iter()
        .map(|param| render_templates(param.trim()).trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    format_coord_components(&positional).unwrap_or_default()
}

fn format_coord_components(params: &[String]) -> Option<String> {
    format_hemisphere_coordinates(params).or_else(|| format_decimal_coordinates(params))
}

fn format_hemisphere_coordinates(params: &[String]) -> Option<String> {
    let lat_hemisphere_index = params
        .iter()
        .position(|param| matches_direction(param, ['N', 'S']))?;
    if !(1..=3).contains(&lat_hemisphere_index) {
        return None;
    }

    let lon_hemisphere_index = params
        .iter()
        .skip(lat_hemisphere_index + 1)
        .position(|param| matches_direction(param, ['E', 'W']))
        .map(|index| index + lat_hemisphere_index + 1)?;
    let lon_component_count = lon_hemisphere_index.checked_sub(lat_hemisphere_index + 1)?;
    if !(1..=3).contains(&lon_component_count) {
        return None;
    }

    let latitude = format_coord_axis(
        &params[..lat_hemisphere_index],
        params[lat_hemisphere_index].chars().next()?,
    )?;
    let longitude = format_coord_axis(
        &params[lat_hemisphere_index + 1..lon_hemisphere_index],
        params[lon_hemisphere_index].chars().next()?,
    )?;

    Some(format!("{latitude} {longitude}"))
}

fn format_coord_axis(parts: &[String], hemisphere: char) -> Option<String> {
    if parts.is_empty()
        || parts.len() > 3
        || !parts.iter().all(|part| coord_component_is_number(part))
    {
        return None;
    }

    let mut rendered = String::new();
    rendered.push_str(parts.first()?.trim());
    rendered.push('°');

    if let Some(minutes) = parts.get(1) {
        rendered.push_str(minutes.trim());
        rendered.push('′');
    }
    if let Some(seconds) = parts.get(2) {
        rendered.push_str(seconds.trim());
        rendered.push('″');
    }

    rendered.push(hemisphere.to_ascii_uppercase());
    Some(rendered)
}

fn format_decimal_coordinates(params: &[String]) -> Option<String> {
    let latitude = params.first()?.trim();
    let longitude = params.get(1)?.trim();
    if !coord_component_is_number(latitude) || !coord_component_is_number(longitude) {
        return None;
    }

    Some(format!("{latitude}, {longitude}"))
}

fn coord_component_is_number(value: &str) -> bool {
    value.trim().parse::<f64>().is_ok()
}

fn matches_direction(value: &str, allowed: [char; 2]) -> bool {
    let trimmed = value.trim();
    trimmed.len() == 1
        && trimmed.chars().next().is_some_and(|ch| {
            allowed
                .iter()
                .any(|direction| ch.eq_ignore_ascii_case(direction))
        })
}

/// [worldhistory](https://en.wikipedia.org/wiki/Template:Worldhistory)
fn render_worldhistory_template(params: &str) -> String {
    let named = template_named_params(params);
    let mut parts = Vec::new();

    if let Some(quote) = template_param(&named, &["quote"]) {
        parts.push(format!("\"{}\"", render_templates(quote)));
    } else {
        parts.push("Citation".to_string());
    }

    parts.push("''The Encyclopedia of World History'' (6th ed.)".to_string());
    parts.join(". ")
}

/// [Shy](https://en.wikipedia.org/wiki/Template:Shy)
fn render_soft_hyphen_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let parts = positional
        .into_iter()
        .filter(|t| !t.trim().is_empty())
        .collect::<Vec<_>>();
    parts.join("\u{00ad}")
}

/// [color box](https://en.wikipedia.org/wiki/Template:Color_box)
fn render_color_box_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(color) = positional.first().filter(|c| !c.trim().is_empty()) else {
        return "■".to_string();
    };
    let color = color.trim();
    format!("__WIKIPEDIA_TO_EPUB_COLOR_BOX_START__{color}__WIKIPEDIA_TO_EPUB_COLOR_BOX_END__")
}

/// [color](https://en.wikipedia.org/wiki/Template:Color)
/// [colour](https://en.wikipedia.org/wiki/Template:Colour)
fn render_color_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let color = template_param(&named, &["1", "color", "colour"])
        .map(|s| s.to_string())
        .or_else(|| positional.first().cloned())
        .unwrap_or_default();

    let text = template_param(&named, &["2", "text", "content"])
        .map(|s| s.to_string())
        .or_else(|| positional.get(1).cloned())
        .unwrap_or_default();

    let color = color.trim();
    let text = text.trim();

    if color.is_empty() || text.is_empty() {
        return render_templates(text);
    }

    let rendered_text = render_templates(text);
    format!(
        "__WIKIPEDIA_TO_EPUB_COLOR_START__{color}__WIKIPEDIA_TO_EPUB_COLOR_MID__{rendered_text}__WIKIPEDIA_TO_EPUB_COLOR_END__"
    )
}

/// [plainlist](https://en.wikipedia.org/wiki/Template:Plainlist)
fn render_plainlist_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let list_content = template_param(&named, &["1"])
        .map(|s| s.to_string())
        .or_else(|| positional.first().cloned())
        .unwrap_or_default();

    render_templates(list_content.trim())
}

/// [note](https://en.wikipedia.org/wiki/Template:Note)
fn render_note_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(label) = positional.get(1).filter(|l| !l.trim().is_empty()) else {
        return String::new();
    };
    format!("'''{}'''", render_templates(label.trim()))
}

/// [fs interlinear](https://en.wikipedia.org/wiki/Template:Fs_interlinear)
fn render_fs_interlinear_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let line1 = positional.first().map(|s| s.trim()).unwrap_or("");
    let line2 = positional.get(1).map(|s| s.trim()).unwrap_or("");
    let line3 = positional.get(2).map(|s| s.trim()).unwrap_or("");
    let line4 = positional.get(3).map(|s| s.trim()).unwrap_or("");

    if line1.is_empty() && line2.is_empty() && line3.is_empty() && line4.is_empty() {
        return String::new();
    }

    let line1_rendered = render_templates(line1);
    let line1_html = if let Some(lang) = template_param(&named, &["lang"]) {
        let lang = lang.trim();
        format!(
            "__WIKIPEDIA_TO_EPUB_LANG_START__{lang}__WIKIPEDIA_TO_EPUB_LANG_VALUE__{line1_rendered}__WIKIPEDIA_TO_EPUB_LANG_END__"
        )
    } else {
        line1_rendered
    };

    let line2_rendered = render_templates(line2);
    let line3_rendered = render_templates(line3);
    let line4_rendered = render_templates(line4);

    let mut html = String::new();
    html.push_str("__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_START__\n");
    if !line1_html.is_empty() {
        html.push_str(&format!(
            "__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__'''{}'''\n",
            line1_html
        ));
    }
    if !line2_rendered.is_empty() {
        html.push_str(&format!(
            "__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__''{}''\n",
            line2_rendered
        ));
    }
    if !line3_rendered.is_empty() {
        html.push_str(&format!(
            "__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__{}\n",
            line3_rendered
        ));
    }
    if !line4_rendered.is_empty() {
        let line4_formatted = if line4_rendered.starts_with('\'')
            && line4_rendered.ends_with('\'')
            && line4_rendered.len() > 1
        {
            format!(
                "''&#39;{}&#39;''",
                &line4_rendered[1..line4_rendered.len() - 1]
            )
        } else {
            format!("''{}''", line4_rendered)
        };
        html.push_str(&format!(
            "__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__{}\n",
            line4_formatted
        ));
    }
    html.push_str("__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_END__");
    html
}

/// [Tooltip](https://en.wikipedia.org/wiki/Template:Tooltip)
fn render_tooltip_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(text) = positional.first().filter(|t| !t.trim().is_empty()) else {
        return String::new();
    };
    let Some(title) = positional.get(1).filter(|t| !t.trim().is_empty()) else {
        return text.to_string();
    };
    let text = render_templates(text.trim());
    let title = render_templates(title.trim());
    format!(
        "__WIKIPEDIA_TO_EPUB_ABBR_START__{title}__WIKIPEDIA_TO_EPUB_ABBR_VALUE__{text}__WIKIPEDIA_TO_EPUB_ABBR_END__"
    )
}

/// [Jaanus](https://en.wikipedia.org/wiki/Template:Jaanus)
fn render_jaanus_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let path = template_param(&named, &["1", "path"])
        .map(|s| s.to_string())
        .or_else(|| positional.first().cloned())
        .unwrap_or_default();

    let label = template_param(&named, &["2", "label", "text"])
        .map(|s| s.to_string())
        .or_else(|| positional.get(1).cloned())
        .unwrap_or_default();

    let path = path.trim();
    let label = label.trim();

    if path.is_empty() {
        return String::new();
    }

    let resolved_label = if label.is_empty() { path } else { label };

    let url = format!("http://www.aisf.or.jp/~jaanus/deta/{}.htm", path);
    format!(
        "[[official-url:{}|{}]] at JAANUS",
        url,
        render_templates(resolved_label)
    )
}

/// [Easy CSS image crop](https://en.wikipedia.org/wiki/Template:Easy_CSS_image_crop)
fn render_easy_css_image_crop_template(params: &str) -> String {
    let named = template_named_params(params);
    let Some(image) = template_param(&named, &["Image", "image"]) else {
        return String::new();
    };
    let image = image.trim();
    if image.is_empty() {
        return String::new();
    }

    let caption = template_param(&named, &["caption", "Caption"])
        .map(|s| s.trim())
        .unwrap_or("");
    let alt = template_param(&named, &["alt", "Alt"])
        .map(|s| s.trim())
        .unwrap_or("");

    if alt.is_empty() {
        format!("[[File:{image}|thumb|{caption}]]")
    } else {
        format!("[[File:{image}|thumb|alt={alt}|{caption}]]")
    }
}

/// [Multiple images](https://en.wikipedia.org/wiki/Template:Multiple_images)
/// [Multiple image](https://en.wikipedia.org/wiki/Template:Multiple_image)
fn render_multiple_images_template(params: &str) -> String {
    let named = template_named_params(params);
    let mut rendered_images = Vec::new();

    let header = template_param(&named, &["header", "Header"])
        .map(|s| s.trim())
        .unwrap_or("");
    let footer = template_param(&named, &["footer", "Footer"])
        .map(|s| s.trim())
        .unwrap_or("");

    if !header.is_empty() {
        rendered_images.push(format!(
            "<p><strong>{}</strong></p>",
            render_templates(header)
        ));
    }

    for i in 1..=10 {
        let img_key = format!("image{i}");
        let cap_key = format!("caption{i}");
        let alt_key = format!("alt{i}");

        let Some(image) = named
            .get(&img_key)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        else {
            continue;
        };

        let caption = named.get(&cap_key).map(|s| s.trim()).unwrap_or("");
        let alt = named.get(&alt_key).map(|s| s.trim()).unwrap_or("");

        let file_link = if alt.is_empty() {
            if caption.is_empty() {
                format!("[[File:{image}|thumb]]")
            } else {
                format!("[[File:{image}|thumb|{caption}]]")
            }
        } else {
            if caption.is_empty() {
                format!("[[File:{image}|thumb|alt={alt}]]")
            } else {
                format!("[[File:{image}|thumb|alt={alt}|{caption}]]")
            }
        };

        rendered_images.push(render_templates(&file_link));
    }

    if !footer.is_empty() {
        rendered_images.push(format!("<p><em>{}</em></p>", render_templates(footer)));
    }

    rendered_images.join("\n")
}

/// [ISSN](https://en.wikipedia.org/wiki/Template:ISSN)
fn render_issn_template(params: &str) -> String {
    let Some(issn) = template_positional_params(params)
        .first()
        .filter(|v| !v.trim().is_empty())
        .cloned()
    else {
        return String::new();
    };
    format!("ISSN {}", render_templates(&issn))
}

pub(crate) fn format_number_with_commas(s: &str) -> String {
    let s = s.trim();
    if s.is_empty() {
        return String::new();
    }
    let (sign, rest) = if let Some(stripped) = s.strip_prefix('-') {
        ("-", stripped)
    } else if let Some(stripped) = s.strip_prefix('+') {
        ("+", stripped)
    } else {
        ("", s)
    };

    let parts: Vec<&str> = rest.split('.').collect();
    let integer_part = parts[0];

    if !integer_part.chars().all(|c| c.is_ascii_digit()) {
        return s.to_string();
    }

    let mut formatted_integer = String::new();
    let bytes = integer_part.as_bytes();
    let len = bytes.len();
    for (i, &byte) in bytes.iter().enumerate() {
        formatted_integer.push(byte as char);
        let remaining = len - 1 - i;
        if remaining > 0 && remaining.is_multiple_of(3) {
            formatted_integer.push(',');
        }
    }

    let mut result = format!("{}{}", sign, formatted_integer);
    if parts.len() > 1 {
        result.push('.');
        result.push_str(&parts[1..].join("."));
    }
    result
}

/// [formatnum](https://en.wikipedia.org/wiki/Template:Formatnum)
pub(crate) fn render_formatnum_template(template: &str, params: &str) -> String {
    let mut num_str = String::new();
    if let Some(colon_idx) = template.find(':') {
        num_str = template[colon_idx + 1..].to_string();
    } else {
        if let Some(first_param) = template_positional_params(params).first() {
            num_str = first_param.clone();
        }
    }
    format_number_with_commas(&num_str)
}

/// [doi](https://en.wikipedia.org/wiki/Template:Doi)
fn render_doi_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let named = template_named_params(params);
    let Some(doi) = positional
        .first()
        .cloned()
        .or_else(|| named.get("1").cloned())
    else {
        return String::new();
    };
    format!("doi:{}", render_templates(&doi))
}

fn calculate_age(y1: i32, m1: i32, d1: i32, y2: i32, m2: i32, d2: i32) -> i32 {
    let mut age = y2 - y1;
    if y1 < 0 && y2 > 0 {
        age -= 1;
    }
    if m2 < m1 || (m2 == m1 && d2 < d1) {
        age -= 1;
    }
    age
}

/// [age](https://en.wikipedia.org/wiki/Template:Age)
fn render_age_template(params: &str) -> String {
    let positional = template_positional_params(params);

    let nums: Vec<i32> = positional
        .iter()
        .map(|s| s.parse::<i32>().unwrap_or(0))
        .collect();

    if nums.len() >= 6 {
        let y1 = nums[0];
        let m1 = nums[1];
        let d1 = nums[2];
        let y2 = nums[3];
        let m2 = nums[4];
        let d2 = nums[5];
        let age = calculate_age(y1, m1, d1, y2, m2, d2);
        age.to_string()
    } else if nums.len() >= 3 {
        let y1 = nums[0];
        let m1 = nums[1];
        let d1 = nums[2];
        let (y2, m2, d2) = current_utc_date();
        let age = calculate_age(y1, m1, d1, y2, m2, d2);
        age.to_string()
    } else {
        String::new()
    }
}

/// [Birth date and age](https://en.wikipedia.org/wiki/Template:Birth_date_and_age)
/// [birth date and age](https://en.wikipedia.org/wiki/Template:Birth_date_and_age)
fn render_birth_date_and_age_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let nums: Vec<i32> = positional
        .iter()
        .map(|s| s.parse::<i32>().unwrap_or(0))
        .collect();

    if nums.len() >= 3 {
        let y = nums[0];
        let m = nums[1];
        let d = nums[2];

        let month_names = [
            "",
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ];

        let month_name = if (1..=12).contains(&m) {
            month_names[m as usize]
        } else {
            ""
        };

        let (cy, cm, cd) = current_utc_date();
        let age = calculate_age(y, m, d, cy, cm, cd);

        let named = template_named_params(params);
        let df_dmy = template_param(&named, &["df"])
            .is_some_and(|v| v.eq_ignore_ascii_case("yes") || v.eq_ignore_ascii_case("dmy"));

        if df_dmy {
            format!("{} {} {} (age {})", d, month_name, y, age)
        } else {
            format!("{} {}, {} (age {})", month_name, d, y, age)
        }
    } else {
        String::new()
    }
}

/// [unbulleted list](https://en.wikipedia.org/wiki/Template:Unbulleted_list)
/// [ubl](https://en.wikipedia.org/wiki/Template:Ubl)
/// [ubli](https://en.wikipedia.org/wiki/Template:Ubli)
/// [unbulleted indent list](https://en.wikipedia.org/wiki/Template:Unbulleted_indent_list)
fn render_unbulleted_list_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let mut items = Vec::new();
    for param in positional {
        let trimmed = param.trim();
        if !trimmed.is_empty() {
            items.push(format!("* {}", render_templates(trimmed)));
        }
    }
    if items.is_empty() {
        String::new()
    } else {
        format!("\n{}", items.join("\n"))
    }
}

/// [native name list](https://en.wikipedia.org/wiki/Template:Native_name_list)
fn render_native_name_list_template(params: &str) -> String {
    let named = template_named_params(params);
    let mut parts = Vec::new();
    for i in 1..=10 {
        let tag_key = format!("tag{}", i);
        let name_key = format!("name{}", i);
        if let Some(name) = named.get(&name_key) {
            let rendered_name = render_templates(name);
            if let Some(tag) = named.get(&tag_key) {
                let tag_trimmed = tag.trim().to_lowercase();
                let lang_name = match tag_trimmed.as_str() {
                    "ja" => "Japanese",
                    "ko" => "Korean",
                    "zh" => "Chinese",
                    "en" => "English",
                    other => other,
                };
                parts.push(format!("{} ({})", rendered_name, lang_name));
            } else {
                parts.push(rendered_name);
            }
        }
    }
    parts.join(", ")
}

/// [hlist](https://en.wikipedia.org/wiki/Template:Hlist)
/// [flatlist](https://en.wikipedia.org/wiki/Template:Flatlist)
fn render_hlist_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let mut items = Vec::new();
    for param in positional {
        let trimmed = param.trim();
        if !trimmed.is_empty() {
            items.push(render_templates(trimmed));
        }
    }
    items.join(", ")
}

/// [Infobox mountain](https://en.wikipedia.org/wiki/Template:Infobox_mountain)
fn render_infobox_mountain_template(params: &str) -> String {
    let named = template_named_params(params);
    let mut rows = Vec::new();

    rows.push("{| class=\"wikitable\"".to_string());

    // 1. Name
    if let Some(name) = template_param(&named, &["name"]) {
        rows.push("|-".to_string());
        rows.push("! Name".to_string());
        rows.push(format!("| {}", render_templates(name)));
    }

    // 2. Native name
    if let Some(native_name) = template_param(&named, &["native_name"]) {
        rows.push("|-".to_string());
        rows.push("! Native name".to_string());
        rows.push(format!("| {}", render_templates(native_name)));
    }

    // 3. Other name
    if let Some(other_name) = template_param(&named, &["other_name"]) {
        rows.push("|-".to_string());
        rows.push("! Other name".to_string());
        rows.push(format!("| {}", render_templates(other_name)));
    }

    // 4. Image
    if let Some(image) = template_param(&named, &["image"]) {
        let caption = template_param(&named, &["image_caption"]).unwrap_or("");
        rows.push("|-".to_string());
        rows.push("! Image".to_string());
        if caption.is_empty() {
            rows.push(format!("| [[File:{}]]", image));
        } else {
            rows.push(format!("| [[File:{}|{}]]", image, caption));
        }
    }

    let add_row = |rows: &mut Vec<String>, label: &str, keys: &[&str]| {
        if let Some(val) = template_param(&named, keys) {
            let rendered_val = render_templates(val);
            rows.push("|-".to_string());
            rows.push(format!("! {}", label));
            rows.push(format!("| {}", rendered_val));
        }
    };

    // 5. Country
    add_row(&mut rows, "Country", &["country"]);

    // 6. Subdivision
    let subdivision_label = template_param(&named, &["subdivision1_type"]).unwrap_or("Subdivision");
    add_row(&mut rows, subdivision_label, &["subdivision1"]);

    // 7. Highest point
    add_row(&mut rows, "Highest point", &["highest"]);

    // 8. Highest location
    add_row(&mut rows, "Highest location", &["highest_location"]);

    // 9. Elevation
    add_row(&mut rows, "Elevation", &["elevation_m", "elevation"]);

    // 10. Coordinates
    add_row(&mut rows, "Coordinates", &["coordinates"]);

    // 11. Geology
    add_row(&mut rows, "Geology", &["geology"]);

    // 12. Orogeny
    add_row(&mut rows, "Orogeny", &["orogeny"]);

    // 13. Dimensions
    add_row(&mut rows, "Length", &["length_km", "length"]);
    add_row(&mut rows, "Width", &["width_km", "width"]);
    add_row(&mut rows, "Area", &["area_km2", "area"]);

    rows.push("|}".to_string());
    rows.join("\n")
}

/// [Infobox country](https://en.wikipedia.org/wiki/Template:Infobox_country)
fn render_infobox_country_template(params: &str) -> String {
    let named = template_named_params(params);
    let mut rows = Vec::new();

    rows.push("{| class=\"wikitable\"".to_string());

    let add_row = |rows: &mut Vec<String>, label: &str, keys: &[&str]| {
        if let Some(val) = template_param(&named, keys) {
            rows.push("|-".to_string());
            rows.push(format!("! {}", label));
            rows.push(format!("| {}", render_templates(val)));
        }
    };

    add_row(
        &mut rows,
        "Name",
        &["conventional_long_name", "common_name", "name"],
    );
    add_row(&mut rows, "Common name", &["common_name"]);
    add_row(&mut rows, "Native name", &["native_name"]);

    if let Some(image_flag) = template_param(&named, &["image_flag"]) {
        rows.push("|-".to_string());
        rows.push("! Flag".to_string());
        rows.push(format!("| {}", render_templates(image_flag)));
    }

    if let Some(image_coat) = template_param(&named, &["image_coat"]) {
        rows.push("|-".to_string());
        let label = template_param(&named, &["symbol_type"]).unwrap_or("Symbol");
        rows.push(format!("! {}", render_templates(label)));
        rows.push(format!("| {}", render_templates(image_coat)));
    }

    if let Some(other_symbol) = template_param(&named, &["other_symbol"]) {
        rows.push("|-".to_string());
        let label = template_param(&named, &["other_symbol_type"]).unwrap_or("Other symbol");
        rows.push(format!("! {}", render_templates(label)));
        rows.push(format!("| {}", render_templates(other_symbol)));
    }

    add_row(&mut rows, "Anthem", &["anthem", "national_anthem"]);
    add_row(&mut rows, "Motto", &["national_motto"]);
    add_row(&mut rows, "Status", &["status"]);
    add_row(&mut rows, "Government type", &["government_type"]);
    add_row(&mut rows, "Capital", &["capital"]);
    add_row(&mut rows, "Largest city", &["largest_city"]);
    add_row(&mut rows, "Coordinates", &["coordinates"]);
    add_row(
        &mut rows,
        "Official languages",
        &["official_languages", "common_languages", "languages"],
    );

    if let (Some(label), Some(value)) = (
        template_param(&named, &["languages_type", "languages2_type"]),
        template_param(&named, &["languages", "languages2"]),
    ) {
        rows.push("|-".to_string());
        rows.push(format!("! {}", render_templates(label)));
        rows.push(format!("| {}", render_templates(value)));
    }

    add_row(&mut rows, "Ethnic groups", &["ethnic_groups"]);
    add_row(&mut rows, "Demonym", &["demonym"]);
    add_row(&mut rows, "Religion", &["religion"]);
    add_row(&mut rows, "Currency", &["currency"]);
    add_row(&mut rows, "Area", &["area_km2", "stat_area1"]);
    add_row(
        &mut rows,
        "Population",
        &["population_total", "stat_pop1", "stat_pop2"],
    );
    add_row(&mut rows, "Year established", &["year_start"]);
    add_row(&mut rows, "Year ended", &["year_end"]);
    add_row(&mut rows, "Preceded by", &["p1"]);
    add_row(&mut rows, "Succeeded by", &["s1"]);
    add_row(&mut rows, "Today", &["today"]);

    for idx in 1..=7 {
        let event_key = if idx == 1 {
            "event_start".to_string()
        } else {
            format!("event{}", idx - 1)
        };
        let date_key = if idx == 1 {
            "date_start".to_string()
        } else {
            format!("date_event{}", idx - 1)
        };
        if let Some(event) = template_param_owned(&named, &[event_key]) {
            let date = template_param_owned(&named, &[date_key]).unwrap_or_default();
            rows.push("|-".to_string());
            rows.push("! Event".to_string());
            if date.is_empty() {
                rows.push(format!("| {}", render_templates(&event)));
            } else {
                rows.push(format!(
                    "| {} ({})",
                    render_templates(&event),
                    render_templates(&date)
                ));
            }
        }
    }

    if let Some(event_end) = template_param(&named, &["event_end"]) {
        let date_end = template_param(&named, &["date_end"]).unwrap_or("");
        rows.push("|-".to_string());
        rows.push("! End".to_string());
        if date_end.is_empty() {
            rows.push(format!("| {}", render_templates(event_end)));
        } else {
            rows.push(format!(
                "| {} ({})",
                render_templates(event_end),
                render_templates(date_end)
            ));
        }
    }

    rows.push("|}".to_string());
    rows.join("\n")
}

/// [Infobox military conflict](https://en.wikipedia.org/wiki/Template:Infobox_military_conflict)
fn render_infobox_military_conflict_template(params: &str) -> String {
    let named = template_named_params(params);
    let mut rows = Vec::new();

    rows.push("{| class=\"wikitable\"".to_string());

    let add_row = |rows: &mut Vec<String>, label: &str, keys: &[&str]| {
        if let Some(val) = template_param(&named, keys) {
            rows.push("|-".to_string());
            rows.push(format!("! {}", label));
            rows.push(format!("| {}", render_templates(val)));
        }
    };

    add_row(&mut rows, "Conflict", &["conflict"]);
    add_row(&mut rows, "Part of", &["partof"]);

    if let Some(image) = template_param(&named, &["image"]) {
        rows.push("|-".to_string());
        rows.push("! Image".to_string());
        let trimmed_image = image.trim();
        let formatted_image = if trimmed_image.starts_with("[[") || trimmed_image.starts_with("{{")
        {
            trimmed_image.to_string()
        } else {
            format!("[[File:{}|thumb]]", trimmed_image)
        };

        if let Some(caption) = template_param(&named, &["caption", "footer"]) {
            rows.push(format!(
                "| {}__WIKIPEDIA_TO_EPUB_BR__{}",
                render_templates(&formatted_image),
                render_templates(caption)
            ));
        } else {
            rows.push(format!("| {}", render_templates(&formatted_image)));
        }
    }

    add_row(&mut rows, "Date", &["date"]);
    add_row(&mut rows, "Place", &["place"]);
    add_row(&mut rows, "Territorial changes", &["territory"]);
    add_row(&mut rows, "Result", &["result"]);
    add_row(&mut rows, "Combatant 1", &["combatant1"]);
    add_row(&mut rows, "Combatant 2", &["combatant2"]);
    add_row(&mut rows, "Commander 1", &["commander1"]);
    add_row(&mut rows, "Commander 2", &["commander2"]);
    add_row(&mut rows, "Strength 1", &["strength1"]);
    add_row(&mut rows, "Strength 2", &["strength2"]);
    add_row(&mut rows, "Casualties 1", &["casualties1"]);
    add_row(&mut rows, "Casualties 2", &["casualties2"]);
    add_row(&mut rows, "Casualties 3", &["casualties3"]);
    add_row(&mut rows, "Notes", &["notes"]);

    rows.push("|}".to_string());
    rows.join("\n")
}

/// [Infobox planet](https://en.wikipedia.org/wiki/Template:Infobox_planet)
fn render_infobox_planet_template(params: &str) -> String {
    fn render_infobox_file_link_label(value: &str) -> String {
        let trimmed = value.trim();
        if !(trimmed.starts_with("[[File:") || trimmed.starts_with("[[Image:"))
            || !trimmed.ends_with("]]")
        {
            return render_templates(trimmed);
        }

        let inner = &trimmed[2..trimmed.len() - 2];
        let parts = split_template_params(inner)
            .into_iter()
            .map(|part| part.trim().to_string())
            .collect::<Vec<_>>();

        let display = parts
            .iter()
            .skip(1)
            .rev()
            .find(|part| {
                let value = part.trim();
                !value.is_empty()
                    && !value.contains('=')
                    && !matches!(
                        value.to_ascii_lowercase().as_str(),
                        "thumb"
                            | "thumbnail"
                            | "right"
                            | "left"
                            | "center"
                            | "frame"
                            | "frameless"
                            | "border"
                    )
                    && !value.to_ascii_lowercase().ends_with("px")
            })
            .cloned();

        display
            .map(|value| render_templates(&value))
            .unwrap_or_default()
    }

    let named = template_named_params(params);
    let mut rows = Vec::new();

    rows.push("{| class=\"wikitable\"".to_string());

    if let Some(name) = template_param(&named, &["name"]) {
        rows.push("|-".to_string());
        rows.push("! Name".to_string());
        rows.push(format!("| {}", render_templates(name)));
    }

    if let Some(symbol) = template_param(&named, &["symbol"]) {
        rows.push("|-".to_string());
        rows.push("! Symbol".to_string());
        rows.push(format!("| {}", render_infobox_file_link_label(symbol)));
    }

    if let Some(image) = template_param(&named, &["image"]) {
        rows.push("|-".to_string());
        rows.push("! Image".to_string());
        if let Some(caption) = template_param(&named, &["caption"]) {
            rows.push(format!(
                "| {}__WIKIPEDIA_TO_EPUB_BR__{}",
                render_templates(image),
                render_templates(caption)
            ));
        } else {
            rows.push(format!("| {}", render_templates(image)));
        }
    }

    let add_row = |rows: &mut Vec<String>, label: &str, keys: &[&str]| {
        if let Some(val) = template_param(&named, keys) {
            rows.push("|-".to_string());
            rows.push(format!("! {}", label));
            rows.push(format!("| {}", render_templates(val)));
        }
    };

    add_row(&mut rows, "Alternative names", &["alt_names"]);
    add_row(&mut rows, "Named after", &["named_after"]);
    add_row(&mut rows, "Adjectives", &["adjectives"]);
    add_row(&mut rows, "Pronunciation", &["pronounced"]);

    add_row(&mut rows, "Epoch", &["epoch"]);
    add_row(&mut rows, "Aphelion", &["aphelion"]);
    add_row(&mut rows, "Perihelion", &["perihelion"]);
    add_row(&mut rows, "Time of perihelion", &["time_periastron"]);
    add_row(&mut rows, "Semi-major axis", &["semimajor"]);
    add_row(&mut rows, "Eccentricity", &["eccentricity"]);
    add_row(&mut rows, "Orbital period", &["period"]);
    add_row(&mut rows, "Synodic period", &["synodic_period"]);
    add_row(&mut rows, "Average speed", &["avg_speed"]);
    add_row(&mut rows, "Mean anomaly", &["mean_anomaly"]);
    add_row(&mut rows, "Inclination", &["inclination"]);
    add_row(&mut rows, "Ascending node", &["asc_node"]);
    add_row(&mut rows, "Argument of perihelion", &["arg_peri"]);
    add_row(&mut rows, "Satellites", &["satellites"]);

    add_row(&mut rows, "Mean radius", &["mean_radius"]);
    add_row(&mut rows, "Equatorial radius", &["equatorial_radius"]);
    add_row(&mut rows, "Polar radius", &["polar_radius"]);
    add_row(&mut rows, "Flattening", &["flattening"]);
    add_row(&mut rows, "Circumference", &["circumference"]);
    add_row(&mut rows, "Surface area", &["surface_area"]);
    add_row(&mut rows, "Volume", &["volume"]);
    add_row(&mut rows, "Mass", &["mass"]);
    add_row(&mut rows, "Density", &["density"]);
    add_row(&mut rows, "Surface gravity", &["surface_grav"]);
    add_row(
        &mut rows,
        "Moment of inertia factor",
        &["moment_of_inertia_factor"],
    );
    add_row(&mut rows, "Escape velocity", &["escape_velocity"]);
    add_row(&mut rows, "Rotation period", &["rotation"]);
    add_row(&mut rows, "Sidereal day", &["sidereal_day"]);
    add_row(&mut rows, "Rotational velocity", &["rot_velocity"]);
    add_row(&mut rows, "Axial tilt", &["axial_tilt"]);
    add_row(
        &mut rows,
        "North pole right ascension",
        &["right_asc_north_pole"],
    );
    add_row(&mut rows, "North pole declination", &["declination"]);
    add_row(&mut rows, "Albedo", &["albedo"]);
    add_row(&mut rows, "Magnitude", &["magnitude"]);
    add_row(&mut rows, "Absolute magnitude", &["abs_magnitude"]);
    add_row(&mut rows, "Angular size", &["angular_size"]);
    add_row(&mut rows, "Single temperature", &["single_temperature"]);

    for idx in 1..=3 {
        let temp_name_key = format!("temp_name{}", idx);
        let min_temp_key = format!("min_temp_{}", idx);
        let mean_temp_key = format!("mean_temp_{}", idx);
        let max_temp_key = format!("max_temp_{}", idx);

        let name = template_param_owned(&named, &[temp_name_key])
            .unwrap_or_else(|| format!("Temperature {idx}"));
        let mut temp_parts = Vec::new();
        if let Some(value) = template_param_owned(&named, &[min_temp_key]) {
            temp_parts.push(format!("min {}", render_templates(&value)));
        }
        if let Some(value) = template_param_owned(&named, &[mean_temp_key]) {
            temp_parts.push(format!("mean {}", render_templates(&value)));
        }
        if let Some(value) = template_param_owned(&named, &[max_temp_key]) {
            temp_parts.push(format!("max {}", render_templates(&value)));
        }
        if !temp_parts.is_empty() {
            rows.push("|-".to_string());
            rows.push(format!("! {}", render_templates(&name)));
            rows.push(format!("| {}", temp_parts.join("__WIKIPEDIA_TO_EPUB_BR__")));
        }
    }

    add_row(
        &mut rows,
        "Surface equivalent dose rate",
        &["surface_equivalent_dose_rate"],
    );
    add_row(
        &mut rows,
        "Surface absorbed dose rate",
        &["surface_absorbed_dose_rate"],
    );
    add_row(&mut rows, "Atmosphere", &["atmosphere"]);
    add_row(&mut rows, "Surface pressure", &["surface_pressure"]);
    add_row(
        &mut rows,
        "Atmosphere composition",
        &["atmosphere_composition"],
    );
    add_row(&mut rows, "Notes", &["note"]);

    rows.push("|}".to_string());
    rows.join("\n")
}

/// [Infobox settlement](https://en.wikipedia.org/wiki/Template:Infobox_settlement)
fn render_infobox_settlement_template(params: &str) -> String {
    let named = template_named_params(params);
    let mut rows = Vec::new();

    rows.push("{| class=\"wikitable\"".to_string());

    if let Some(name) = template_param(&named, &["name"]) {
        rows.push("|-".to_string());
        rows.push("! Name".to_string());
        rows.push(format!("| {}", render_templates(name)));
    }

    if let Some(official_name) = template_param(&named, &["official_name"]) {
        rows.push("|-".to_string());
        rows.push("! Official name".to_string());
        rows.push(format!("| {}", render_templates(official_name)));
    }

    if let Some(native_name) = template_param(&named, &["native_name"]) {
        rows.push("|-".to_string());
        rows.push("! Native name".to_string());
        rows.push(format!("| {}", render_templates(native_name)));
    }

    if let Some(settlement_type) = template_param(&named, &["settlement_type"]) {
        rows.push("|-".to_string());
        rows.push("! Settlement type".to_string());
        rows.push(format!("| {}", render_templates(settlement_type)));
    }

    let add_row = |rows: &mut Vec<String>, label: &str, keys: &[&str]| {
        if let Some(val) = template_param(&named, keys) {
            let rendered_val = render_templates(val);
            rows.push("|-".to_string());
            rows.push(format!("! {}", label));
            rows.push(format!("| {}", rendered_val));
        }
    };

    // Images: image_skyline
    if let Some(image) = template_param(&named, &["image_skyline"]) {
        rows.push("|-".to_string());
        rows.push("! Image".to_string());
        rows.push(format!("| {}", render_templates(image)));
    }

    for i in 1..=4 {
        let type_key = if i == 1 {
            "subdivision_type".to_string()
        } else {
            format!("subdivision_type{}", i - 1)
        };
        let name_key = if i == 1 {
            "subdivision_name".to_string()
        } else {
            format!("subdivision_name{}", i - 1)
        };
        if let (Some(label), Some(val)) = (
            template_param(&named, &[&type_key]),
            template_param(&named, &[&name_key]),
        ) {
            rows.push("|-".to_string());
            rows.push(format!("! {}", render_templates(label)));
            rows.push(format!("| {}", render_templates(val)));
        }
    }

    add_row(&mut rows, "Governing body", &["governing_body"]);

    if let (Some(leader_title), Some(leader_name)) = (
        template_param(&named, &["leader_title"]),
        template_param(&named, &["leader_name"]),
    ) {
        rows.push("|-".to_string());
        rows.push(format!("! {}", render_templates(leader_title)));
        rows.push(format!("| {}", render_templates(leader_name)));
    }

    add_row(&mut rows, "Area", &["area_total_km2"]);
    add_row(&mut rows, "Population", &["population_total"]);
    add_row(&mut rows, "Density", &["population_density_km2"]);
    add_row(&mut rows, "Time zone", &["timezone1"]);
    add_row(&mut rows, "Coordinates", &["coordinates"]);

    for sec in 1..=5 {
        let sec_name_key = format!("blank_name_sec{}", sec);
        let sec_info_key = format!("blank_info_sec{}", sec);
        if let Some(sec_name) = template_param(&named, &[&sec_name_key]) {
            rows.push("|-".to_string());
            rows.push(format!("! {}", render_templates(sec_name)));
            if let Some(sec_info) = template_param(&named, &[&sec_info_key]) {
                rows.push(format!("| {}", render_templates(sec_info)));
            } else {
                rows.push("|".to_string());
            }
        }
        for idx in 1..=7 {
            let name_key = format!("blank{}_name_sec{}", idx, sec);
            let info_key = format!("blank{}_info_sec{}", idx, sec);
            if let (Some(name), Some(info)) = (
                template_param(&named, &[&name_key]),
                template_param(&named, &[&info_key]),
            ) {
                rows.push("|-".to_string());
                rows.push(format!("! {}", render_templates(name)));
                rows.push(format!("| {}", render_templates(info)));
            }
        }
    }

    add_row(&mut rows, "Website", &["website"]);

    rows.push("|}".to_string());
    rows.join("\n")
}

/// [Infobox](https://en.wikipedia.org/wiki/Template:Infobox)
fn render_infobox_generic_template(params: &str) -> String {
    let named = template_named_params(params);
    let mut rows = Vec::new();

    rows.push("{| class=\"wikitable\"".to_string());

    if let Some(title) = template_param(&named, &["title"]) {
        rows.push("|-".to_string());
        rows.push(format!("! colspan=\"2\" | {}", render_templates(title)));
    }

    if let Some(image) = template_param(&named, &["image"]) {
        rows.push("|-".to_string());
        if let Some(caption) = template_param(&named, &["caption"]) {
            rows.push(format!(
                "| colspan=\"2\" | {}\n<br/>{}",
                render_templates(image),
                render_templates(caption)
            ));
        } else {
            rows.push(format!("| colspan=\"2\" | {}", render_templates(image)));
        }
    }

    for i in 1..=120 {
        let header_key = format!("header{}", i);
        let label_key = format!("label{}", i);
        let data_key = format!("data{}", i);

        if let Some(header) = template_param(&named, &[&header_key]) {
            rows.push("|-".to_string());
            rows.push(format!("! colspan=\"2\" | {}", render_templates(header)));
        } else if let Some(data) = template_param(&named, &[&data_key]) {
            rows.push("|-".to_string());
            if let Some(label) = template_param(&named, &[&label_key]) {
                rows.push(format!("! {}", render_templates(label)));
                rows.push(format!("| {}", render_templates(data)));
            } else {
                rows.push(format!("| colspan=\"2\" | {}", render_templates(data)));
            }
        }
    }

    rows.push("|}".to_string());
    rows.join("\n")
}

fn get_date_from_params(
    positional: &[String],
    start_idx: usize,
    len: usize,
) -> Option<(i32, i32, i32)> {
    if start_idx + len > positional.len() {
        return None;
    }

    if len == 3 {
        let y = positional[start_idx].parse::<i32>().ok()?;
        let m = positional[start_idx + 1].parse::<i32>().ok()?;
        let d = positional[start_idx + 2].parse::<i32>().ok()?;
        Some((y, m, d))
    } else if len == 1 {
        parse_date_string(&positional[start_idx])
    } else {
        None
    }
}

fn days_from_year_zero(year: i32, month: i32, day: i32) -> i32 {
    let mut y = year;
    if y < 0 {
        y += 1;
    }

    let mut days = day;

    let prev_y = y - 1;
    days += prev_y * 365 + prev_y / 4 - prev_y / 100 + prev_y / 400;

    let is_leap = (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0);
    let month_lengths = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    days += month_lengths.iter().take(month as usize - 1).sum::<i32>();

    days
}

fn days_between_dates(y1: i32, m1: i32, d1: i32, y2: i32, m2: i32, d2: i32) -> i32 {
    let days1 = days_from_year_zero(y1, m1, d1);
    let days2 = days_from_year_zero(y2, m2, d2);
    days2 - days1
}

fn calculate_age_in_years_and_days(
    y1: i32,
    m1: i32,
    d1: i32,
    y2: i32,
    m2: i32,
    d2: i32,
) -> (i32, i32) {
    let mut years = y2 - y1;
    if y1 < 0 && y2 > 0 {
        years -= 1;
    }

    let anniversary_passed = m2 > m1 || (m2 == m1 && d2 >= d1);

    let (anniversary_year, years_actual) = if anniversary_passed {
        (y2, years)
    } else {
        let prev_year = if y2 == 1 && y1 < 0 { -1 } else { y2 - 1 };
        (prev_year, years - 1)
    };

    let days = days_between_dates(anniversary_year, m1, d1, y2, m2, d2);

    (years_actual, days)
}

/// [ayd](https://en.wikipedia.org/wiki/Template:Ayd)
/// [age in years and days nts](https://en.wikipedia.org/wiki/Template:Age_in_years_and_days_nts)
/// [Age in years and days nts](https://en.wikipedia.org/wiki/Template:Age_in_years_and_days_nts)
fn render_ayd_template(params: &str) -> String {
    let positional = template_positional_params(params);

    let date1_opt;
    let date2_opt;

    if positional.len() >= 6
        && positional[0].parse::<i32>().is_ok()
        && positional[3].parse::<i32>().is_ok()
    {
        date1_opt = get_date_from_params(&positional, 0, 3);
        date2_opt = get_date_from_params(&positional, 3, 3);
    } else if positional.len() >= 3 && positional[0].parse::<i32>().is_ok() {
        date1_opt = get_date_from_params(&positional, 0, 3);
        date2_opt = Some(current_utc_date());
    } else if positional.len() >= 2 {
        date1_opt = get_date_from_params(&positional, 0, 1);
        date2_opt = get_date_from_params(&positional, 1, 1);
    } else if !positional.is_empty() {
        date1_opt = get_date_from_params(&positional, 0, 1);
        date2_opt = Some(current_utc_date());
    } else {
        return String::new();
    }

    let Some((y1, m1, d1)) = date1_opt else {
        return String::new();
    };
    let Some((y2, m2, d2)) = date2_opt else {
        return String::new();
    };

    let (years, days) = calculate_age_in_years_and_days(y1, m1, d1, y2, m2, d2);

    let years_str = if years == 1 {
        "1 year".to_string()
    } else {
        format!("{years} years")
    };
    let days_str = if days == 1 {
        "1 day".to_string()
    } else {
        format!("{days} days")
    };

    format!("{years_str}, {days_str}")
}

/// [RouteBox](https://en.wikipedia.org/wiki/Template:RouteBox)
fn render_route_box_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let named = template_named_params(params);

    let label = positional
        .first()
        .cloned()
        .or_else(|| named.get("1").cloned())
        .unwrap_or_default();
    let link = positional
        .get(1)
        .cloned()
        .or_else(|| named.get("2").cloned())
        .unwrap_or_default();
    let bg_color = positional
        .get(2)
        .cloned()
        .or_else(|| named.get("3").cloned())
        .unwrap_or_else(|| "#333333".to_string());
    let text_color = positional
        .get(3)
        .cloned()
        .or_else(|| named.get("4").cloned())
        .unwrap_or_else(|| "white".to_string());

    let label = render_templates(&label);
    let link = render_templates(&link);
    let bg_color = render_templates(&bg_color);
    let text_color = render_templates(&text_color);

    let link_wikitext = if link.is_empty() {
        label.clone()
    } else if label == link {
        format!("[[{link}]]")
    } else {
        format!("[[{link}|{label}]]")
    };

    let bg_color = bg_color.trim();
    let text_color = text_color.trim();
    format!(
        "__WIKIPEDIA_TO_EPUB_ROUTE_BOX_START__{bg_color}__WIKIPEDIA_TO_EPUB_ROUTE_BOX_MID__{text_color}__WIKIPEDIA_TO_EPUB_ROUTE_BOX_TEXT__{link_wikitext}__WIKIPEDIA_TO_EPUB_ROUTE_BOX_END__"
    )
}

/// [STN](https://en.wikipedia.org/wiki/Template:STN)
fn render_stn_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|p| p.trim().to_string())
        .collect::<Vec<_>>();
    if params.is_empty() {
        return String::new();
    }
    let name = &params[0];
    if name.is_empty() {
        return String::new();
    }

    let mut capitalize = true;
    let mut disambig = None;
    let mut custom_label = None;

    if params.len() > 1 {
        let p1 = &params[1];
        if p1 == "x" {
            capitalize = true;
        } else if !p1.is_empty() && !p1.contains('=') {
            disambig = Some(p1);
        }
    }

    if params.len() > 2 {
        let p2 = &params[2];
        if !p2.is_empty() && !p2.contains('=') {
            custom_label = Some(p2);
        }
    }

    let suffix = if capitalize { "Station" } else { "station" };

    let target = match disambig {
        Some(d) => format!("{} {} ({})", name, suffix, d),
        None => format!("{} {}", name, suffix),
    };

    let label = match custom_label {
        Some(l) => l.to_string(),
        None => name.to_string(),
    };

    format!("[[{}|{}]]", target, render_templates(&label))
}

/// [rail-interchange](https://en.wikipedia.org/wiki/Template:Rail-interchange)
/// [ric](https://en.wikipedia.org/wiki/Template:Ric)
/// [rint](https://en.wikipedia.org/wiki/Template:Rint)
fn render_ric_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let named = template_named_params(params);

    let system = positional
        .first()
        .cloned()
        .or_else(|| named.get("1").cloned())
        .unwrap_or_default();
    let line = positional
        .get(1)
        .cloned()
        .or_else(|| named.get("2").cloned())
        .unwrap_or_default();

    let system = render_templates(&system);
    let line = render_templates(&line);

    let line = line.trim();
    if line.is_empty() {
        system.trim().to_string()
    } else {
        format!("[{}]", line)
    }
}

/// [Line link](https://en.wikipedia.org/wiki/Template:Line_link)
/// [lnl](https://en.wikipedia.org/wiki/Template:Lnl)
fn render_lnl_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let named = template_named_params(params);

    let system = positional
        .first()
        .cloned()
        .or_else(|| named.get("1").cloned())
        .unwrap_or_default();
    let line = positional
        .get(1)
        .cloned()
        .or_else(|| named.get("2").cloned())
        .unwrap_or_default();

    let system = render_templates(&system).trim().to_string();
    let line = render_templates(&line).trim().to_string();

    if system.is_empty() {
        return String::new();
    }
    if line.is_empty() {
        return format!("[[{system}]]");
    }

    if system.eq_ignore_ascii_case("JR East") {
        let (link, label) = match line.to_ascii_uppercase().as_str() {
            "JY" => ("Yamanote Line", "Yamanote Line"),
            "JK" => ("Keihin–Tōhoku Line", "Keihin–Tōhoku Line"),
            "JU" => ("Utsunomiya Line", "Utsunomiya Line"),
            "JC" => ("Chūō Line (Rapid)", "Chūō Line"),
            "JO" => ("Yokosuka Line", "Yokosuka Line"),
            "JB" => ("Chūō–Sōbu Line", "Chūō–Sōbu Line"),
            "JE" => ("Keiyō Line", "Keiyō Line"),
            "JH" => ("Yokohama Line", "Yokohama Line"),
            "JT" => ("Tōkaidō Line (JR East)", "Tōkaidō Line"),
            "JJ" => ("Jōban Line", "Jōban Line (Rapid)"),
            "JM" => ("Musashino Line", "Musashino Line"),
            "JN" => ("Nambu Line", "Nambu Line"),
            "JI" => ("Tsurumi Line", "Tsurumi Line"),
            _ => (line.as_str(), line.as_str()),
        };
        if link == label {
            format!("[[{link}]]")
        } else {
            format!("[[{link}|{label}]]")
        }
    } else {
        // Fallback for other systems
        if line.len() <= 2 {
            // Likely a code, e.g. M, H
            format!("[[{system} {line} Line|{line} Line]]")
        } else {
            format!("[[{line}]]")
        }
    }
}

/// [GBurl](https://en.wikipedia.org/wiki/Template:GBurl)
fn render_gburl_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);
    render_google_books_url(&named, &positional).unwrap_or_default()
}

/// [Google books](https://en.wikipedia.org/wiki/Template:Google_books)
fn render_google_books_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);
    let Some(url) = render_google_books_url(&named, &positional) else {
        return String::new();
    };

    if template_param(&named, &["plainurl"])
        .map(|value| value.trim())
        .is_some_and(|value| value.eq_ignore_ascii_case("yes") || value.eq_ignore_ascii_case("y"))
    {
        return url;
    }

    let label = template_param(&named, &["2", "text", "label", "title"])
        .or_else(|| positional.get(1).map(String::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Google Books");

    format!("[[official-url:{url}|{}]]", render_templates(label))
}

fn render_google_books_url(
    named: &std::collections::HashMap<String, String>,
    positional: &[String],
) -> Option<String> {
    let id = template_param(named, &["id", "1"])
        .or_else(|| positional.first().map(String::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())?;
    let mut url = format!("https://books.google.com/books?id={id}");

    if let Some(pg) = template_param(named, &["pg"]) {
        url.push_str(&format!("&pg={pg}"));
    } else if let Some(p) = template_param(named, &["p", "page"]) {
        if p.chars().all(|c| c.is_ascii_digit()) {
            url.push_str(&format!("&pg=PA{p}"));
        } else {
            url.push_str(&format!("&pg={p}"));
        }
    }

    if let Some(q) = template_param(named, &["q", "keywords"]) {
        url.push_str(&format!("&q={}", q.replace(' ', "+")));
    } else if let Some(dq) = template_param(named, &["dq", "text"]) {
        url.push_str(&format!("&dq={}", dq.replace(' ', "+")));
    }

    Some(url)
}

/// [usurped](https://en.wikipedia.org/wiki/Template:Usurped)
fn render_usurped_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let url = template_param(&named, &["1", "url"])
        .or_else(|| positional.first().map(String::as_str))
        .map(|s| s.trim())
        .unwrap_or("");

    render_templates(url)
}

/// [Break](https://en.wikipedia.org/wiki/Template:Break)
/// [br](https://en.wikipedia.org/wiki/Template:Br)
/// [brk](https://en.wikipedia.org/wiki/Template:Brk)
/// [crlf](https://en.wikipedia.org/wiki/Template:Crlf)
fn render_break_template(params: &str) -> String {
    let n = template_positional_params(params)
        .first()
        .and_then(|val| val.trim().parse::<usize>().ok())
        .unwrap_or(1);
    "__WIKIPEDIA_TO_EPUB_BR__".repeat(n)
}

/// [as of](https://en.wikipedia.org/wiki/Template:As_of)
fn render_as_of_template(params: &str) -> String {
    let named = template_named_params(params);
    if let Some(alt) = template_param(&named, &["alt"]) {
        return render_templates(alt);
    }

    let positional = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .map(|param| render_templates(&param))
        .collect::<Vec<_>>();

    let Some(year) = positional.first() else {
        return String::new();
    };

    let date = as_of_date(&positional, template_param(&named, &["df"]));
    let prefix = if template_param_truthy(&named, &["lc"]) {
        "as of"
    } else {
        "As of"
    };

    if date.is_empty() {
        render_templates(year)
    } else {
        format!("{prefix} {date}")
    }
}

/// [died-in](https://en.wikipedia.org/wiki/Template:Died-in)
fn render_died_in_template(params: &str) -> String {
    let date = render_passthrough_template(params);
    if date.trim().is_empty() {
        String::new()
    } else {
        format!("d. {}", date.trim())
    }
}

fn as_of_date(positional: &[String], date_format: Option<&str>) -> String {
    let year = positional.first().map(String::as_str).unwrap_or_default();
    let Some(month) = positional.get(1).map(String::as_str) else {
        return year.to_string();
    };

    let month = as_of_month_name(month).unwrap_or(month);
    let Some(day) = positional.get(2).map(String::as_str) else {
        return format!("{month} {year}");
    };

    if date_format.is_some_and(|value| value.eq_ignore_ascii_case("dmy")) {
        format!("{day} {month} {year}")
    } else {
        format!("{month} {day}, {year}")
    }
}

fn as_of_month_name(month: &str) -> Option<&'static str> {
    match month.trim().parse::<usize>().ok()? {
        1 => Some("January"),
        2 => Some("February"),
        3 => Some("March"),
        4 => Some("April"),
        5 => Some("May"),
        6 => Some("June"),
        7 => Some("July"),
        8 => Some("August"),
        9 => Some("September"),
        10 => Some("October"),
        11 => Some("November"),
        12 => Some("December"),
        _ => None,
    }
}

fn template_param_truthy(named: &HashMap<String, String>, keys: &[&str]) -> bool {
    template_param(named, keys).is_some_and(|value| {
        value.eq_ignore_ascii_case("y")
            || value.eq_ignore_ascii_case("yes")
            || value.eq_ignore_ascii_case("true")
            || value == "1"
    })
}

/// [Quote box](https://en.wikipedia.org/wiki/Template:Quote_box)
/// [Quote](https://en.wikipedia.org/wiki/Template:Quote)
/// [blockquote](https://en.wikipedia.org/wiki/Template:Blockquote)
fn render_blockquote_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let text = template_param(&named, &["text", "quote", "1"])
        .map(str::to_string)
        .or_else(|| positional.first().cloned())
        .map(|value| render_templates(&value).replace('\n', " "))
        .unwrap_or_default();

    let source = template_param(&named, &["source", "author", "cite", "2"])
        .map(str::to_string)
        .or_else(|| positional.get(1).cloned())
        .map(|value| render_templates(&value).replace('\n', " "))
        .unwrap_or_default();

    if text.trim().is_empty() {
        return String::new();
    }

    let mut rendered = format!(
        "\n__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_START__\n__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__{}\n",
        text.trim()
    );
    if !source.trim().is_empty() {
        rendered.push_str(&format!(
            "__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_SOURCE__{}\n",
            source.trim()
        ));
    }
    rendered.push_str("__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_END__\n");
    rendered
}

/// [Poem quote](https://en.wikipedia.org/wiki/Template:Poem_quote)
/// [poemquote](https://en.wikipedia.org/wiki/Template:Poemquote)
fn render_poem_quote_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let text = template_param(&named, &["text", "quote", "1"])
        .map(str::to_string)
        .or_else(|| positional.first().cloned())
        .unwrap_or_default();

    let source = template_param(&named, &["source", "author", "cite", "2"])
        .map(str::to_string)
        .or_else(|| positional.get(1).cloned())
        .unwrap_or_default();

    if text.trim().is_empty() {
        return String::new();
    }

    let mut rendered = String::new();
    rendered.push_str("\n__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_START__\n");
    for line in text.lines() {
        let rendered_line = render_templates(line);
        rendered.push_str(&format!(
            "__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__{}\n",
            rendered_line.trim()
        ));
    }

    let rendered_source = render_templates(&source);
    if !rendered_source.trim().is_empty() {
        rendered.push_str(&format!(
            "__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_SOURCE__{}\n",
            rendered_source.trim()
        ));
    }
    rendered.push_str("__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_END__\n");
    rendered
}

/// [Verse translation](https://en.wikipedia.org/wiki/Template:Verse_translation)
fn render_verse_translation_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let text1 = template_param(&named, &["1"])
        .map(str::to_string)
        .or_else(|| positional.first().cloned())
        .unwrap_or_default();

    let text2 = template_param(&named, &["2"])
        .map(str::to_string)
        .or_else(|| positional.get(1).cloned())
        .unwrap_or_default();

    if text1.trim().is_empty() && text2.trim().is_empty() {
        return String::new();
    }

    let italicsoff = template_param(&named, &["italicsoff"])
        .is_some_and(|v| v.eq_ignore_ascii_case("y") || v.eq_ignore_ascii_case("yes"));

    let mut rendered = String::new();
    rendered.push_str("\n__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_START__\n");

    if !text1.trim().is_empty() {
        for line in text1.lines() {
            let rendered_line = render_templates(line);
            let formatted_line = if italicsoff {
                rendered_line.trim().to_string()
            } else {
                format!("''{}''", rendered_line.trim())
            };
            rendered.push_str(&format!(
                "__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__{}\n",
                formatted_line
            ));
        }
        if !text2.trim().is_empty() {
            rendered.push_str("__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__\n");
        }
    }

    if !text2.trim().is_empty() {
        for line in text2.lines() {
            let rendered_line = render_templates(line);
            rendered.push_str(&format!(
                "__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__{}\n",
                rendered_line.trim()
            ));
        }
    }

    rendered.push_str("__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_END__\n");
    rendered
}

/// [Verse transliteration-translation](https://en.wikipedia.org/wiki/Template:Verse_transliteration-translation)
fn render_verse_transliteration_translation_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let text1 = template_param(&named, &["1"])
        .map(str::to_string)
        .or_else(|| positional.first().cloned())
        .unwrap_or_default();

    let text2 = template_param(&named, &["2"])
        .map(str::to_string)
        .or_else(|| positional.get(1).cloned())
        .unwrap_or_default();

    let text3 = template_param(&named, &["3"])
        .map(str::to_string)
        .or_else(|| positional.get(2).cloned())
        .unwrap_or_default();

    if text1.trim().is_empty() && text2.trim().is_empty() && text3.trim().is_empty() {
        return String::new();
    }

    let mut rendered = String::new();
    rendered.push_str("\n__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_START__\n");

    if !text1.trim().is_empty() {
        for line in text1.lines() {
            let rendered_line = render_templates(line);
            rendered.push_str(&format!(
                "__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__{}\n",
                rendered_line.trim()
            ));
        }
        if !text2.trim().is_empty() || !text3.trim().is_empty() {
            rendered.push_str("__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__\n");
        }
    }

    if !text2.trim().is_empty() {
        for line in text2.lines() {
            let rendered_line = render_templates(line);
            rendered.push_str(&format!(
                "__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__''{}''\n",
                rendered_line.trim()
            ));
        }
        if !text3.trim().is_empty() {
            rendered.push_str("__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__\n");
        }
    }

    if !text3.trim().is_empty() {
        for line in text3.lines() {
            let rendered_line = render_templates(line);
            rendered.push_str(&format!(
                "__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__{}\n",
                rendered_line.trim()
            ));
        }
    }

    rendered.push_str("__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_END__\n");
    rendered
}

/// [for timeline](https://en.wikipedia.org/wiki/Template:For_timeline)
fn render_for_timeline_template(params: &str) -> String {
    let articles = template_article_params(params);

    match articles.as_slice() {
        [] => String::new(),
        [article] => format!("For a timeline, see: [[{article}]]"),
        articles => format!("For timelines, see: {}", join_template_articles(articles)),
    }
}

/// [legend](https://en.wikipedia.org/wiki/Template:Legend)
/// [legend0](https://en.wikipedia.org/wiki/Template:Legend0)
fn render_legend_template(params: &str) -> String {
    let params = template_positional_params(params);
    let Some(label) = params.get(1).map(String::as_str) else {
        return String::new();
    };

    render_templates(label)
}

/// [numero](https://en.wikipedia.org/wiki/Template:Numero)
fn render_numero_template(params: &str) -> String {
    let number = render_passthrough_template(params);
    if number.trim().is_empty() {
        String::new()
    } else {
        format!("No. {}", number.trim())
    }
}

/// [anl](https://en.wikipedia.org/wiki/Template:Anl)
fn render_article_link_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(article) = positional
        .first()
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
    else {
        return String::new();
    };
    let label = positional
        .get(1)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(article);

    format!("[[{article}|{}]]", render_templates(label))
}

/// [for](https://en.wikipedia.org/wiki/Template:For)
fn render_for_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(topic) = positional
        .first()
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
    else {
        return String::new();
    };
    let articles = positional
        .iter()
        .skip(1)
        .filter(|article| !article.trim().is_empty())
        .cloned()
        .collect::<Vec<_>>();

    if articles.is_empty() {
        render_templates(topic)
    } else {
        format!(
            "For {}, see: {}",
            render_templates(topic),
            join_template_articles(&articles)
        )
    }
}

/// [excerpt](https://en.wikipedia.org/wiki/Template:Excerpt)
fn render_excerpt_template(params: &str) -> String {
    let articles = template_article_params(params);

    match articles.as_slice() {
        [] => String::new(),
        [article] => format!("Excerpt from: [[{article}]]"),
        articles => format!("Excerpts from: {}", join_template_articles(articles)),
    }
}

/// [main](https://en.wikipedia.org/wiki/Template:Main)
/// [Main article](https://en.wikipedia.org/wiki/Template:Main_article)
fn render_main_template(params: &str) -> String {
    let articles = template_article_params(params);

    match articles.as_slice() {
        [] => String::new(),
        [article] => format!("Main article: [[{article}]]"),
        articles => format!("Main articles: {}", join_template_articles(articles)),
    }
}

/// [Main list](https://en.wikipedia.org/wiki/Template:Main_list)
fn render_main_list_template(params: &str) -> String {
    let named = template_named_params(params);
    let articles = template_article_params(params);
    let more = template_param(&named, &["more"])
        .map(|v| !v.eq_ignore_ascii_case("no"))
        .unwrap_or(true);

    if articles.is_empty() {
        return String::new();
    }

    let prefix = if more {
        if articles.len() == 1 {
            "For a more comprehensive list, see "
        } else {
            "For more comprehensive lists, see "
        }
    } else {
        if articles.len() == 1 {
            "For a comprehensive list, see "
        } else {
            "For comprehensive lists, see "
        }
    };

    format!("{}{}", prefix, join_template_articles(&articles))
}

/// [dts](https://en.wikipedia.org/wiki/Template:Dts)
fn render_dts_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let mut year = None;
    let mut month = None;
    let mut day = None;

    if positional.len() >= 3 {
        year = positional[0].trim().parse::<i32>().ok();
        let m_str = positional[1].trim();
        month = m_str.parse::<i32>().ok().or_else(|| {
            let months = [
                "january",
                "february",
                "march",
                "april",
                "may",
                "june",
                "july",
                "august",
                "september",
                "october",
                "november",
                "december",
            ];
            months
                .iter()
                .position(|&m| m.eq_ignore_ascii_case(m_str))
                .map(|idx| idx as i32 + 1)
        });
        day = positional[2].trim().parse::<i32>().ok();
    } else if let Some(first_param) = positional.first() {
        let first_param = first_param.trim();
        let parts: Vec<&str> = first_param.split('-').collect();
        if parts.len() == 3 {
            year = parts[0].parse::<i32>().ok();
            month = parts[1].parse::<i32>().ok();
            day = parts[2].parse::<i32>().ok();
        } else if let Some((y, m, d)) = parse_date_string(first_param) {
            year = Some(y);
            month = Some(m);
            day = Some(d);
        }
    }

    let Some(y) = year else {
        return String::new();
    };
    let Some(m) = month else {
        return String::new();
    };
    let Some(d) = day else {
        return String::new();
    };

    let months = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    if !(1..=12).contains(&m) {
        return String::new();
    }
    let month_name = months[m as usize - 1];

    let format_param = template_param(&named, &["format"]);
    let is_dmy = format_param.is_some_and(|fmt| fmt.eq_ignore_ascii_case("dmy"));

    let bc = template_param(&named, &["bc"]).is_some()
        || positional.iter().any(|p| p.eq_ignore_ascii_case("bc"));

    let base = if is_dmy {
        format!("{} {} {}", d, month_name, y)
    } else {
        format!("{} {}, {}", month_name, d, y)
    };

    if bc { format!("{} BC", base) } else { base }
}

/// [see also](https://en.wikipedia.org/wiki/Template:See_also)
/// [also](https://en.wikipedia.org/wiki/Template:Also)
fn render_see_also_template(params: &str) -> String {
    let articles = template_article_params(params);

    if articles.is_empty() {
        String::new()
    } else {
        format!("See also: {}", join_template_articles(&articles))
    }
}

/// [further](https://en.wikipedia.org/wiki/Template:Further)
fn render_further_template(params: &str) -> String {
    let named = template_named_params(params);
    let articles = template_article_params(params);

    if articles.is_empty() {
        String::new()
    } else if let Some(topic) = template_param(&named, &["topic"]) {
        format!(
            "Further information about {}: {}",
            render_templates(topic),
            join_template_articles(&articles)
        )
    } else {
        format!("Further information: {}", join_template_articles(&articles))
    }
}

/// [wiktionary](https://en.wikipedia.org/wiki/Template:Wiktionary)
fn render_wiktionary_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    let Some(title) = params.first() else {
        return String::new();
    };
    let label = params.get(1).unwrap_or(title);
    let target = format!("wikt:{title}");

    format!("Wiktionary: [[{target}|{label}]]")
}

/// [wikivoyage](https://en.wikipedia.org/wiki/Template:Wikivoyage)
/// [wikivoyage-inline](https://en.wikipedia.org/wiki/Template:Wikivoyage-inline)
/// [wikivoyage inline](https://en.wikipedia.org/wiki/Template:Wikivoyage_inline)
fn render_wikivoyage_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    let Some(title) = params.first() else {
        return String::new();
    };
    let label = params.get(1).unwrap_or(title);
    let target = format!("voy:{title}");

    format!("Wikivoyage: [[{target}|{label}]]")
}

/// [wikisource](https://en.wikipedia.org/wiki/Template:Wikisource)
fn render_wikisource_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    let Some(title) = params.first() else {
        return String::new();
    };
    let label = params.get(1).unwrap_or(title);
    let target = format!("src:{title}");

    format!("Wikisource: [[{target}|{label}]]")
}

/// [wikibooks](https://en.wikipedia.org/wiki/Template:Wikibooks)
fn render_wikibooks_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);
    let book = template_param(&named, &["1"])
        .or_else(|| positional.first().map(String::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let page = template_param(&named, &["2"])
        .or_else(|| positional.get(1).map(String::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let Some(book) = book else {
        return String::new();
    };

    let target = if let Some(page) = page {
        format!("b:{book}/{page}")
    } else {
        format!("b:{book}")
    };
    let label = template_param(&named, &["3"])
        .or_else(|| positional.get(2).map(String::as_str))
        .or(page)
        .unwrap_or(book);

    format!("Wikibooks: [[{target}|{}]]", render_templates(label))
}

/// [britannica](https://en.wikipedia.org/wiki/Template:Britannica)
fn render_britannica_template(params: &str) -> String {
    let params = template_positional_params(params);
    let Some(article_id) = params
        .first()
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
    else {
        return String::new();
    };
    let label = params
        .get(1)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Encyclopaedia Britannica");
    let url = format!("https://www.britannica.com/EBchecked/topic/{article_id}");

    format!(
        "Britannica: [[official-url:{url}|{}]]",
        render_templates(label)
    )
}

/// [official website](https://en.wikipedia.org/wiki/Template:Official_website)
fn render_official_website_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let named = template_named_params(params);

    let url = positional
        .first()
        .map(String::as_str)
        .or_else(|| template_param(&named, &["url", "website"]))
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let Some(url) = url else {
        return String::new();
    };

    let label = template_param(&named, &["name", "title"])
        .or_else(|| positional.get(1).map(String::as_str).map(str::trim))
        .filter(|value| !value.is_empty())
        .unwrap_or("Official website");

    format!("[[official-url:{url}|{}]]", render_templates(label))
}

/// [url](https://en.wikipedia.org/wiki/Template:Url)
fn render_url_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let named = template_named_params(params);

    let url = template_param(&named, &["1", "url"])
        .or_else(|| positional.first().map(String::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let Some(url) = url else {
        return String::new();
    };

    let label = template_param(&named, &["2", "name", "title"])
        .or_else(|| positional.get(1).map(String::as_str).map(str::trim))
        .filter(|value| !value.is_empty())
        .unwrap_or(url);

    format!("[[official-url:{url}|{}]]", render_templates(label))
}

/// [osmrelation-inline](https://en.wikipedia.org/wiki/Template:Osmrelation-inline)
/// [OSM relation](https://en.wikipedia.org/wiki/Template:OSM_relation)
fn render_openstreetmap_relation_template(params: &str) -> String {
    let params = template_positional_params(params);
    let Some(relation_id) = params.first().map(String::as_str) else {
        return String::new();
    };
    let relation_id = relation_id.trim();
    if relation_id.is_empty() {
        return String::new();
    }

    format!("[[osmrelation:{relation_id}|OpenStreetMap relation {relation_id}]]")
}

/// [osmway](https://en.wikipedia.org/wiki/Template:Osmway)
/// [OSM way](https://en.wikipedia.org/wiki/Template:OSM_way)
fn render_openstreetmap_way_template(params: &str) -> String {
    let params = template_positional_params(params);
    let Some(way_id) = params.first().map(String::as_str) else {
        return String::new();
    };
    let way_id = way_id.trim();
    if way_id.is_empty() {
        return String::new();
    }

    format!("[[osmway:{way_id}|OpenStreetMap way {way_id}]]")
}

/// [webarchive](https://en.wikipedia.org/wiki/Template:Webarchive)
fn render_webarchive_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let named = template_named_params(params);

    let url = template_param(&named, &["url"])
        .or_else(|| {
            positional
                .iter()
                .find_map(|param| template_url_value(param))
        })
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let Some(url) = url else {
        return String::new();
    };

    let label = template_param(&named, &["date"])
        .map(|date| format!("Archived on {}", render_templates(date)))
        .unwrap_or_else(|| "Archived copy".to_string());

    format!("[[official-url:{url}|{label}]]")
}

fn template_url_value(value: &str) -> Option<&str> {
    let value = value.trim();
    if value.starts_with("http://") || value.starts_with("https://") || value.starts_with("//") {
        Some(value)
    } else {
        None
    }
}

/// [largest cities](https://en.wikipedia.org/wiki/Template:Largest_cities)
fn render_largest_cities_template(params: &str) -> String {
    let named = template_named_params(params);
    let country = template_param(&named, &["country"])
        .map(render_templates)
        .filter(|value| !value.is_empty());
    let mut lines = Vec::new();

    for index in 1..=100 {
        let city_key = format!("city_{index}");
        let Some(city) = named.get(&city_key).map(String::as_str).map(str::trim) else {
            continue;
        };
        if city.is_empty() {
            continue;
        }

        let city = render_largest_city_name(city);
        let division = named
            .get(&format!("div_{index}"))
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(render_templates);
        let population = named
            .get(&format!("pop_{index}"))
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(render_templates);

        let mut details = Vec::new();
        if let Some(division) = division {
            details.push(division);
        }
        if let Some(population) = population {
            details.push(format!("population {population}"));
        }

        if details.is_empty() {
            lines.push(format!("* {city}"));
        } else {
            lines.push(format!("* {city} ({})", details.join(", ")));
        }
    }

    if lines.is_empty() {
        return String::new();
    }

    let heading = country
        .map(|country| format!("Largest cities in {country}:"))
        .unwrap_or_else(|| "Largest cities:".to_string());
    format!("\n{heading}\n{}\n", lines.join("\n"))
}

/// [historical populations](https://en.wikipedia.org/wiki/Template:Historical_populations)
fn render_historical_populations_template(params: &str) -> String {
    let entries = historical_population_entries(params);
    if entries.is_empty() {
        return String::new();
    }

    let lines = entries
        .into_iter()
        .map(|(year, population)| format!("* {year}: {population}"))
        .collect::<Vec<_>>();
    format!("\nHistorical populations:\n{}\n", lines.join("\n"))
}

/// [climate chart](https://en.wikipedia.org/wiki/Template:Climate_chart)
fn render_climate_chart_template(params: &str) -> String {
    let params = template_positional_params(params)
        .into_iter()
        .map(|param| render_templates(&param).trim().to_string())
        .filter(|param| !param.is_empty())
        .collect::<Vec<_>>();

    let Some(location) = params.first() else {
        return String::new();
    };

    let entries = params.iter().skip(1).take(36).collect::<Vec<_>>();
    if entries.len() < 36 {
        return String::new();
    }

    let month_names = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let lines = month_names
        .iter()
        .zip(entries.chunks_exact(3))
        .map(|(month, values)| {
            format!(
                "* {month}: {} to {} °C, {} mm",
                format_convert_value(values[0]),
                format_convert_value(values[1]),
                format_convert_value(values[2])
            )
        })
        .collect::<Vec<_>>();

    format!(
        "\nClimate chart for {}:\n{}\n",
        render_templates(location),
        lines.join("\n")
    )
}

fn historical_population_entries(params: &str) -> Vec<(String, String)> {
    let values = split_template_params(params)
        .into_iter()
        .filter_map(|param| {
            let trimmed = param.trim();
            if trimmed.is_empty() {
                return None;
            }

            match trimmed.split_once('=') {
                Some((key, value)) if key.trim().parse::<usize>().is_ok() => {
                    Some(value.trim().to_string())
                }
                Some(_) => None,
                None => Some(trimmed.to_string()),
            }
        })
        .map(|value| render_templates(&value).trim().to_string())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();

    values
        .chunks(2)
        .filter_map(|chunk| {
            let [year, population] = chunk else {
                return None;
            };

            Some((year.to_string(), format_historical_population(population)))
        })
        .collect()
}

fn format_historical_population(value: &str) -> String {
    let trimmed = value.trim();
    match trimmed.parse::<i64>() {
        Ok(number) => format_population_number(number),
        Err(_) => trimmed.to_string(),
    }
}

fn format_population_number(value: i64) -> String {
    let digits = value.abs().to_string();
    let grouped = digits
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or_default())
        .collect::<Vec<_>>()
        .join(",");

    if value < 0 {
        format!("-{grouped}")
    } else {
        grouped
    }
}

fn render_largest_city_name(city: &str) -> String {
    let city = render_templates(city).trim().to_string();
    if city.contains("[[") {
        city
    } else {
        format!("[[{city}]]")
    }
}

/// [sclass](https://en.wikipedia.org/wiki/Template:Sclass)
fn render_ship_class_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| render_templates(param.trim()).trim().to_string())
        .collect::<Vec<_>>();

    let class_name = params.first().map(String::as_str).unwrap_or("").trim();
    let ship_type = params.get(1).map(String::as_str).unwrap_or("").trim();
    if class_name.is_empty() || ship_type.is_empty() {
        return String::new();
    }

    let format = params.get(2).map(String::as_str).unwrap_or("").trim();
    let ship_type_disambiguation = params.get(3).map(String::as_str).unwrap_or("").trim();
    let class_disambiguation = params.get(4).map(String::as_str).unwrap_or("").trim();

    let mut class_target = format!("{class_name}-class {ship_type}");
    if !class_disambiguation.is_empty() {
        class_target.push_str(&format!(" ({class_disambiguation})"));
    }

    let class_label = match format {
        "1" => format!("''{class_name}''-class {ship_type}"),
        "4" => format!("''{class_name}'' class"),
        "5" => format!("''{class_name}''"),
        _ => format!("''{class_name}''-class"),
    };

    let class_link = format!("[[{class_target}|{class_label}]]");
    match format {
        "0" | "4" | "5" => class_link,
        "1" => class_link,
        "2" => format!("{class_link} {ship_type}"),
        "" | "3" => {
            let ship_type_link = if ship_type_disambiguation.is_empty() {
                format!("[[{ship_type}]]")
            } else {
                format!("[[{ship_type} ({ship_type_disambiguation})|{ship_type}]]")
            };
            format!("{class_link} {ship_type_link}")
        }
        _ => class_link,
    }
}

/// [Arrow](https://en.wikipedia.org/wiki/Template:Arrow)
fn render_arrow_template(params: &str) -> String {
    let params = template_positional_params(params);
    match params
        .first()
        .map(|value| value.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("l" | "left" | "w") => "←".to_string(),
        Some("u" | "up" | "n") => "↑".to_string(),
        Some("d" | "down" | "s") => "↓".to_string(),
        Some("ne") => "↗".to_string(),
        Some("nw") => "↖".to_string(),
        Some("se") => "↘".to_string(),
        Some("sw") => "↙".to_string(),
        _ => "→".to_string(),
    }
}

/// [ROKS](https://en.wikipedia.org/wiki/Template:ROKS)
fn render_republic_of_korea_ship_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| render_templates(&param).trim().to_string())
        .filter(|param| !param.contains('='))
        .collect::<Vec<_>>();

    let Some(name) = params
        .first()
        .map(String::as_str)
        .filter(|name| !name.is_empty())
    else {
        return "ROKS".to_string();
    };

    let disambiguator = params
        .get(1)
        .map(String::as_str)
        .filter(|value| !value.is_empty());
    let target = match disambiguator {
        Some(disambiguator) => format!("ROKS {name} ({disambiguator})"),
        None => format!("ROKS {name}"),
    };

    format!("[[{target}|ROKS ''{name}'']]")
}

/// [For-multi](https://en.wikipedia.org/wiki/Template:For-multi)
fn render_for_multi_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let mut chunks = Vec::new();
    let mut iter = positional.into_iter();
    while let Some(topic) = iter.next() {
        if let Some(article) = iter.next()
            && !topic.trim().is_empty()
            && !article.trim().is_empty()
        {
            chunks.push(format!("{}, see [[{}]]", render_templates(&topic), article));
        }
    }

    if chunks.is_empty() {
        return String::new();
    }

    format!("For {}.", chunks.join("; for "))
}

/// [USS](https://en.wikipedia.org/wiki/Template:USS)
/// [HMS](https://en.wikipedia.org/wiki/Template:HMS)
pub(crate) fn render_ship_template(prefix: &str, params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(name) = positional
        .first()
        .map(String::as_str)
        .filter(|name| !name.is_empty())
    else {
        return prefix.to_string();
    };

    let id = positional
        .get(1)
        .map(String::as_str)
        .filter(|val| !val.is_empty());

    let format_val = positional
        .get(2)
        .map(String::as_str)
        .filter(|val| !val.is_empty())
        .and_then(|val| val.parse::<i32>().ok())
        .unwrap_or(0);

    let target = match id {
        Some(id_val) => format!("{prefix} {name} ({id_val})"),
        None => format!("{prefix} {name}"),
    };

    let display = match format_val {
        6 => format!("''{name}''"),
        2 => match id {
            Some(id_val) => format!("''{name}'' ({id_val})"),
            None => format!("''{name}''"),
        },
        3 => format!("{prefix} ''{name}''"),
        _ => match id {
            Some(id_val) => format!("{prefix} ''{name}'' ({id_val})"),
            None => format!("{prefix} ''{name}''"),
        },
    };

    format!("[[{target}|{display}]]")
}

/// [Nb5](https://en.wikipedia.org/wiki/Template:Nb5)
pub(crate) fn render_five_nonbreaking_spaces_template() -> String {
    "\u{00A0}\u{00A0}\u{00A0}\u{00A0}\u{00A0}".to_string()
}

/// [ship](https://en.wikipedia.org/wiki/Template:Ship)
fn render_generic_ship_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(prefix) = positional
        .first()
        .map(String::as_str)
        .filter(|p| !p.is_empty())
    else {
        return String::new();
    };

    let Some(name) = positional
        .get(1)
        .map(String::as_str)
        .filter(|name| !name.is_empty())
    else {
        return prefix.to_string();
    };

    let id = positional
        .get(2)
        .map(String::as_str)
        .filter(|val| !val.is_empty());

    let format_val = positional
        .get(3)
        .map(String::as_str)
        .filter(|val| !val.is_empty())
        .and_then(|val| val.parse::<i32>().ok())
        .unwrap_or(0);

    let target = match id {
        Some(id_val) => format!("{prefix} {name} ({id_val})"),
        None => format!("{prefix} {name}"),
    };

    let display = match format_val {
        6 => format!("''{name}''"),
        2 => match id {
            Some(id_val) => format!("''{name}'' ({id_val})"),
            None => format!("''{name}''"),
        },
        3 => format!("{prefix} ''{name}''"),
        _ => match id {
            Some(id_val) => format!("{prefix} ''{name}'' ({id_val})"),
            None => format!("{prefix} ''{name}''"),
        },
    };

    format!("[[{target}|{display}]]")
}

/// [Proto](https://en.wikipedia.org/wiki/Template:Proto)
fn render_proto_template(params: &str) -> String {
    let parts = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .collect::<Vec<_>>();
    if parts.len() < 2 {
        return String::new();
    }
    let lang_raw = &parts[0];
    let word = &parts[1];

    let lang_cap = lang_raw
        .split('-')
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join("-");

    format!("Proto-{} *{}", lang_cap, word)
}

/// [chem2](https://en.wikipedia.org/wiki/Template:Chem2)
fn render_chem2_template(params: &str) -> String {
    let positional = template_positional_params(params);
    if let Some(formula) = positional.first() {
        let re = Regex::new(r"([a-zA-Z])([0-9]+)").unwrap();
        re.replace_all(
            formula,
            "${1}__WIKIPEDIA_TO_EPUB_SUB_START__${2}__WIKIPEDIA_TO_EPUB_SUB_END__",
        )
        .into_owned()
    } else {
        String::new()
    }
}

/// [sup](https://en.wikipedia.org/wiki/Template:Sup)
fn render_sup_template(params: &str) -> String {
    let positional = template_positional_params(params);
    if let Some(text) = positional.first() {
        format!(
            "__WIKIPEDIA_TO_EPUB_SUP_START__{}__WIKIPEDIA_TO_EPUB_SUP_END__",
            render_templates(text)
        )
    } else {
        String::new()
    }
}

/// [sub](https://en.wikipedia.org/wiki/Template:Sub)
fn render_sub_template(params: &str) -> String {
    let positional = template_positional_params(params);
    if let Some(text) = positional.first() {
        format!(
            "__WIKIPEDIA_TO_EPUB_SUB_START__{}__WIKIPEDIA_TO_EPUB_SUB_END__",
            render_templates(text)
        )
    } else {
        String::new()
    }
}

/// [su](https://en.wikipedia.org/wiki/Template:Su)
fn render_su_template(params: &str) -> String {
    let named = template_named_params(params);
    let p = template_param(&named, &["p"]);
    let b = template_param(&named, &["b"]);

    let mut rendered = String::new();
    if let Some(sup_val) = p {
        rendered.push_str("__WIKIPEDIA_TO_EPUB_SUP_START__");
        rendered.push_str(&render_templates(sup_val));
        rendered.push_str("__WIKIPEDIA_TO_EPUB_SUP_END__");
    }
    if let Some(sub_val) = b {
        rendered.push_str("__WIKIPEDIA_TO_EPUB_SUB_START__");
        rendered.push_str(&render_templates(sub_val));
        rendered.push_str("__WIKIPEDIA_TO_EPUB_SUB_END__");
    }
    rendered
}

/// [e](https://en.wikipedia.org/wiki/Template:E)
fn render_e_template(params: &str) -> String {
    let positional = template_positional_params(params);
    if let Some(power) = positional.first() {
        format!(
            "× 10__WIKIPEDIA_TO_EPUB_SUP_START__{}__WIKIPEDIA_TO_EPUB_SUP_END__",
            render_templates(power)
        )
    } else {
        String::new()
    }
}

/// [mpl](https://en.wikipedia.org/wiki/Template:Mpl)
fn render_mpl_template(params: &str) -> String {
    let positional = template_positional_params(params);
    positional.join("")
}

/// [columns list](https://en.wikipedia.org/wiki/Template:Columns_list)
fn render_columns_list_template(params: &str) -> String {
    let positional = template_positional_params(params);
    if let Some(content) = positional.first() {
        render_templates(content)
    } else {
        String::new()
    }
}

/// [annotated link](https://en.wikipedia.org/wiki/Template:Annotated_link)
fn render_annotated_link_template(params: &str) -> String {
    let positional = template_positional_params(params);
    if let Some(target) = positional.first() {
        format!("[[{}]]", target)
    } else {
        String::new()
    }
}

/// [Dp](https://en.wikipedia.org/wiki/Template:Dp)
/// [dp](https://en.wikipedia.org/wiki/Template:Dp)
fn render_dp_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let Some(name) = positional.first() else {
        return String::new();
    };
    let target = match name.trim().to_lowercase().as_str() {
        "ceres" => "Ceres (dwarf planet)",
        "eris" => "Eris (dwarf planet)",
        "orcus" => "90482 Orcus",
        "quaoar" => "50000 Quaoar",
        "gonggong" => "225088 Gonggong",
        "sedna" => "90377 Sedna",
        "pluto" => "Pluto",
        "makemake" => "Makemake",
        "haumea" => "Haumea",
        _ => name.trim(),
    };
    format!("[[{}|{}]]", target, name.trim())
}

/// [Visible anchor](https://en.wikipedia.org/wiki/Template:Visible_anchor)
/// [visible anchor](https://en.wikipedia.org/wiki/Template:Visible_anchor)
fn render_visible_anchor_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let text = template_param(&named, &["text", "2"])
        .or_else(|| positional.get(1).map(String::as_str))
        .or_else(|| positional.first().map(String::as_str));

    match text {
        Some(t) => render_templates(t),
        None => String::new(),
    }
}

/// [L1](https://en.wikipedia.org/wiki/Template:L1)
/// [L2](https://en.wikipedia.org/wiki/Template:L2)
/// [L3](https://en.wikipedia.org/wiki/Template:L3)
/// [L4](https://en.wikipedia.org/wiki/Template:L4)
/// [L5](https://en.wikipedia.org/wiki/Template:L5)
pub(crate) fn render_lagrange_template(point: &str) -> String {
    format!(
        "L__WIKIPEDIA_TO_EPUB_SUB_START__{}__WIKIPEDIA_TO_EPUB_SUB_END__",
        point
    )
}

/// [spaces](https://en.wikipedia.org/wiki/Template:Spaces)
fn render_spaces_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let count = positional
        .first()
        .and_then(|val| val.trim().parse::<usize>().ok())
        .unwrap_or(1);
    "\u{00A0}".repeat(count)
}

/// [mpl-](https://en.wikipedia.org/wiki/Template:Mpl-)
fn render_mpl_dash_template(params: &str) -> String {
    let positional = template_positional_params(params);
    if positional.len() >= 3 {
        let number = &positional[0];
        let desig = &positional[1];
        let suffix = &positional[2];
        format!("[[({}) {}{}]]", number, desig, suffix)
    } else if positional.len() == 2 {
        let number = &positional[0];
        let desig = &positional[1];
        format!("[[({}) {}]]", number, desig)
    } else if let Some(first) = positional.first() {
        first.to_string()
    } else {
        String::new()
    }
}

/// [chem](https://en.wikipedia.org/wiki/Template:Chem)
fn render_chem_template(params: &str) -> String {
    fn is_charge(s: &str) -> bool {
        if s == "+" || s == "-" {
            return true;
        }
        if s.ends_with('+') || s.ends_with('-') {
            let num_part = &s[..s.len() - 1];
            return !num_part.is_empty() && num_part.chars().all(|c| c.is_ascii_digit());
        }
        if s.starts_with('+') || s.starts_with('-') {
            let num_part = &s[1..];
            return !num_part.is_empty() && num_part.chars().all(|c| c.is_ascii_digit());
        }
        false
    }

    let positional = template_positional_params(params);
    let mut rendered = String::new();
    for part in positional {
        let trimmed = part.trim();
        if trimmed.chars().all(|c| c.is_ascii_digit()) && !trimmed.is_empty() {
            rendered.push_str("__WIKIPEDIA_TO_EPUB_SUB_START__");
            rendered.push_str(trimmed);
            rendered.push_str("__WIKIPEDIA_TO_EPUB_SUB_END__");
        } else if is_charge(trimmed) && !trimmed.is_empty() {
            rendered.push_str("__WIKIPEDIA_TO_EPUB_SUP_START__");
            rendered.push_str(trimmed);
            rendered.push_str("__WIKIPEDIA_TO_EPUB_SUP_END__");
        } else {
            rendered.push_str(trimmed);
        }
    }
    render_templates(&rendered)
}

/// [solar radius](https://en.wikipedia.org/wiki/Template:Solar_radius)
fn render_solar_radius_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let value = positional.first().map(String::as_str).unwrap_or("");
    let symbol = "R__WIKIPEDIA_TO_EPUB_SUB_START__☉__WIKIPEDIA_TO_EPUB_SUB_END__";
    if value.is_empty() {
        symbol.to_string()
    } else {
        format!("{} {}", value, symbol)
    }
}

/// [±](https://en.wikipedia.org/wiki/Template:%C2%B1)
fn render_plus_minus_template(params: &str) -> String {
    let positional = template_positional_params(params);
    if positional.is_empty() {
        "±".to_string()
    } else {
        format!("± {}", positional.join(" "))
    }
}

/// [Collapsible list](https://en.wikipedia.org/wiki/Template:Collapsible_list)
fn render_collapsible_list_template(params: &str) -> String {
    let named = template_named_params(params);
    let title = template_param(&named, &["title"]);
    let positional = template_positional_params(params);

    let mut parts = Vec::new();
    if let Some(t) = title {
        let t_rendered = render_templates(t);
        if !t_rendered.trim().is_empty() {
            parts.push(t_rendered.trim().to_string());
        }
    }

    for item in positional {
        let item_rendered = render_templates(&item);
        if !item_rendered.trim().is_empty() {
            parts.push(format!("* {}", item_rendered.trim()));
        }
    }

    if parts.is_empty() {
        return String::new();
    }

    format!("\n{}\n", parts.join("\n"))
}

/// [Internet Archive short film](https://en.wikipedia.org/wiki/Template:Internet_Archive_short_film)
fn render_internet_archive_short_film_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let named = template_named_params(params);

    let id = template_param(&named, &["1", "id"])
        .or_else(|| positional.first().map(String::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let Some(id) = id else {
        return String::new();
    };

    let name = template_param(&named, &["2", "name"])
        .or_else(|| positional.get(1).map(String::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Internet Archive short film");

    let url = format!("https://archive.org/details/{id}");
    format!(
        "[[official-url:{url}|''{}'']] at the Internet Archive",
        render_templates(name)
    )
}

/// [ill](https://en.wikipedia.org/wiki/Template:Ill)
/// [illm](https://en.wikipedia.org/wiki/Template:Illm)
/// [Interlanguage link](https://en.wikipedia.org/wiki/Template:Interlanguage_link)
/// [Interlanguage link multi](https://en.wikipedia.org/wiki/Template:Interlanguage_link_multi)
fn render_interlanguage_link_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .collect::<Vec<_>>();

    let Some(article) = params.first().filter(|article| !article.is_empty()) else {
        return String::new();
    };

    let label = params
        .iter()
        .filter_map(|param| param.split_once('='))
        .find_map(|(key, value)| {
            if key.trim().eq_ignore_ascii_case("lt") {
                Some(value.trim())
            } else {
                None
            }
        })
        .filter(|value| !value.is_empty())
        .unwrap_or(article);

    if label == article {
        format_interlanguage_link(article, None, params.get(1))
    } else {
        format_interlanguage_link(article, Some(label), params.get(1))
    }
}

/// [reign](https://en.wikipedia.org/wiki/Template:Reign)
fn render_reign_template(params: &str) -> String {
    let mut positional = Vec::new();
    let mut named = HashMap::new();

    if !params.trim().is_empty() {
        for param in split_template_params(params)
            .into_iter()
            .map(|param| param.trim().to_string())
        {
            if let Some((key, value)) = param.split_once('=') {
                named.insert(key.trim().to_lowercase(), value.trim().to_string());
            } else {
                positional.push(param);
            }
        }
    }

    let label = reign_label(&named);
    let era = named.get("era").map(String::as_str);
    let mut dates = Vec::new();

    if let Some(pre_date) = named.get("pre-date").filter(|value| !value.is_empty()) {
        dates.push(pre_date.to_string());
    }

    if let Some(single) = named
        .get("single")
        .or_else(|| named.get("post-date"))
        .filter(|value| !value.is_empty() && positional.is_empty() && dates.is_empty())
    {
        dates.push(single.to_string());
    } else if !positional.is_empty() {
        dates.push(format_reign_range(
            positional.first().map(String::as_str),
            positional.get(1).map(String::as_str),
        ));
    }

    if let Some(mid_date) = named.get("mid-date").filter(|value| !value.is_empty()) {
        dates.push(mid_date.to_string());
    }

    if positional.get(1).is_some() && positional.get(3).is_some() {
        dates.push(format_reign_range(
            positional.get(2).map(String::as_str),
            positional.get(3).map(String::as_str),
        ));
    }

    if let Some(post_date) = named
        .get("post-date")
        .filter(|value| !value.is_empty() && !positional.is_empty())
    {
        dates.push(post_date.to_string());
    }

    if let Some(era) = era.filter(|value| !value.trim().is_empty())
        && let Some(last) = dates.last_mut()
    {
        last.push(' ');
        last.push_str(era.trim());
    }

    match (label.as_str(), dates.is_empty()) {
        ("", true) => String::new(),
        ("", false) => dates.join(", "),
        (_, true) => label,
        (_, false) => format!("{label} {}", dates.join(", ")),
    }
}

/// [open access](https://en.wikipedia.org/wiki/Template:Open_access)
/// [free access](https://en.wikipedia.org/wiki/Template:Free_access)
pub(crate) fn render_open_access_template() -> String {
    "__WIKIPEDIA_TO_EPUB_OPEN_ACCESS__".to_string()
}

/// [rp](https://en.wikipedia.org/wiki/Template:Rp)
/// [Reference page](https://en.wikipedia.org/wiki/Template:Reference_page)
fn render_reference_page_template(params: &str) -> String {
    let named = template_named_params(params);

    if let Some(pages_val) = template_param(&named, &["pages"]) {
        let rendered_page = render_templates(pages_val).trim().to_string();
        if !rendered_page.is_empty() {
            return format!(" pp. {rendered_page}");
        }
    }

    if let Some(page_val) = template_param(&named, &["page", "1"]) {
        let rendered_page = render_templates(page_val).trim().to_string();
        if !rendered_page.is_empty() {
            return format!(" p. {rendered_page}");
        }
    }

    let pages = split_template_params(params)
        .into_iter()
        .map(|param| render_templates(param.trim()).trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    match pages.as_slice() {
        [] => String::new(),
        [page] => format!(" p. {page}"),
        pages => format!(" pp. {}", pages.join(", ")),
    }
}

fn reign_label(named: &HashMap<String, String>) -> String {
    if let Some(label) = named.get("label").filter(|value| !value.trim().is_empty()) {
        return label.trim().to_string();
    }

    let show = named
        .get("show")
        .or_else(|| named.get("link"))
        .or_else(|| named.get("lk"))
        .map(String::as_str)
        .unwrap_or_default()
        .trim()
        .to_lowercase();
    let capitalized = named.contains_key("cap");

    match show.as_str() {
        "none" | "no" | "n" | "off" | "false" | "0" | "blank" => String::new(),
        "word" => {
            if capitalized {
                "Reigned".to_string()
            } else {
                "reigned".to_string()
            }
        }
        "colon" => {
            if capitalized {
                "Reign:".to_string()
            } else {
                "reign:".to_string()
            }
        }
        "lword" => {
            if capitalized {
                "[[Reign|Reigned]]".to_string()
            } else {
                "[[Reign|reigned]]".to_string()
            }
        }
        "lcolon" => {
            if capitalized {
                "[[Reign|Reign]]:".to_string()
            } else {
                "[[Reign|reign]]:".to_string()
            }
        }
        "link" | "yes" | "y" | "on" | "true" | "1" => {
            if capitalized {
                "[[Reign|R.]]".to_string()
            } else {
                "[[Reign|r.]]".to_string()
            }
        }
        _ => {
            if capitalized {
                "R.".to_string()
            } else {
                "r.".to_string()
            }
        }
    }
}

fn format_reign_range(start: Option<&str>, end: Option<&str>) -> String {
    let start = start.unwrap_or("").trim();
    let end = end.unwrap_or("").trim();
    let start = if start.is_empty() { "?" } else { start };
    let separator = if start.contains(char::is_whitespace) || end.contains(char::is_whitespace) {
        " – "
    } else {
        "–"
    };

    format!("{start}{separator}{end}")
}

fn template_article_params(params: &str) -> Vec<String> {
    split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>()
}

fn join_template_articles(articles: &[String]) -> String {
    let links = articles
        .iter()
        .map(|article| format!("[[{article}]]"))
        .collect::<Vec<_>>();

    join_plain_items(&links)
}

pub(crate) fn join_plain_items(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [link] => link.to_string(),
        [first, second] => format!("{first} and {second}"),
        _ => {
            let last = items.last().cloned().unwrap_or_default();
            let leading = &items[..items.len() - 1];
            format!("{}, and {last}", leading.join(", "))
        }
    }
}

fn format_convert_value(value: &str) -> String {
    value.trim().replace("&minus;", "−")
}

/// [AWOL](https://en.wikipedia.org/wiki/Template:AWOL)
pub(crate) fn render_awol_template() -> String {
    render_templates("&nbsp;([[Absent without leave|{{abbr|AWOL|Desertion}}]])")
}

/// [Assassinated](https://en.wikipedia.org/wiki/Template:Assassinated)
fn render_assassinated_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);
    let link = positional
        .first()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("Assassination");
    let alt = template_param(&named, &["alt"])
        .map(|s| s.trim())
        .unwrap_or("");
    let bold = template_param(&named, &["bold"])
        .map(|s| s.trim())
        .unwrap_or("");

    let label = if alt == "yes" {
        "(Assassinated)".to_string()
    } else if bold == "no" {
        "X".to_string()
    } else {
        "'''X'''".to_string()
    };

    format!("&nbsp;[[{link}|{label}]]")
}

/// [DOW](https://en.wikipedia.org/wiki/Template:DOW)
/// [Died of wounds](https://en.wikipedia.org/wiki/Template:Died_of_wounds)
pub(crate) fn render_died_of_wounds_template() -> String {
    render_templates("&nbsp;([[Killed in action|{{abbr|DOW|Died of wounds}}]])")
}

/// [Executed](https://en.wikipedia.org/wiki/Template:Executed)
fn render_executed_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let link = positional
        .first()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("Capital punishment");
    format!("&nbsp;[[File:Skull and Crossbones.svg|14px|Executed|link={link}]]")
}

/// [KIA](https://en.wikipedia.org/wiki/Template:KIA)
fn render_kia_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);
    let link = positional
        .first()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("Killed in action");
    let alt = template_param(&named, &["alt"])
        .map(|s| s.trim())
        .unwrap_or("");
    let bold = template_param(&named, &["bold"])
        .map(|s| s.trim())
        .unwrap_or("");

    let label = if alt == "yes" {
        "(KIA)".to_string()
    } else if bold == "no" {
        "†".to_string()
    } else {
        "'''†'''".to_string()
    };

    format!("&nbsp;[[{link}|{label}]]")
}

/// [KIA2](https://en.wikipedia.org/wiki/Template:KIA2)
fn render_kia2_template(_params: &str) -> String {
    render_kia_template("alt=yes")
}

/// [MIA](https://en.wikipedia.org/wiki/Template:MIA)
pub(crate) fn render_mia_template() -> String {
    render_templates("&nbsp;([[Missing in action|{{abbr|MIA|Missing in action}}]])")
}

/// [Natural Causes](https://en.wikipedia.org/wiki/Template:Natural_Causes)
fn render_natural_causes_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);
    let link = positional
        .first()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("Manner of death#Natural causes of death");
    let alt = template_param(&named, &["alt"])
        .map(|s| s.trim())
        .unwrap_or("");
    let bold = template_param(&named, &["bold"])
        .map(|s| s.trim())
        .unwrap_or("");

    let label = if alt == "yes" {
        "(Natural causes)".to_string()
    } else if bold == "no" {
        "#".to_string()
    } else {
        "'''#'''".to_string()
    };

    let tooltip_wikitext = format!("{{{{tooltip|{label}|Natural causes}}}}");
    let rendered_tooltip = render_templates(&tooltip_wikitext);

    format!("&nbsp;[[{link}|{rendered_tooltip}]]")
}

/// [PKIA](https://en.wikipedia.org/wiki/Template:PKIA)
pub(crate) fn render_pkia_template() -> String {
    render_templates("&nbsp;([[Killed in action|{{abbr|PKIA|Presumed killed in action}}]])")
}

/// [POW](https://en.wikipedia.org/wiki/Template:POW)
pub(crate) fn render_pow_template() -> String {
    render_templates(
        "&#x20;<span style=\"white-space:nowrap\">([[Prisoner of war|{{abbr|POW|Prisoner of war}}]])</span>",
    )
}

/// [Suicide](https://en.wikipedia.org/wiki/Template:Suicide)
fn render_suicide_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);
    let link = positional
        .first()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("Suicide");
    let alt = template_param(&named, &["alt"])
        .map(|s| s.trim())
        .unwrap_or("");
    let bold = template_param(&named, &["bold"])
        .map(|s| s.trim())
        .unwrap_or("");

    let label = if alt == "yes" {
        let abbr_wikitext = "{{abbr|Suicide|[[Suicide]]}}";
        format!("({})", render_templates(abbr_wikitext))
    } else if bold == "no" {
        "‡‡".to_string()
    } else {
        "'''‡‡'''".to_string()
    };

    format!("&nbsp;[[{link}|{label}]]")
}

/// [Surrendered](https://en.wikipedia.org/wiki/Template:Surrendered)
fn render_surrendered_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let link = positional
        .first()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("Surrender (military)");
    format!("&nbsp;[[File:White flag icon.svg|14px|Surrendered|link={link}]]")
}

/// [Turncoat](https://en.wikipedia.org/wiki/Template:Turncoat)
fn render_turncoat_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let link = positional
        .first()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("Turncoat");
    format!("&nbsp;[[File:Black flag icon.svg|14px|Turncoat|link={link}]]")
}

/// [WIA](https://en.wikipedia.org/wiki/Template:WIA)
pub(crate) fn render_wia_template() -> String {
    render_templates("&nbsp;([[Wounded in action|{{abbr|WIA|Wounded in action}}]])")
}

/// [NDLDC](https://en.wikipedia.org/wiki/Template:NDLDC)
fn render_ndldc_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let id = template_param(&named, &["id", "1"])
        .map(|s| s.trim())
        .or_else(|| positional.first().map(|s| s.trim()))
        .unwrap_or("");

    let format_param = template_param(&named, &["format"])
        .map(|s| s.trim().to_lowercase())
        .unwrap_or_default();

    if id.is_empty() {
        return String::new();
    }

    if format_param.is_empty() {
        // Raw link format
        format!("https://dl.ndl.go.jp/info:ndljp/pid/{id}")
    } else {
        match format_param.as_str() {
            "url" => {
                let param2 = template_param(&named, &["2"])
                    .or_else(|| positional.get(1).map(|s| s.as_str()))
                    .unwrap_or("");
                format!("https://dl.ndl.go.jp/en/pid/{id}{param2}")
            }
            "pid" => {
                format!("[[ndlpid (identifier)|ndlpid]]:[https://dl.ndl.go.jp/en/pid/{id} {id}]")
            }
            "digimeta" => {
                format!("[[ndlpid (identifier)|ndlpid]]:[https://dl.ndl.go.jp/en/pid/{id} {id}]")
            }
            "ndljp" => {
                format!(
                    "[[ndlpid (identifier)|ndljp]]:[https://dl.ndl.go.jp/info:ndljp/pid/{id} {id}]"
                )
            }
            "doi" => {
                format!("doi:10.11501/{id}")
            }
            "hdl" => {
                format!("hdl:10.11501/{id}")
            }
            "external" => {
                let param2 = template_param(&named, &["2"])
                    .or_else(|| positional.get(1).map(|s| s.as_str()))
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .unwrap_or("");
                let link_text = if param2.is_empty() {
                    "NDL Digital Collections"
                } else {
                    param2
                };
                format!(
                    "\"[https://dl.ndl.go.jp/info:ndljp/pid/{id} {link_text}]\" - [[National Diet Library#National Diet Library Digital Collections|NDL Digital Collections]]"
                )
            }
            _ => {
                format!("https://dl.ndl.go.jp/info:ndljp/pid/{id}")
            }
        }
    }
}

/// [Station](https://en.wikipedia.org/wiki/Template:Station)
fn render_station_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let station_name = positional
        .first()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("");
    if station_name.is_empty() {
        return String::new();
    }

    let capitalize = positional
        .get(1)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .is_some();
    let suffix = positional
        .get(2)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("");
    let label = template_param(&named, &["alt"])
        .or_else(|| positional.get(3).map(String::as_str))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(station_name);

    let station_word = if capitalize { "Station" } else { "station" };

    let target = if suffix.is_empty() {
        format!("{station_name} {station_word}")
    } else {
        format!("{station_name} {station_word} ({suffix})")
    };

    format!("[[{target}|{label}]]")
}

struct TrackGaugeData {
    formatted: &'static str,
    alias: &'static str,
    alias_link: &'static str,
}

fn normalize_gauge_input(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect()
}

fn lookup_track_gauge(normalized: &str) -> Option<TrackGaugeData> {
    match normalized {
        "1435mm" | "1435" | "standardgauge" | "standard" | "4ft812in" | "4ft85in" | "4ft812" => {
            Some(TrackGaugeData {
                formatted: "1,435 mm (4 ft 8+1\u{2044}2 in)",
                alias: "standard gauge",
                alias_link: "[[standard-gauge railway|standard gauge]]",
            })
        }
        "1067mm" | "1067" | "capegauge" | "cape" | "3ft6in" | "3ft6" => Some(TrackGaugeData {
            formatted: "1,067 mm (3 ft 6 in)",
            alias: "Cape gauge",
            alias_link: "[[Cape gauge]]",
        }),
        "1000mm" | "1000" | "metregauge" | "metergauge" | "metre" | "meter" | "3ft338in"
        | "3ft338" => Some(TrackGaugeData {
            formatted: "1,000 mm (3 ft 3+3\u{2044}8 in)",
            alias: "metre gauge",
            alias_link: "[[Meter gauge|metre gauge]]",
        }),
        "1520mm" | "1520" | "russiangauge" | "russian" | "4ft112732in" | "4ft112732" => {
            Some(TrackGaugeData {
                formatted: "1,520 mm (4 ft 11+27\u{2044}32 in)",
                alias: "Russian gauge",
                alias_link: "[[5 ft and 1520 mm track gauge|Russian gauge]]",
            })
        }
        "1524mm" | "1524" | "5ft" | "5ftgauge" => Some(TrackGaugeData {
            formatted: "1,524 mm (5 ft)",
            alias: "5 ft gauge",
            alias_link: "[[5 ft and 1520 mm track gauge|5 ft gauge]]",
        }),
        "1668mm" | "1668" | "iberiangauge" | "iberian" | "5ft52132in" | "5ft52132" => {
            Some(TrackGaugeData {
                formatted: "1,668 mm (5 ft 5+21\u{2044}32 in)",
                alias: "Iberian gauge",
                alias_link: "[[Iberian gauge]]",
            })
        }
        "1676mm" | "1676" | "indiangauge" | "indian" | "5ft6in" | "5ft6" => Some(TrackGaugeData {
            formatted: "1,676 mm (5 ft 6 in)",
            alias: "Indian gauge",
            alias_link: "[[5 ft 6 in gauge|Indian gauge]]",
        }),
        "762mm" | "762" | "2ft6in" | "2ft6" | "762mmgauge" => Some(TrackGaugeData {
            formatted: "762 mm (2 ft 6 in)",
            alias: "2 ft 6 in gauge",
            alias_link: "[[2 ft 6 in gauge]]",
        }),
        "600mm" | "600" | "2ft" | "2ftgauge" | "1ft1158in" | "1ft1158" => Some(TrackGaugeData {
            formatted: "600 mm (1 ft 11+5\u{2044}8 in)",
            alias: "2 ft gauge",
            alias_link: "[[2 ft and 600 mm gauge railways|2 ft gauge]]",
        }),
        _ => None,
    }
}

/// [Track gauge](https://en.wikipedia.org/wiki/Template:Track_gauge)
/// [RailGauge](https://en.wikipedia.org/wiki/Template:Track_gauge)
fn render_track_gauge_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let raw_input = positional
        .first()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or("");
    if raw_input.is_empty() {
        return String::new();
    }

    let normalized = normalize_gauge_input(raw_input);

    let al = template_param(&named, &["al"])
        .map(|s| s.trim().to_lowercase())
        .unwrap_or_default();
    let allk = template_param(&named, &["allk"])
        .map(|s| s.trim().to_lowercase())
        .unwrap_or_default();
    let is_al = al == "on" || al == "yes";
    let is_allk = allk == "on" || allk == "yes";

    if let Some(gauge) = lookup_track_gauge(&normalized) {
        let mut result = gauge.formatted.to_string();
        if is_allk {
            result.push(' ');
            result.push_str(gauge.alias_link);
        } else if is_al {
            result.push(' ');
            result.push_str(gauge.alias);
        }
        result
    } else {
        raw_input.to_string()
    }
}

/// [JPN](https://en.wikipedia.org/wiki/Template:JPN)
fn render_jpn_template(params: &str) -> String {
    let named = template_named_params(params);
    if let Some(name) = template_param(&named, &["name"])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        format!("🇯🇵 [[Japan|{name}]]")
    } else {
        "🇯🇵 [[Japan]]".to_string()
    }
}

/// [flagdeco](https://en.wikipedia.org/wiki/Template:Flagdeco)
fn render_flagdeco_template(_params: &str) -> String {
    String::new()
}

/// [pprime](https://en.wikipedia.org/wiki/Template:Pprime)
fn render_pprime_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let first = positional.first().cloned().unwrap_or_default();
    format!("{}″", first.trim())
}

/// [RA](https://en.wikipedia.org/wiki/Template:Right_ascension)
fn render_ra_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let mut parts = Vec::new();
    if let Some(h) = positional.first()
        && !h.trim().is_empty()
    {
        parts.push(format!(
            "{}__WIKIPEDIA_TO_EPUB_SUP_START__h__WIKIPEDIA_TO_EPUB_SUP_END__",
            h.trim()
        ));
    }
    if let Some(m) = positional.get(1)
        && !m.trim().is_empty()
    {
        parts.push(format!(
            "{}__WIKIPEDIA_TO_EPUB_SUP_START__m__WIKIPEDIA_TO_EPUB_SUP_END__",
            m.trim()
        ));
    }
    if let Some(s) = positional.get(2)
        && !s.trim().is_empty()
    {
        parts.push(format!(
            "{}__WIKIPEDIA_TO_EPUB_SUP_START__s__WIKIPEDIA_TO_EPUB_SUP_END__",
            s.trim()
        ));
    }
    parts.join(" ")
}

/// [Hyphen](https://en.wikipedia.org/wiki/Template:Hyphen)
fn render_hyphen_template(_params: &str) -> String {
    "-".to_string()
}

fn format_interlanguage_link(
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

pub(crate) fn get_dispatch_table() -> DispatchTable {
    HashMap::from([
        (
            "airport codes",
            render_airport_codes_template as TemplateHandler,
        ),
        (
            "airport-dest-list",
            render_airport_dest_list_template as TemplateHandler,
        ),
        (
            "nws-current",
            render_nws_current_template as TemplateHandler,
        ),
        ("right", render_right_template as TemplateHandler),
        (
            "wikibooks inline",
            render_wikibooks_inline_template as TemplateHandler,
        ),
        (
            "wikibooks-inline",
            render_wikibooks_inline_template as TemplateHandler,
        ),
        ("refh", render_refh_template as TemplateHandler),
        ("M", render_m_template as TemplateHandler),
        ("m", render_m_template as TemplateHandler),
        ("earthquake magnitude", render_m_template as TemplateHandler),
        ("nowrap", render_passthrough_template as TemplateHandler),
        ("nobr", render_passthrough_template as TemplateHandler),
        (
            "collapse top",
            render_collapse_top_template as TemplateHandler,
        ),
        ("var", render_var_template as TemplateHandler),
        ("gaps", render_gaps_template as TemplateHandler),
        ("center", render_passthrough_template as TemplateHandler),
        (
            "crossreference",
            render_passthrough_template as TemplateHandler,
        ),
        ("nobold", render_passthrough_template as TemplateHandler),
        ("stack", render_passthrough_template as TemplateHandler),
        ("longitem", render_passthrough_template as TemplateHandler),
        ("xref", render_passthrough_template as TemplateHandler),
        ("quote box", render_blockquote_template as TemplateHandler),
        ("quote", render_blockquote_template as TemplateHandler),
        ("cquote", render_cquote_template as TemplateHandler),
        ("term", render_term_template as TemplateHandler),
        ("defn", render_defn_template as TemplateHandler),
        ("us$", render_us_dollar_template as TemplateHandler),
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
        ("smaller", render_smaller_template as TemplateHandler),
        ("small", render_smaller_template as TemplateHandler),
        ("sic", render_sic_template as TemplateHandler),
        ("circa", render_circa_template as TemplateHandler),
        ("c.", render_circa_template as TemplateHandler),
        ("cx", render_circa_template as TemplateHandler),
        ("isbn", render_isbn_template as TemplateHandler),
        ("asin", render_asin_template as TemplateHandler),
        ("oclc", render_oclc_template as TemplateHandler),
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
        ("sfrac", render_sfrac_template as TemplateHandler),
        ("mvar", render_mvar_template as TemplateHandler),
        ("math", render_math_template as TemplateHandler),
        ("tmath", render_tmath_template as TemplateHandler),
        ("jstor", render_jstor_template as TemplateHandler),
        ("wspsm", render_wspsm_template as TemplateHandler),
        ("em", render_em_template as TemplateHandler),
        ("mathworld", render_mathworld_template as TemplateHandler),
        ("as ref", render_as_ref_template as TemplateHandler),
        ("oeis2c", render_oeis2c_template as TemplateHandler),
        ("thinsp", render_thinsp_template as TemplateHandler),
        ("dfn", render_dfn_template as TemplateHandler),
        ("subsup", render_subsup_template as TemplateHandler),
        ("abs", render_abs_template as TemplateHandler),
        ("mono", render_mono_template as TemplateHandler),
        ("pi", render_pi_template as TemplateHandler),
        ("springer", render_springer_template as TemplateHandler),
        (
            "closed-open",
            render_closed_open_template as TemplateHandler,
        ),
        ("sqrt", render_sqrt_template as TemplateHandler),
        (
            "section link",
            render_section_link_template as TemplateHandler,
        ),
        ("mset", render_mset_template as TemplateHandler),
        (
            "hidden begin",
            render_hidden_begin_template as TemplateHandler,
        ),
        ("floruit", render_floruit_template as TemplateHandler),
        ("coord", render_coord_template as TemplateHandler),
        ("rp", render_reference_page_template as TemplateHandler),
        (
            "reference page",
            render_reference_page_template as TemplateHandler,
        ),
        ("as of", render_as_of_template as TemplateHandler),
        ("died-in", render_died_in_template as TemplateHandler),
        ("blockquote", render_blockquote_template as TemplateHandler),
        ("for", render_for_template as TemplateHandler),
        (
            "for timeline",
            render_for_timeline_template as TemplateHandler,
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
        ("flagdeco", render_flagdeco_template as TemplateHandler),
        ("pprime", render_pprime_template as TemplateHandler),
        ("ra", render_ra_template as TemplateHandler),
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
        ("ship", render_generic_ship_template as TemplateHandler),
        ("proto", render_proto_template as TemplateHandler),
        (
            "infobox",
            render_infobox_generic_template as TemplateHandler,
        ),
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
        ("note", render_note_template as TemplateHandler),
        (
            "fs interlinear",
            render_fs_interlinear_template as TemplateHandler,
        ),
        ("tooltip", render_tooltip_template as TemplateHandler),
        ("jaanus", render_jaanus_template as TemplateHandler),
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
        ("usurped", render_usurped_template as TemplateHandler),
        ("break", render_break_template as TemplateHandler),
        ("br", render_break_template as TemplateHandler),
        ("brk", render_break_template as TemplateHandler),
        ("crlf", render_break_template as TemplateHandler),
        ("jct", render_jct_template as TemplateHandler),
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
        ("ublist", render_unbulleted_list_template as TemplateHandler),
        ("parabr", render_parabr_template as TemplateHandler),
        (
            "age in years, months, weeks and days",
            render_age_in_years_months_weeks_days_template as TemplateHandler,
        ),
        ("est.", render_est_template as TemplateHandler),
        (
            "britannica url",
            render_britannica_url_template as TemplateHandler,
        ),
        ("olist", render_ordered_list_template as TemplateHandler),
        (
            "ordered list",
            render_ordered_list_template as TemplateHandler,
        ),
        ("webtrans", render_webtrans_template as TemplateHandler),
        ("osm", render_osm_template as TemplateHandler),
        (
            "wiktionary-inline",
            render_wiktionary_inline_template as TemplateHandler,
        ),
        (
            "wiktionary inline",
            render_wiktionary_inline_template as TemplateHandler,
        ),
        ("wti", render_wiktionary_inline_template as TemplateHandler),
        ("colorbull", render_colorbull_template as TemplateHandler),
        (
            "portal-inline",
            render_portal_inline_template as TemplateHandler,
        ),
        (
            "portal inline",
            render_portal_inline_template as TemplateHandler,
        ),
        ("mp", render_mp_template as TemplateHandler),
        ("minor planet", render_mp_template as TemplateHandler),
    ])
}

fn render_parabr_template(_params: &str) -> String {
    "__WIKIPEDIA_TO_EPUB_BR____WIKIPEDIA_TO_EPUB_BR__".to_string()
}

fn render_age_in_years_months_weeks_days_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let mut y1 = named.get("year1").and_then(|s| s.parse::<i32>().ok());
    let mut m1 = named.get("month1").and_then(|s| s.parse::<i32>().ok());
    let mut d1 = named.get("day1").and_then(|s| s.parse::<i32>().ok());

    let mut y2 = named.get("year2").and_then(|s| s.parse::<i32>().ok());
    let mut m2 = named.get("month2").and_then(|s| s.parse::<i32>().ok());
    let mut d2 = named.get("day2").and_then(|s| s.parse::<i32>().ok());

    if y1.is_none() || m1.is_none() || d1.is_none() {
        if positional.len() >= 6 {
            y1 = positional[0].parse::<i32>().ok();
            m1 = positional[1].parse::<i32>().ok();
            d1 = positional[2].parse::<i32>().ok();
            y2 = positional[3].parse::<i32>().ok();
            m2 = positional[4].parse::<i32>().ok();
            d2 = positional[5].parse::<i32>().ok();
        } else if positional.len() >= 3 {
            y1 = positional[0].parse::<i32>().ok();
            m1 = positional[1].parse::<i32>().ok();
            d1 = positional[2].parse::<i32>().ok();
        }
    }

    let Some(y1) = y1 else {
        return String::new();
    };
    let Some(m1) = m1 else {
        return String::new();
    };
    let Some(d1) = d1 else {
        return String::new();
    };

    let (y2, m2, d2) = if let (Some(y), Some(m), Some(d)) = (y2, m2, d2) {
        (y, m, d)
    } else {
        current_utc_date()
    };

    let days1 = days_from_year_zero(y1, m1, d1);
    let days2 = days_from_year_zero(y2, m2, d2);
    if days1 > days2 {
        return String::new();
    }

    let mut years = y2 - y1;
    let mut months = m2 - m1;
    let mut days = d2 - d1;

    if days < 0 {
        months -= 1;
        let prev_m = if m2 == 1 { 12 } else { m2 - 1 };
        let prev_y = if m2 == 1 { y2 - 1 } else { y2 };
        let is_leap = (prev_y % 4 == 0 && prev_y % 100 != 0) || (prev_y % 400 == 0);
        let month_lengths = if is_leap {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };
        days += month_lengths[prev_m as usize - 1];
    }

    if months < 0 {
        years -= 1;
        months += 12;
    }

    let mut parts = Vec::new();
    if years > 0 {
        parts.push(if years == 1 {
            "1 year".to_string()
        } else {
            format!("{} years", years)
        });
    }
    if months > 0 {
        parts.push(if months == 1 {
            "1 month".to_string()
        } else {
            format!("{} months", months)
        });
    }
    let weeks = days / 7;
    let rem_days = days % 7;
    if weeks > 0 {
        parts.push(if weeks == 1 {
            "1 week".to_string()
        } else {
            format!("{} weeks", weeks)
        });
    }
    if rem_days > 0 {
        parts.push(if rem_days == 1 {
            "1 day".to_string()
        } else {
            format!("{} days", rem_days)
        });
    }

    if parts.is_empty() {
        return "0 days".to_string();
    }

    if parts.len() == 1 {
        parts[0].clone()
    } else {
        let last = parts.pop().unwrap();
        format!("{} and {}", parts.join(", "), last)
    }
}

fn render_est_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let val = positional.first().map(|s| s.trim()).unwrap_or("");

    let abbr = "__WIKIPEDIA_TO_EPUB_ABBR_START__estimate__WIKIPEDIA_TO_EPUB_ABBR_VALUE__est.__WIKIPEDIA_TO_EPUB_ABBR_END__";
    if val.is_empty() {
        abbr.to_string()
    } else {
        format!("{} {}", abbr, render_templates(val))
    }
}

fn render_britannica_url_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let url = named
        .get("url")
        .or_else(|| named.get("1"))
        .or_else(|| positional.first())
        .map(|s| s.trim())
        .unwrap_or("");
    let title = named
        .get("title")
        .or_else(|| named.get("2"))
        .or_else(|| positional.get(1))
        .map(|s| s.trim())
        .unwrap_or("Encyclopædia Britannica");
    let author = named
        .get("author")
        .or_else(|| named.get("3"))
        .or_else(|| positional.get(2))
        .map(|s| s.trim())
        .unwrap_or("");

    if url.is_empty() {
        return String::new();
    }

    let link = format!("\"[[official-url:{}|{}]]\"", url, render_templates(title));
    if !author.is_empty() {
        format!(
            "{} by {} at ''Encyclopædia Britannica''",
            link,
            render_templates(author)
        )
    } else {
        format!("{} at ''Encyclopædia Britannica''", link)
    }
}

fn render_ordered_list_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let mut items = Vec::new();
    for param in positional {
        let trimmed = param.trim();
        if !trimmed.is_empty() {
            items.push(format!("# {}", render_templates(trimmed)));
        }
    }
    if items.is_empty() {
        String::new()
    } else {
        format!("\n{}", items.join("\n"))
    }
}

fn render_webtrans_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let named = template_named_params(params);

    let url = template_param(&named, &["url"])
        .or_else(|| positional.first().map(String::as_str))
        .map(str::trim)
        .unwrap_or("");
    let title = template_param(&named, &["title"])
        .or_else(|| positional.get(1).map(String::as_str))
        .map(str::trim)
        .unwrap_or(url);
    let lang = template_param(&named, &["lang"])
        .or_else(|| positional.get(2).map(String::as_str))
        .map(str::trim)
        .unwrap_or("");

    if url.is_empty() {
        return String::new();
    }

    let rendered_title = render_templates(title);
    let mut link = format!("[[official-url:{}|{}]]", url, rendered_title);
    if !lang.is_empty() {
        let lang_lower = lang.to_ascii_lowercase();
        let lang_name = match lang_lower.as_str() {
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
            other => other,
        };
        link = format!("{} (in {})", link, lang_name);
    }
    link
}

fn render_osm_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let named = template_named_params(params);

    if let Some(relation_id) = template_param(&named, &["relation"]) {
        let id = relation_id.trim();
        if !id.is_empty() {
            return format!("[[osmrelation:{}|{}]]", id, id);
        }
    }

    let type_val = template_param(&named, &["1"])
        .or_else(|| positional.first().map(String::as_str))
        .map(str::trim)
        .unwrap_or("")
        .to_lowercase();
    let id = template_param(&named, &["2"])
        .or_else(|| positional.get(1).map(String::as_str))
        .map(str::trim)
        .unwrap_or("");
    let name = template_param(&named, &["3"])
        .or_else(|| positional.get(2).map(String::as_str))
        .map(str::trim)
        .unwrap_or("");

    if id.is_empty() {
        return String::new();
    }

    let label = if name.is_empty() {
        format!("{} on OpenStreetMap", id)
    } else {
        format!("{} {} on OpenStreetMap", id, render_templates(name))
    };

    let type_char = type_val.chars().next().unwrap_or(' ');
    match type_char {
        'r' => format!("[[osmrelation:{}|{}]]", id, label),
        'w' => format!("[[osmway:{}|{}]]", id, label),
        _ => format!(
            "[[official-url:https://www.openstreetmap.org/node/{}|{}]]",
            id, label
        ),
    }
}

fn render_wiktionary_inline_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let named = template_named_params(params);

    let term = template_param(&named, &["1", "term"])
        .or_else(|| positional.first().map(String::as_str))
        .map(str::trim)
        .unwrap_or("");
    let label = template_param(&named, &["2", "displayed text"])
        .or_else(|| positional.get(1).map(String::as_str))
        .map(str::trim)
        .unwrap_or(term);
    let extratext = template_param(&named, &["extratext"])
        .map(str::trim)
        .unwrap_or("");

    if term.is_empty() {
        return String::new();
    }

    let rendered_label = render_templates(label);
    if extratext.is_empty() {
        format!(
            "The dictionary definition of [[wikt:{}|{}]] at Wiktionary",
            term, rendered_label
        )
    } else {
        format!(
            "The dictionary definition of [[wikt:{}|{}]] at Wiktionary, {}",
            term,
            rendered_label,
            render_templates(extratext)
        )
    }
}

fn render_colorbull_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let named = template_named_params(params);

    let color = template_param(&named, &["1"])
        .or_else(|| positional.first().map(String::as_str))
        .map(str::trim)
        .unwrap_or("black");
    let shape = template_param(&named, &["2"])
        .or_else(|| positional.get(1).map(String::as_str))
        .map(str::trim)
        .unwrap_or("square")
        .to_lowercase();
    let wikilink = template_param(&named, &["3"])
        .or_else(|| positional.get(2).map(String::as_str))
        .map(str::trim)
        .unwrap_or("");

    let shape_char = match shape.as_str() {
        "c" | "circle" | "r" | "round" => "○",
        "d" | "diamond" => "◇",
        "tu" | "up" | "uptriangle" => "△",
        "td" | "dn" | "downtriangle" => "▽",
        "tl" | "lt" | "lefttriangle" => "◁",
        "tr" | "rt" | "righttriangle" => "▷",
        _ => "■",
    };

    let colored_shape = format!(
        "__WIKIPEDIA_TO_EPUB_COLOR_START__{}__WIKIPEDIA_TO_EPUB_COLOR_MID__{}__WIKIPEDIA_TO_EPUB_COLOR_END__",
        color, shape_char
    );

    if wikilink.is_empty() {
        colored_shape
    } else {
        format!("[[{}|{}]]", wikilink, colored_shape)
    }
}

fn render_portal_inline_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let named = template_named_params(params);

    let portal_name = template_param(&named, &["1"])
        .or_else(|| positional.first().map(String::as_str))
        .map(str::trim)
        .unwrap_or("");
    let text = template_param(&named, &["text"])
        .map(str::trim)
        .unwrap_or("");
    let short = template_param(&named, &["short"])
        .map(str::trim)
        .unwrap_or("");

    if portal_name.is_empty() {
        return String::new();
    }

    let label = if !text.is_empty() {
        render_templates(text)
    } else if !short.is_empty() {
        render_templates(portal_name)
    } else {
        format!("{} portal", render_templates(portal_name))
    };

    format!("[[Portal:{}|{}]]", portal_name, label)
}

fn render_mp_template(params: &str) -> String {
    let positional = template_positional_params(params);
    if positional.is_empty() {
        return String::new();
    }

    // Check if satellite
    if positional[0].trim().eq_ignore_ascii_case("S") {
        match positional.len() {
            1 => return "S/".to_string(),
            2 => return format!("S/{}", positional[1].trim()),
            3 => return format!("S/{} ({})", positional[1].trim(), positional[2].trim()),
            4 => {
                return format!(
                    "S/{} ({}) {}",
                    positional[1].trim(),
                    positional[2].trim(),
                    positional[3].trim()
                );
            }
            _ => {
                let year = positional[1].trim();
                let primary = positional[2].trim();
                let sub_val = positional[3].trim();
                let trailing = positional[4].trim();
                return format!(
                    "S/{} ({}__WIKIPEDIA_TO_EPUB_SUB_START__{}__WIKIPEDIA_TO_EPUB_SUB_END__) {}",
                    year, primary, sub_val, trailing
                );
            }
        }
    }

    match positional.len() {
        1 => positional[0].trim().to_string(),
        2 => {
            let p0 = positional[0].trim();
            let p1 = positional[1].trim();
            if p0.chars().all(|c| c.is_ascii_digit()) {
                format!("({}) {}", p0, p1)
            } else {
                format!(
                    "{}__WIKIPEDIA_TO_EPUB_SUB_START__{}__WIKIPEDIA_TO_EPUB_SUB_END__",
                    p0, p1
                )
            }
        }
        _ => {
            let p0 = positional[0].trim();
            let p1 = positional[1].trim();
            let p2 = positional[2].trim();
            format!(
                "({}) {}__WIKIPEDIA_TO_EPUB_SUB_START__{}__WIKIPEDIA_TO_EPUB_SUB_END__",
                p0, p1, p2
            )
        }
    }
}

fn render_airport_codes_template(params: &str) -> String {
    let parts = split_template_params(params);
    let named = template_named_params(params);

    let mut positional = Vec::new();
    for part in parts {
        let trimmed = part.trim();
        if split_parameter_by_equals(trimmed).is_none() {
            positional.push(trimmed.to_string());
        }
    }

    let labels = ["IATA", "ICAO", "FAA", "TC", "GPS", "CAAC"];
    let mut codes = Vec::new();
    for (i, val) in positional.iter().enumerate() {
        if i < labels.len() {
            let val = val.trim();
            if !val.is_empty() {
                codes.push(format!("{}: {}", labels[i], val));
            }
        }
    }

    if codes.is_empty() {
        return String::new();
    }

    let joined = codes.join(", ");
    let p = template_param(&named, &["p"]);
    if p == Some("n") {
        joined
    } else {
        format!("({joined})")
    }
}

fn render_airport_dest_list_template(params: &str) -> String {
    let parts = split_template_params(params);
    let named = template_named_params(params);

    let mut positional = Vec::new();
    for part in parts {
        let trimmed = part.trim();
        if split_parameter_by_equals(trimmed).is_none() {
            positional.push(trimmed.to_string());
        }
    }

    let mut rows = vec![
        "{| class=\"wikitable\"".to_string(),
        "|-".to_string(),
        "! Airlines".to_string(),
        "! Destinations".to_string(),
    ];

    let col3_title = template_param(&named, &["3rdcoltitle", "3rdcol"]);
    if let Some(title) = col3_title {
        rows.push(format!("! {}", render_templates(title)));
    }

    let chunk_size = if col3_title.is_some() { 3 } else { 2 };
    for chunk in positional.chunks(chunk_size) {
        if chunk.len() >= 2 {
            rows.push("|-".to_string());
            rows.push(format!("| {}", render_templates(&chunk[0])));
            rows.push(format!("| {}", render_templates(&chunk[1])));
            if chunk_size == 3 && chunk.len() >= 3 {
                rows.push(format!("| {}", render_templates(&chunk[2])));
            } else if chunk_size == 3 {
                rows.push("|".to_string());
            }
        }
    }

    rows.push("|}".to_string());
    rows.join("\n")
}

fn render_nws_current_template(params: &str) -> String {
    let parts = split_template_params(params);
    let mut positional = Vec::new();
    for part in parts {
        let trimmed = part.trim();
        if split_parameter_by_equals(trimmed).is_none() {
            positional.push(trimmed.to_string());
        }
    }

    if positional.is_empty() {
        return String::new();
    }

    let icao = &positional[0];
    let name = if positional.len() > 1 && !positional[1].is_empty() {
        &positional[1]
    } else {
        icao
    };

    format!(
        "[http://tgftp.nws.noaa.gov/weather/current/{}.html Current weather for {}] at NOAA/NWS",
        icao, name
    )
}

fn render_right_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let named = template_named_params(params);

    let content = template_param(&named, &["1"])
        .or_else(|| positional.first().map(String::as_str))
        .map(str::trim)
        .unwrap_or("");

    if content.is_empty() {
        "style=\"text-align:right\"|".to_string()
    } else {
        format!(
            "<div style=\"float:right;\">{}</div>",
            render_templates(content)
        )
    }
}

fn render_wikibooks_inline_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    if let Some(links) = template_param(&named, &["links"]) {
        return format!("{} at Wikibooks", render_templates(links));
    }

    let book = template_param(&named, &["1"])
        .or_else(|| positional.first().map(String::as_str))
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let Some(book) = book else {
        return String::new();
    };

    let label = template_param(&named, &["2"])
        .or_else(|| positional.get(1).map(String::as_str))
        .map(str::trim)
        .unwrap_or(book);

    format!("[[b:{}|{}]] at Wikibooks", book, render_templates(label))
}

fn render_refh_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);
    let multi = template_param(&named, &["multi"])
        .or_else(|| positional.first().map(String::as_str))
        .unwrap_or("yes");

    if multi == "no" {
        "__WIKIPEDIA_TO_EPUB_ABBR_START__Reference__WIKIPEDIA_TO_EPUB_ABBR_VALUE__Ref.__WIKIPEDIA_TO_EPUB_ABBR_END__".to_string()
    } else {
        "__WIKIPEDIA_TO_EPUB_ABBR_START__References__WIKIPEDIA_TO_EPUB_ABBR_VALUE__Refs.__WIKIPEDIA_TO_EPUB_ABBR_END__".to_string()
    }
}

fn render_m_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let code = template_param(&named, &["1"])
        .or_else(|| positional.first().map(String::as_str))
        .map(str::trim)
        .unwrap_or("");

    if code.is_empty() {
        return String::new();
    }

    let val = template_param(&named, &["2"])
        .or_else(|| positional.get(1).map(String::as_str))
        .map(str::trim)
        .unwrap_or("");

    let link = template_param(&named, &["link"]).is_some();
    let src = template_param(&named, &["src"])
        .map(str::trim)
        .unwrap_or("");

    // Special cases
    let (label, _anchor) = if code == "Magnitude" || code == "magnitude" {
        ("[[Seismic magnitude scales|Magnitude]]".to_string(), None)
    } else if code == "M" {
        ("[[Seismic magnitude scales|M]]".to_string(), None)
    } else if code == "Mag" || code == "Mag." || code == "mag" || code == "mag." {
        ("[[Seismic magnitude scales|Mag.]]".to_string(), None)
    } else {
        let code_lc = code.to_lowercase();
        let (label_str, anchor_str) = match code_lc.as_str() {
            "?" => ("M", None),
            "??" => ("M", None),
            "r?" => ("M", Some("ML")),
            "uk" | "unk" | "ukn" | "unknown" => ("M<sub>uk</sub>", Some("Muk")),
            "l" => ("M<sub>L</sub>", Some("ML")),
            "jma" | "j" => ("M<sub>JMA</sub>", Some("Mjma")),
            "h" => ("M<sub>h</sub>", Some("Mh")),
            "0" => ("M<sub>0</sub>", Some("M0")),
            "0tex" => ("M<sub>0</sub>", Some("M0")),
            "." => ("M", Some("Mw")),
            "w" | "mw" => ("M<sub>w</sub>", Some("Mw")),
            "wp" | "mwp" => ("M<sub>wp</sub>", Some("Mwp")),
            "wpd" | "mwpd" => ("M<sub>wpd</sub>", Some("Mwpd")),
            "wb" | "mwb" => ("M<sub>wb</sub>", Some("Mwb")),
            "wc" | "mwc" => ("M<sub>wc</sub>", Some("Mwc")),
            "wr" | "mwr" => ("M<sub>wr</sub>", Some("Mwr")),
            "ww" | "mww" => ("M<sub>ww</sub>", Some("Mww")),
            "s" => {
                if code == "S" {
                    ("M<sub>S</sub>", Some("Ms"))
                } else {
                    ("M<sub>s</sub>", Some("Ms"))
                }
            }
            "gr" => ("M<sub>GR</sub>", Some("Mgr")),
            "s20" => ("M<sub>s20</sub>", Some("Ms")),
            "sbb" => ("M<sub>sBB</sub>", Some("Ms")),
            "z" => ("M<sub>z</sub>", Some("Mz")),
            "s7" => ("M<sub>s7</sub>", Some("Ms7")),
            "sn" => ("M<sub>sn</sub>", Some("Msn")),
            "lh" => ("M<sub>LH</sub>", Some("MLH")),
            "v" => ("M<sub>V</sub>", Some("MV")),
            "r" => ("M<sub>R</sub>", Some("MR")),
            "b" => {
                if code == "B" {
                    ("mB", Some("mB"))
                } else {
                    ("mb", Some("mb"))
                }
            }
            "bigb" => ("mB", Some("mB")),
            "bbb" => ("mB<sub>BB</sub>", Some("mB")),
            "bc" => ("mB<sub>c</sub>", Some("mBc")),
            "blg" => ("mb<sub>Lg</sub>", Some("mbLg")),
            "n" => ("m<sub>N</sub>", Some("mN")),
            "c" => ("M<sub>c</sub>", Some("Mc")),
            "d" => ("M<sub>d</sub>", Some("Md")),
            "e" => ("M<sub>e</sub>", Some("Me")),
            "k" => ("M<sub>(K)</sub>", Some("MK")),
            "t" => ("M<sub>t</sub>", Some("Mt")),
            "m" => ("M<sub>m</sub>", Some("Mm")),
            "ms" => ("M<sub>ms</sub>", Some("Mms")),
            "fa" => ("M<sub>fa</sub>", Some("Mfa")),
            "la" => ("M<sub>la</sub>", Some("Mla")),
            "i" => {
                if code == "I" {
                    ("M<sub>I</sub>", Some("MI"))
                } else {
                    ("M<sub>i</sub>", Some("Mi"))
                }
            }
            "x" => ("M<sub>x</sub>", Some("Magnitude scales")),
            _ => ("", None),
        };

        if label_str.is_empty() {
            return String::new();
        }

        let label_owned = if let (true, Some(anchor)) = (link, anchor_str) {
            format!("[[Seismic magnitude scales#{}|{}]]", anchor, label_str)
        } else {
            label_str.to_string()
        };

        (label_owned, anchor_str)
    };

    let mut result = label;

    if !src.is_empty() {
        result.push_str(&format!("<sup>({})</sup>", render_templates(src)));
    }

    if !val.is_empty() {
        result.push_str(&format!("\u{2009}{}", render_templates(val)));
    }

    result
}

pub(crate) fn get_empty_dispatch_table() -> HashMap<&'static str, fn() -> String> {
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
    empty_lookup
}

fn render_cquote_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let text = template_param(&named, &["text", "quote", "1"])
        .map(str::to_string)
        .or_else(|| positional.first().cloned())
        .map(|value| render_templates(&value).replace('\n', " "))
        .unwrap_or_default();

    let author = template_param(&named, &["author", "2"])
        .map(str::to_string)
        .or_else(|| positional.get(1).cloned())
        .map(|value| render_templates(&value).replace('\n', " "));

    let source = template_param(&named, &["source", "cite", "3"])
        .map(str::to_string)
        .or_else(|| positional.get(2).cloned())
        .map(|value| render_templates(&value).replace('\n', " "));

    if text.trim().is_empty() {
        return String::new();
    }

    let mut attribution = String::new();
    if let Some(auth_val) = author {
        let auth_trimmed = auth_val.trim();
        if !auth_trimmed.is_empty() {
            attribution.push_str(auth_trimmed);
        }
    }
    if let Some(src_val) = source {
        let src_trimmed = src_val.trim();
        if !src_trimmed.is_empty() {
            if !attribution.is_empty() {
                attribution.push_str(", ");
            }
            attribution.push_str(src_trimmed);
        }
    }

    let mut rendered = format!(
        "\n__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_START__\n__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_TEXT__{}\n",
        text.trim()
    );
    if !attribution.is_empty() {
        rendered.push_str(&format!(
            "__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_SOURCE__{}\n",
            attribution
        ));
    }
    rendered.push_str("__WIKIPEDIA_TO_EPUB_BLOCKQUOTE_END__\n");
    rendered
}

fn render_term_template(params: &str) -> String {
    let named = template_named_params(params);
    let term = if let Some(val) = template_param(&named, &["1", "term"]) {
        val.to_string()
    } else {
        let positional = template_positional_params(params);
        if let Some(val) = positional.first() {
            val.to_string()
        } else {
            String::new()
        }
    };
    if term.is_empty() {
        String::new()
    } else {
        format!("'''{}'''", render_templates(&term))
    }
}

fn render_defn_template(params: &str) -> String {
    let named = template_named_params(params);
    let content = if let Some(val) = template_param(&named, &["1", "defn"]) {
        val.to_string()
    } else {
        let positional = template_positional_params(params);
        if let Some(val) = positional.first() {
            val.to_string()
        } else {
            String::new()
        }
    };
    if content.is_empty() {
        String::new()
    } else {
        render_templates(&content)
    }
}

fn render_us_dollar_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let amount = template_param(&named, &["1"])
        .map(str::to_string)
        .or_else(|| positional.first().cloned())
        .map(|v| v.trim().to_string())
        .unwrap_or_default();

    if amount.is_empty() {
        "US$".to_string()
    } else {
        format!("US${}", render_templates(&amount))
    }
}
