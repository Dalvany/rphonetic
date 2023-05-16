/*
 * Licensed to the Apache Software Foundation (ASF) under one or more
 * contributor license agreements.  See the NOTICE file distributed with
 * this work for additional information regarding copyright ownership.
 * The ASF licenses this file to You under the Apache License, Version 2.0
 * (the "License"); you may not use this file except in compliance with
 * the License.  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use std::collections::BTreeMap;

use crate::helper::is_vowel;
use crate::{
    build_error, end_of_line, folding, multiline_comment, quadruplet, Encoder, PhoneticError,
};

#[cfg(feature = "embedded_dm")]
const DEFAULT_RULES: &str = include_str!("../rules/dmrules.txt");

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
    /// Finish matching [MAX_LENGTH] by appending `0`.
    fn finish(&mut self) {
        while self.builder.len() < MAX_LENGTH {
            self.builder.push('0');
        }
    }

    fn process_next_replacement(&mut self, replacement: &'a str, append_force: bool) {
        let append = self
            .last_replacement
            .map_or(true, |v| !v.ends_with(replacement))
            || append_force;

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
struct Rule {
    pattern: String,
    replacement_at_start: Vec<String>,
    replacement_before_vowel: Vec<String>,
    replacement_default: Vec<String>,
}

impl Rule {
    fn parse_branch(part: &str) -> Vec<String> {
        part.split('|').map(|v| v.to_string()).collect()
    }

    fn get_pattern_length(&self) -> usize {
        self.pattern.len()
    }

    fn matches(&self, context: &str) -> bool {
        context.starts_with(&self.pattern)
    }

    fn get_replacements(&self, context: &str, at_start: bool) -> &Vec<String> {
        if at_start {
            return &self.replacement_at_start;
        }

        let next_index = self.get_pattern_length();
        let next_char_is_vowel =
            next_index < context.len() && is_vowel(context.chars().nth(next_index), false);
        if next_char_is_vowel {
            return &self.replacement_before_vowel;
        }

        &self.replacement_default
    }
}

impl TryFrom<(&str, &str, &str, &str)> for Rule {
    type Error = PhoneticError;

    fn try_from(
        (part1, part2, part3, part4): (&str, &str, &str, &str),
    ) -> Result<Self, Self::Error> {
        let pattern = part1.to_string();
        let replacement_at_start: Vec<String> = Rule::parse_branch(part2);
        let replacement_before_vowel: Vec<String> = Rule::parse_branch(part3);
        let replacement_default: Vec<String> = Rule::parse_branch(part4);
        Ok(Self {
            pattern,
            replacement_at_start,
            replacement_before_vowel,
            replacement_default,
        })
    }
}

/// This the [Daitch Mokotoff soundex](https://en.wikipedia.org/wiki/Daitch%E2%80%93Mokotoff_Soundex) implementation.
///
/// When `embedded_dm` feature is enabled, then there is a [Default] implementation
/// that uses [commons-codec rules](https://github.com/apache/commons-codec/blob/master/src/main/resources/org/apache/commons/codec/language/dmrules.txt).
///
/// It can be constructed with custom rules using [TryFrom].
///
/// A rule is either in the form of :
/// * `char`=`char` (a char is converted into another char, this is used for ASCII folding)
/// * "`pattern`" "`replacement_at_start`" "`replacement_before_vowel`" "`default_replacement`"
///     * `pattern` : a string to match
///     * `replacement_at_start` : the code to replace `pattern` with if `pattern` is at the start of the word.
///     * `Replacement_before_vowel`: the code to replace `pattern` with if `pattern` is before a vowel inside the word.
///     * `default_replacement`: the code to replace `pattern` with for other cases.
/// To support branching, any pattern can be in the form of `code|code|...`.
///
/// Rules are separated by `\n`.
///
/// Parse supports single line comment using `//` and multiline comments using `/* ... */`.
/// Note that multiline comment must start at the beginning of a line.
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
/// À=a // You can put a one line comment at the end of a rule.
/// This rule is for ASCII folding.
/// /*
/// This rule converts the substring `sh` into
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
/// This rule converts the substring `sh` into
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
/// The algorithm, first, removes all spaces and, if enables, apply ASCII folding
/// with provided rules.
///
/// # Encoding
///
/// There are 2 methods to encode a string:
/// * [DaitchMokotoffSoundex](#encode) that encode without branching.
/// Only one code is returned
/// * [DaitchMokotoffSoundex](#soundex) that encode with branching.
/// Multiple codes, separated by a `|` are returned.
///
/// There is a [helper function](DaitchMokotoffSoundex#method.inner_soundex) that returns code(s) in the form
/// of a vec, avoiding parsing the output.
///
/// # Exemples
///
/// ## Encode methode
///
/// ```rust
/// # fn main() -> Result<(), rphonetic::PhoneticError> {
/// use rphonetic::{DaitchMokotoffSoundex, DaitchMokotoffSoundexBuilder, Encoder};
///
/// const COMMONS_CODEC_RULES: &str = include_str!("../rules/dmrules.txt");
///
/// let encoder = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;
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
/// const COMMONS_CODEC_RULES: &str = include_str!("../rules/dmrules.txt");
///
/// let encoder = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;
///
/// assert_eq!(encoder.soundex("Rosochowaciec"), "944744|944745|944754|944755|945744|945745|945754|945755");
/// #   Ok(())
/// # }
/// ```
///
/// A [Default] implementation with default rules is provided when feature `embedded_dm` is enabled.
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct DaitchMokotoffSoundex {
    ascii_folding: bool,
    rules: BTreeMap<char, Vec<Rule>>,
    ascii_folding_rules: BTreeMap<char, char>,
}

