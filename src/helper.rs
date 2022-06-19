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
