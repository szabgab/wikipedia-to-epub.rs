use std::collections::{HashMap, HashSet};

use regex::Regex;

use crate::tools::{matching_template_end, split_template_name};
use crate::wikitext::normalize_reference_attr;

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

pub(crate) fn collect_reference_groups(text: &str) -> HashMap<String, Vec<String>> {
    let mut named_definitions = HashMap::<(String, String), String>::new();
    for tag in parse_reference_tags(text) {
        if let (Some(name), Some(content)) = (tag.name, tag.content) {
            named_definitions.insert((tag.group, name), content);
        }
    }

    let mut groups = HashMap::<String, Vec<String>>::new();
    let mut seen_named = HashSet::<(String, String)>::new();
    let occurrence_text = strip_reflist_templates(text);

    for tag in parse_reference_tags(&occurrence_text) {
        match (tag.name, tag.content) {
            (Some(name), Some(content)) => {
                if seen_named.insert((tag.group.clone(), name)) {
                    groups.entry(tag.group).or_default().push(content);
                }
            }
            (Some(name), None) => {
                let key = (tag.group.clone(), name.clone());
                if seen_named.insert(key.clone())
                    && let Some(content) = named_definitions.get(&key)
                {
                    groups.entry(tag.group).or_default().push(content.clone());
                }
            }
            (None, Some(content)) => {
                groups.entry(tag.group).or_default().push(content);
            }
            (None, None) => {}
        }
    }

    groups
}

pub(crate) fn strip_reflist_templates(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut offset = 0usize;

    while let Some(start) = text[offset..].find("{{").map(|index| offset + index) {
        output.push_str(&text[offset..start]);
        if let Some(end) = matching_template_end(text, start) {
            let content = &text[start + 2..end];
            let (template, _) = split_template_name(content);
            if template.trim().eq_ignore_ascii_case("reflist") {
                offset = end + 2;
                continue;
            }
        }
        output.push_str("{{");
        offset = start + 2;
    }

    output.push_str(&text[offset..]);
    output
}

#[cfg(test)]
mod tests {
    use super::collect_reference_groups;
    use super::parse_reference_tags;
    use super::strip_reflist_templates;

    #[test]
    fn collect_reference_groups_collects_anonymous_references() {
        let groups = collect_reference_groups("Text <ref>One</ref> more <ref>Two</ref>");
        assert_eq!(
            groups.get(""),
            Some(&vec!["One".to_string(), "Two".to_string()])
        );
    }

    #[test]
    fn collect_reference_groups_collects_grouped_references() {
        let groups = collect_reference_groups(
            "Text <ref group=note>Note 1</ref> <ref>Ref 1</ref> <ref group=note>Note 2</ref>",
        );
        assert_eq!(
            groups.get("note"),
            Some(&vec!["Note 1".to_string(), "Note 2".to_string()])
        );
        assert_eq!(groups.get(""), Some(&vec!["Ref 1".to_string()]));
    }

    #[test]
    fn collect_reference_groups_resolves_named_references() {
        let groups =
            collect_reference_groups("Text <ref name=abc>Content</ref> and reuse <ref name=abc />");
        assert_eq!(groups.get(""), Some(&vec!["Content".to_string()]));
    }

    #[test]
    fn collect_reference_groups_resolves_out_of_order_named_references() {
        let groups = collect_reference_groups(
            "Reuse <ref name=abc /> before definition <ref name=abc>Content</ref>",
        );
        assert_eq!(groups.get(""), Some(&vec!["Content".to_string()]));
    }

    #[test]
    fn collect_reference_groups_ignores_reflist_content() {
        let groups = collect_reference_groups("<ref>Keep</ref> {{reflist|<ref>Ignore</ref>}}");
        assert_eq!(groups.get(""), Some(&vec!["Keep".to_string()]));
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

    #[test]
    fn strip_reflist_templates_removes_simple_reflist_template() {
        assert_eq!(
            strip_reflist_templates("Before {{reflist}} after"),
            "Before  after"
        );
    }

    #[test]
    fn strip_reflist_templates_removes_reflist_template_with_parameters() {
        assert_eq!(
            strip_reflist_templates("Before {{reflist|group=note}} after"),
            "Before  after"
        );
        assert_eq!(
            strip_reflist_templates("Before {{RefList|1}} after"),
            "Before  after"
        );
    }

    #[test]
    fn strip_reflist_templates_preserves_other_templates() {
        assert_eq!(
            strip_reflist_templates("Before {{other|reflist}} after"),
            "Before {{other|reflist}} after"
        );
    }

    #[test]
    fn strip_reflist_templates_preserves_unclosed_reflist_template() {
        assert_eq!(
            strip_reflist_templates("Before {{reflist after"),
            "Before {{reflist after"
        );
    }

    #[test]
    fn strip_reflist_templates_removes_multiple_reflist_templates() {
        assert_eq!(
            strip_reflist_templates("{{reflist}} middle {{Reflist|group=n}}"),
            " middle "
        );
    }
}
