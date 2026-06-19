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
