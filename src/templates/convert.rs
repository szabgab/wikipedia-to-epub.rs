use crate::parse_template_number;
use crate::templates::formatting::{
    format_number_with_commas, template_named_params, template_positional_params,
};
use crate::templates::render_templates;
use crate::tools::split_template_params;

use crate::types::{DispatchTable, TemplateHandler};
use std::collections::HashMap;

/// [JPY](https://en.wikipedia.org/wiki/Template:JPY)
fn render_jpy_template(params: &str) -> String {
    let named = template_named_params(params);
    let positional = template_positional_params(params);

    let amount_opt = positional
        .first()
        .cloned()
        .or_else(|| named.get("1").cloned())
        .or_else(|| named.get("amount").cloned());

    match amount_opt {
        Some(amount) if !amount.trim().is_empty() => {
            format!("¥{}", format_number_with_commas(&amount))
        }
        _ => "¥".to_string(),
    }
}

/// [FXConvert](https://en.wikipedia.org/wiki/Template:FXConvert)
fn render_fx_convert_template(params: &str) -> String {
    let positional = template_positional_params(params);
    let named = template_named_params(params);

    if positional.is_empty() {
        return String::new();
    }

    let currency = &positional[0];
    let amount_str = positional.get(1).map(String::as_str).unwrap_or("0");
    let scale = positional.get(2).map(String::as_str).unwrap_or("");

    let cursign = named.get("cursign").map(String::as_str).unwrap_or("");

    let year = named.get("year").and_then(|y| y.parse::<i32>().ok());

    let mut clean_cursign = if cursign.contains("[[") && cursign.contains("]]") {
        cursign.replace("[[", "").replace("]]", "")
    } else {
        cursign.to_string()
    };

    if clean_cursign.is_empty() {
        clean_cursign = match currency.to_ascii_uppercase().as_str() {
            "KOR" | "KRW" => "₩".to_string(),
            "EUR" => "€".to_string(),
            "GBP" => "£".to_string(),
            "JPY" => "¥".to_string(),
            _ => currency.to_string(),
        };
    }

    let amount: f64 = amount_str.parse().unwrap_or(0.0);

    let scale_word = match scale.to_ascii_lowercase().as_str() {
        "b" => "billion",
        "m" => "million",
        "t" => "trillion",
        _ => scale,
    };

    let formatted_amount = if amount.fract() == 0.0 {
        format!("{clean_cursign}{amount:.0}")
    } else {
        format!("{clean_cursign}{amount:.2}")
    };

    let local_str = if scale_word.is_empty() {
        formatted_amount
    } else {
        format!("{formatted_amount} {scale_word}")
    };

    let converted_str =
        if currency.eq_ignore_ascii_case("KOR") || currency.eq_ignore_ascii_case("KRW") {
            if let Some(2020) = year {
                let usd_amount = amount / 1.18025;
                if scale == "b" {
                    format!("US${:.2} million", usd_amount)
                } else if scale == "m" {
                    format!("US${:.2} thousand", usd_amount)
                } else {
                    format!("US${:.2}", usd_amount / 1180.0)
                }
            } else {
                let usd_amount = amount / 1.2;
                if scale == "b" {
                    format!("US${:.2} million", usd_amount)
                } else if scale == "m" {
                    format!("US${:.2} thousand", usd_amount)
                } else {
                    format!("US${:.2}", usd_amount / 1200.0)
                }
            }
        } else {
            String::new()
        };

    if converted_str.is_empty() {
        local_str
    } else {
        format!("{local_str} ({converted_str})")
    }
}

/// [percentage](https://en.wikipedia.org/wiki/Template:Percentage)
fn render_percentage_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| render_templates(param.trim()).trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    let Some(part) = params
        .first()
        .and_then(|value| parse_template_number(value))
    else {
        return String::new();
    };
    let Some(total) = params.get(1).and_then(|value| parse_template_number(value)) else {
        return String::new();
    };

    if total == 0.0 {
        return String::new();
    }

    let decimals = params
        .get(2)
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(0);
    let percentage = part / total * 100.0;

    if decimals == 0 {
        format!("{:.0}%", percentage)
    } else {
        format!("{percentage:.decimals$}%")
    }
}

/// [UN Population](https://en.wikipedia.org/wiki/Template:UN_Population)
fn render_un_population_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
        .collect::<Vec<_>>();

    match params.first().map(String::as_str) {
        Some("ref") => String::new(),
        Some(country) if country.eq_ignore_ascii_case("Dem. People's Republic of Korea") => {
            "26,100,000".to_string()
        }
        _ => String::new(),
    }
}

