use crate::daitch_mokotoff::Rule;

fn parse_branch(part: &str) -> Vec<String> {
    part.split('|').map(|v| v.to_string()).collect()
}

impl From<(&str, &str, &str, &str)> for Rule {
    fn from((part1, part2, part3, part4): (&str, &str, &str, &str)) -> Self {
        let pattern = part1.to_string();
        let replacement_at_start: Vec<String> = parse_branch(part2);
        let replacement_before_vowel: Vec<String> = parse_branch(part3);
        let replacement_default: Vec<String> = parse_branch(part4);
        Self {
            pattern,
            replacement_at_start,
            replacement_before_vowel,
            replacement_default,
        }
    }
}
