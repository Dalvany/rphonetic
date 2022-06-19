/// Replace regex like "s+" by a single char "S"
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

pub fn replace_end<'a>(mut string: String, pattern: &'a str, to: &'a str) -> String {
    if string.ends_with(pattern) {
        string.replace_range(string.len() - pattern.len().., to);
    }
    string
}

pub fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}

fn is_letter(c: char) -> bool {
    matches!(c, 'a'..='z')
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
        .filter(|&c| is_letter(c))
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

    #[test]
    fn test_is_letter() {
        assert!(is_letter('a'));
        assert!(is_letter('b'));
        assert!(is_letter('c'));
        assert!(is_letter('d'));
        assert!(is_letter('e'));
        assert!(is_letter('f'));
        assert!(is_letter('g'));
        assert!(is_letter('h'));
        assert!(is_letter('i'));
        assert!(is_letter('j'));
        assert!(is_letter('k'));
        assert!(is_letter('l'));
        assert!(is_letter('m'));
        assert!(is_letter('n'));
        assert!(is_letter('o'));
        assert!(is_letter('p'));
        assert!(is_letter('q'));
        assert!(is_letter('r'));
        assert!(is_letter('s'));
        assert!(is_letter('t'));
        assert!(is_letter('u'));
        assert!(is_letter('v'));
        assert!(is_letter('w'));
        assert!(is_letter('x'));
        assert!(is_letter('y'));
        assert!(is_letter('z'));
        assert!(!is_letter('A'));
        assert!(!is_letter('Z'));
        assert!(!is_letter('1'));
        assert!(!is_letter('0'));
        assert!(!is_letter('O'));
        assert!(!is_letter('\n'));
        assert!(!is_letter('こ'));
    }
}
