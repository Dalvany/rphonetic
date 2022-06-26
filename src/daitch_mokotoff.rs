use std::collections::BTreeMap;

use crate::{Encoder, PhoneticError, RULE_LINE};
use crate::helper::is_vowel;

const DEFAULT_RULES: &str = include_str!("rules/dmrules.txt");

/// Max length of a DM soundex value.
const MAX_LENGTH: usize = 6;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct Branch<'a> {
    builder: String,
    last_replacement: Option<&'a str>,
}

impl<'a> Default for Branch<'a> {
    fn default() -> Self {
        Self {
            builder: String::with_capacity(MAX_LENGTH),
            last_replacement: None,
        }
    }
}


impl<'a> Branch<'a> {
    /// Finish to match [MAX_LENGTH] by appending `0`.
    fn finish(&mut self) {
        while self.builder.len() < MAX_LENGTH {
            self.builder.push('0');
        }
    }

    fn process_next_replacement(&mut self, replacement: &'a str, append_force: bool) {
        let append = self.last_replacement.map_or(true, |v| !v.ends_with(replacement)) || append_force;

        if append && self.builder.len() < MAX_LENGTH {
            self.builder.push_str(replacement);
            if self.builder.len() > MAX_LENGTH {
                self.builder = self.builder[0..MAX_LENGTH].to_string();
            }
        }

        self.last_replacement = Some(replacement);
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
struct Rule<'a> {
    pattern: &'a str,
    replacement_at_start: Vec<&'a str>,
    replacement_before_vowel: Vec<&'a str>,
    replacement_default: Vec<&'a str>,
}

impl<'a> Rule<'a> {
    fn parse_branch(part: &'a str) -> Vec<&'a str> {
        part.split('|').collect()
    }

    fn get_pattern_length(&self) -> usize {
        self.pattern.len()
    }

    fn matches(&self, context: &str) -> bool {
        context.starts_with(self.pattern)
    }

    fn get_replacements(&self, context: &str, at_start: bool) -> &Vec<&'a str> {
        if at_start {
            return &self.replacement_at_start;
        }

        let next_index = self.get_pattern_length();
        let next_char_is_vowel = next_index < context.len() && is_vowel(context.chars().nth(next_index).unwrap());
        if next_char_is_vowel {
            return &self.replacement_before_vowel;
        }

        &self.replacement_default
    }
}

impl<'a> TryFrom<&'a str> for Rule<'a> {
    type Error = PhoneticError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let cap_opt = RULE_LINE.captures(value);

        match cap_opt {
            None => Err(PhoneticError::ParseRuleError(format!("Rule doesn't follow format \"pattern\" \"replacement at start\" \"replacement before vowel\" \"default replacement\" or char=char. Got : {}", value))),
            Some(cap) => {
                let pattern = cap.get(1).unwrap().as_str();
                let replacement_at_start: Vec<&str> = Rule::parse_branch(cap.get(2).unwrap().as_str());
                let replacement_before_vowel: Vec<&str> = Rule::parse_branch(cap.get(3).unwrap().as_str());
                let replacement_default: Vec<&str> = Rule::parse_branch(cap.get(4).unwrap().as_str());
                Ok(Self {
                    pattern,
                    replacement_at_start,
                    replacement_before_vowel,
                    replacement_default,
                })
            }
        }
    }
}