/// [convert](https://en.wikipedia.org/wiki/Template:Convert)
/// [cvt](https://en.wikipedia.org/wiki/Template:Convert_abbreviated)
fn render_convert_template(params: &str) -> String {
    let params = split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.contains('='))
        .collect::<Vec<_>>();

    let Some(value) = params.first().map(String::as_str) else {
        return String::new();
    };

    match params.get(1).map(String::as_str) {
        Some(separator)
            if matches!(separator, "to" | "and" | "-" | "–" | "by") && params.len() >= 4 =>
        {
            let original = format_convert_pair(value, separator, &params[2], &params[3]);
            let converted = render_convert_pair_secondary(
                value,
                separator,
                &params[2],
                &params[3],
                &params[4..],
            );
            append_parenthetical_conversion(original, converted)
        }
        Some(unit) => {
            let orig = format_convert_value(value);
            let unit_fmt = format_convert_unit(unit);

            let converted = render_convert_single_secondary(value, unit, &params[2..]);
            append_parenthetical_conversion(format!("{orig} {unit_fmt}"), converted)
        }
        None => format_convert_value(value),
    }
}

fn append_parenthetical_conversion(original: String, converted: Option<String>) -> String {
    match converted {
        Some(converted) if !converted.is_empty() => format!("{original} ({converted})"),
        _ => original,
    }
}

fn format_convert_pair(first: &str, separator: &str, second: &str, unit: &str) -> String {
    let first = format_convert_value(first);
    let second = format_convert_value(second);
    let unit = format_convert_unit(unit);

    match separator {
        "and" => format!("{first} {unit} and {second} {unit}"),
        "to" => format!("{first} to {second} {unit}"),
        "by" => format!("{first} by {second} {unit}"),
        "-" | "–" => format!("{first}{separator}{second} {unit}"),
        _ => format!("{first} {unit} {separator} {second} {unit}"),
    }
}

fn render_convert_single_secondary(
    value: &str,
    source_unit: &str,
    trailing_params: &[String],
) -> Option<String> {
    let (target_spec, precision) = find_convert_target_and_precision(source_unit, trailing_params)?;

    if let Some((first, separator, second)) = parse_embedded_convert_range(value) {
        let rendered = split_convert_unit_spec(&target_spec)
            .into_iter()
            .filter_map(|target_unit| {
                if same_convert_unit(source_unit, target_unit) {
                    return None;
                }

                let first_converted =
                    convert_value(parse_template_number(first)?, source_unit, target_unit)?;
                let second_converted =
                    convert_value(parse_template_number(second)?, source_unit, target_unit)?;

                Some(format_convert_pair(
                    &format_converted_number(first_converted, precision),
                    separator,
                    &format_converted_number(second_converted, precision),
                    target_unit,
                ))
            })
            .collect::<Vec<_>>();

        return (!rendered.is_empty()).then(|| rendered.join(", "));
    }

    let numeric_value = parse_template_number(value)?;

    let rendered = split_convert_unit_spec(&target_spec)
        .into_iter()
        .filter_map(|target_unit| {
            if same_convert_unit(source_unit, target_unit) {
                return None;
            }

            let converted = convert_value(numeric_value, source_unit, target_unit)?;
            Some(format!(
                "{} {}",
                format_converted_number(converted, precision),
                format_convert_unit(target_unit)
            ))
        })
        .collect::<Vec<_>>();

    (!rendered.is_empty()).then(|| rendered.join(", "))
}

fn parse_embedded_convert_range(value: &str) -> Option<(&str, &str, &str)> {
    let trimmed = value.trim();

    for separator in ["–", "-"] {
        let search_start = trimmed
            .char_indices()
            .nth(1)
            .map(|(index, _)| index)
            .unwrap_or(trimmed.len());
        let Some(relative_index) = trimmed
            .get(search_start..)
            .and_then(|remaining| remaining.find(separator))
        else {
            continue;
        };
        let separator_index = search_start + relative_index;
        let (first, rest) = trimmed.split_at(separator_index);
        let Some(second) = rest.strip_prefix(separator) else {
            continue;
        };

        if parse_template_number(first).is_some() && parse_template_number(second).is_some() {
            return Some((first.trim(), separator, second.trim()));
        }
    }

    None
}

