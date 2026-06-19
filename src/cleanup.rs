use regex::Regex;

pub(crate) fn normalize_reference_attr(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
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

#[cfg(test)]
mod tests {
    use super::normalize_reference_attr;
    use super::parse_reference_tags;

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

    #[test]
    fn parse_reference_tags_reads_named_reference_with_content() {
        let tags = parse_reference_tags(
            r#"Intro <ref name="alpha" group="note"> Example reference </ref> outro."#,
        );

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name.as_deref(), Some("alpha"));
        assert_eq!(tags[0].group, "note");
        assert_eq!(tags[0].content.as_deref(), Some("Example reference"));
    }

    #[test]
    fn parse_reference_tags_reads_self_closing_named_reference() {
        let tags = parse_reference_tags(r#"Intro <ref group='n' name='alpha' /> outro."#);

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name.as_deref(), Some("alpha"));
        assert_eq!(tags[0].group, "n");
        assert_eq!(tags[0].content, None);
    }

    #[test]
    fn parse_reference_tags_reads_unquoted_attributes() {
        let tags = parse_reference_tags(r#"<ref name=alpha group=n>Body</ref>"#);

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name.as_deref(), Some("alpha"));
        assert_eq!(tags[0].group, "n");
        assert_eq!(tags[0].content.as_deref(), Some("Body"));
    }

    #[test]
    fn parse_reference_tags_filters_empty_name_and_content() {
        let tags = parse_reference_tags(r#"<ref name="">   </ref>"#);

        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, None);
        assert_eq!(tags[0].group, "");
        assert_eq!(tags[0].content, None);
    }

    #[test]
    fn parse_reference_tags_preserves_reference_order() {
        let tags = parse_reference_tags(
            r#"<ref name="first">One</ref> middle <ref name="second">Two</ref>"#,
        );

        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].name.as_deref(), Some("first"));
        assert_eq!(tags[0].content.as_deref(), Some("One"));
        assert_eq!(tags[1].name.as_deref(), Some("second"));
        assert_eq!(tags[1].content.as_deref(), Some("Two"));
    }
}
