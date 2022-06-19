/// Replace regex like "s+" by a single char "S"
pub fn replace_compact_all(string: String, search: char, replace: char) -> String {
    let mut already_replaced = false;
    let mut ret = String::with_capacity(string.len());

    string.chars().for_each(|ch| {
        if ch != search {
            ret.push(ch);
            already_replaced = false;
        } else {
            if !already_replaced {
                ret.push(replace);
                already_replaced = true;
            }
        }
    });

    ret
}