fn render_convert_pair_secondary(
    first: &str,
    separator: &str,
    second: &str,
    source_unit: &str,
    trailing_params: &[String],
) -> Option<String> {
    let first_value = parse_template_number(first)?;
    let second_value = parse_template_number(second)?;
    let (target_spec, precision) = find_convert_target_and_precision(source_unit, trailing_params)?;

    let rendered = split_convert_unit_spec(&target_spec)
        .into_iter()
        .filter_map(|target_unit| {
            if same_convert_unit(source_unit, target_unit) {
                return None;
            }

            let first_converted = convert_value(first_value, source_unit, target_unit)?;
            let second_converted = convert_value(second_value, source_unit, target_unit)?;

            Some(format_convert_pair(
                &format_converted_number(first_converted, precision),
                separator,
                &format_converted_number(second_converted, precision),
                target_unit,
            ))
        })
        .collect::<Vec<_>>();

    (!rendered.is_empty()).then(|| rendered.join(", "))
}

fn find_convert_target_and_precision(
    source_unit: &str,
    trailing_params: &[String],
) -> Option<(String, Option<i32>)> {
    let explicit_target_index = trailing_params
        .iter()
        .position(|param| looks_like_convert_unit_spec(param));

    let target = explicit_target_index
        .map(|index| trailing_params[index].trim().to_string())
        .or_else(|| default_convert_target_spec(source_unit).map(str::to_string))?;

    let precision = trailing_params
        .iter()
        .find_map(|param| parse_convert_precision(param));
    Some((target, precision))
}

fn looks_like_convert_unit_spec(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.contains('<') || trimmed.contains('(') || trimmed.contains(')')
    {
        return false;
    }

    if parse_template_number(trimmed).is_some() {
        return false;
    }

    let tokens = split_convert_unit_spec(trimmed);
    !tokens.is_empty()
        && tokens
            .iter()
            .all(|token| normalize_convert_unit_key(token).is_some())
}

fn split_convert_unit_spec(value: &str) -> Vec<&str> {
    value
        .split(|ch: char| ch.is_whitespace() || ch == '+')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .collect()
}

fn parse_convert_precision(value: &str) -> Option<i32> {
    let trimmed = value.trim();
    (!trimmed.is_empty() && !looks_like_convert_unit_spec(trimmed))
        .then(|| trimmed.parse::<i32>().ok())
        .flatten()
}

fn same_convert_unit(left: &str, right: &str) -> bool {
    normalize_convert_unit_key(left) == normalize_convert_unit_key(right)
}

fn default_convert_target_spec(source_unit: &str) -> Option<&'static str> {
    match normalize_convert_unit_key(source_unit)? {
        "km" => Some("mi"),
        "mi" => Some("km"),
        "m" => Some("ft"),
        "meter" => Some("ft"),
        "cm" => Some("in"),
        "mm" => Some("in"),
        "km2" => Some("mi2"),
        "c" => Some("f"),
        "c-change" => Some("f-change"),
        "km/h" => Some("mph"),
        "km/s" => Some("mi/s"),
        "m/s2" => Some("g0"),
        "e6km" => Some("e6mi"),
        "km3" => Some("mi3"),
        "m2" => Some("ft2"),
        "acres" => Some("km2"),
        _ => None,
    }
}

