use regex::Regex;

pub(crate) fn normalize_reference_attr(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::normalize_reference_attr;

    #[test]
    fn normalize_reference_attr_trims_outer_whitespace() {
        assert_eq!(normalize_reference_attr("  alpha  "), "alpha");
    }

    #[test]
    fn normalize_reference_attr_removes_outer_double_quotes() {
        assert_eq!(normalize_reference_attr(r#"  "alpha"  "#), "alpha");
    }

    #[test]
    fn normalize_reference_attr_removes_outer_single_quotes() {
        assert_eq!(normalize_reference_attr("  'alpha'  "), "alpha");
    }

    #[test]
    fn normalize_reference_attr_trims_whitespace_inside_quotes() {
        assert_eq!(normalize_reference_attr(r#""  alpha  ""#), "alpha");
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ReferenceTag {
    pub(crate) group: String,
    pub(crate) name: Option<String>,
    pub(crate) content: Option<String>,
}

pub(crate) fn parse_reference_tags(text: &str) -> Vec<ReferenceTag> {
    let ref_re = Regex::new(r#"(?is)<ref\b([^>/]*?)/>|<ref\b([^>]*)>(.*?)</ref>"#).unwrap();
    let name_re = Regex::new(r#"(?i)\bname\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#).unwrap();
    let group_re = Regex::new(r#"(?i)\bgroup\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#).unwrap();

    ref_re
        .captures_iter(text)
        .map(|captures| {
            let attrs = captures
                .get(1)
                .or_else(|| captures.get(2))
                .map(|m| m.as_str())
                .unwrap_or("");
            let name = name_re
                .captures(attrs)
                .and_then(|caps| caps.get(1).or_else(|| caps.get(2)).or_else(|| caps.get(3)))
                .map(|m| normalize_reference_attr(m.as_str()))
                .filter(|value| !value.is_empty());
            let group = group_re
                .captures(attrs)
                .and_then(|caps| caps.get(1).or_else(|| caps.get(2)).or_else(|| caps.get(3)))
                .map(|m| normalize_reference_attr(m.as_str()))
                .unwrap_or_default();
            let content = captures
                .get(3)
                .map(|m| m.as_str().trim().to_string())
                .filter(|value| !value.is_empty());

            ReferenceTag {
                group,
                name,
                content,
            }
        })
        .collect()
}