/// This the [Daitch Mokotoff soundex](https://en.wikipedia.org/wiki/Daitch%E2%80%93Mokotoff_Soundex) implementation.
///
/// [Default] implementation use [commons-codec rules](https://github.com/apache/commons-codec/blob/master/src/main/resources/org/apache/commons/codec/language/dmrules.txt).
///
/// It can be constructed with custom rules using [TryFrom].
///
/// A rule is either in the form of :
/// * `char`=`char` (a char is converted into another char, this is use for ASCII folding)
/// * "`pattern`" "`replacement_at_start`" "`replacement_before_vowel`" "`default_replacement`"
///     * `pattern` : a string to match
///     * `replacement_at_start` : the code to replace `pattern` with if `pattern` is at the start of the word.
///     * `replacement_before_vowel` : the code to replace `pattern` with if `pattern` is before a vowel inside the word.
///     * `default_replacement` : the code to replace `pattern` with for other case.
/// To support branching, any pattern can be in the form of `code|code|...`.
///
/// Rules are separated by `\n`.
///
/// Parse support single line comment using `//` and multiline comments using `/* ... */`. Note that multiline comment must start
/// at the beginning of a line.
///
/// # Example :
///
/// Here is an example of rules :
/// ```norust
/// /*
/// This
/// is
/// a
/// multiline
/// comment
///  */
///
/// // This is a single line comment.
///
/// à=a // You can put a one line comment at the end of a rule. This rule is for ASCII folding.
/// /*
/// This rule convert the substring `sh` into
///  - `0` if at the start of the word
///  - an empty string if before a vowel
///  - otherwise it does a branching with code `0` and code `1`
///  */
/// "sh" "0" "" "0|1"
/// ```
///
/// In the following example, we construct a [DaitchMokotoffSoundex] using the previous rule :
///
/// ```rust
/// # fn main() -> Result<(), rphonetic::PhoneticError> {
/// use rphonetic::{DaitchMokotoffSoundex, DaitchMokotoffSoundexBuilder};
/// let rules = "/*
/// This
/// is
/// a
/// multiline
/// comment
///  */
///
/// // This is a single line comment.
///
/// à=a // You can put a one line comment at the end of a rule. This rule is for ASCII folding.
/// /*
/// This rule convert the substring `sh` into
///  - `0` if at the start of the word
///  - an empty string if before a vowel
///  - otherwise it does a branching with code `0` and code `1`
///  */
/// \"sh\" \"0\" \"\" \"0|1\"";///
///
/// let daitch_mokotoff = DaitchMokotoffSoundexBuilder::with_rules(rules).build()?;
/// #   Ok(())
/// # }
/// ```
///
/// The algorithm, first, removes all whitespaces and, if enables, apply ASCII folding
/// with provided rules.
///
/// # Encoding
///
/// There is 2 method to encode a string :
/// * [DaitchMokotoffSoundex](#encode) that encode without branching. Only one code is returned
/// * [DaitchMokotoffSoundex](#soundex) that encode with branching. Multiple code, separated by a `|` are returned.
///
/// # Exemples
///
/// ## Encode methode
///
/// ```rust
/// # fn main() -> Result<(), rphonetic::PhoneticError> {
/// use rphonetic::{DaitchMokotoffSoundex, DaitchMokotoffSoundexBuilder, Encoder};
///
/// let encoder = DaitchMokotoffSoundexBuilder::default().build()?;
///
/// assert_eq!(encoder.encode("Rosochowaciec"), "944744");
/// #   Ok(())
/// # }
/// ```
///
/// ## Soundex
///
/// ```rust
/// # fn main() -> Result<(), rphonetic::PhoneticError> {
/// use rphonetic::{DaitchMokotoffSoundex, DaitchMokotoffSoundexBuilder, Encoder};
///
/// let encoder = DaitchMokotoffSoundexBuilder::default().build()?;
///
/// assert_eq!(encoder.soundex("Rosochowaciec"), "944744|944745|944754|944755|945744|945745|945754|945755");
/// #   Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct DaitchMokotoffSoundex<'a> {
    ascii_folding: bool,
    rules: BTreeMap<char, Vec<Rule<'a>>>,
    ascii_folding_rules: BTreeMap<char, char>,
}

impl<'a> DaitchMokotoffSoundex<'a> {
    /// Encode the string with branching.
    pub fn soundex(&self, value: &'a str) -> String {
        self.inner_soundex(value, true).join("|")
    }

    fn inner_soundex(&self, value: &'a str, branching: bool) -> Vec<String> {
        let source = value.chars().filter(|ch| !ch.is_whitespace()).map(|ch| {
            let lower = ch.to_lowercase().next();
            match lower {
                None => ch,
                Some(mut lower) => {
                    if self.ascii_folding && self.ascii_folding_rules.contains_key(&lower) {
                        lower = *self.ascii_folding_rules.get(&lower).unwrap();
                    }

                    lower
                }
            }
        }).collect::<String>();

        let mut current_branches: Vec<Branch> = vec![Branch::default()];

        let mut last_char = '\0';
        let mut iterator = source.char_indices();
        while let Some((index, ch)) = iterator.next() {
            // Get context
            let context = &source[index..];

            // Get rules for character
            let rules = self.rules.get(&ch);

            if let Some(rules) = rules {
                for rule in rules {
                    if rule.matches(context) {
                        let mut next_branches: Vec<Branch> = Vec::new();

                        let replacement = rule.get_replacements(context, last_char == '\0');

                        for branch in current_branches.iter() {
                            for next_replacement in replacement.iter() {
                                let mut next_branch = branch.clone();
                                let force = (last_char == 'm' && ch == 'n') || (last_char == 'n' && ch == 'm');
                                next_branch.process_next_replacement(next_replacement, force);
                                // Perhaps use the crate "linked-hash-map" but its major version is 0 and I want to release a major version
                                if !next_branches.contains(&next_branch) {
                                    next_branches.push(next_branch);
                                }
                                if !branching {
                                    break;
                                }
                            }
                        }

                        current_branches = next_branches;

                        let l = rule.get_pattern_length();
                        // Since nth(..) is 0 base, nth(0) while call "next()", resulting
                        // in a supplementary call.
                        // So we need to "skip" if length >= 2 and we need to substract 2.
                        if l > 1 {
                            let _ = iterator.nth(rule.get_pattern_length() - 2);
                        }
                        break;
                    }
                }
                last_char = ch;
            }
        }

        let mut result: Vec<String> = Vec::with_capacity(current_branches.len());
        for branch in current_branches.iter_mut() {
            branch.finish();
            result.push(branch.builder.clone());
        }

        result
    }