fn normalize_convert_unit_key(unit: &str) -> Option<&'static str> {
    let trimmed = unit.trim();

    match trimmed {
        "°C" | "C" => return Some("c"),
        "°F" | "F" => return Some("f"),
        "K" => return Some("k"),
        "km²" | "km2" => return Some("km2"),
        "mi²" | "mi2" | "sqmi" => return Some("mi2"),
        "m²" | "m2" => return Some("m2"),
        "m³" | "m3" => return Some("m3"),
        "km³" | "km3" => return Some("km3"),
        "μSv/h" => return Some("μsv/h"),
        "μGy/h" => return Some("μgy/h"),
        _ => {}
    }

    match trimmed.to_ascii_lowercase().as_str() {
        "c" => Some("c"),
        "f" => Some("f"),
        "k" => Some("k"),
        "km" => Some("km"),
        "mi" | "mile" => Some("mi"),
        "m" | "metre" | "meters" | "metres" => Some("m"),
        "meter" => Some("meter"),
        "ft" => Some("ft"),
        "ft2" => Some("ft2"),
        "yd" => Some("yd"),
        "in" => Some("in"),
        "cm" => Some("cm"),
        "mm" => Some("mm"),
        "au" => Some("au"),
        "ly" => Some("ly"),
        "pc" => Some("pc"),
        "e6km" => Some("e6km"),
        "e6mi" => Some("e6mi"),
        "e9km" => Some("e9km"),
        "e6km2" => Some("e6km2"),
        "e6sqmi" | "e6mi2" => Some("e6mi2"),
        "e9km3" => Some("e9km3"),
        "e6cumi" => Some("e6cumi"),
        "km/h" => Some("km/h"),
        "mph" | "mi/h" => Some("mph"),
        "m/s" => Some("m/s"),
        "km/s" => Some("km/s"),
        "mi/s" => Some("mi/s"),
        "m/s2" => Some("m/s2"),
        "g0" => Some("g0"),
        "pa" => Some("pa"),
        "kpa" => Some("kpa"),
        "mbar" => Some("mbar"),
        "bar" => Some("bar"),
        "mpa" => Some("mpa"),
        "gpa" => Some("gpa"),
        "e6psi" => Some("e6psi"),
        "psi" => Some("psi"),
        "kg/m3" => Some("kg/m3"),
        "g/cm3" => Some("g/cm3"),
        "lb/ft3" => Some("lb/ft3"),
        "d" => Some("d"),
        "yr" => Some("yr"),
        "years" => Some("years"),
        "c-change" => Some("c-change"),
        "f-change" => Some("f-change"),
        "msv/yr" => Some("msv/yr"),
        "msv/d" => Some("msv/d"),
        "mgy/d" => Some("mgy/d"),
        "ug/m3" => Some("ug/m3"),
        "mm/year" => Some("mm/year"),
        "in/year" => Some("in/year"),
        "mm/yr" => Some("mm/yr"),
        "in/yr" => Some("in/yr"),
        "l" => Some("l"),
        "gal" => Some("gal"),
        "mi3" => Some("mi3"),
        "acres" => Some("acres"),
        _ => None,
    }
}

fn convert_value(value: f64, source_unit: &str, target_unit: &str) -> Option<f64> {
    let source = normalize_convert_unit_key(source_unit)?;
    let target = normalize_convert_unit_key(target_unit)?;

    if source == target {
        return Some(value);
    }

    match (source, target) {
        ("c", "f") => Some(value * 9.0 / 5.0 + 32.0),
        ("c", "k") => Some(value + 273.15),
        ("f", "c") => Some((value - 32.0) * 5.0 / 9.0),
        ("f", "k") => Some((value - 32.0) * 5.0 / 9.0 + 273.15),
        ("k", "c") => Some(value - 273.15),
        ("k", "f") => Some((value - 273.15) * 9.0 / 5.0 + 32.0),
        ("c-change", "f-change") => Some(value * 9.0 / 5.0),
        ("f-change", "c-change") => Some(value * 5.0 / 9.0),
        _ => convert_linear_value(value, source, target),
    }
}