#[cfg(feature = "embedded_dm")]
impl Default for DaitchMokotoffSoundex {
    fn default() -> Self {
        DaitchMokotoffSoundexBuilder::default().build().unwrap()
    }
}

impl DaitchMokotoffSoundex {
    /// Encode the string with branching.
    /// Multiple codes might be generated, separated by a pipe.
    ///
    /// # Example :
    ///
    /// ```rust
    /// # fn main() -> Result<(), rphonetic::PhoneticError> {
    /// use rphonetic::{DaitchMokotoffSoundex, DaitchMokotoffSoundexBuilder, Encoder};
    ///
    /// const COMMONS_CODEC_RULES: &str = include_str!("../rules/dmrules.txt");
    ///
    /// let encoder = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;
    ///
    /// // With branching
    /// assert_eq!(encoder.soundex("Rosochowaciec"), "944744|944745|944754|944755|945744|945745|945754|945755");
    /// #   Ok(())
    /// # }
    /// ```
    pub fn soundex(&self, value: &str) -> String {
        self.inner_soundex(value, true).join("|")
    }

    /// Encode a string and return vector of codes avoiding a parsing result
    ///
    /// # Parameters :
    ///
    /// * `value` : value to encode
    /// * `branching`: if `true` branching will be enabled and multiple code can
    /// be generated, otherwise the result will contain only one code.
    ///
    /// # Result :
    ///
    /// A list of code.
    /// If branching is disabled, a result will contain only one code;
    /// otherwise it might contain multiple codes.
    ///
    /// # Example :
    ///
    /// ```rust
    /// # fn main() -> Result<(), rphonetic::PhoneticError> {
    /// use rphonetic::{DaitchMokotoffSoundex, DaitchMokotoffSoundexBuilder, Encoder};
    ///
    /// const COMMONS_CODEC_RULES: &str = include_str!("../rules/dmrules.txt");
    ///
    /// let encoder = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;
    ///
    /// // With branching
    /// assert_eq!(encoder.inner_soundex("Rosochowaciec", true), vec!["944744","944745","944754","944755","945744","945745","945754","945755"]);
    ///
    /// // Without branching
    /// assert_eq!(encoder.inner_soundex("Rosochowaciec", false), vec!["944744"]);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn inner_soundex(&self, value: &str, branching: bool) -> Vec<String> {
        let source = value
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .map(|ch| {
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
            })
            .collect::<String>();

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
                                let force = (last_char == 'm' && ch == 'n')
                                    || (last_char == 'n' && ch == 'm');
                                next_branch.process_next_replacement(next_replacement, force);
                                // Perhaps use the crate "linked-hash-map" but its major version is 0, and I want to release a major version
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
                        // So we need to "skip" if length >= 2, and we need to substract 2.
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
}

