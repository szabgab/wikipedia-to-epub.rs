use regex::Regex;

use crate::tools::split_template_name;

pub(crate) fn cleanup_wikitext(text: &str) -> String {
    let text = text.replace("\r\n", "\n");
    remove_comments(&text)
}

fn remove_comments(text: &str) -> String {
    Regex::new(r"(?s)<!--.*?-->")
        .unwrap()
        .replace_all(text, "")
        .into_owned()
}

pub(crate) fn remove_some_html_tags(text: &str) -> String {
    let mut text = text.to_string();
    for tag in ["gallery", "timeline", "math", "score", "syntaxhighlight"] {
        let pattern = format!(r"(?is)<{tag}\b[^>]*>.*?</{tag}>");
        text = Regex::new(&pattern)
            .unwrap()
            .replace_all(&text, "")
            .into_owned();
    }

    text = Regex::new(r"(?i)<br\s*/?>")
        .unwrap()
        .replace_all(&text, "\n")
        .into_owned();

    text
}

pub(crate) fn normalize_reference_attr(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
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
    use super::matching_template_end;
    use super::normalize_reference_attr;
    use super::remove_some_html_tags;
    use super::strip_reflist_templates;

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

    #[test]
    fn remove_some_html_tags_removes_gallery_tags() {
        assert_eq!(
            remove_some_html_tags("<gallery>Image1.png|Label1\nImage2.png</gallery>"),
            ""
        );
        assert_eq!(
            remove_some_html_tags("<gallery class=\"abc\">Image1.png|Label1</gallery>"),
            ""
        );
    }

    #[test]
    fn remove_some_html_tags_removes_math_tags() {
        assert_eq!(remove_some_html_tags("<math>1 + 1 = 2</math>"), "");
    }

    #[test]
    fn remove_some_html_tags_removes_timeline_score_syntaxhighlight_tags() {
        assert_eq!(
            remove_some_html_tags(
                "<timeline>Timeline info</timeline> <score>Score info</score> <syntaxhighlight lang=\"rust\">println!(\"Hello\");</syntaxhighlight>"
            ),
            "  "
        );
    }

    #[test]
    fn remove_some_html_tags_converts_br_to_newline() {
        assert_eq!(
            remove_some_html_tags("Line1<br>Line2<br/>Line3<br   />Line4"),
            "Line1\nLine2\nLine3\nLine4"
        );
        assert_eq!(remove_some_html_tags("Line1<BR>Line2"), "Line1\nLine2");
    }

    #[test]
    fn remove_some_html_tags_preserves_other_html() {
        assert_eq!(
            remove_some_html_tags("<p>Paragraph</p> <b>bold</b>"),
            "<p>Paragraph</p> <b>bold</b>"
        );
    }
}