    fn ascii_folding(&mut self, ascii_folding: bool) {
        self.ascii_folding = ascii_folding;
    }
}

/// Construct a [DaitchMokotoffSoundex] encoder with `ascii_folding` enable and
/// using [commons-codec](https://github.com/apache/commons-codec/blob/master/src/main/resources/org/apache/commons/codec/language/dmrules.txt) rules.
///
/// # Warning
///
/// This will probably removed from futur version. Use [DaitchMokotoffSoundexBuilder] instead.
impl<'a> Default for DaitchMokotoffSoundex<'a> {
    fn default() -> Self {
        // Test ensures that unwrap is safe
        Self::try_from(DEFAULT_RULES).unwrap()
    }
}


/// Construct a [DaitchMokotoffSoundex] encoder from custom rules and with `ascii_folding` enable.
impl<'a> TryFrom<&'a str> for DaitchMokotoffSoundex<'a> {
    type Error = PhoneticError;

    fn try_from(custom_rules: &'a str) -> Result<Self, Self::Error> {
        let mut rules: BTreeMap<char, Vec<Rule<'a>>> = BTreeMap::new();
        let mut ascii_folding_rules: BTreeMap<char, char> = BTreeMap::new();
        let mut multiline_comment = false;
        for mut line in custom_rules.split('\n') {
            line = line.trim();

            // Start to test multiline comment ends, thus we can collapse some 'if'.
            if line.ends_with("*/") {
                multiline_comment = false;
                continue;
            } else if line.is_empty() || line.starts_with("//") || multiline_comment {
                continue;
            } else if line.starts_with("/*") {
                multiline_comment = true;
                continue;
            }

            if let Some(index) = line.find('=') {
                let ch = line[0..index].chars().next();
                let replacement = line[index + 1..index + 2].chars().next();

                let _ = match (ch, replacement) {
                    (Some(pattern), Some(replace_by)) => ascii_folding_rules.insert(pattern, replace_by),
                    (_, _) => return Err(PhoneticError::ParseRuleError(format!("Line contains an '=' but is not a char replacement. Got : {}", line))),
                };
            } else {
                let rule = Rule::try_from(line)?;
                // There's always at least one char, the regex ensures that.
                let ch = rule.pattern.chars().next().unwrap();
                rules.entry(ch).or_insert_with(Vec::new).push(rule);
            }
        }

        // Ordering by pattern length decreasing.
        rules.values_mut().for_each(|v| v.sort_by(|a, b| a.pattern.len().cmp(&b.pattern.len()).reverse()));

        Ok(Self {
            ascii_folding: true,
            rules,
            ascii_folding_rules,
        })
    }
}

impl<'a> Encoder for DaitchMokotoffSoundex<'a> {
    /// Encode a string without branching.
    fn encode(&self, s: &str) -> String {
        self.inner_soundex(s, false).get(0).map(|v| v.to_string()).unwrap_or_else(|| "".to_string())
    }
}

/// This is a builder for [DaitchMokotoffSoundex].
#[derive(Clone, Debug)]
pub struct DaitchMokotoffSoundexBuilder<'a> {
    rules: &'a str,
    ascii_folding: bool,
}

/// Create a [DaitchMokotoffSoundexBuilder] with
/// [commons-codec](https://github.com/apache/commons-codec/blob/master/src/main/resources/org/apache/commons/codec/language/dmrules.txt)
/// rules and `ascii_folding` enable.
impl<'a> Default for DaitchMokotoffSoundexBuilder<'a> {
    fn default() -> Self {
        Self {
            rules: DEFAULT_RULES,
            ascii_folding: true,
        }
    }
}

impl<'a> DaitchMokotoffSoundexBuilder<'a> {
    /// Create a [DaitchMokotoffSoundexBuilder] with custom rules and `ascii_folding` enable.
    pub fn with_rules(rules: &'a str) -> Self {
        Self {
            rules,
            ascii_folding: true,
        }
    }

    /// Enable or disable ASCII folding rules.
    pub fn ascii_folding(mut self, ascii_folding: bool) -> Self {
        self.ascii_folding = ascii_folding;

        self
    }

