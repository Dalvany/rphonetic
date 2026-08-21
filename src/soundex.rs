use std::str::FromStr;

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
use serde::{Deserialize, Serialize};

use crate::soundex_commons::SoundexUtils;
use crate::{Encoder, SoundexCommons, SoundexConvertError, SoundexEncodeError};

const SILENT: char = '-';

/// This is the default mapping character for soundex.
/// * `A` is encoded into `0`
/// * `B` is encoded into `1`
/// * `C` is encoded into `2`
/// * `D` is encoded into `3`
/// * `E` is encoded into `0`
/// * ...etc
///
/// There silent (`-`) code for any character so `̀H` and `W` will be treated differently (they are
/// considered as silence).
pub const DEFAULT_US_ENGLISH_MAPPING_SOUNDEX: [char; 26] = [
    '0', '1', '2', '3', '0', '1', '2', '0', '0', '2', '2', '4', '5', '5', '0', '1', '2', '6', '2',
    '3', '0', '1', '0', '2', '0', '2',
];

/// A mapping from [Genealogy](http://www.genealogy.com/articles/research/00000060.html) site.
/// * `A` is encoded into `-` (silent)
/// * `B` is encoded into `1`
/// * `C` is encoded into `2`
/// * `D` is encoded into `3`
/// * `E` is encoded into `-` (silent)
/// * ...etc
///
/// Except from vowels that are mapped to silence, it is the same mapping as [DEFAULT_US_ENGLISH_MAPPING_SOUNDEX].
///
/// As there are silent in this mapping, `H` and `W` won't be treated differently.
pub const DEFAULT_US_ENGLISH_GENEALOGY_MAPPING_SOUNDEX: [char; 26] = [
    '-', '1', '2', '3', '-', '1', '2', '-', '-', '2', '2', '4', '5', '5', '-', '1', '2', '6', '2',
    '3', '-', '1', '-', '2', '-', '2',
];

fn has_silent_in_mapping(mapping: [char; 26]) -> bool {
    mapping.iter().any(|c| c == &SILENT)
}

/// This is the [Soundex](https://en.wikipedia.org/wiki/Soundex) implementation of [Encoder].
///
/// The code will have a constant length of 4.
///
/// Although it was primarily done for names, [Soundex] can be used for general words.
///
/// # Example :
///
/// ```rust
/// # fn main() -> anyhow::Result<()> {
/// use rphonetic::{Encoder, Soundex};
///
/// let soundex = Soundex::default();
/// assert_eq!(soundex.encode("jumped")?, "J513");
///
/// #   Ok(())
/// # }
/// ```
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Soundex {
    mapping: [char; 26],
    special_case_h_w: bool,
}

impl Soundex {
    /// Construct a new [Soundex] with the provided mapping.
    ///
    /// There are implementations of [TryFrom] for convenience.
    ///
    /// # Parameter
    ///
    /// * `mapping`: mapping array.
    ///   It contains for each letter its corresponding code.
    ///   Index 0 is the code for `A`, index is for `B` and so on for
    ///   each letter of the latin alphabet.
    ///   Code `-` is treated as silent (eg [DEFAULT_US_ENGLISH_GENEALOGY_MAPPING_SOUNDEX]).
    /// * `special_case_h_w`: a boolean to indicate that `H` and `W` should be treated as silence.
    pub fn new(mapping: [char; 26], special_case_h_w: bool) -> Self {
        Self {
            mapping,
            special_case_h_w,
        }
    }
}

/// This is the [Default] implementation for [Soundex], it returns an instance
/// with [DEFAULT_US_ENGLISH_MAPPING_SOUNDEX] and, therefor, with a special
/// treatment for `H` and `W`̀: they are considered as silence.
impl Default for Soundex {
    fn default() -> Self {
        Self {
            mapping: DEFAULT_US_ENGLISH_MAPPING_SOUNDEX,
            special_case_h_w: true,
        }
    }
}

impl From<[char; 26]> for Soundex {
    fn from(mapping: [char; 26]) -> Self {
        let special_case_h_w = !has_silent_in_mapping(mapping);
        Self {
            mapping,
            special_case_h_w,
        }
    }
}

impl TryFrom<&str> for Soundex {
    type Error = SoundexConvertError;