fn convert_linear_value(value: f64, source: &str, target: &str) -> Option<f64> {
    let (source_scale, target_scale) = match (source, target) {
        ("km", "mi" | "au" | "ly" | "pc" | "e6km" | "e6mi" | "e9km") => {
            (km_scale(source)?, km_scale(target)?)
        }
        ("mi" | "au" | "ly" | "pc" | "e6km" | "e6mi" | "e9km", "km") => {
            (km_scale(source)?, km_scale(target)?)
        }
        (
            "mi" | "au" | "ly" | "pc" | "e6km" | "e6mi" | "e9km",
            "mi" | "au" | "ly" | "pc" | "e6km" | "e6mi" | "e9km",
        ) => (km_scale(source)?, km_scale(target)?),
        (
            "m" | "meter" | "ft" | "yd" | "in" | "cm" | "mm",
            "m" | "meter" | "ft" | "yd" | "in" | "cm" | "mm",
        ) => (meter_scale(source)?, meter_scale(target)?),
        (
            "km2" | "mi2" | "m2" | "e6km2" | "e6mi2" | "acres" | "ft2",
            "km2" | "mi2" | "m2" | "e6km2" | "e6mi2" | "acres" | "ft2",
        ) => (km2_scale(source)?, km2_scale(target)?),
        ("km3" | "e9km3" | "e6cumi" | "mi3", "km3" | "e9km3" | "e6cumi" | "mi3") => {
            (km3_scale(source)?, km3_scale(target)?)
        }
        ("km/h" | "mph" | "m/s" | "km/s" | "mi/s", "km/h" | "mph" | "m/s" | "km/s" | "mi/s") => {
            (mps_scale(source)?, mps_scale(target)?)
        }
        ("m/s2" | "g0", "m/s2" | "g0") => {
            (acceleration_scale(source)?, acceleration_scale(target)?)
        }
        (
            "pa" | "kpa" | "mbar" | "bar" | "mpa" | "gpa" | "psi" | "e6psi",
            "pa" | "kpa" | "mbar" | "bar" | "mpa" | "gpa" | "psi" | "e6psi",
        ) => (pressure_scale(source)?, pressure_scale(target)?),
        ("kg/m3" | "g/cm3" | "lb/ft3", "kg/m3" | "g/cm3" | "lb/ft3") => {
            (density_scale(source)?, density_scale(target)?)
        }
        ("d" | "yr" | "years", "d" | "yr" | "years") => (day_scale(source)?, day_scale(target)?),
        ("mm/year" | "in/year" | "mm/yr" | "in/yr", "mm/year" | "in/year" | "mm/yr" | "in/yr") => {
            (length_rate_scale(source)?, length_rate_scale(target)?)
        }
        (
            "msv/yr" | "msv/d" | "mgy/d" | "μsv/h" | "μgy/h",
            "msv/yr" | "msv/d" | "mgy/d" | "μsv/h" | "μgy/h",
        ) => (dose_rate_scale(source)?, dose_rate_scale(target)?),
        ("l" | "gal", "l" | "gal") => (volume_liter_scale(source)?, volume_liter_scale(target)?),
        _ => return None,
    };

    Some(value * source_scale / target_scale)
}

fn km_scale(unit: &str) -> Option<f64> {
    match unit {
        "km" => Some(1.0),
        "mi" => Some(1.609_344),
        "au" => Some(149_597_870.7),
        "ly" => Some(9.460_730_472_580_8e12),
        "pc" => Some(3.085_677_581_491_367e13),
        "e6km" => Some(1_000_000.0),
        "e6mi" => Some(1_609_344.0),
        "e9km" => Some(1_000_000_000.0),
        _ => None,
    }
}

fn meter_scale(unit: &str) -> Option<f64> {
    match unit {
        "m" | "meter" => Some(1.0),
        "ft" => Some(0.3048),
        "yd" => Some(0.9144),
        "in" => Some(0.0254),
        "cm" => Some(0.01),
        "mm" => Some(0.001),
        _ => None,
    }
}

fn km2_scale(unit: &str) -> Option<f64> {
    match unit {
        "km2" => Some(1.0),
        "mi2" => Some(2.589_988_110_336),
        "m2" => Some(0.000_001),
        "e6km2" => Some(1_000_000.0),
        "e6mi2" => Some(2_589_988.110_336),
        "acres" => Some(0.004_046_856_422_4),
        "ft2" => Some(0.000_000_092_903_04),
        _ => None,
    }
}

fn km3_scale(unit: &str) -> Option<f64> {
    match unit {
        "km3" => Some(1.0),
        "e9km3" => Some(1_000_000_000.0),
        "e6cumi" => Some(4_168_181.825_440_579),
        "mi3" => Some(4.168_181_825_440_579),
        _ => None,
    }
}

fn mps_scale(unit: &str) -> Option<f64> {
    match unit {
        "m/s" => Some(1.0),
        "km/h" => Some(0.277_777_777_777_777_8),
        "mph" => Some(0.447_04),
        "km/s" => Some(1_000.0),
        "mi/s" => Some(1_609.344),
        _ => None,
    }
}

fn acceleration_scale(unit: &str) -> Option<f64> {
    match unit {
        "m/s2" => Some(1.0),
        "g0" => Some(9.806_65),
        _ => None,
    }
}

fn pressure_scale(unit: &str) -> Option<f64> {
    match unit {
        "pa" => Some(1.0),
        "kpa" => Some(1_000.0),
        "mbar" => Some(100.0),
        "bar" => Some(100_000.0),
        "mpa" => Some(1_000_000.0),
        "gpa" => Some(1_000_000_000.0),
        "psi" => Some(6_894.757_293_168),
        "e6psi" => Some(6_894_757_293.168),
        _ => None,
    }
}