    /// Construct a new [DaitchMokotoffSoundex] encoder.
    ///
    /// # Error
    ///
    /// This method return an error in case it can't parse the rules.
    pub fn build(self) -> Result<DaitchMokotoffSoundex<'a>, PhoneticError> {
        let mut result = DaitchMokotoffSoundex::try_from(self.rules)?;
        result.ascii_folding(self.ascii_folding);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_rules() -> Result<(), PhoneticError> {
        let result = DaitchMokotoffSoundexBuilder::default().build()?;

        let mut ascii_folding_rules: BTreeMap<char, char> = BTreeMap::new();
        ascii_folding_rules.insert('ß', 's');
        ascii_folding_rules.insert('à', 'a');
        ascii_folding_rules.insert('á', 'a');
        ascii_folding_rules.insert('â', 'a');
        ascii_folding_rules.insert('ã', 'a');
        ascii_folding_rules.insert('ä', 'a');
        ascii_folding_rules.insert('å', 'a');
        ascii_folding_rules.insert('æ', 'a');
        ascii_folding_rules.insert('ç', 'c');
        ascii_folding_rules.insert('è', 'e');
        ascii_folding_rules.insert('é', 'e');
        ascii_folding_rules.insert('ê', 'e');
        ascii_folding_rules.insert('ë', 'e');
        ascii_folding_rules.insert('ì', 'i');
        ascii_folding_rules.insert('í', 'i');
        ascii_folding_rules.insert('î', 'i');
        ascii_folding_rules.insert('ï', 'i');
        ascii_folding_rules.insert('ð', 'd');
        ascii_folding_rules.insert('ñ', 'n');
        ascii_folding_rules.insert('ò', 'o');
        ascii_folding_rules.insert('ó', 'o');
        ascii_folding_rules.insert('ô', 'o');
        ascii_folding_rules.insert('õ', 'o');
        ascii_folding_rules.insert('ö', 'o');
        ascii_folding_rules.insert('ø', 'o');
        ascii_folding_rules.insert('ù', 'u');
        ascii_folding_rules.insert('ú', 'u');
        ascii_folding_rules.insert('û', 'u');
        ascii_folding_rules.insert('ý', 'y');
        ascii_folding_rules.insert('ý', 'y');
        ascii_folding_rules.insert('þ', 'b');
        ascii_folding_rules.insert('ÿ', 'y');
        ascii_folding_rules.insert('ć', 'c');
        ascii_folding_rules.insert('ł', 'l');
        ascii_folding_rules.insert('ś', 's');
        ascii_folding_rules.insert('ż', 'z');
        ascii_folding_rules.insert('ź', 'z');

        let mut rules: BTreeMap<char, Vec<Rule>> = BTreeMap::new();
        rules.insert('ą', vec![
            Rule {
                pattern: "ą",
                replacement_at_start: vec![""],
                replacement_before_vowel: vec![""],
                replacement_default: vec!["", "6"],
            }
        ]);
        rules.insert('ę', vec![
            Rule {
                pattern: "ę",
                replacement_at_start: vec![""],
                replacement_before_vowel: vec![""],
                replacement_default: vec!["", "6"],
            }
        ]);
        rules.insert('ț', vec![
            Rule {
                pattern: "ț",
                replacement_at_start: vec!["3", "4"],
                replacement_before_vowel: vec!["3", "4"],
                replacement_default: vec!["3", "4"],
            }
        ]);
        rules.insert('a', vec![
            Rule {
                pattern: "ai",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec!["1"],
                replacement_default: vec![""],
            },
            Rule {
                pattern: "aj",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec!["1"],
                replacement_default: vec![""],
            },
            Rule {
                pattern: "ay",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec!["1"],
                replacement_default: vec![""],
            },
            Rule {
                pattern: "au",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec!["7"],
                replacement_default: vec![""],
            },
            Rule {
                pattern: "a",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec![""],
                replacement_default: vec![""],
            },
        ]);
        rules.insert('b', vec![
            Rule {
                pattern: "b",
                replacement_at_start: vec!["7"],
                replacement_before_vowel: vec!["7"],
                replacement_default: vec!["7"],
            }
        ]);
        rules.insert('c', vec![
            Rule {
                pattern: "chs",
                replacement_at_start: vec!["5"],
                replacement_before_vowel: vec!["54"],
                replacement_default: vec!["54"],
            },
            Rule {
                pattern: "csz",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "czs",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "cz",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "cs",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "ch",
                replacement_at_start: vec!["4", "5"],
                replacement_before_vowel: vec!["4", "5"],
                replacement_default: vec!["4", "5"],
            },
            Rule {
                pattern: "ck",
                replacement_at_start: vec!["5", "45"],
                replacement_before_vowel: vec!["5", "45"],
                replacement_default: vec!["5", "45"],
            },
            Rule {
                pattern: "c",
                replacement_at_start: vec!["4", "5"],
                replacement_before_vowel: vec!["4", "5"],
                replacement_default: vec!["4", "5"],
            },
        ]);
        rules.insert('ţ', vec![
            Rule {
                pattern: "ţ",
                replacement_at_start: vec!["3", "4"],
                replacement_before_vowel: vec!["3", "4"],
                replacement_default: vec!["3", "4"],
            }
        ]);
        rules.insert('d', vec![
            Rule {
                pattern: "drz",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "drs",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "dsh",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "dsz",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "dzh",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "dzs",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "ds",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "dz",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "dt",
                replacement_at_start: vec!["3"],
                replacement_before_vowel: vec!["3"],
                replacement_default: vec!["3"],
            },
            Rule {
                pattern: "d",
                replacement_at_start: vec!["3"],
                replacement_before_vowel: vec!["3"],
                replacement_default: vec!["3"],
            },
        ]);
        rules.insert('e', vec![
            Rule {
                pattern: "ei",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec!["1"],
                replacement_default: vec![""],
            },
            Rule {
                pattern: "ej",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec!["1"],
                replacement_default: vec![""],
            },
            Rule {
                pattern: "ey",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec!["1"],
                replacement_default: vec![""],
            },
            Rule {
                pattern: "eu",
                replacement_at_start: vec!["1"],
                replacement_before_vowel: vec!["1"],
                replacement_default: vec![""],
            },
            Rule {
                pattern: "e",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec![""],
                replacement_default: vec![""],
            },
        ]);
        rules.insert('f', vec![
            Rule {
                pattern: "fb",
                replacement_at_start: vec!["7"],
                replacement_before_vowel: vec!["7"],
                replacement_default: vec!["7"],
            },
            Rule {
                pattern: "f",
                replacement_at_start: vec!["7"],
                replacement_before_vowel: vec!["7"],
                replacement_default: vec!["7"],
            },
        ]);
        rules.insert('g', vec![
            Rule {
                pattern: "g",
                replacement_at_start: vec!["5"],
                replacement_before_vowel: vec!["5"],
                replacement_default: vec!["5"],
            },
        ]);
        rules.insert('h', vec![
            Rule {
                pattern: "h",
                replacement_at_start: vec!["5"],
                replacement_before_vowel: vec!["5"],
                replacement_default: vec![""],
            },
        ]);
        rules.insert('i', vec![
            Rule {
                pattern: "ia",
                replacement_at_start: vec!["1"],
                replacement_before_vowel: vec![""],
                replacement_default: vec![""],
            },
            Rule {
                pattern: "ie",
                replacement_at_start: vec!["1"],
                replacement_before_vowel: vec![""],
                replacement_default: vec![""],
            },
            Rule {
                pattern: "io",
                replacement_at_start: vec!["1"],
                replacement_before_vowel: vec![""],
                replacement_default: vec![""],
            },
            Rule {
                pattern: "iu",
                replacement_at_start: vec!["1"],
                replacement_before_vowel: vec![""],
                replacement_default: vec![""],
            },
            Rule {
                pattern: "i",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec![""],
                replacement_default: vec![""],
            },
        ]);
        rules.insert('j', vec![
            Rule {
                pattern: "j",
                replacement_at_start: vec!["1", "4"],
                replacement_before_vowel: vec!["", "4"],
                replacement_default: vec!["", "4"],
            },
        ]);
        rules.insert('k', vec![
            Rule {
                pattern: "ks",
                replacement_at_start: vec!["5"],
                replacement_before_vowel: vec!["54"],
                replacement_default: vec!["54"],
            },
            Rule {
                pattern: "kh",
                replacement_at_start: vec!["5"],
                replacement_before_vowel: vec!["5"],
                replacement_default: vec!["5"],
            },
            Rule {
                pattern: "k",
                replacement_at_start: vec!["5"],
                replacement_before_vowel: vec!["5"],
                replacement_default: vec!["5"],
            },
        ]);
        rules.insert('l', vec![
            Rule {
                pattern: "l",
                replacement_at_start: vec!["8"],
                replacement_before_vowel: vec!["8"],
                replacement_default: vec!["8"],
            },
        ]);
        rules.insert('m', vec![
            Rule {
                pattern: "mn",
                replacement_at_start: vec!["66"],
                replacement_before_vowel: vec!["66"],
                replacement_default: vec!["66"],
            },
            Rule {
                pattern: "m",
                replacement_at_start: vec!["6"],
                replacement_before_vowel: vec!["6"],
                replacement_default: vec!["6"],
            },
        ]);
        rules.insert('n', vec![
            Rule {
                pattern: "nm",
                replacement_at_start: vec!["66"],
                replacement_before_vowel: vec!["66"],
                replacement_default: vec!["66"],
            },
            Rule {
                pattern: "n",
                replacement_at_start: vec!["6"],
                replacement_before_vowel: vec!["6"],
                replacement_default: vec!["6"],
            },
        ]);
        rules.insert('o', vec![
            Rule {
                pattern: "oi",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec!["1"],
                replacement_default: vec![""],
            },
            Rule {
                pattern: "oj",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec!["1"],
                replacement_default: vec![""],
            },
            Rule {
                pattern: "oy",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec!["1"],
                replacement_default: vec![""],
            },
            Rule {
                pattern: "o",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec![""],
                replacement_default: vec![""],
            },
        ]);
        rules.insert('p', vec![
            Rule {
                pattern: "pf",
                replacement_at_start: vec!["7"],
                replacement_before_vowel: vec!["7"],
                replacement_default: vec!["7"],
            },
            Rule {
                pattern: "ph",
                replacement_at_start: vec!["7"],
                replacement_before_vowel: vec!["7"],
                replacement_default: vec!["7"],
            },
            Rule {
                pattern: "p",
                replacement_at_start: vec!["7"],
                replacement_before_vowel: vec!["7"],
                replacement_default: vec!["7"],
            },
        ]);
        rules.insert('q', vec![
            Rule {
                pattern: "q",
                replacement_at_start: vec!["5"],
                replacement_before_vowel: vec!["5"],
                replacement_default: vec!["5"],
            },
        ]);
        rules.insert('r', vec![
            Rule {
                pattern: "rs",
                replacement_at_start: vec!["4", "94"],
                replacement_before_vowel: vec!["4", "94"],
                replacement_default: vec!["4", "94"],
            },
            Rule {
                pattern: "rz",
                replacement_at_start: vec!["4", "94"],
                replacement_before_vowel: vec!["4", "94"],
                replacement_default: vec!["4", "94"],
            },
            Rule {
                pattern: "r",
                replacement_at_start: vec!["9"],
                replacement_before_vowel: vec!["9"],
                replacement_default: vec!["9"],
            },
        ]);
        rules.insert('s', vec![
            Rule {
                pattern: "schtsch",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "schtsh",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "schtch",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "shtch",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "shtsh",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "stsch",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "shch",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "scht",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["43"],
                replacement_default: vec!["43"],
            },
            Rule {
                pattern: "schd",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["43"],
                replacement_default: vec!["43"],
            },
            Rule {
                pattern: "stch",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "strz",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "strs",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "stsh",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "szcz",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "szcs",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "sch",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "sht",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["43"],
                replacement_default: vec!["43"],
            },
            Rule {
                pattern: "szt",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["43"],
                replacement_default: vec!["43"],
            },
            Rule {
                pattern: "shd",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["43"],
                replacement_default: vec!["43"],
            },
            Rule {
                pattern: "szd",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["43"],
                replacement_default: vec!["43"],
            },
            Rule {
                pattern: "sh",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "sc",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "st",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["43"],
                replacement_default: vec!["43"],
            },
            Rule {
                pattern: "sd",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["43"],
                replacement_default: vec!["43"],
            },
            Rule {
                pattern: "sz",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "s",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
        ]);
        rules.insert('t', vec![
            Rule {
                pattern: "ttsch",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "ttch",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "tsch",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "ttsz",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "tch",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "trz",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "trs",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "tsh",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "tts",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "ttz",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "tzs",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "tsz",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "th",
                replacement_at_start: vec!["3"],
                replacement_before_vowel: vec!["3"],
                replacement_default: vec!["3"],
            },
            Rule {
                pattern: "ts",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "tc",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "tz",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "t",
                replacement_at_start: vec!["3"],
                replacement_before_vowel: vec!["3"],
                replacement_default: vec!["3"],
            },
        ]);
        rules.insert('u', vec![
            Rule {
                pattern: "ui",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec!["1"],
                replacement_default: vec![""],
            },
            Rule {
                pattern: "uj",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec!["1"],
                replacement_default: vec![""],
            },
            Rule {
                pattern: "uy",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec!["1"],
                replacement_default: vec![""],
            },
            Rule {
                pattern: "ue",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec!["1"],
                replacement_default: vec![""],
            },
            Rule {
                pattern: "u",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec![""],
                replacement_default: vec![""],
            },
        ]);
        rules.insert('v', vec![
            Rule {
                pattern: "v",
                replacement_at_start: vec!["7"],
                replacement_before_vowel: vec!["7"],
                replacement_default: vec!["7"],
            },
        ]);
        rules.insert('w', vec![
            Rule {
                pattern: "w",
                replacement_at_start: vec!["7"],
                replacement_before_vowel: vec!["7"],
                replacement_default: vec!["7"],
            },
        ]);
        rules.insert('x', vec![
            Rule {
                pattern: "x",
                replacement_at_start: vec!["5"],
                replacement_before_vowel: vec!["54"],
                replacement_default: vec!["54"],
            },
        ]);
        rules.insert('y', vec![
            Rule {
                pattern: "y",
                replacement_at_start: vec!["1"],
                replacement_before_vowel: vec![""],
                replacement_default: vec![""],
            },
        ]);
        rules.insert('z', vec![
            Rule {
                pattern: "zhdzh",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "zdzh",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "zsch",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "zdz",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "zhd",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["43"],
                replacement_default: vec!["43"],
            },
            Rule {
                pattern: "zsh",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "zd",
                replacement_at_start: vec!["2"],
                replacement_before_vowel: vec!["43"],
                replacement_default: vec!["43"],
            },
            Rule {
                pattern: "zh",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "zs",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            },
            Rule {
                pattern: "z",
                replacement_at_start: vec!["4"],
                replacement_before_vowel: vec!["4"],
                replacement_default: vec!["4"],
            }, ]);

        let expected = DaitchMokotoffSoundex {
            ascii_folding: true,
            rules,
            ascii_folding_rules,
        };

        let iter1 = result.rules.into_iter().zip(expected.rules.into_iter());
        for ((ch1, rules1), (ch2, rules2)) in iter1 {
            assert_eq!(ch1, ch2, "Rule key differ");
            let iter2 = rules1.into_iter().zip(rules2.into_iter());
            for (rule1, rule2) in iter2 {
                assert_eq!(rule1, rule2, "Rules differ at key {}", ch1);
            }
        }

        assert_eq!(result.ascii_folding_rules, expected.ascii_folding_rules);

        Ok(())
    }

    #[test]
    fn test_custom_rule() -> Result<(), PhoneticError> {
        let rules = "/*
This
is
a
multiline
comment
 */
///
// This is a single line comment.
///
à=a // You can put a one line comment at the end of a rule. This rule is for ASCII folding.
/*
This rule convert the substring `sh` into
 - `0` if at the start of the word
 - an empty string if before a vowel
 - otherwise it does a branching with code `0` and code `1`
 */
\"sh\" \"0\" \"\" \"0|1\"";

        let result = DaitchMokotoffSoundexBuilder::with_rules(rules).build()?;

        let mut ascii_folding_rules: BTreeMap<char, char> = BTreeMap::new();
        ascii_folding_rules.insert('à', 'a');
        let mut rules: BTreeMap<char, Vec<Rule>> = BTreeMap::new();
        rules.insert('s', vec![
            Rule {
                pattern: "sh",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec![""],
                replacement_default: vec!["0", "1"],
            }
        ]);
        let expected = DaitchMokotoffSoundex {
            ascii_folding: true,
            rules,
            ascii_folding_rules,
        };

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_without_ascii_folding() -> Result<(), PhoneticError> {
        let rules = "/*
This
is
a
multiline
comment
 */
///
// This is a single line comment.
///
à=a // You can put a one line comment at the end of a rule. This rule is for ASCII folding.
/*
This rule convert the substring `sh` into
 - `0` if at the start of the word
 - an empty string if before a vowel
 - otherwise it does a branching with code `0` and code `1`
 */
\"sh\" \"0\" \"\" \"0|1\"";

        let result = DaitchMokotoffSoundexBuilder::with_rules(rules).ascii_folding(false).build()?;

        let mut ascii_folding_rules: BTreeMap<char, char> = BTreeMap::new();
        ascii_folding_rules.insert('à', 'a');
        let mut rules: BTreeMap<char, Vec<Rule>> = BTreeMap::new();
        rules.insert('s', vec![
            Rule {
                pattern: "sh",
                replacement_at_start: vec!["0"],
                replacement_before_vowel: vec![""],
                replacement_default: vec!["0", "1"],
            }
        ]);
        let expected = DaitchMokotoffSoundex {
            ascii_folding: false,
            rules,
            ascii_folding_rules,
        };

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_malformed_custom_rule() {
        let result = DaitchMokotoffSoundexBuilder::with_rules("This is wrong.").build();
        assert_eq!(result, Err(PhoneticError::ParseRuleError("Rule doesn't follow format \"pattern\" \"replacement at start\" \"replacement before vowel\" \"default replacement\" or char=char. Got : This is wrong.".to_string())));
    }

    #[test]
    fn test_accented_character_folding() -> Result<(), PhoneticError> {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::default().build()?;

        assert_eq!(daitch_mokotoff.soundex("Straßburg"), "294795");
        assert_eq!(daitch_mokotoff.soundex("Strasburg"), "294795");

        assert_eq!(daitch_mokotoff.soundex("Éregon"), "095600");
        assert_eq!(daitch_mokotoff.soundex("Eregon"), "095600");

        Ok(())
    }

    #[test]
    fn test_adjacent_codes() -> Result<(), PhoneticError> {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::default().build()?;

        // AKSSOL
        // A-KS-S-O-L
        // 0-54-4---8 -> wrong
        // 0-54-----8 -> correct
        assert_eq!(daitch_mokotoff.soundex("AKSSOL"), "054800");

        // GERSCHFELD
        // G-E-RS-CH-F-E-L-D
        // 5--4/94-5/4-7-8-3 -> wrong
        // 5--4/94-5/--7-8-3 -> correct
        assert_eq!(daitch_mokotoff.soundex("GERSCHFELD"), "547830|545783|594783|594578");

        Ok(())
    }

    #[test]
    fn test_encode_basic() -> Result<(), PhoneticError> {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::default().build()?;

        assert_eq!(daitch_mokotoff.encode("AUERBACH"), "097400");
        assert_eq!(daitch_mokotoff.encode("OHRBACH"), "097400");
        assert_eq!(daitch_mokotoff.encode("LIPSHITZ"), "874400");
        assert_eq!(daitch_mokotoff.encode("LIPPSZYC"), "874400");
        assert_eq!(daitch_mokotoff.encode("LEWINSKY"), "876450");
        assert_eq!(daitch_mokotoff.encode("LEVINSKI"), "876450");
        assert_eq!(daitch_mokotoff.encode("SZLAMAWICZ"), "486740");
        assert_eq!(daitch_mokotoff.encode("SHLAMOVITZ"), "486740");

        Ok(())
    }

    #[test]
    fn test_encode_ignore_apostrophes() -> Result<(), PhoneticError> {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::default().build()?;

        for v in vec!["OBrien", "'OBrien", "O'Brien", "OB'rien", "OBr'ien", "OBri'en", "OBrie'n", "OBrien'"].iter() {
            assert_eq!(daitch_mokotoff.encode(v), "079600", "Error for {}", v);
        }

        Ok(())
    }

    #[test]
    fn test_encode_ignore_hyphens() -> Result<(), PhoneticError> {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::default().build()?;

        for v in vec!["KINGSMITH", "-KINGSMITH", "K-INGSMITH", "KI-NGSMITH", "KIN-GSMITH", "KING-SMITH", "KINGS-MITH", "KINGSM-ITH",
                      "KINGSMI-TH", "KINGSMIT-H", "KINGSMITH-"].iter() {
            assert_eq!(daitch_mokotoff.encode(v), "565463", "Error for {}", v);
        }

        Ok(())
    }

    #[test]
    fn test_encode_ignore_trimmable() -> Result<(), PhoneticError> {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::default().build()?;

        assert_eq!(daitch_mokotoff.encode(" \t\n\r Washington \t\n\r "), "746536");
        assert_eq!(daitch_mokotoff.encode("Washington"), "746536");

        Ok(())
    }

    #[test]
    fn test_soundex_basic() -> Result<(), PhoneticError> {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::default().build()?;

        assert_eq!(daitch_mokotoff.soundex("GOLDEN"), "583600");
        assert_eq!(daitch_mokotoff.soundex("Alpert"), "087930");
        assert_eq!(daitch_mokotoff.soundex("Breuer"), "791900");
        assert_eq!(daitch_mokotoff.soundex("Haber"), "579000");
        assert_eq!(daitch_mokotoff.soundex("Mannheim"), "665600");
        assert_eq!(daitch_mokotoff.soundex("Mintz"), "664000");
        assert_eq!(daitch_mokotoff.soundex("Topf"), "370000");
        assert_eq!(daitch_mokotoff.soundex("Kleinmann"), "586660");
        assert_eq!(daitch_mokotoff.soundex("Ben Aron"), "769600");

        assert_eq!(daitch_mokotoff.soundex("AUERBACH"), "097400|097500");
        assert_eq!(daitch_mokotoff.soundex("OHRBACH"), "097400|097500");
        assert_eq!(daitch_mokotoff.soundex("LIPSHITZ"), "874400");
        assert_eq!(daitch_mokotoff.soundex("LIPPSZYC"), "874400|874500");
        assert_eq!(daitch_mokotoff.soundex("LEWINSKY"), "876450");
        assert_eq!(daitch_mokotoff.soundex("LEVINSKI"), "876450");
        assert_eq!(daitch_mokotoff.soundex("SZLAMAWICZ"), "486740");
        assert_eq!(daitch_mokotoff.soundex("SHLAMOVITZ"), "486740");

        Ok(())
    }

    #[test]
    fn test_soundex_basic2() -> Result<(), PhoneticError> {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::default().build()?;

        assert_eq!(daitch_mokotoff.soundex("Ceniow"), "467000|567000");
        assert_eq!(daitch_mokotoff.soundex("Tsenyuv"), "467000");
        assert_eq!(daitch_mokotoff.soundex("Holubica"), "587400|587500");
        assert_eq!(daitch_mokotoff.soundex("Golubitsa"), "587400");
        assert_eq!(daitch_mokotoff.soundex("Przemysl"), "746480|794648");
        assert_eq!(daitch_mokotoff.soundex("Pshemeshil"), "746480");
        assert_eq!(daitch_mokotoff.soundex("Rosochowaciec"), "944744|944745|944754|944755|945744|945745|945754|945755");
        assert_eq!(daitch_mokotoff.soundex("Rosokhovatsets"), "945744");

        Ok(())
    }

    #[test]
    fn test_soundex_basic3() -> Result<(), PhoneticError> {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::default().build()?;

        assert_eq!(daitch_mokotoff.soundex("Peters"), "734000|739400");
        assert_eq!(daitch_mokotoff.soundex("Peterson"), "734600|739460");
        assert_eq!(daitch_mokotoff.soundex("Moskowitz"), "645740");
        assert_eq!(daitch_mokotoff.soundex("Moskovitz"), "645740");
        assert_eq!(daitch_mokotoff.soundex("Jackson"), "154600|145460|454600|445460");
        assert_eq!(daitch_mokotoff.soundex("Jackson-Jackson"), "154654|154645|154644|145465|145464|454654|454645|454644|445465|445464");

        Ok(())
    }

    #[test]
    fn test_special_romanian_characters() -> Result<(), PhoneticError> {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::default().build()?;

        assert_eq!(daitch_mokotoff.soundex("ţamas"), "364000|464000");
        assert_eq!(daitch_mokotoff.soundex("țamas"), "364000|464000");

        Ok(())
    }
}