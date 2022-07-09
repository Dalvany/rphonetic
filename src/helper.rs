/// Replace regex like "s+" by a single char "S".
pub fn replace_compact_all_to_uppercase(string: String, chars: Vec<char>) -> String {
    let mut ret = String::with_capacity(string.len());
    let mut previous: Option<char> = None;

    string.chars().for_each(|ch| {
        if chars.contains(&ch) {
            if let Some(prev) = previous {
                if prev != ch {
                    ret.push(ch.to_ascii_uppercase());
                    previous = Some(ch);
                }
            } else {
                ret.push(ch.to_ascii_uppercase());
                previous = Some(ch);
            }
        } else {
            ret.push(ch);
            previous = None;
        }
    });

    ret
}

/// Test if `string` ends with `pattern` and replace it by `to`.
pub fn replace_end<'a>(mut string: String, pattern: &'a str, to: &'a str) -> String {
    if string.ends_with(pattern) {
        string.replace_range(string.len() - pattern.len().., to);
    }
    string
}

/// Test if a char is a vowel.
pub fn is_vowel(c: Option<char>, include_y: bool) -> bool {
    match c {
        Some(ch) => matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u') || (include_y && ch == 'y'),
        None => false,
    }
}

pub fn replace_char<F>(string: String, f: F) -> String
where
    F: FnMut((usize, char)) -> char,
{
    string
        .chars()
        .into_iter()
        .enumerate()
        .map(f)
        .collect::<String>()
}

pub fn remove_all_nonletter(string: String) -> String {
    string
        .chars()
        .into_iter()
        .filter(|&c| c.is_lowercase())
        .collect::<String>()
}

pub fn soundex_clean(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_alphabetic())
        .map(|c| c.to_uppercase().collect::<String>())
        .collect()
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_vowel() {
        assert!(is_vowel(Some('a'), false));
        assert!(is_vowel(Some('e'), false));
        assert!(is_vowel(Some('i'), false));
        assert!(is_vowel(Some('o'), false));
        assert!(is_vowel(Some('u'), false));
        assert!(!is_vowel(Some('b'), false));
        assert!(!is_vowel(Some('d'), false));
        assert!(!is_vowel(Some('p'), false));
        assert!(!is_vowel(Some('q'), false));
        assert!(!is_vowel(Some('z'), false));
        assert!(!is_vowel(Some('A'), false));
        assert!(!is_vowel(Some('I'), false));
        assert!(!is_vowel(Some('3'), false));

        assert!(!is_vowel(Some('y'), false));
        assert!(is_vowel(Some('y'), true));

        assert!(!is_vowel(None, false));
    }

    #[test]
    fn test_replace_compact_all_to_uppercase_nothing_to_compact() {
        let result =
            replace_compact_all_to_uppercase("aaaabbbbccccdddd".to_string(), vec!['e', 'f', 'g']);
        assert_eq!(result, "aaaabbbbccccdddd");
    }

    #[test]
    fn test_replace_compact_all_to_uppercase_compact_all() {
        let result = replace_compact_all_to_uppercase(
            "aaaabbbbccccdddd".to_string(),
            vec!['a', 'b', 'c', 'd'],
        );
        assert_eq!(result, "ABCD");
    }

    #[test]
    fn test_replace_compact_all_to_uppercase() {
        let result =
            replace_compact_all_to_uppercase("aaaabbbbccccdddd".to_string(), vec!['b', 'd']);
        assert_eq!(result, "aaaaBccccD");
    }

    #[test]
    fn test_soundex_clean() {
        let result =
            soundex_clean("This is a test ! With numbers like 0 and other alphabet like 中 or δ.");

        assert_eq!(
            result,
            "THISISATESTWITHNUMBERSLIKEANDOTHERALPHABETLIKE中ORΔ"
        );
    }
}
