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

pub(crate) fn template_named_params(params: &str) -> HashMap<String, String> {
    split_template_params(params)
        .into_iter()
        .filter_map(|param| {
            let (key, value) = param.split_once('=')?;
            Some((key.trim().to_lowercase(), value.trim().to_string()))
        })
        .collect()
}

pub(crate) fn template_positional_params(params: &str) -> Vec<String> {
    split_template_params(params)
        .into_iter()
        .map(|param| param.trim().to_string())
        .filter(|param| !param.is_empty() && !param.contains('='))
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
