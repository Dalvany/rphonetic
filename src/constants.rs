use regex::Regex;

lazy_static! {
    pub static ref RULE_LINE: Regex = Regex::new(
        r"\s*\x22(.+?)\x22\s+\x22(.*?)\x22\s+\x22(.*?)\x22\s+\x22(.*?)\x22\s*(//.*){0,1}$"
    )
    .unwrap();
    pub static ref RULE_LANG_LINE: Regex =
        Regex::new(r"\s*(.+?)\s+(.*?)\s+(.*?)\s*(//.*){0,1}$").unwrap();
    pub static ref DM_LANGUAGE_LINE: Regex = Regex::new(r"^\s*(.+?)\s*(//.*){0,1}$").unwrap();
    pub static ref BM_INCLUDE_LINE: Regex =
        Regex::new(r"^\s*#include\s+([a-z_]+?)\s*(//.*){0,1}$").unwrap();
}

pub const SINGLE_LINE_COMMENT: &str = "//";
pub const MULTI_LINE_COMMENT_START: &str = "/*";
pub const MULTI_LINE_COMMENT_END: &str = "*/";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regexp() {
        let data = "  \"part1\"   \"part2\" \"part3\"\t\"part4\"";
        assert!(RULE_LINE.is_match(data));
        for cap in RULE_LINE.captures_iter(data) {
            assert_eq!(&cap[1], "part1");
            assert_eq!(&cap[2], "part2");
            assert_eq!(&cap[3], "part3");
            assert_eq!(&cap[4], "part4");
        }
    }

    #[test]
    fn test_regexp_with_one_line_comment() {
        let data =
            "  \"part1\"   \"part2\"\t \"part3\"\t\"part4\"\t\t // This is a one line comment";
        assert!(RULE_LINE.is_match(data));
        for cap in RULE_LINE.captures_iter(data) {
            assert_eq!(&cap[1], "part1");
            assert_eq!(&cap[2], "part2");
            assert_eq!(&cap[3], "part3");
            assert_eq!(&cap[4], "part4");
        }
    }

    #[test]
    fn test_regexp_with_empty_parts() {
        let data = "  \"part1\"   \"part2\"\t \"\"\t\"\"\t\t";
        assert!(RULE_LINE.is_match(data));
        for cap in RULE_LINE.captures_iter(data) {
            assert_eq!(&cap[1], "part1");
            assert_eq!(&cap[2], "part2");
            assert_eq!(&cap[3], "");
            assert_eq!(&cap[4], "");
        }
    }

    #[test]
    fn test_regexp_no_match() {
        let data = "  \"part1\"   \t \"part3\"\t\"part4\"\t\t // This is not a match, missing a part \"test\"";
        assert!(!RULE_LINE.is_match(data));
    }

    #[test]
    fn test_regexp_whatever() {
        let data = "This is not a match";
        assert!(!RULE_LINE.is_match(data));
    }
}
