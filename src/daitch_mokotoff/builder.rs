use std::collections::BTreeMap;

use nom::Parser;

use crate::daitch_mokotoff::{DaitchMokotoffSoundex, Rule};
use crate::{build_parse_error, end_of_line, folding, multiline_comment, quadruplet, ParseError};

/// This is a builder for [DaitchMokotoffSoundex].
#[derive(Clone, Debug)]
pub struct DaitchMokotoffSoundexBuilder<'a> {
    rules: &'a str,
    ascii_folding: bool,
}

/// Create a [DaitchMokotoffSoundexBuilder] with
/// [commons-codec](https://github.com/apache/commons-codec/blob/master/src/main/resources/org/apache/commons/codec/language/dmrules.txt)
/// rules and `ascii_folding` enable.
#[cfg(feature = "embedded_dm")]
impl Default for DaitchMokotoffSoundexBuilder<'_> {
    fn default() -> Self {
        Self {
            rules: crate::daitch_mokotoff::DEFAULT_RULES,
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
    /// This method returns an error in case it can't parse the rules.
    pub fn build(self) -> Result<DaitchMokotoffSoundex, ParseError> {
        let mut rules: BTreeMap<char, Vec<Rule>> = BTreeMap::new();
        let mut ascii_folding_rules: BTreeMap<char, char> = BTreeMap::new();
        let mut remains = self.rules;
        let mut line_number: usize = 0;
        while !remains.is_empty() {
            line_number += 1;

            // Parrsing test from more probable to less probable.

            // Try quadruplet rule
            if let Ok((rm, quadruplet)) = quadruplet().parse(remains) {
                let rule = Rule::from(quadruplet);
                // There's always at least one char, the regex ensures that.
                let ch = rule.pattern.chars().next().unwrap();
                rules.entry(ch).or_default().push(rule);
                remains = rm;
                continue;
            }

            // Try folding rule
            if let Ok((rm, (pattern, replacement))) = folding().parse(remains) {
                ascii_folding_rules.insert(pattern, replacement);
                remains = rm;
                continue;
            }

            // Try single line comment
            if let Ok((rm, _)) = end_of_line().parse(remains) {
                remains = rm;
                continue;
            }

            // Try multiline comment
            if let Ok((rm, ln)) = multiline_comment().parse(remains) {
                line_number += ln;
                remains = rm;
                continue;
            }

            // Everything fails, then return an error...
            return Err(build_parse_error(
                line_number,
                None,
                remains,
                "Can't recognize line".to_string(),
            ));
        }

        // Ordering by pattern length decreasing.
        rules
            .values_mut()
            .for_each(|v| v.sort_by(|a, b| a.pattern.len().cmp(&b.pattern.len()).reverse()));

        Ok(DaitchMokotoffSoundex {
            ascii_folding: self.ascii_folding,
            rules,
            ascii_folding_rules,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMMONS_CODEC_RULES: &str = include_str!("../../rules/dmrules.txt");

    #[test]
    fn test_default_rules() {
        let result = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES)
            .build()
            .unwrap();

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
        rules.insert(
            'ą',
            vec![Rule {
                pattern: "ą".to_string(),
                replacement_at_start: vec!["".to_string()],
                replacement_before_vowel: vec!["".to_string()],
                replacement_default: vec!["".to_string(), "6".to_string()],
            }],
        );
        rules.insert(
            'ę',
            vec![Rule {
                pattern: "ę".to_string(),
                replacement_at_start: vec!["".to_string()],
                replacement_before_vowel: vec!["".to_string()],
                replacement_default: vec!["".to_string(), "6".to_string()],
            }],
        );
        rules.insert(
            'ț',
            vec![Rule {
                pattern: "ț".to_string(),
                replacement_at_start: vec!["3".to_string(), "4".to_string()],
                replacement_before_vowel: vec!["3".to_string(), "4".to_string()],
                replacement_default: vec!["3".to_string(), "4".to_string()],
            }],
        );
        rules.insert(
            'a',
            vec![
                Rule {
                    pattern: "ai".to_string(),
                    replacement_at_start: vec!["0".to_string()],
                    replacement_before_vowel: vec!["1".to_string()],
                    replacement_default: vec!["".to_string()],
                },
                Rule {
                    pattern: "aj".to_string(),
                    replacement_at_start: vec!["0".to_string()],
                    replacement_before_vowel: vec!["1".to_string()],
                    replacement_default: vec!["".to_string()],
                },
                Rule {
                    pattern: "ay".to_string(),
                    replacement_at_start: vec!["0".to_string()],
                    replacement_before_vowel: vec!["1".to_string()],
                    replacement_default: vec!["".to_string()],
                },
                Rule {
                    pattern: "au".to_string(),
                    replacement_at_start: vec!["0".to_string()],
                    replacement_before_vowel: vec!["7".to_string()],
                    replacement_default: vec!["".to_string()],
                },
                Rule {
                    pattern: "a".to_string(),
                    replacement_at_start: vec!["0".to_string()],
                    replacement_before_vowel: vec!["".to_string()],
                    replacement_default: vec!["".to_string()],
                },
            ],
        );
        rules.insert(
            'b',
            vec![Rule {
                pattern: "b".to_string(),
                replacement_at_start: vec!["7".to_string()],
                replacement_before_vowel: vec!["7".to_string()],
                replacement_default: vec!["7".to_string()],
            }],
        );
        rules.insert(
            'c',
            vec![
                Rule {
                    pattern: "chs".to_string(),
                    replacement_at_start: vec!["5".to_string()],
                    replacement_before_vowel: vec!["54".to_string()],
                    replacement_default: vec!["54".to_string()],
                },
                Rule {
                    pattern: "csz".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "czs".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "cz".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "cs".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "ch".to_string(),
                    replacement_at_start: vec!["4".to_string(), "5".to_string()],
                    replacement_before_vowel: vec!["4".to_string(), "5".to_string()],
                    replacement_default: vec!["4".to_string(), "5".to_string()],
                },
                Rule {
                    pattern: "ck".to_string(),
                    replacement_at_start: vec!["5".to_string(), "45".to_string()],
                    replacement_before_vowel: vec!["5".to_string(), "45".to_string()],
                    replacement_default: vec!["5".to_string(), "45".to_string()],
                },
                Rule {
                    pattern: "c".to_string(),
                    replacement_at_start: vec!["4".to_string(), "5".to_string()],
                    replacement_before_vowel: vec!["4".to_string(), "5".to_string()],
                    replacement_default: vec!["4".to_string(), "5".to_string()],
                },
            ],
        );
        rules.insert(
            'ţ',
            vec![Rule {
                pattern: "ţ".to_string(),
                replacement_at_start: vec!["3".to_string(), "4".to_string()],
                replacement_before_vowel: vec!["3".to_string(), "4".to_string()],
                replacement_default: vec!["3".to_string(), "4".to_string()],
            }],
        );
        rules.insert(
            'd',
            vec![
                Rule {
                    pattern: "drz".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "drs".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "dsh".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "dsz".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "dzh".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "dzs".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "ds".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "dz".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "dt".to_string(),
                    replacement_at_start: vec!["3".to_string()],
                    replacement_before_vowel: vec!["3".to_string()],
                    replacement_default: vec!["3".to_string()],
                },
                Rule {
                    pattern: "d".to_string(),
                    replacement_at_start: vec!["3".to_string()],
                    replacement_before_vowel: vec!["3".to_string()],
                    replacement_default: vec!["3".to_string()],
                },
            ],
        );
        rules.insert(
            'e',
            vec![
                Rule {
                    pattern: "ei".to_string(),
                    replacement_at_start: vec!["0".to_string()],
                    replacement_before_vowel: vec!["1".to_string()],
                    replacement_default: vec!["".to_string()],
                },
                Rule {
                    pattern: "ej".to_string(),
                    replacement_at_start: vec!["0".to_string()],
                    replacement_before_vowel: vec!["1".to_string()],
                    replacement_default: vec!["".to_string()],
                },
                Rule {
                    pattern: "ey".to_string(),
                    replacement_at_start: vec!["0".to_string()],
                    replacement_before_vowel: vec!["1".to_string()],
                    replacement_default: vec!["".to_string()],
                },
                Rule {
                    pattern: "eu".to_string(),
                    replacement_at_start: vec!["1".to_string()],
                    replacement_before_vowel: vec!["1".to_string()],
                    replacement_default: vec!["".to_string()],
                },
                Rule {
                    pattern: "e".to_string(),
                    replacement_at_start: vec!["0".to_string()],
                    replacement_before_vowel: vec!["".to_string()],
                    replacement_default: vec!["".to_string()],
                },
            ],
        );
        rules.insert(
            'f',
            vec![
                Rule {
                    pattern: "fb".to_string(),
                    replacement_at_start: vec!["7".to_string()],
                    replacement_before_vowel: vec!["7".to_string()],
                    replacement_default: vec!["7".to_string()],
                },
                Rule {
                    pattern: "f".to_string(),
                    replacement_at_start: vec!["7".to_string()],
                    replacement_before_vowel: vec!["7".to_string()],
                    replacement_default: vec!["7".to_string()],
                },
            ],
        );
        rules.insert(
            'g',
            vec![Rule {
                pattern: "g".to_string(),
                replacement_at_start: vec!["5".to_string()],
                replacement_before_vowel: vec!["5".to_string()],
                replacement_default: vec!["5".to_string()],
            }],
        );
        rules.insert(
            'h',
            vec![Rule {
                pattern: "h".to_string(),
                replacement_at_start: vec!["5".to_string()],
                replacement_before_vowel: vec!["5".to_string()],
                replacement_default: vec!["".to_string()],
            }],
        );
        rules.insert(
            'i',
            vec![
                Rule {
                    pattern: "ia".to_string(),
                    replacement_at_start: vec!["1".to_string()],
                    replacement_before_vowel: vec!["".to_string()],
                    replacement_default: vec!["".to_string()],
                },
                Rule {
                    pattern: "ie".to_string(),
                    replacement_at_start: vec!["1".to_string()],
                    replacement_before_vowel: vec!["".to_string()],
                    replacement_default: vec!["".to_string()],
                },
                Rule {
                    pattern: "io".to_string(),
                    replacement_at_start: vec!["1".to_string()],
                    replacement_before_vowel: vec!["".to_string()],
                    replacement_default: vec!["".to_string()],
                },
                Rule {
                    pattern: "iu".to_string(),
                    replacement_at_start: vec!["1".to_string()],
                    replacement_before_vowel: vec!["".to_string()],
                    replacement_default: vec!["".to_string()],
                },
                Rule {
                    pattern: "i".to_string(),
                    replacement_at_start: vec!["0".to_string()],
                    replacement_before_vowel: vec!["".to_string()],
                    replacement_default: vec!["".to_string()],
                },
            ],
        );
        rules.insert(
            'j',
            vec![Rule {
                pattern: "j".to_string(),
                replacement_at_start: vec!["1".to_string(), "4".to_string()],
                replacement_before_vowel: vec!["".to_string(), "4".to_string()],
                replacement_default: vec!["".to_string(), "4".to_string()],
            }],
        );
        rules.insert(
            'k',
            vec![
                Rule {
                    pattern: "ks".to_string(),
                    replacement_at_start: vec!["5".to_string()],
                    replacement_before_vowel: vec!["54".to_string()],
                    replacement_default: vec!["54".to_string()],
                },
                Rule {
                    pattern: "kh".to_string(),
                    replacement_at_start: vec!["5".to_string()],
                    replacement_before_vowel: vec!["5".to_string()],
                    replacement_default: vec!["5".to_string()],
                },
                Rule {
                    pattern: "k".to_string(),
                    replacement_at_start: vec!["5".to_string()],
                    replacement_before_vowel: vec!["5".to_string()],
                    replacement_default: vec!["5".to_string()],
                },
            ],
        );
        rules.insert(
            'l',
            vec![Rule {
                pattern: "l".to_string(),
                replacement_at_start: vec!["8".to_string()],
                replacement_before_vowel: vec!["8".to_string()],
                replacement_default: vec!["8".to_string()],
            }],
        );
        rules.insert(
            'm',
            vec![
                Rule {
                    pattern: "mn".to_string(),
                    replacement_at_start: vec!["66".to_string()],
                    replacement_before_vowel: vec!["66".to_string()],
                    replacement_default: vec!["66".to_string()],
                },
                Rule {
                    pattern: "m".to_string(),
                    replacement_at_start: vec!["6".to_string()],
                    replacement_before_vowel: vec!["6".to_string()],
                    replacement_default: vec!["6".to_string()],
                },
            ],
        );
        rules.insert(
            'n',
            vec![
                Rule {
                    pattern: "nm".to_string(),
                    replacement_at_start: vec!["66".to_string()],
                    replacement_before_vowel: vec!["66".to_string()],
                    replacement_default: vec!["66".to_string()],
                },
                Rule {
                    pattern: "n".to_string(),
                    replacement_at_start: vec!["6".to_string()],
                    replacement_before_vowel: vec!["6".to_string()],
                    replacement_default: vec!["6".to_string()],
                },
            ],
        );
        rules.insert(
            'o',
            vec![
                Rule {
                    pattern: "oi".to_string(),
                    replacement_at_start: vec!["0".to_string()],
                    replacement_before_vowel: vec!["1".to_string()],
                    replacement_default: vec!["".to_string()],
                },
                Rule {
                    pattern: "oj".to_string(),
                    replacement_at_start: vec!["0".to_string()],
                    replacement_before_vowel: vec!["1".to_string()],
                    replacement_default: vec!["".to_string()],
                },
                Rule {
                    pattern: "oy".to_string(),
                    replacement_at_start: vec!["0".to_string()],
                    replacement_before_vowel: vec!["1".to_string()],
                    replacement_default: vec!["".to_string()],
                },
                Rule {
                    pattern: "o".to_string(),
                    replacement_at_start: vec!["0".to_string()],
                    replacement_before_vowel: vec!["".to_string()],
                    replacement_default: vec!["".to_string()],
                },
            ],
        );
        rules.insert(
            'p',
            vec![
                Rule {
                    pattern: "pf".to_string(),
                    replacement_at_start: vec!["7".to_string()],
                    replacement_before_vowel: vec!["7".to_string()],
                    replacement_default: vec!["7".to_string()],
                },
                Rule {
                    pattern: "ph".to_string(),
                    replacement_at_start: vec!["7".to_string()],
                    replacement_before_vowel: vec!["7".to_string()],
                    replacement_default: vec!["7".to_string()],
                },
                Rule {
                    pattern: "p".to_string(),
                    replacement_at_start: vec!["7".to_string()],
                    replacement_before_vowel: vec!["7".to_string()],
                    replacement_default: vec!["7".to_string()],
                },
            ],
        );
        rules.insert(
            'q',
            vec![Rule {
                pattern: "q".to_string(),
                replacement_at_start: vec!["5".to_string()],
                replacement_before_vowel: vec!["5".to_string()],
                replacement_default: vec!["5".to_string()],
            }],
        );
        rules.insert(
            'r',
            vec![
                Rule {
                    pattern: "rs".to_string(),
                    replacement_at_start: vec!["4".to_string(), "94".to_string()],
                    replacement_before_vowel: vec!["4".to_string(), "94".to_string()],
                    replacement_default: vec!["4".to_string(), "94".to_string()],
                },
                Rule {
                    pattern: "rz".to_string(),
                    replacement_at_start: vec!["4".to_string(), "94".to_string()],
                    replacement_before_vowel: vec!["4".to_string(), "94".to_string()],
                    replacement_default: vec!["4".to_string(), "94".to_string()],
                },
                Rule {
                    pattern: "r".to_string(),
                    replacement_at_start: vec!["9".to_string()],
                    replacement_before_vowel: vec!["9".to_string()],
                    replacement_default: vec!["9".to_string()],
                },
            ],
        );
        rules.insert(
            's',
            vec![
                Rule {
                    pattern: "schtsch".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "schtsh".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "schtch".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "shtch".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "shtsh".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "stsch".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "shch".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "scht".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["43".to_string()],
                    replacement_default: vec!["43".to_string()],
                },
                Rule {
                    pattern: "schd".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["43".to_string()],
                    replacement_default: vec!["43".to_string()],
                },
                Rule {
                    pattern: "stch".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "strz".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "strs".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "stsh".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "szcz".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "szcs".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "sch".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "sht".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["43".to_string()],
                    replacement_default: vec!["43".to_string()],
                },
                Rule {
                    pattern: "szt".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["43".to_string()],
                    replacement_default: vec!["43".to_string()],
                },
                Rule {
                    pattern: "shd".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["43".to_string()],
                    replacement_default: vec!["43".to_string()],
                },
                Rule {
                    pattern: "szd".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["43".to_string()],
                    replacement_default: vec!["43".to_string()],
                },
                Rule {
                    pattern: "sh".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "sc".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "st".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["43".to_string()],
                    replacement_default: vec!["43".to_string()],
                },
                Rule {
                    pattern: "sd".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["43".to_string()],
                    replacement_default: vec!["43".to_string()],
                },
                Rule {
                    pattern: "sz".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "s".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
            ],
        );
        rules.insert(
            't',
            vec![
                Rule {
                    pattern: "ttsch".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "ttch".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "tsch".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "ttsz".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "tch".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "trz".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "trs".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "tsh".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "tts".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "ttz".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "tzs".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "tsz".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "th".to_string(),
                    replacement_at_start: vec!["3".to_string()],
                    replacement_before_vowel: vec!["3".to_string()],
                    replacement_default: vec!["3".to_string()],
                },
                Rule {
                    pattern: "ts".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "tc".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "tz".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "t".to_string(),
                    replacement_at_start: vec!["3".to_string()],
                    replacement_before_vowel: vec!["3".to_string()],
                    replacement_default: vec!["3".to_string()],
                },
            ],
        );
        rules.insert(
            'u',
            vec![
                Rule {
                    pattern: "ui".to_string(),
                    replacement_at_start: vec!["0".to_string()],
                    replacement_before_vowel: vec!["1".to_string()],
                    replacement_default: vec!["".to_string()],
                },
                Rule {
                    pattern: "uj".to_string(),
                    replacement_at_start: vec!["0".to_string()],
                    replacement_before_vowel: vec!["1".to_string()],
                    replacement_default: vec!["".to_string()],
                },
                Rule {
                    pattern: "uy".to_string(),
                    replacement_at_start: vec!["0".to_string()],
                    replacement_before_vowel: vec!["1".to_string()],
                    replacement_default: vec!["".to_string()],
                },
                Rule {
                    pattern: "ue".to_string(),
                    replacement_at_start: vec!["0".to_string()],
                    replacement_before_vowel: vec!["1".to_string()],
                    replacement_default: vec!["".to_string()],
                },
                Rule {
                    pattern: "u".to_string(),
                    replacement_at_start: vec!["0".to_string()],
                    replacement_before_vowel: vec!["".to_string()],
                    replacement_default: vec!["".to_string()],
                },
            ],
        );
        rules.insert(
            'v',
            vec![Rule {
                pattern: "v".to_string(),
                replacement_at_start: vec!["7".to_string()],
                replacement_before_vowel: vec!["7".to_string()],
                replacement_default: vec!["7".to_string()],
            }],
        );
        rules.insert(
            'w',
            vec![Rule {
                pattern: "w".to_string(),
                replacement_at_start: vec!["7".to_string()],
                replacement_before_vowel: vec!["7".to_string()],
                replacement_default: vec!["7".to_string()],
            }],
        );
        rules.insert(
            'x',
            vec![Rule {
                pattern: "x".to_string(),
                replacement_at_start: vec!["5".to_string()],
                replacement_before_vowel: vec!["54".to_string()],
                replacement_default: vec!["54".to_string()],
            }],
        );
        rules.insert(
            'y',
            vec![Rule {
                pattern: "y".to_string(),
                replacement_at_start: vec!["1".to_string()],
                replacement_before_vowel: vec!["".to_string()],
                replacement_default: vec!["".to_string()],
            }],
        );
        rules.insert(
            'z',
            vec![
                Rule {
                    pattern: "zhdzh".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "zdzh".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "zsch".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "zdz".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "zhd".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["43".to_string()],
                    replacement_default: vec!["43".to_string()],
                },
                Rule {
                    pattern: "zsh".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "zd".to_string(),
                    replacement_at_start: vec!["2".to_string()],
                    replacement_before_vowel: vec!["43".to_string()],
                    replacement_default: vec!["43".to_string()],
                },
                Rule {
                    pattern: "zh".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "zs".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
                Rule {
                    pattern: "z".to_string(),
                    replacement_at_start: vec!["4".to_string()],
                    replacement_before_vowel: vec!["4".to_string()],
                    replacement_default: vec!["4".to_string()],
                },
            ],
        );

        let expected = DaitchMokotoffSoundex {
            ascii_folding: true,
            rules,
            ascii_folding_rules,
        };

        let iter1 = result.rules.into_iter().zip(expected.rules);
        for ((ch1, rules1), (ch2, rules2)) in iter1 {
            assert_eq!(ch1, ch2, "Rule key differ");
            let iter2 = rules1.into_iter().zip(rules2);
            for (rule1, rule2) in iter2 {
                assert_eq!(rule1, rule2, "Rules differ at key {ch1}");
            }
        }

        assert_eq!(result.ascii_folding_rules, expected.ascii_folding_rules);
    }

    #[test]
    fn test_custom_rule() {
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

        let result = DaitchMokotoffSoundexBuilder::with_rules(rules).build();

        let mut ascii_folding_rules: BTreeMap<char, char> = BTreeMap::new();
        ascii_folding_rules.insert('à', 'a');
        let mut rules: BTreeMap<char, Vec<Rule>> = BTreeMap::new();
        rules.insert(
            's',
            vec![Rule {
                pattern: "sh".to_string(),
                replacement_at_start: vec!["0".to_string()],
                replacement_before_vowel: vec!["".to_string()],
                replacement_default: vec!["0".to_string(), "1".to_string()],
            }],
        );
        let expected = DaitchMokotoffSoundex {
            ascii_folding: true,
            rules,
            ascii_folding_rules,
        };

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_without_ascii_folding() {
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

        let result = DaitchMokotoffSoundexBuilder::with_rules(rules)
            .ascii_folding(false)
            .build();

        let mut ascii_folding_rules: BTreeMap<char, char> = BTreeMap::new();
        ascii_folding_rules.insert('à', 'a');
        let mut rules: BTreeMap<char, Vec<Rule>> = BTreeMap::new();
        rules.insert(
            's',
            vec![Rule {
                pattern: "sh".to_string(),
                replacement_at_start: vec!["0".to_string()],
                replacement_before_vowel: vec!["".to_string()],
                replacement_default: vec!["0".to_string(), "1".to_string()],
            }],
        );
        let expected = DaitchMokotoffSoundex {
            ascii_folding: false,
            rules,
            ascii_folding_rules,
        };

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_malformed_custom_rule() {
        let result = DaitchMokotoffSoundexBuilder::with_rules("This is wrong.").build();
        assert_eq!(
            result,
            Err(ParseError {
                line_number: 1,
                filename: None,
                line_content: "This is wrong.".to_string(),
                description: "Can't recognize line".to_string(),
            })
        );
    }
}
