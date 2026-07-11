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
use std::convert::Infallible;

pub use crate::daitch_mokotoff::builder::*;
use crate::helper::is_vowel;
use crate::Encoder;

mod builder;
mod parser;

#[cfg(feature = "embedded_dm")]
const DEFAULT_RULES: &str = include_str!("../../rules/dmrules.txt");

/// Max length of a DM soundex value.
const MAX_LENGTH: usize = 6;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct Branch<'a> {
    builder: String,
    last_replacement: Option<&'a str>,
}

impl Default for Branch<'_> {
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
///
///   To support branching, any pattern can be in the form of `code|code|...`.
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
/// # fn main() -> anyhow::Result<()> {
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
///   Only one code is returned
/// * [DaitchMokotoffSoundex](#soundex) that encode with branching.
///   Multiple codes, separated by a `|` are returned.
///
/// There is a [helper function](DaitchMokotoffSoundex#method.inner_soundex) that returns code(s) in the form
/// of a vec, avoiding parsing the output.
///
/// # Exemples
///
/// ## Encode methode
///
/// ```rust
/// # fn main() -> anyhow::Result<()> {
/// use rphonetic::{DaitchMokotoffSoundex, DaitchMokotoffSoundexBuilder, Encoder};
///
/// const COMMONS_CODEC_RULES: &str = include_str!("../../rules/dmrules.txt");
///
/// let encoder = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;
///
/// assert_eq!(encoder.encode("Rosochowaciec")?, "944744");
/// #   Ok(())
/// # }
/// ```
///
/// ## Soundex
///
/// ```rust
/// # fn main() -> anyhow::Result<()> {
/// use rphonetic::{DaitchMokotoffSoundex, DaitchMokotoffSoundexBuilder, Encoder};
///
/// const COMMONS_CODEC_RULES: &str = include_str!("../../rules/dmrules.txt");
///
/// let encoder = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;
///
/// assert_eq!(encoder.soundex("Rosochowaciec")?, "944744|944745|944754|944755|945744|945745|945754|945755");
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
    /// # fn main() -> anyhow::Result<()> {
    /// use rphonetic::{DaitchMokotoffSoundex, DaitchMokotoffSoundexBuilder, Encoder};
    ///
    /// const COMMONS_CODEC_RULES: &str = include_str!("../../rules/dmrules.txt");
    ///
    /// let encoder = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;
    ///
    /// // With branching
    /// assert_eq!(encoder.soundex("Rosochowaciec")?, "944744|944745|944754|944755|945744|945745|945754|945755");
    /// #   Ok(())
    /// # }
    /// ```
    pub fn soundex(&self, value: &str) -> Result<String, Infallible> {
        self.inner_soundex(value, true).map(|v| v.join("|"))
    }

    /// Encode a string and return vector of codes avoiding a parsing result
    ///
    /// # Parameters :
    ///
    /// * `value` : value to encode
    /// * `branching`: if `true` branching will be enabled and multiple code can
    ///   be generated, otherwise the result will contain only one code.
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
    /// # fn main() -> anyhow::Result<()> {
    /// use rphonetic::{DaitchMokotoffSoundex, DaitchMokotoffSoundexBuilder, Encoder};
    ///
    /// const COMMONS_CODEC_RULES: &str = include_str!("../../rules/dmrules.txt");
    ///
    /// let encoder = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;
    ///
    /// // With branching
    /// assert_eq!(encoder.inner_soundex("Rosochowaciec", true)?, vec!["944744","944745","944754","944755","945744","945745","945754","945755"]);
    ///
    /// // Without branching
    /// assert_eq!(encoder.inner_soundex("Rosochowaciec", false)?, vec!["944744"]);
    /// #   Ok(())
    /// # }
    /// ```
    pub fn inner_soundex(&self, value: &str, branching: bool) -> Result<Vec<String>, Infallible> {
        let source = value
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .map(|ch| {
                let lower = ch.to_lowercase().next();
                match lower {
                    None => ch,
                    Some(mut lower) => {
                        if self.ascii_folding {
                            if let Some(elem) = self.ascii_folding_rules.get(&lower) {
                                lower = *elem
                            }
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

        Ok(result)
    }
}

impl Encoder for DaitchMokotoffSoundex {
    type Error = Infallible;

    /// Encode a string without branching, only one code will be generated
    ///
    /// # Example :
    ///
    /// ```rust
    /// # fn main() -> anyhow::Result<()> {
    /// use rphonetic::{DaitchMokotoffSoundex, DaitchMokotoffSoundexBuilder, Encoder};
    ///
    /// const COMMONS_CODEC_RULES: &str = include_str!("../../rules/dmrules.txt");
    ///
    /// let encoder = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES).build()?;
    ///
    ///
    /// // Without branching
    /// assert_eq!(encoder.encode("Rosochowaciec")?, "944744");
    /// #   Ok(())
    /// # }
    /// ```
    fn encode(&self, s: &str) -> Result<String, Infallible> {
        let result = self
            .inner_soundex(s, false)?
            .first()
            .map(|v| v.to_string())
            .unwrap_or_default();

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMMONS_CODEC_RULES: &str = include_str!("../../rules/dmrules.txt");

    #[test]
    fn test_accented_character_folding() {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES)
            .build()
            .unwrap();

        assert_eq!(
            daitch_mokotoff.soundex("Straßburg"),
            Ok("294795".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("Strasburg"),
            Ok("294795".to_string())
        );

        assert_eq!(daitch_mokotoff.soundex("Éregon"), Ok("095600".to_string()));
        assert_eq!(daitch_mokotoff.soundex("Eregon"), Ok("095600".to_string()));
    }

    #[test]
    fn test_adjacent_codes() {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES)
            .build()
            .unwrap();

        // AKSSOL
        // A-KS-S-O-L
        // 0-54-4---8 -> wrong
        // 0-54-----8 -> correct
        assert_eq!(daitch_mokotoff.soundex("AKSSOL"), Ok("054800".to_string()));

        // GERSCHFELD
        // G-E-RS-CH-F-E-L-D
        // 5--4/94-5/4-7-8-3 -> wrong
        // 5--4/94-5/--7-8-3 -> correct
        assert_eq!(
            daitch_mokotoff.soundex("GERSCHFELD"),
            Ok("547830|545783|594783|594578".to_string())
        );
    }

    #[test]
    fn test_encode_basic() {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES)
            .build()
            .unwrap();

        assert_eq!(daitch_mokotoff.encode("AUERBACH"), Ok("097400".to_string()));
        assert_eq!(daitch_mokotoff.encode("OHRBACH"), Ok("097400".to_string()));
        assert_eq!(daitch_mokotoff.encode("LIPSHITZ"), Ok("874400".to_string()));
        assert_eq!(daitch_mokotoff.encode("LIPPSZYC"), Ok("874400".to_string()));
        assert_eq!(daitch_mokotoff.encode("LEWINSKY"), Ok("876450".to_string()));
        assert_eq!(daitch_mokotoff.encode("LEVINSKI"), Ok("876450".to_string()));
        assert_eq!(
            daitch_mokotoff.encode("SZLAMAWICZ"),
            Ok("486740".to_string())
        );
        assert_eq!(
            daitch_mokotoff.encode("SHLAMOVITZ"),
            Ok("486740".to_string())
        );
    }

    #[test]
    fn test_encode_ignore_apostrophes() {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES)
            .build()
            .unwrap();

        for v in [
            "OBrien", "'OBrien", "O'Brien", "OB'rien", "OBr'ien", "OBri'en", "OBrie'n", "OBrien'",
        ]
        .iter()
        {
            assert_eq!(
                daitch_mokotoff.encode(v),
                Ok("079600".to_string()),
                "Error for {v}"
            );
        }
    }

    #[test]
    fn test_encode_ignore_hyphens() {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES)
            .build()
            .unwrap();

        for v in [
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
            assert_eq!(
                daitch_mokotoff.encode(v),
                Ok("565463".to_string()),
                "Error for {v}"
            );
        }
    }

    #[test]
    fn test_encode_ignore_trimmable() {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES)
            .build()
            .unwrap();

        assert_eq!(
            daitch_mokotoff.encode(" \t\n\r Washington \t\n\r "),
            Ok("746536".to_string())
        );
        assert_eq!(
            daitch_mokotoff.encode("Washington"),
            Ok("746536".to_string())
        );
    }

    #[test]
    fn test_soundex_basic() {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES)
            .build()
            .unwrap();

        assert_eq!(daitch_mokotoff.soundex("GOLDEN"), Ok("583600".to_string()));
        assert_eq!(daitch_mokotoff.soundex("Alpert"), Ok("087930".to_string()));
        assert_eq!(daitch_mokotoff.soundex("Breuer"), Ok("791900".to_string()));
        assert_eq!(daitch_mokotoff.soundex("Haber"), Ok("579000".to_string()));
        assert_eq!(
            daitch_mokotoff.soundex("Mannheim"),
            Ok("665600".to_string())
        );
        assert_eq!(daitch_mokotoff.soundex("Mintz"), Ok("664000".to_string()));
        assert_eq!(daitch_mokotoff.soundex("Topf"), Ok("370000".to_string()));
        assert_eq!(
            daitch_mokotoff.soundex("Kleinmann"),
            Ok("586660".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("Ben Aron"),
            Ok("769600".to_string())
        );

        assert_eq!(
            daitch_mokotoff.soundex("AUERBACH"),
            Ok("097400|097500".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("OHRBACH"),
            Ok("097400|097500".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("LIPSHITZ"),
            Ok("874400".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("LIPPSZYC"),
            Ok("874400|874500".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("LEWINSKY"),
            Ok("876450".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("LEVINSKI"),
            Ok("876450".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("SZLAMAWICZ"),
            Ok("486740".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("SHLAMOVITZ"),
            Ok("486740".to_string())
        );
    }

    #[test]
    fn test_soundex_basic2() {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES)
            .build()
            .unwrap();

        assert_eq!(
            daitch_mokotoff.soundex("Ceniow"),
            Ok("467000|567000".to_string())
        );
        assert_eq!(daitch_mokotoff.soundex("Tsenyuv"), Ok("467000".to_string()));
        assert_eq!(
            daitch_mokotoff.soundex("Holubica"),
            Ok("587400|587500".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("Golubitsa"),
            Ok("587400".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("Przemysl"),
            Ok("746480|794648".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("Pshemeshil"),
            Ok("746480".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("Rosochowaciec"),
            Ok("944744|944745|944754|944755|945744|945745|945754|945755".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("Rosokhovatsets"),
            Ok("945744".to_string())
        );
    }

    #[test]
    fn test_soundex_basic3() {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES)
            .build()
            .unwrap();

        assert_eq!(
            daitch_mokotoff.soundex("Peters"),
            Ok("734000|739400".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("Peterson"),
            Ok("734600|739460".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("Moskowitz"),
            Ok("645740".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("Moskovitz"),
            Ok("645740".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("Jackson"),
            Ok("154600|145460|454600|445460".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("Jackson-Jackson"),
            Ok("154654|154645|154644|145465|145464|454654|454645|454644|445465|445464".to_string())
        );
    }

    #[test]
    fn test_special_romanian_characters() {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::with_rules(COMMONS_CODEC_RULES)
            .build()
            .unwrap();

        assert_eq!(
            daitch_mokotoff.soundex("ţamas"),
            Ok("364000|464000".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("țamas"),
            Ok("364000|464000".to_string())
        );
    }

    #[test]
    #[cfg(feature = "embedded_dm")]
    fn test_embedded_dm() {
        let daitch_mokotoff = DaitchMokotoffSoundexBuilder::default().build().unwrap();

        assert_eq!(
            daitch_mokotoff.soundex("ţamas"),
            Ok("364000|464000".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("țamas"),
            Ok("364000|464000".to_string())
        );
    }

    #[test]
    #[cfg(feature = "embedded_dm")]
    fn test_default_daitch_mokotoff() {
        let daitch_mokotoff = DaitchMokotoffSoundex::default();

        assert_eq!(
            daitch_mokotoff.soundex("ţamas"),
            Ok("364000|464000".to_string())
        );
        assert_eq!(
            daitch_mokotoff.soundex("țamas"),
            Ok("364000|464000".to_string())
        );
    }
}
