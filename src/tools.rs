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

#[cfg(test)]
mod tests {
    use super::{
        balanced_wiki_link_end, is_file_link_start, matching_template_end, parse_template_number,
        split_template_name,
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
}