    /// Construct a [Soundex] from the mapping in parameter. This [str] will
    /// be converted into an array of 26 chars, so `mapping`'s length must be 26.
    ///
    /// Mapping can contain `-` for silent. See [DEFAULT_US_ENGLISH_GENEALOGY_MAPPING_SOUNDEX].
    ///
    /// # Parameters
    ///
    /// * `mapping`: str that contains the corresponding code for each character.
    ///
    /// # Errors
    ///
    /// Returns an error the number of [char] in the [str] isn't equals to 26.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> anyhow::Result<()> {
    /// use rphonetic::{Encoder, Soundex};
    ///
    /// // Construct an encoder with 'A' coded into '0', 'B' into '1', 'C' into '3', 'D' into '6', 'E' into '0', ...etc
    /// // (this is the default mapping)
    /// let soundex = Soundex::try_from("01360240043788015936020505")?;
    ///
    /// assert_eq!(soundex.encode("jumped")?, "J816");
    /// #    Ok(())
    /// # }
    /// ```
    fn try_from(mapping: &str) -> Result<Self, Self::Error> {
        let mapping: [char; 26] = mapping
            .chars()
            .collect::<Vec<char>>()
            .try_into()
            .map_err(SoundexConvertError)?;
        Ok(Self::from(mapping))
    }
}

impl FromStr for Soundex {
    type Err = SoundexConvertError;

    /// Construct a [Soundex] from the mapping in parameter. This [str] will
    /// be converted into an array of 26 chars, so `mapping`'s length must be 26.
    ///
    /// Mapping can contain `-` for silent. See [DEFAULT_US_ENGLISH_GENEALOGY_MAPPING_SOUNDEX].
    ///
    /// # Parameters
    ///
    /// * `mapping`: str that contains the corresponding code for each character.
    ///
    /// # Errors
    ///
    /// Returns an error the number of [char] in the [str] isn't equals to 26.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> anyhow::Result<()> {
    /// use rphonetic::{Encoder, Soundex};
    ///
    /// // Construct an encoder with 'A' coded into '0', 'B' into '1', 'C' into '3', 'D' into '6', 'E' into '0', ...etc
    /// // (this is the default mapping)
    /// let soundex = "01360240043788015936020505".parse::<Soundex>()?;
    ///
    /// assert_eq!(soundex.encode("jumped")?, "J816");
    /// #    Ok(())
    /// # }
    /// ```
    fn from_str(mapping: &str) -> Result<Self, Self::Err> {
        Self::try_from(mapping)
    }
}

impl TryFrom<String> for Soundex {
    type Error = SoundexConvertError;

    /// Construct a [Soundex] from the mapping in parameter. This [String] will
    /// be converted into an array of 26 chars, so `mapping`'s length must be 26.
    ///
    /// Mapping can contain `-` for silent. See [DEFAULT_US_ENGLISH_GENEALOGY_MAPPING_SOUNDEX].
    ///
    /// # Parameters
    ///
    /// * `mapping`: str that contains the corresponding code for each character.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> anyhow::Result<()> {
    /// use rphonetic::{Encoder, Soundex};
    ///
    /// // Construct an encoder with 'A' coded into '0', 'B' into '1', 'C' into '3', 'D' into '6', 'E' into '0', ...etc
    /// // (this is the default mapping)
    /// let soundex = Soundex::try_from("01360240043788015936020505".to_string())?;
    ///
    /// assert_eq!(soundex.encode("jumped")?, "J816");
    /// #    Ok(())
    /// # }
    /// ```
    fn try_from(mapping: String) -> Result<Self, Self::Error> {
        Self::try_from(mapping.as_str())
    }
}

/// [Encoder] implementation.
///
/// Note that it should be safe to use the `unchecked` method
/// for this algorithm because non ASCII letters are removed. Then
/// all `get(...)` calls on slice must be safe.
impl Encoder for Soundex {
    type Error = SoundexEncodeError;

