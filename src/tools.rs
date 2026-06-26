use regex::Regex;
use std::collections::HashMap;

///
/// ```
/// # use wikipedia_to_epub::tools::split_template_params;
/// let params = "param1|param2|param3";
/// let parts = split_template_params(params);
/// assert_eq!(parts, vec!["param1", "param2", "param3"]);
/// ```
pub fn split_template_params(params: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut template_depth = 0usize;
    let mut link_depth = 0usize;
    let mut chars = params.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '[' && chars.peek() == Some(&'[') {
            current.push(ch);
            current.push(chars.next().unwrap());
            link_depth += 1;
        } else if ch == ']' && chars.peek() == Some(&']') {
            current.push(ch);
            current.push(chars.next().unwrap());
            link_depth = link_depth.saturating_sub(1);
        } else if ch == '{' && chars.peek() == Some(&'{') {
            current.push(ch);
            current.push(chars.next().unwrap());
            template_depth += 1;
        } else if ch == '}' && chars.peek() == Some(&'}') {
            current.push(ch);
            current.push(chars.next().unwrap());
            template_depth = template_depth.saturating_sub(1);
        } else if ch == '|' && template_depth == 0 && link_depth == 0 {
            parts.push(current);
            current = String::new();
        } else {
            current.push(ch);
        }
    }

    parts.push(current);
    parts
}

/// Returns the possible keys for a person's first name based on a base name and index.
///
/// ```
/// # use wikipedia_to_epub::tools::person_first_keys;
/// assert_eq!(person_first_keys("first", 0), vec!["first", "given"]);
/// assert_eq!(person_first_keys("first", 2), vec!["first2", "given2"]);
/// assert_eq!(person_first_keys("editor-first", 0), vec!["editor-first", "editor-given", "editor-first1", "editor-given1"]);
/// ```
pub fn person_first_keys(base: &str, index: usize) -> Vec<String> {
    if index == 0 {
        match base {
            "first" => vec!["first".to_string(), "given".to_string()],
            "editor-first" => vec![
                "editor-first".to_string(),
                "editor-given".to_string(),
                "editor-first1".to_string(),
                "editor-given1".to_string(),
            ],
            _ => vec![base.to_string()],
        }
    } else {
        match base {
            "first" => vec![format!("first{index}"), format!("given{index}")],
            "editor-first" => vec![
                format!("editor-first{index}"),
                format!("editor-given{index}"),
            ],
            _ => vec![format!("{base}{index}")],
        }
    }
}

/// Returns the possible keys for a person's last name based on a base name and index.
///
/// ```
/// # use wikipedia_to_epub::tools::person_last_keys;
/// assert_eq!(person_last_keys("last", 0), vec!["last", "surname"]);
/// assert_eq!(person_last_keys("last", 3), vec!["last3", "surname3"]);
/// assert_eq!(person_last_keys("editor-last", 0), vec!["editor-last", "editor-surname", "editor-last1", "editor-surname1"]);
/// ```
pub fn person_last_keys(base: &str, index: usize) -> Vec<String> {
    if index == 0 {
        match base {
            "last" => vec!["last".to_string(), "surname".to_string()],
            "editor-last" => vec![
                "editor-last".to_string(),
                "editor-surname".to_string(),
                "editor-last1".to_string(),
                "editor-surname1".to_string(),
            ],
            _ => vec![base.to_string()],
        }
    } else {
        match base {
            "last" => vec![format!("last{index}"), format!("surname{index}")],
            "editor-last" => vec![
                format!("editor-last{index}"),
                format!("editor-surname{index}"),
            ],
            _ => vec![format!("{base}{index}")],
        }
    }
}