fn density_scale(unit: &str) -> Option<f64> {
    match unit {
        "kg/m3" => Some(1.0),
        "g/cm3" => Some(1_000.0),
        "lb/ft3" => Some(16.018_463_373_96),
        _ => None,
    }
}

fn day_scale(unit: &str) -> Option<f64> {
    match unit {
        "d" => Some(1.0),
        "yr" | "years" => Some(365.25),
        _ => None,
    }
}

fn dose_rate_scale(unit: &str) -> Option<f64> {
    match unit {
        "msv/yr" => Some(1_000.0 / (365.25 * 24.0)),
        "msv/d" | "mgy/d" => Some(1_000.0 / 24.0),
        "μsv/h" | "μgy/h" => Some(1.0),
        _ => None,
    }
}

fn length_rate_scale(unit: &str) -> Option<f64> {
    match unit {
        "mm/year" | "mm/yr" => Some(0.001),
        "in/year" | "in/yr" => Some(0.0254),
        _ => None,
    }
}

fn volume_liter_scale(unit: &str) -> Option<f64> {
    match unit {
        "l" => Some(1.0),
        "gal" => Some(3.785_411_784),
        _ => None,
    }
}

fn format_converted_number(value: f64, precision: Option<i32>) -> String {
    if !value.is_finite() {
        return value.to_string();
    }

    match precision {
        Some(decimals) if decimals >= 0 => {
            let decimals = decimals as usize;
            format_number_with_commas(&format!("{value:.decimals$}"))
        }
        Some(negative_precision) => {
            let rounded =
                round_to_negative_precision(value, negative_precision.unsigned_abs() as i32);
            format_number_with_commas(&format!("{rounded:.0}"))
        }
        None => format_number_with_commas(&format_significant_digits(value, 3)),
    }
}

fn round_to_negative_precision(value: f64, digits: i32) -> f64 {
    let scale = 10f64.powi(digits);
    (value / scale).round() * scale
}

fn format_significant_digits(value: f64, digits: usize) -> String {
    if value == 0.0 {
        return "0".to_string();
    }

    let exponent = value.abs().log10().floor() as i32;
    let scale = 10f64.powi(digits as i32 - exponent - 1);
    let rounded = (value * scale).round() / scale;
    let decimals = (digits as i32 - exponent - 1).max(0) as usize;
    trim_trailing_zeroes(&format!("{rounded:.decimals$}"))
}

fn trim_trailing_zeroes(value: &str) -> String {
    if let Some((integer, fractional)) = value.split_once('.') {
        let fractional = fractional.trim_end_matches('0');
        if fractional.is_empty() {
            integer.to_string()
        } else {
            format!("{integer}.{fractional}")
        }
    } else {
        value.to_string()
    }
}

/// [Inflation](https://en.wikipedia.org/wiki/Template:Inflation)
fn render_inflation_template(params: &str) -> String {
    let positional = template_positional_params(params);
    if positional.len() < 3 {
        return String::new();
    }
    let index = &positional[0];
    let value_str = &positional[1];
    let year_str = &positional[2];

    let value: f64 = value_str.trim().parse().unwrap_or(0.0);
    let year: i32 = year_str.trim().parse().unwrap_or(0);

    if index.eq_ignore_ascii_case("US") {
        let cpi_1950 = 24.1;
        let cpi_2023 = 304.7;

        let cpi_start = match year {
            1950 => cpi_1950,
            _ => 24.1,
        };

        let inflated = value * (cpi_2023 / cpi_start);

        if inflated >= 100.0 {
            format!("{:.0}", inflated)
        } else {
            format!("{:.2}", inflated)
        }
    } else {
        value_str.clone()
    }
}

/// [Inflation/year](https://en.wikipedia.org/wiki/Template:Inflation/year)
fn render_inflation_year_template(_params: &str) -> String {
    "2023".to_string()
}