    fn encode(&self, value: &str) -> Result<String, Self::Error> {
        let value = Self::soundex_clean(value);
        let mut iterator = value.chars();

        let mut code: [char; 4] = ['0', '0', '0', '0'];

        match iterator.next() {
            Some(ch) => code[0] = ch,
            None => return Ok(value),
        }

        let mut count = 1;
        let mut previous = crate::soundex_commons::get_mapping_code(&self.mapping, code[0])?;
        while count < code.len() {
            match iterator.next() {
                None => break,
                Some(ch) => {
                    if self.special_case_h_w && (ch == 'H' || ch == 'W') {
                        continue;
                    }
                    let digit = crate::soundex_commons::get_mapping_code(&self.mapping, ch)?;
                    if digit == SILENT {
                        continue;
                    }
                    if digit != '0' && digit != previous {
                        code[count] = digit;
                        count += 1;
                    }

                    previous = digit;
                }
            }
        }

        Ok(code.iter().collect())
    }
}

impl SoundexUtils for Soundex {}

impl SoundexCommons for Soundex {}

#[cfg(test)]
mod tests {
    // Note : can't test characters outside ascii letter range, because call to 'soundex_clean'
    // prevent this case.
    use super::*;

    fn check_encoding(data: Vec<&str>, expected: &str) {
        let soundex = Soundex::default();

        for v in data {
            assert_eq!(
                soundex.encode(v),
                Ok(expected.to_string()),
                "Encoding {v} should return {expected}"
            );
        }
    }

    #[test]
    fn test_b650() {
        let data = vec![
            "BARHAM", "BARONE", "BARRON", "BERNA", "BIRNEY", "BIRNIE", "BOOROM", "BOREN", "BORN",
            "BOURN", "BOURNE", "BOWRON", "BRAIN", "BRAME", "BRANN", "BRAUN", "BREEN", "BRIEN",
            "BRIM", "BRIMM", "BRINN", "BRION", "BROOM", "BROOME", "BROWN", "BROWNE", "BRUEN",
            "BRUHN", "BRUIN", "BRUMM", "BRUN", "BRUNO", "BRYAN", "BURIAN", "BURN", "BURNEY",
            "BYRAM", "BYRNE", "BYRON", "BYRUM",
        ];

        check_encoding(data, "B650");
    }

    #[test]
    fn test_bad_characters() {
        let soundex = Soundex::default();

        assert_eq!(soundex.encode("HOL>MES"), Ok("H452".to_string()));
    }

    #[test]
    fn test_difference() {
        let soundex = Soundex::default();

        assert_eq!(soundex.difference(" ", " "), Ok(0));
        assert_eq!(soundex.difference("Smith", "Smythe"), Ok(4));
        assert_eq!(soundex.difference("Ann", "Andrew"), Ok(2));
        assert_eq!(soundex.difference("Margaret", "Andrew"), Ok(1));
        assert_eq!(soundex.difference("Janet", "Margaret"), Ok(0));
        assert_eq!(soundex.difference("Green", "Greene"), Ok(4));
        assert_eq!(soundex.difference("Blotchet-Halls", "Greene"), Ok(0));
        assert_eq!(soundex.difference("Smith", "Smythe"), Ok(4));
        assert_eq!(soundex.difference("Smithers", "Smythers"), Ok(4));
        assert_eq!(soundex.difference("Anothers", "Brothers"), Ok(2));
    }

    #[test]
    fn test_encode_basic() {
        let soundex = Soundex::default();

        assert_eq!(soundex.encode("testing"), Ok("T235".to_string()));
        assert_eq!(soundex.encode("The"), Ok("T000".to_string()));
        assert_eq!(soundex.encode("quick"), Ok("Q200".to_string()));
        assert_eq!(soundex.encode("brown"), Ok("B650".to_string()));
        assert_eq!(soundex.encode("fox"), Ok("F200".to_string()));
        assert_eq!(soundex.encode("jumped"), Ok("J513".to_string()));
        assert_eq!(soundex.encode("over"), Ok("O160".to_string()));
        assert_eq!(soundex.encode("the"), Ok("T000".to_string()));
        assert_eq!(soundex.encode("lazy"), Ok("L200".to_string()));
        assert_eq!(soundex.encode("dogs"), Ok("D200".to_string()));
    }