/// Returns the possible keys for a person's link based on a base name and index.
///
/// ```
/// # use wikipedia_to_epub::tools::person_link_keys;
/// assert_eq!(person_link_keys("author-link", 0), vec!["author-link", "authorlink"]);
/// assert_eq!(person_link_keys("author-link", 2), vec!["author-link2", "authorlink2"]);
/// assert_eq!(person_link_keys("editor-link", 0), vec!["editor-link", "editorlink", "editor-link1", "editorlink1"]);
/// ```
pub fn person_link_keys(base: &str, index: usize) -> Vec<String> {
    if index == 0 {
        match base {
            "author-link" => vec!["author-link".to_string(), "authorlink".to_string()],
            "editor-link" => vec![
                "editor-link".to_string(),
                "editorlink".to_string(),
                "editor-link1".to_string(),
                "editorlink1".to_string(),
            ],
            _ => vec![base.to_string()],
        }
    } else {
        match base {
            "author-link" => vec![format!("author-link{index}"), format!("authorlink{index}")],
            "editor-link" => vec![format!("editor-link{index}"), format!("editorlink{index}")],
            _ => vec![format!("{base}{index}")],
        }
    }
}

pub(crate) fn parse_template_number(value: &str) -> Option<f64> {
    let number = value
        .trim()
        .replace([',', ' '], "")
        .replace("&minus;", "-")
        .replace('−', "-");

    number.parse::<f64>().ok()
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

pub(crate) fn split_parameter_by_equals(param: &str) -> Option<(&str, &str)> {
    let mut template_depth = 0usize;
    let mut link_depth = 0usize;
    let mut chars = param.char_indices().peekable();

    while let Some((index, ch)) = chars.next() {
        if ch == '[' && chars.peek().is_some_and(|(_, c)| *c == '[') {
            chars.next();
            link_depth += 1;
        } else if ch == ']' && chars.peek().is_some_and(|(_, c)| *c == ']') {
            chars.next();
            link_depth = link_depth.saturating_sub(1);
        } else if ch == '{' && chars.peek().is_some_and(|(_, c)| *c == '{') {
            chars.next();
            template_depth += 1;
        } else if ch == '}' && chars.peek().is_some_and(|(_, c)| *c == '}') {
            chars.next();
            template_depth = template_depth.saturating_sub(1);
        } else if ch == '=' && template_depth == 0 && link_depth == 0 {
            return Some((&param[..index], &param[index + 1..]));
        }
    }

    None
}

pub(crate) fn template_named_params(params: &str) -> HashMap<String, String> {
    split_template_params(params)
        .into_iter()
        .filter_map(|param| {
            let (key, value) = split_parameter_by_equals(&param)?;
            Some((key.trim().to_lowercase(), value.trim().to_string()))
        })
        .collect()
}

pub(crate) fn template_positional_params(params: &str) -> Vec<String> {
    split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && split_parameter_by_equals(param).is_none())
        .collect()
}