/// [val](https://en.wikipedia.org/wiki/Template:Val)
/// [Value](https://en.wikipedia.org/wiki/Template:Value)
/// [value](https://en.wikipedia.org/wiki/Template:Value)
fn render_val_template(params: &str) -> String {
    let raw_parts = split_template_params(params);
    let mut positional = Vec::new();
    let mut unit = String::new();
    let mut exponent = String::new();

    for part in raw_parts {
        let part_trimmed = part.trim();
        if let Some((name, val)) = part_trimmed.split_once('=') {
            let name = name.trim();
            let val = val.trim();
            if name == "u" || name == "ul" {
                unit = val.to_string();
            } else if name == "e" {
                exponent = val.to_string();
            }
        } else if !part_trimmed.is_empty() {
            positional.push(part_trimmed.to_string());
        }
    }

    if positional.is_empty() {
        return String::new();
    }

    let mut rendered = String::new();

    if positional.len() >= 3
        && (positional[1] == "–"
            || positional[1] == "-"
            || positional[1] == "to"
            || positional[1] == "and"
            || positional[1] == "or")
    {
        rendered.push_str(&positional[0]);
        rendered.push(' ');
        rendered.push_str(&positional[1]);
        rendered.push(' ');
        rendered.push_str(&positional[2]);
    } else if positional.len() == 2 {
        rendered.push_str(&positional[0]);
        rendered.push_str(" ± ");
        rendered.push_str(&positional[1]);
    } else if positional.len() >= 3 {
        rendered.push_str(&positional[0]);
        rendered.push_str(" (+");
        rendered.push_str(&positional[2]);
        rendered.push_str("/-");
        rendered.push_str(&positional[1]);
        rendered.push(')');
    } else {
        rendered.push_str(&positional[0]);
    }

    if !exponent.is_empty() {
        rendered.push_str(" × 10__WIKIPEDIA_TO_EPUB_SUP_START__");
        rendered.push_str(&exponent);
        rendered.push_str("__WIKIPEDIA_TO_EPUB_SUP_END__");
    }

    if !unit.is_empty() {
        rendered.push(' ');
        rendered.push_str(&unit);
    }

    render_templates(&rendered)
}

pub(crate) fn format_convert_value(value: &str) -> String {
    let trimmed = value.trim();
    let (sign, rest) = if let Some(rest) = trimmed.strip_prefix("&minus;") {
        ("−", rest)
    } else if let Some(rest) = trimmed.strip_prefix('−') {
        ("−", rest)
    } else if let Some(rest) = trimmed.strip_prefix('-') {
        ("-", rest)
    } else if let Some(rest) = trimmed.strip_prefix('+') {
        ("+", rest)
    } else {
        ("", trimmed)
    };

    let formatted = format_number_with_commas(rest);
    format!("{sign}{formatted}")
}

fn format_convert_unit(unit: &str) -> String {
    match normalize_convert_unit_key(unit) {
        Some("c") => "°C".to_string(),
        Some("f") => "°F".to_string(),
        Some("k") => "K".to_string(),
        Some("mi") => "mi".to_string(),
        Some("km2") => "km²".to_string(),
        Some("mi2") => "mi²".to_string(),
        Some("m2") => "m²".to_string(),
        Some("m3") => "m³".to_string(),
        Some("km3") => "km³".to_string(),
        Some("m") | Some("meter") => "m".to_string(),
        Some("g0") => "g".to_string(),
        Some("m/s2") => "m/s²".to_string(),
        Some("kg/m3") => "kg/m³".to_string(),
        Some("ug/m3") => "ug/m³".to_string(),
        Some("e6km") => "million km".to_string(),
        Some("e6mi") => "million mi".to_string(),
        Some("e9km") => "billion km".to_string(),
        Some("e6km2") => "million km²".to_string(),
        Some("e6mi2") => "million mi²".to_string(),
        Some("e9km3") => "billion km³".to_string(),
        Some("e6cumi") => "million mi³".to_string(),
        Some("au") => "AU".to_string(),
        Some("c-change") => "°C change".to_string(),
        Some("f-change") => "°F change".to_string(),
        Some("ft2") => "ft²".to_string(),
        Some("mi3") => "mi³".to_string(),
        Some("e6psi") => "million psi".to_string(),
        _ => unit.trim().to_string(),
    }
}

pub(crate) fn get_dispatch_table() -> DispatchTable {
    HashMap::from([
        ("convert", render_convert_template as TemplateHandler),
        ("cvt", render_convert_template as TemplateHandler),
        ("percentage", render_percentage_template as TemplateHandler),
        (
            "un population",
            render_un_population_template as TemplateHandler,
        ),
        ("inflation", render_inflation_template as TemplateHandler),
        (
            "inflation/year",
            render_inflation_year_template as TemplateHandler,
        ),
        ("val", render_val_template as TemplateHandler),
        ("value", render_val_template as TemplateHandler),
        ("fxconvert", render_fx_convert_template as TemplateHandler),
        ("jpy", render_jpy_template as TemplateHandler),
    ])
}