    #[test]
    fn test_encode_batch2() {
        let soundex = Soundex::default();

        assert_eq!(soundex.encode("Allricht"), Ok("A462".to_string()));
        assert_eq!(soundex.encode("Eberhard"), Ok("E166".to_string()));
        assert_eq!(soundex.encode("Engebrethson"), Ok("E521".to_string()));
        assert_eq!(soundex.encode("Heimbach"), Ok("H512".to_string()));
        assert_eq!(soundex.encode("Hanselmann"), Ok("H524".to_string()));
        assert_eq!(soundex.encode("Hildebrand"), Ok("H431".to_string()));
        assert_eq!(soundex.encode("Kavanagh"), Ok("K152".to_string()));
        assert_eq!(soundex.encode("Lind"), Ok("L530".to_string()));
        assert_eq!(soundex.encode("Lukaschowsky"), Ok("L222".to_string()));
        assert_eq!(soundex.encode("McDonnell"), Ok("M235".to_string()));
        assert_eq!(soundex.encode("McGee"), Ok("M200".to_string()));
        assert_eq!(soundex.encode("Opnian"), Ok("O155".to_string()));
        assert_eq!(soundex.encode("Oppenheimer"), Ok("O155".to_string()));
        assert_eq!(soundex.encode("Riedemanas"), Ok("R355".to_string()));
        assert_eq!(soundex.encode("Zita"), Ok("Z300".to_string()));
        assert_eq!(soundex.encode("Zitzmeinn"), Ok("Z325".to_string()));
    }

    #[test]
    fn test_encode_batch3() {
        let soundex = Soundex::default();

        assert_eq!(soundex.encode("Washington"), Ok("W252".to_string()));
        assert_eq!(soundex.encode("Lee"), Ok("L000".to_string()));
        assert_eq!(soundex.encode("Gutierrez"), Ok("G362".to_string()));
        assert_eq!(soundex.encode("Pfister"), Ok("P236".to_string()));
        assert_eq!(soundex.encode("Jackson"), Ok("J250".to_string()));
        assert_eq!(soundex.encode("Tymczak"), Ok("T522".to_string()));
        assert_eq!(soundex.encode("VanDeusen"), Ok("V532".to_string()));
    }

    #[test]
    fn test_encode_batch4() {
        let soundex = Soundex::default();

        assert_eq!(soundex.encode("HOLMES"), Ok("H452".to_string()));
        assert_eq!(soundex.encode("ADOMOMI"), Ok("A355".to_string()));
        assert_eq!(soundex.encode("VONDERLEHR"), Ok("V536".to_string()));
        assert_eq!(soundex.encode("BALL"), Ok("B400".to_string()));
        assert_eq!(soundex.encode("SHAW"), Ok("S000".to_string()));
        assert_eq!(soundex.encode("JACKSON"), Ok("J250".to_string()));
        assert_eq!(soundex.encode("SCANLON"), Ok("S545".to_string()));
        assert_eq!(soundex.encode("SAINTJOHN"), Ok("S532".to_string()));
    }

    #[test]
    fn test_encode_ignore_apostrophes() {
        let data = vec![
            "OBrien", "'OBrien", "O'Brien", "OB'rien", "OBr'ien", "OBri'en", "OBrie'n", "OBrien'",
        ];

        check_encoding(data, "O165");
    }

    #[test]
    fn test_encode_ignore_hyphens() {
        let data = vec![
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
        ];

        check_encoding(data, "K525");
    }

    #[test]
    fn test_encode_ignore_trimmable() {
        let soundex = Soundex::default();

        assert_eq!(
            soundex.encode(" \t\n\r Washington \t\n\r "),
            Ok("W252".to_string())
        );
    }

    #[test]
    fn test_hw_rule_ex1() {
        let soundex = Soundex::default();

        assert_eq!(soundex.encode("Ashcraft"), Ok("A261".to_string()));
        assert_eq!(soundex.encode("Ashcroft"), Ok("A261".to_string()));
        assert_eq!(soundex.encode("yehudit"), Ok("Y330".to_string()));
        assert_eq!(soundex.encode("yhwdyt"), Ok("Y330".to_string()));
    }

    #[test]
    fn test_hw_rule_ex2() {
        let soundex = Soundex::default();

        assert_eq!(soundex.encode("BOOTHDAVIS"), Ok("B312".to_string()));
        assert_eq!(soundex.encode("BOOTH-DAVIS"), Ok("B312".to_string()));
    }

    #[test]
    fn test_hw_rule_ex3() {
        let soundex = Soundex::default();

        assert_eq!(soundex.encode("Sgler"), Ok("S460".to_string()));
        assert_eq!(soundex.encode("Swhgler"), Ok("S460".to_string()));

        let data = vec![
            "SAILOR", "SALYER", "SAYLOR", "SCHALLER", "SCHELLER", "SCHILLER", "SCHOOLER",
            "SCHULER", "SCHUYLER", "SEILER", "SEYLER", "SHOLAR", "SHULER", "SILAR", "SILER",
            "SILLER",
        ];
        check_encoding(data, "S460");
    }