impl Encoder for DaitchMokotoffSoundex {
    /// Encode a string without branching, only one code will be generated
    ///
    /// # Example :
    ///
    /// ```rust
    /// # fn main() -> Result<(), rphonetic::PhoneticError> {
    /// use rphonetic::{DaitchMokotoffSoundex, DaitchMokotoffSoundexBuilder, Encoder};
    ///
    /// const COMMONS_CODEC_RULES: &str = include_str!("../rules/dmrules.txt");
    ///
    /// let encoder = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;
    ///
    ///
    /// // Without branching
    /// assert_eq!(encoder.encode("Rosochowaciec"), "944744");
    /// #   Ok(())
    /// # }
    /// ```
    fn encode(&self, s: &str) -> String {
        self.inner_soundex(s, false)
            .get(0)
            .map(|v| v.to_string())
            .unwrap_or_else(|| "".to_string())
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
#[cfg(feature = "embedded_dm")]
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
    /// This method returns an error in case it can't parse the rules.
    pub fn build(self) -> Result<DaitchMokotoffSoundex, PhoneticError> {
        let mut rules: BTreeMap<char, Vec<Rule>> = BTreeMap::new();
        let mut ascii_folding_rules: BTreeMap<char, char> = BTreeMap::new();
        let mut remains = self.rules;
        let mut line_number: usize = 0;
        while !remains.is_empty() {
            line_number += 1;

            // Parrsing test from more probable to less probable.

            // Try quadruplet rule
            if let Ok((rm, quadruplet)) = quadruplet()(remains) {
                let rule = Rule::try_from(quadruplet)?;
                // There's always at least one char, the regex ensures that.
                let ch = rule.pattern.chars().next().unwrap();
                rules.entry(ch).or_insert_with(Vec::new).push(rule);
                remains = rm;
                continue;
            }

            // Try folding rule
            if let Ok((rm, (pattern, replacement))) = folding()(remains) {
                ascii_folding_rules.insert(pattern, replacement);
                remains = rm;
                continue;
            }

            // Try single line comment
            if let Ok((rm, _)) = end_of_line()(remains) {
                remains = rm;
                continue;
            }

            // Try multiline comment
            if let Ok((rm, ln)) = multiline_comment()(remains) {
                line_number += ln;
                remains = rm;
                continue;
            }

            // Everything fails, then return an error...
            return Err(build_error(
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
    use crate::ParseError;

    const COMMONS_CODEC_RULES: &str = include_str!("../rules/dmrules.txt");

    #[test]
    fn test_default_rules() -> Result<(), PhoneticError> {
        let result = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;

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

        let iter1 = result.rules.into_iter().zip(expected.rules.into_iter());
        for ((ch1, rules1), (ch2, rules2)) in iter1 {
            assert_eq!(ch1, ch2, "Rule key differ");
            let iter2 = rules1.into_iter().zip(rules2.into_iter());
            for (rule1, rule2) in iter2 {
                assert_eq!(rule1, rule2, "Rules differ at key {ch1}");
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

        let result = DaitchMokotoffSoundexBuilder::with_rules(rules)
            .ascii_folding(false)
            .build()?;

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

        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_malformed_custom_rule() {
        let result = DaitchMokotoffSoundexBuilder::with_rules("This is wrong.").build();
        assert_eq!(
            result,
            Err(PhoneticError::ParseRuleError(ParseError {
                line_number: 1,
                filename: None,
                line_content: "This is wrong.".to_string(),
                description: "Can't recognize line".to_string(),
            }))
        );
    }

    #[test]
    fn test_accented_character_folding() -> Result<(), PhoneticError> {
        let daitch_mokotoff =
            DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;

        assert_eq!(daitch_mokotoff.soundex("Straßburg"), "294795");
        assert_eq!(daitch_mokotoff.soundex("Strasburg"), "294795");

        assert_eq!(daitch_mokotoff.soundex("Éregon"), "095600");
        assert_eq!(daitch_mokotoff.soundex("Eregon"), "095600");

        Ok(())
    }

    #[test]
    fn test_adjacent_codes() -> Result<(), PhoneticError> {
        let daitch_mokotoff =
            DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;

        // AKSSOL
        // A-KS-S-O-L
        // 0-54-4---8 -> wrong
        // 0-54-----8 -> correct
        assert_eq!(daitch_mokotoff.soundex("AKSSOL"), "054800");

        // GERSCHFELD
        // G-E-RS-CH-F-E-L-D
        // 5--4/94-5/4-7-8-3 -> wrong
        // 5--4/94-5/--7-8-3 -> correct
        assert_eq!(
            daitch_mokotoff.soundex("GERSCHFELD"),
            "547830|545783|594783|594578"
        );

        Ok(())
    }

    #[test]
    fn test_encode_basic() -> Result<(), PhoneticError> {
        let daitch_mokotoff =
            DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;

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
        let daitch_mokotoff =
            DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;

        for v in vec![
            "OBrien", "'OBrien", "O'Brien", "OB'rien", "OBr'ien", "OBri'en", "OBrie'n", "OBrien'",
        ]
        .iter()
        {
            assert_eq!(daitch_mokotoff.encode(v), "079600", "Error for {v}");
        }

        Ok(())
    }

    #[test]
    fn test_encode_ignore_hyphens() -> Result<(), PhoneticError> {
        let daitch_mokotoff =
            DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;

        for v in vec![
            "KINGSMITH",
            "-KINGSMITH",
            "K-INGSMITH",
            "KI-NGSMITH",
            "KIN-GSMITH",
            "KING-SMITH",
            "KINGS-MITH",
            "KINGSM-ITH",
            "KINGSMI-TH",
            "KINGSMIT-H",
            "KINGSMITH-",
        ]
        .iter()
        {
            assert_eq!(daitch_mokotoff.encode(v), "565463", "Error for {v}");
        }

        Ok(())
    }

    #[test]
    fn test_encode_ignore_trimmable() -> Result<(), PhoneticError> {
        let daitch_mokotoff =
            DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;

        assert_eq!(
            daitch_mokotoff.encode(" \t\n\r Washington \t\n\r "),
            "746536"
        );
        assert_eq!(daitch_mokotoff.encode("Washington"), "746536");

        Ok(())
    }

    #[test]
    fn test_soundex_basic() -> Result<(), PhoneticError> {
        let daitch_mokotoff =
            DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;

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
        let daitch_mokotoff =
            DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;

        assert_eq!(daitch_mokotoff.soundex("Ceniow"), "467000|567000");
        assert_eq!(daitch_mokotoff.soundex("Tsenyuv"), "467000");
        assert_eq!(daitch_mokotoff.soundex("Holubica"), "587400|587500");
        assert_eq!(daitch_mokotoff.soundex("Golubitsa"), "587400");
        assert_eq!(daitch_mokotoff.soundex("Przemysl"), "746480|794648");
        assert_eq!(daitch_mokotoff.soundex("Pshemeshil"), "746480");
        assert_eq!(
            daitch_mokotoff.soundex("Rosochowaciec"),
            "944744|944745|944754|944755|945744|945745|945754|945755"
        );
        assert_eq!(daitch_mokotoff.soundex("Rosokhovatsets"), "945744");

        Ok(())
    }

    #[test]
    fn test_soundex_basic3() -> Result<(), PhoneticError> {
        let daitch_mokotoff =
            DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;

        assert_eq!(daitch_mokotoff.soundex("Peters"), "734000|739400");
        assert_eq!(daitch_mokotoff.soundex("Peterson"), "734600|739460");
        assert_eq!(daitch_mokotoff.soundex("Moskowitz"), "645740");
        assert_eq!(daitch_mokotoff.soundex("Moskovitz"), "645740");
        assert_eq!(
            daitch_mokotoff.soundex("Jackson"),
            "154600|145460|454600|445460"
        );
        assert_eq!(
            daitch_mokotoff.soundex("Jackson-Jackson"),
            "154654|154645|154644|145465|145464|454654|454645|454644|445465|445464"
        );

        Ok(())
    }

    #[test]
    fn test_special_romanian_characters() -> Result<(), PhoneticError> {
        let daitch_mokotoff =
            DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;

        assert_eq!(daitch_mokotoff.soundex("ţamas"), "364000|464000");
        assert_eq!(daitch_mokotoff.soundex("țamas"), "364000|464000");

        Ok(())
    }

    #[test]
    #[cfg(feature = "embedded_dm")]
    fn test_embedded_dm() -> Result<(), PhoneticError> {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::default().build()?;

        assert_eq!(daitch_mokotoff.soundex("ţamas"), "364000|464000");
        assert_eq!(daitch_mokotoff.soundex("țamas"), "364000|464000");

        Ok(())
    }

    #[test]
    #[cfg(feature = "embedded_dm")]
    fn test_default_daitch_mokotoff() {
        let daitch_mokotoff = DaitchMokotoffSoundex::default();

        assert_eq!(daitch_mokotoff.soundex("ţamas"), "364000|464000");
        assert_eq!(daitch_mokotoff.soundex("țamas"), "364000|464000");
    }
}
