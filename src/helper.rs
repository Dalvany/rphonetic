/// Replace regex like "s+" by a single char "S"
pub fn replace_compact_all(string: String, pattern: char, to: char) -> String {
    let mut already_replaced = false;
    let mut ret = String::with_capacity(string.len());

    string.chars().for_each(|ch| {
        if ch != pattern {
            ret.push(ch);
            already_replaced = false;
        } else {
            if !already_replaced {
                ret.push(to);
                already_replaced = true;
            }
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
    match c {
        'a' | 'e' | 'i' | 'o' | 'u' => true,
        _ => false,
    }
}

fn is_letter(c: char) -> bool {
    match c {
        'a'..='z' => true,
        _ => false,
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
        .filter(|&c| is_letter(c))
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_vowel() {
        assert_eq!(is_vowel('a'), true);
        assert_eq!(is_vowel('e'), true);
        assert_eq!(is_vowel('i'), true);
        assert_eq!(is_vowel('o'), true);
        assert_eq!(is_vowel('u'), true);
        assert_eq!(is_vowel('b'), false);
        assert_eq!(is_vowel('d'), false);
        assert_eq!(is_vowel('p'), false);
        assert_eq!(is_vowel('q'), false);
        assert_eq!(is_vowel('z'), false);
        assert_eq!(is_vowel('A'), false);
        assert_eq!(is_vowel('I'), false);
        assert_eq!(is_vowel('3'), false);
    }

    #[test]
    fn test_is_letter() {
        assert_eq!(is_letter('a'), true);
        assert_eq!(is_letter('b'), true);
        assert_eq!(is_letter('c'), true);
        assert_eq!(is_letter('d'), true);
        assert_eq!(is_letter('e'), true);
        assert_eq!(is_letter('f'), true);
        assert_eq!(is_letter('g'), true);
        assert_eq!(is_letter('h'), true);
        assert_eq!(is_letter('i'), true);
        assert_eq!(is_letter('j'), true);
        assert_eq!(is_letter('k'), true);
        assert_eq!(is_letter('l'), true);
        assert_eq!(is_letter('m'), true);
        assert_eq!(is_letter('n'), true);
        assert_eq!(is_letter('o'), true);
        assert_eq!(is_letter('p'), true);
        assert_eq!(is_letter('q'), true);
        assert_eq!(is_letter('r'), true);
        assert_eq!(is_letter('s'), true);
        assert_eq!(is_letter('t'), true);
        assert_eq!(is_letter('u'), true);
        assert_eq!(is_letter('v'), true);
        assert_eq!(is_letter('w'), true);
        assert_eq!(is_letter('x'), true);
        assert_eq!(is_letter('y'), true);
        assert_eq!(is_letter('z'), true);
        assert_eq!(is_letter('A'), false);
        assert_eq!(is_letter('Z'), false);
        assert_eq!(is_letter('1'), false);
        assert_eq!(is_letter('0'), false);
        assert_eq!(is_letter('O'), false);
        assert_eq!(is_letter('\n'), false);
        assert_eq!(is_letter('こ'), false);
    }
}
