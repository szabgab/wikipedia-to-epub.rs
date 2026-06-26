use regex::Regex;

pub(crate) fn remove_comments(text: &str) -> String {
    Regex::new(r"(?s)<!--.*?-->")
        .unwrap()
        .replace_all(text, "")
        .into_owned()
}

pub(crate) fn remove_some_html_tags(text: &str) -> String {
    let mut text = text.to_string();
    for tag in ["gallery", "timeline", "score", "syntaxhighlight"] {
        let pattern = format!(r"(?is)<{tag}\b[^>]*>.*?</{tag}>");
        text = Regex::new(&pattern)
            .unwrap()
            .replace_all(&text, "")
            .into_owned();
    }

    // https://en.wikipedia.org/wiki/Help:Displaying_a_formula
    let math_re = Regex::new(r"(?is)<math\b[^>]*>(.*?)</math>").unwrap();
    text = math_re
        .replace_all(&text, |caps: &regex::Captures| {
            crate::tools::clean_math_latex(&caps[1])
        })
        .into_owned();

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

#[cfg(test)]
mod tests {
    use super::normalize_reference_attr;
    use super::remove_some_html_tags;

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
    fn remove_some_html_tags_cleans_math_tags() {
        assert_eq!(remove_some_html_tags("<math>1 + 1 = 2</math>"), "1 + 1 = 2");
        assert_eq!(
            remove_some_html_tags("<math display=\"inline\">\\binom{n}{k}</math>"),
            "(n/k)"
        );
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