pub(crate) fn template_param<'a>(
    named: &'a HashMap<String, String>,
    keys: &[&str],
) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| named.get(*key))
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub(crate) fn template_param_owned(
    named: &HashMap<String, String>,
    keys: &[String],
) -> Option<String> {
    keys.iter()
        .find_map(|key| named.get(key))
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(String::from)
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

pub(crate) fn is_file_link_start(text: &str) -> bool {
    let trimmed = text.trim_start();
    let lowercase = trimmed.chars().take(6).collect::<String>().to_lowercase();
    lowercase.starts_with("file:") || lowercase.starts_with("image:")
}

pub(crate) fn balanced_wiki_link_end(text: &str, start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = start;

    while index < text.len() {
        let remaining = &text[index..];

        if remaining.starts_with("[[") {
            depth += 1;
            index += 2;
            continue;
        }

        if remaining.starts_with("]]") && depth > 0 {
            depth -= 1;
            index += 2;
            if depth == 0 {
                return Some(index);
            }
            continue;
        }

        let ch = remaining.chars().next().unwrap();
        index += ch.len_utf8();
    }

    None
}

pub fn clean_math_latex(latex: &str) -> String {
    let mut text = latex.to_string();

    // 1. Remove comments
    let comment_re = Regex::new(r"%.*").unwrap();
    text = comment_re.replace_all(&text, "").into_owned();

    // 2. Remove LaTeX formatting commands like \textstyle, \displaystyle, etc.
    for cmd in &[
        r"\\textstyle([^a-zA-Z]|$)",
        r"\\displaystyle([^a-zA-Z]|$)",
        r"\\scriptstyle([^a-zA-Z]|$)",
        r"\\scriptscriptstyle([^a-zA-Z]|$)",
        r"\\limits([^a-zA-Z]|$)",
        r"\\nolimits([^a-zA-Z]|$)",
    ] {
        let cmd_re = Regex::new(cmd).unwrap();
        text = cmd_re.replace_all(&text, "$1").into_owned();
    }

    // 3. Helper regexes for superscripts and subscripts
    let superscript_re = Regex::new(r"\^\{([^{}]+)\}").unwrap();
    while superscript_re.is_match(&text) {
        text = superscript_re.replace_all(&text, "^($1)").into_owned();
    }

    let subscript_re = Regex::new(r"_\{([^{}]+)\}").unwrap();
    while subscript_re.is_match(&text) {
        text = subscript_re.replace_all(&text, "_($1)").into_owned();
    }

    // 3b. Handle \frac{num}{den} and \binom{num}{den} with matching braces
    let start_re = Regex::new(r"\\(?:d|t)?(?:frac|binom)\s*\{").unwrap();
    while let Some(m) = start_re.find(&text) {
        let frac_start = m.start();
        let first_open = text[frac_start..].find('{').map(|idx| frac_start + idx);
        let first_close =
            first_open.and_then(|fo| find_matching_brace(&text, fo).map(|fc| (fo, fc)));
        if let Some((first_open, first_close)) = first_close {
            let num = &text[first_open + 1..first_close];
            let remaining = &text[first_close + 1..];
            if let Some(second_open_rel) = remaining.find('{') {
                let between = &remaining[..second_open_rel];
                if between.trim().is_empty() {
                    let second_open = first_close + 1 + second_open_rel;
                    if let Some(second_close) = find_matching_brace(&text, second_open) {
                        let den = &text[second_open + 1..second_close];
                        let cmd_part = &text[frac_start..first_open];
                        let is_binom = cmd_part.contains("binom");

                        let replacement = if is_binom {
                            format!("({num}/{den})")
                        } else {
                            let cleaned_num = wrap_frac_term(num);
                            let cleaned_den = wrap_frac_term(den);
                            format!("{cleaned_num}/{cleaned_den}")
                        };

                        text.replace_range(frac_start..second_close + 1, &replacement);
                        continue;
                    }
                }
            }
        }
        break;
    }

    // 3c. Handle \frac <single-char> {den}
    let frac_single_re = Regex::new(r"\\(?:d|t)?frac\s*([0-9a-zA-Z])\s*\{").unwrap();
    while let Some(m) = frac_single_re.find(&text) {
        let frac_start = m.start();
        let cap_match = frac_single_re.captures(&text[frac_start..]).unwrap();
        let num = cap_match[1].to_string();
        let first_open = frac_start + cap_match.get(0).unwrap().as_str().find('{').unwrap();
        if let Some(second_close) = find_matching_brace(&text, first_open) {
            let den = &text[first_open + 1..second_close];
            let cleaned_den = wrap_frac_term(den);
            let replacement = format!("{num}/{cleaned_den}");
            text.replace_range(frac_start..second_close + 1, &replacement);
            continue;
        }
        break;
    }

    // 3d. Handle \frac <single-char> <single-char>
    let frac_double_single_re =
        Regex::new(r"\\(?:d|t)?frac\s*([0-9a-zA-Z])\s*([0-9a-zA-Z])").unwrap();
    while frac_double_single_re.is_match(&text) {
        text = frac_double_single_re
            .replace_all(&text, "$1/$2")
            .into_owned();
    }

    // 3e. Handle {num \choose den} with matching braces
    while let Some(choose_idx) = text.find(r"\choose") {
        if let Some((open_idx, close_idx)) = find_enclosing_braces(&text, choose_idx) {
            let num = &text[open_idx + 1..choose_idx].trim();
            let den = &text[choose_idx + r"\choose".len()..close_idx].trim();
            let replacement = format!("({num}/{den})");
            text.replace_range(open_idx..close_idx + 1, &replacement);
            continue;
        }
        break;
    }

    // 3f. Handle \sqrt{arg}
    let sqrt_re = Regex::new(r"\\sqrt\s*\{").unwrap();
    while let Some(m) = sqrt_re.find(&text) {
        let sqrt_start = m.start();
        let open_brace = sqrt_start + text[sqrt_start..].find('{').unwrap();
        if let Some(close_brace) = find_matching_brace(&text, open_brace) {
            let arg = &text[open_brace + 1..close_brace];
            let cleaned_arg = clean_math_latex(arg);
            let replacement = if cleaned_arg.len() > 1 {
                format!("√({cleaned_arg})")
            } else {
                format!("√{cleaned_arg}")
            };
            text.replace_range(sqrt_start..close_brace + 1, &replacement);
            continue;
        }
        break;
    }

    // 3g. Handle \mathbb
    let mathbb_re = Regex::new(r"\\mathbb\{([a-zA-Z])\}").unwrap();
    while mathbb_re.is_match(&text) {
        text = mathbb_re
            .replace_all(&text, |caps: &regex::Captures| match &caps[1] {
                "N" => "ℕ".to_string(),
                "Z" => "ℤ".to_string(),
                "Q" => "ℚ".to_string(),
                "R" => "ℝ".to_string(),
                "C" => "ℂ".to_string(),
                val => val.to_string(),
            })
            .into_owned();
    }

    // 4. Specific symbol replacements
    let symbol_replacements = [
        (r"\\left\\\{", "#LEFT_BRACE#"),
        (r"\\right\\\}", "#RIGHT_BRACE#"),
        (r"\\\{", "#LEFT_BRACE#"),
        (r"\\\}", "#RIGHT_BRACE#"),
        (r"\\left\s*\(", "("),
        (r"\\right\s*\)", ")"),
        (r"\\left\s*\[", "["),
        (r"\\right\s*\]", "]"),
        (r"\\operatorname([^a-zA-Z]|$)", "$1"),
        (r"\\text([^a-zA-Z]|$)", "$1"),
        (r"\\isin([^a-zA-Z]|$)", " ∈ $1"),
        (r"\\in([^a-zA-Z]|$)", " ∈ $1"),
        (r"\\notin([^a-zA-Z]|$)", " ∉ $1"),
        (r"\\le([^a-zA-Z]|$)", " ≤ $1"),
        (r"\\leq([^a-zA-Z]|$)", " ≤ $1"),
        (r"\\ge([^a-zA-Z]|$)", " ≥ $1"),
        (r"\\geq([^a-zA-Z]|$)", " ≥ $1"),
        (r"\\approx([^a-zA-Z]|$)", " ≈ $1"),
        (r"\\ne([^a-zA-Z]|$)", " ≠ $1"),
        (r"\\neq([^a-zA-Z]|$)", " ≠ $1"),
        (r"\\pm([^a-zA-Z]|$)", " ± $1"),
        (r"\\times([^a-zA-Z]|$)", " × $1"),
        (r"\\div([^a-zA-Z]|$)", " ÷ $1"),
        (r"\\cdot([^a-zA-Z]|$)", " · $1"),
        (r"\\lfloor([^a-zA-Z]|$)", " ⌊ $1"),
        (r"\\rfloor([^a-zA-Z]|$)", " ⌋ $1"),
        (r"\\lceil([^a-zA-Z]|$)", " ⌈ $1"),
        (r"\\rceil([^a-zA-Z]|$)", " ⌉ $1"),
        (r"\\infty([^a-zA-Z]|$)", " ∞ $1"),
        (r"\\to([^a-zA-Z]|$)", " → $1"),
        (r"\\rightarrow([^a-zA-Z]|$)", " → $1"),
        (r"\\ldots([^a-zA-Z]|$)", "...$1"),
        (r"\\dots([^a-zA-Z]|$)", "...$1"),
        (r"\\quad([^a-zA-Z]|$)", " $1"),
        (r"\\qquad([^a-zA-Z]|$)", " $1"),
        (r"\\ ", " "),
        (r"\\,", " "),
        (r"\\;", " "),
        (r"\\!", ""),
        (r"\\Pr([^a-zA-Z]|$)", "Pr$1"),
        (r"\\sigma([^a-zA-Z]|$)", "σ$1"),
        (r"\\mu([^a-zA-Z]|$)", "μ$1"),
        (r"\\pi([^a-zA-Z]|$)", "π$1"),
        (r"\\alpha([^a-zA-Z]|$)", "α$1"),
        (r"\\beta([^a-zA-Z]|$)", "β$1"),
        (r"\\theta([^a-zA-Z]|$)", "θ$1"),
        (r"\\lambda([^a-zA-Z]|$)", "λ$1"),
        (r"\\epsilon([^a-zA-Z]|$)", "ε$1"),
        (r"\\eta([^a-zA-Z]|$)", "η$1"),
        (r"\\phi([^a-zA-Z]|$)", "φ$1"),
        (r"\\rho([^a-zA-Z]|$)", "ρ$1"),
        (r"\\tau([^a-zA-Z]|$)", "τ$1"),
        (r"\\omega([^a-zA-Z]|$)", "ω$1"),
        (r"\\Gamma([^a-zA-Z]|$)", "Γ$1"),
        (r"\\Delta([^a-zA-Z]|$)", "Δ$1"),
        (r"\\Omega([^a-zA-Z]|$)", "Ω$1"),
        (r"\\sqrt([^a-zA-Z]|$)", "√$1"),
        (r"\\bar([^a-zA-Z]|$)", "$1"),
        (r"\\hat([^a-zA-Z]|$)", "$1"),
        (r"\\tilde([^a-zA-Z]|$)", "$1"),
        (r"\\overline([^a-zA-Z]|$)", "$1"),
        (r"\\log([^a-zA-Z]|$)", "log$1"),
        (r"\\ln([^a-zA-Z]|$)", "ln$1"),
        (r"\\exp([^a-zA-Z]|$)", "exp$1"),
        (r"\\int([^a-zA-Z]|$)", "∫$1"),
        (r"\\mathcal([^a-zA-Z]|$)", "$1"),
    ];

    for &(pattern, replacement) in &symbol_replacements {
        let re = Regex::new(pattern).unwrap();
        text = re.replace_all(&text, replacement).into_owned();
    }

    // 5. Remove environments & clean layout
    let env_re = Regex::new(r"\\(?:begin|end)\{[a-zA-Z0-9*]+\}").unwrap();
    text = env_re.replace_all(&text, "").into_owned();

    let newline_re = Regex::new(r"\\\\").unwrap();
    text = newline_re.replace_all(&text, "\n").into_owned();

    text = text.replace("&", "");

    // 6. Clean remaining LaTeX commands starting with backslash (e.g. \sin -> sin)
    let fallback_cmd_re = Regex::new(r"\\([a-zA-Z]+)([^a-zA-Z]|$)").unwrap();
    text = fallback_cmd_re.replace_all(&text, "$1$2").into_owned();

    // 7. Remove all remaining grouping braces { and }
    text = text.replace("{", "").replace("}", "");

    // 8. Restore escaped braces
    text = text
        .replace("#LEFT_BRACE#", "{")
        .replace("#RIGHT_BRACE#", "}");

    // 8b. Clean up spaces inside parentheses, brackets, braces
    let space_open_re = Regex::new(r"\(\s+").unwrap();
    text = space_open_re.replace_all(&text, "(").into_owned();
    let space_close_re = Regex::new(r"\s+\)").unwrap();
    text = space_close_re.replace_all(&text, ")").into_owned();

    let space_open_bracket_re = Regex::new(r"\[\s+").unwrap();
    text = space_open_bracket_re.replace_all(&text, "[").into_owned();
    let space_close_bracket_re = Regex::new(r"\s+\]").unwrap();
    text = space_close_bracket_re.replace_all(&text, "]").into_owned();

    let space_open_brace_re = Regex::new(r"\{\s+").unwrap();
    text = space_open_brace_re.replace_all(&text, "{").into_owned();
    let space_close_brace_re = Regex::new(r"\s+\}").unwrap();
    text = space_close_brace_re.replace_all(&text, "}").into_owned();

    // 9. Extra whitespace cleanup
    let multiple_spaces_re = Regex::new(r"[ \t]+").unwrap();
    text = multiple_spaces_re.replace_all(&text, " ").into_owned();

    text.trim().to_string()
}

fn wrap_frac_term(term: &str) -> String {
    let term = term.trim();
    if term.contains('+')
        || term.contains('-')
        || term.contains(' ')
        || term.contains('·')
        || term.contains('×')
    {
        format!("({term})")
    } else {
        term.to_string()
    }
}

fn find_matching_brace(text: &str, start: usize) -> Option<usize> {
    let mut depth = 0;
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let char_pos = chars.iter().position(|&(i, _)| i == start)?;

    for &(i, ch) in &chars[char_pos..] {
        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

fn find_enclosing_braces(text: &str, pos: usize) -> Option<(usize, usize)> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let char_pos = chars.iter().position(|&(i, _)| i == pos)?;

    let mut depth = 0;
    let mut open_idx = None;
    for &(i, ch) in chars[..char_pos].iter().rev() {
        if ch == '}' {
            depth += 1;
        } else if ch == '{' {
            if depth == 0 {
                open_idx = Some(i);
                break;
            } else {
                depth -= 1;
            }
        }
    }

    let mut depth = 0;
    let mut close_idx = None;
    for &(i, ch) in &chars[char_pos..] {
        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            if depth == 0 {
                close_idx = Some(i);
                break;
            } else {
                depth -= 1;
            }
        }
    }

    match (open_idx, close_idx) {
        (Some(o), Some(c)) => Some((o, c)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        balanced_wiki_link_end, clean_math_latex, is_file_link_start, matching_template_end,
        parse_template_number, split_template_name,
    };

    #[test]
    fn test_parse_template_number() {
        assert_eq!(parse_template_number("123"), Some(123.0));
        assert_eq!(parse_template_number("1,234.56"), Some(1234.56));
        assert_eq!(parse_template_number(" 1 000 "), Some(1000.0));
        assert_eq!(parse_template_number("-50"), Some(-50.0));
        assert_eq!(parse_template_number("&minus;10.5"), Some(-10.5));
        assert_eq!(parse_template_number("−20"), Some(-20.0));
        assert_eq!(parse_template_number("abc"), None);
        assert_eq!(parse_template_number(""), None);
    }
    #[test]
    fn test_split_template_name() {
        assert_eq!(
            split_template_name("name|param1|param2"),
            ("name", "param1|param2")
        );
        assert_eq!(split_template_name("name"), ("name", ""));
        assert_eq!(split_template_name("name|"), ("name", ""));
        assert_eq!(
            split_template_name("name|a={{b|c}}|d"),
            ("name", "a={{b|c}}|d")
        );
        assert_eq!(
            split_template_name("name|a=[[b|c]]|d"),
            ("name", "a=[[b|c]]|d")
        );
        assert_eq!(split_template_name(" name | p=1 "), (" name ", " p=1 "));
    }

    #[test]
    fn matching_template_end_finds_simple_template_end() {
        let text = "Before {{Main|Korea}} after";
        let start = text.find("{{").unwrap();

        assert_eq!(matching_template_end(text, start), Some(19));
    }

    #[test]
    fn matching_template_end_finds_outer_nested_template_end() {
        let text = "{{Outer|before {{Inner|value}} after}} tail";
        let start = text.find("{{").unwrap();

        assert_eq!(matching_template_end(text, start), Some(36));
    }

    #[test]
    fn matching_template_end_returns_none_for_unclosed_template() {
        let text = "Before {{Outer|{{Inner}} after";
        let start = text.find("{{").unwrap();

        assert_eq!(matching_template_end(text, start), None);
    }

    #[test]
    fn matching_template_end_uses_the_requested_start_offset() {
        let text = "{{First}} text {{Second|value}}";
        let start = text.find("{{Second").unwrap();

        assert_eq!(matching_template_end(text, start), Some(29));
    }

    #[test]
    fn test_is_file_link_start_returns_true_for_file_prefix() {
        assert!(is_file_link_start("file:example.jpg"));
        assert!(is_file_link_start("File:example.jpg"));
        assert!(is_file_link_start("  file:example.jpg"));
    }

    #[test]
    fn test_is_file_link_start_returns_true_for_image_prefix() {
        assert!(is_file_link_start("image:example.jpg"));
        assert!(is_file_link_start("IMAGE:example.jpg"));
        assert!(is_file_link_start("  image:example.jpg"));
    }

    #[test]
    fn test_is_file_link_start_returns_false_for_other_prefixes() {
        assert!(!is_file_link_start("not_file:example.jpg"));
        assert!(!is_file_link_start("fil:example.jpg"));
        assert!(!is_file_link_start("imag:example.jpg"));
        assert!(!is_file_link_start(""));
    }

    #[test]
    fn balanced_wiki_link_end_finds_simple_link_end() {
        let text = "Before [[Link]] after";
        let start = text.find("[[").unwrap();

        assert_eq!(balanced_wiki_link_end(text, start), Some(15));
    }

    #[test]
    fn balanced_wiki_link_end_finds_outer_nested_link_end() {
        let text = "[[File:Image.png|[[Inner link]] and [[Other]]]] tail";
        let start = text.find("[[").unwrap();

        assert_eq!(balanced_wiki_link_end(text, start), Some(47));
    }

    #[test]
    fn balanced_wiki_link_end_returns_none_for_unclosed_link() {
        let text = "Before [[Link without end";
        let start = text.find("[[").unwrap();

        assert_eq!(balanced_wiki_link_end(text, start), None);
    }

    #[test]
    fn balanced_wiki_link_end_returns_none_for_unopened_link() {
        let text = "Link]] without start";
        let start = 0;

        assert_eq!(balanced_wiki_link_end(text, start), None);
    }

    #[test]
    fn balanced_wiki_link_end_handles_non_ascii() {
        let text = "[[Lïnk é]] suffix";
        let start = 0;

        assert_eq!(balanced_wiki_link_end(text, start), Some(12));
    }

    #[test]
    fn balanced_wiki_link_end_uses_requested_start_offset() {
        let text = "[[First]] text [[Second|value]]";
        let start = text.find("[[Second").unwrap();

        assert_eq!(balanced_wiki_link_end(text, start), Some(31));
    }

    #[test]
    fn test_clean_math_latex() {
        assert_eq!(
            clean_math_latex(r"\textstyle \Pr(X = k) = \binom{n}{k} p^k (1-p)^{n-k}"),
            "Pr(X = k) = (n/k) p^k (1-p)^(n-k)"
        );
        assert_eq!(clean_math_latex(r"\textstyle \tfrac{n}{2}"), "n/2");
        assert_eq!(clean_math_latex(r"\textstyle k > \tfrac{n}{2}"), "k > n/2");
        assert_eq!(clean_math_latex(r"n \isin \mathbb{N}"), "n ∈ ℕ");
        assert_eq!(clean_math_latex(r"p \in [0,1]"), "p ∈ [0,1]");
        assert_eq!(clean_math_latex(r"\frac{1-6pq}{npq}"), "(1-6pq)/npq");
        assert_eq!(
            clean_math_latex(r"f(k,n,p)=f(n-k,n,1-p)."),
            "f(k,n,p)=f(n-k,n,1-p)."
        );
        assert_eq!(clean_math_latex(r"\{0, 1, 2, \ldots\}"), "{0, 1, 2, ...}");
        assert_eq!(
            clean_math_latex(r"\left( 1/(3 \cdot 332,946) \right)^{1/3} = 0.01"),
            "(1/(3 · 332,946))^(1/3) = 0.01"
        );
        assert_eq!(clean_math_latex(r"\mathbb{V}(X)"), "V(X)");
        assert_eq!(
            clean_math_latex(
                r"\frac{1}{\sqrt{2\pi\sigma^2}} \exp\left(-\frac{(x-\mu)^2}{2\sigma^2}\right)"
            ),
            "1/√(2πσ^2) exp(-((x-μ)^2)/2σ^2)"
        );
    }
}
