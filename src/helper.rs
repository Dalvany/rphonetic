/// Replace regex like "s+" by a single char "S".
pub fn replace_compact_all(string: String, pattern: char, to: char) -> String {
    let mut already_replaced = false;
    let mut ret = String::with_capacity(string.len());

    string.chars().for_each(|ch| {
        if ch != pattern {
            ret.push(ch);
            already_replaced = false;
        } else if !already_replaced {
            ret.push(to);
            already_replaced = true;
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
pub fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
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
        .filter(|&c| c.is_ascii_lowercase())
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_vowel() {
        assert!(is_vowel('a'));
        assert!(is_vowel('e'));
        assert!(is_vowel('i'));
        assert!(is_vowel('o'));
        assert!(is_vowel('u'));
        assert!(!is_vowel('b'));
        assert!(!is_vowel('d'));
        assert!(!is_vowel('p'));
        assert!(!is_vowel('q'));
        assert!(!is_vowel('z'));
        assert!(!is_vowel('A'));
        assert!(!is_vowel('I'));
        assert!(!is_vowel('3'));
    }
}