    #[test]
    fn test_ms_sql_server1() {
        let soundex = Soundex::default();

        assert_eq!(soundex.encode("Smith"), Ok("S530".to_string()));
        assert_eq!(soundex.encode("Smythe"), Ok("S530".to_string()));
    }

    #[test]
    fn test_ms_sql_server2() {
        let data = vec![
            "Erickson", "Erickson", "Erikson", "Ericson", "Ericksen", "Ericsen",
        ];

        check_encoding(data, "E625");
    }

    #[test]
    fn test_ms_sql_server3() {
        let soundex = Soundex::default();

        assert_eq!(soundex.encode("Ann"), Ok("A500".to_string()));
        assert_eq!(soundex.encode("Andrew"), Ok("A536".to_string()));
        assert_eq!(soundex.encode("Janet"), Ok("J530".to_string()));
        assert_eq!(soundex.encode("Margaret"), Ok("M626".to_string()));
        assert_eq!(soundex.encode("Steven"), Ok("S315".to_string()));
        assert_eq!(soundex.encode("Michael"), Ok("M240".to_string()));
        assert_eq!(soundex.encode("Robert"), Ok("R163".to_string()));
        assert_eq!(soundex.encode("Laura"), Ok("L600".to_string()));
        assert_eq!(soundex.encode("Anne"), Ok("A500".to_string()));
    }

    #[test]
    fn test_wikipedia_american_soundex() {
        let soundex = Soundex::default();

        assert_eq!(soundex.encode("Robert"), Ok("R163".to_string()));
        assert_eq!(soundex.encode("Rupert"), Ok("R163".to_string()));
        assert_eq!(soundex.encode("Ashcraft"), Ok("A261".to_string()));
        assert_eq!(soundex.encode("Ashcroft"), Ok("A261".to_string()));
        assert_eq!(soundex.encode("Tymczak"), Ok("T522".to_string()));
        assert_eq!(soundex.encode("Pfister"), Ok("P236".to_string()));
    }

    #[test]
    fn test_genealogy() {
        let soundex = Soundex::from(DEFAULT_US_ENGLISH_GENEALOGY_MAPPING_SOUNDEX);

        assert_eq!(soundex.encode("Heggenburger"), Ok("H251".to_string()));
        assert_eq!(soundex.encode("Blackman"), Ok("B425".to_string()));
        assert_eq!(soundex.encode("Schmidt"), Ok("S530".to_string()));
        assert_eq!(soundex.encode("Lippmann"), Ok("L150".to_string()));
        assert_eq!(soundex.encode("Dodds"), Ok("D200".to_string()));
        assert_eq!(soundex.encode("Dhdds"), Ok("D200".to_string()));
        assert_eq!(soundex.encode("Dwdds"), Ok("D200".to_string()));
    }

    #[test]
    fn test_simplified_soundex() {
        let soundex = Soundex::new(DEFAULT_US_ENGLISH_MAPPING_SOUNDEX, false);

        assert_eq!(soundex.encode("WILLIAMS"), Ok("W452".to_string()));
        assert_eq!(soundex.encode("BARAGWANATH"), Ok("B625".to_string()));
        assert_eq!(soundex.encode("DONNELL"), Ok("D540".to_string()));
        assert_eq!(soundex.encode("LLOYD"), Ok("L300".to_string()));
        assert_eq!(soundex.encode("WOOLCOCK"), Ok("W422".to_string()));
        assert_eq!(soundex.encode("Dodds"), Ok("D320".to_string()));
        assert_eq!(soundex.encode("Dhdds"), Ok("D320".to_string()));
        assert_eq!(soundex.encode("Dwdds"), Ok("D320".to_string()));
    }

    #[test]
    fn test_try_from_str() {
        let result = Soundex::try_from("01230120022455012623010202");

        assert_eq!(result, Ok(Soundex::default()));
    }

    #[test]
    fn test_try_from_string() {
        let result = Soundex::try_from("01230120022455012623010202".to_string());

        assert_eq!(result, Ok(Soundex::default()));
    }
}
