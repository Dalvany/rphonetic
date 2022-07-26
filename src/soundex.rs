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

use crate::{Encoder, SoundexCommons, SoundexUtils};

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
/// Although it was primary done for names, [Soundex] can be used for general words.
///
/// # Example :
///
/// ```rust
/// use rphonetic::{Encoder, Soundex};
///
/// let soundex = Soundex::default();
/// assert_eq!(soundex.encode("jumped"), "J513");
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
    /// * `mapping` : mapping array. It contains, for each letter its corresponding code. Index 0 is the code for `A`, index 1
    /// is for `B`and so on for each letter of the latin alphabet. Code `-` is treated as silent (eg [DEFAULT_US_ENGLISH_GENEALOGY_MAPPING_SOUNDEX]).
    /// * `special_case_h_w` : a boolean to indicate that  ̀H` and `W` should be treated as silence.
    pub fn new(mapping: [char; 26], special_case_h_w: bool) -> Self {
        Self {
            mapping,
            special_case_h_w,
        }
    }

    fn get_mapping_code(&self, ch: char) -> char {
        self.mapping[ch as usize - 65]
    }
}

/// This is the [Default] implementation for [Soundex], it returns an instance
/// with [DEFAULT_US_ENGLISH_MAPPING_SOUNDEX] and, therefor, with a special
/// treatement for `H` and `W`̀ : they are considered as silence.
impl Default for Soundex {
    fn default() -> Self {
        Self {
            mapping: DEFAULT_US_ENGLISH_MAPPING_SOUNDEX,
            special_case_h_w: true,
        }
    }
}

impl TryFrom<[char; 26]> for Soundex {
    type Error = Vec<char>;

    fn try_from(mapping: [char; 26]) -> Result<Self, Self::Error> {
        let special_case_h_w = !has_silent_in_mapping(mapping);
        Ok(Self {
            mapping,
            special_case_h_w,
        })
    }
}

impl TryFrom<&str> for Soundex {
    type Error = Vec<char>;

    /// Construct a [Soundex] from the mapping in parameter. This [str] will
    /// be converted into an array of 26 chars, so `mapping`'s length must be 26.
    ///
    /// Mapping can contains `-` for silent. See [DEFAULT_US_ENGLISH_GENEALOGY_MAPPING_SOUNDEX].
    ///
    /// # Parameters
    ///
    /// * `mapping` : str that contains the corresponding code for each character.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Vec<char>> {
    /// use rphonetic::{Encoder, Soundex};
    ///
    /// // Construct an encoder with 'A' coded into '0', 'B' into '1', 'C' into '3', 'D' into '6', 'E' into '0', ...etc
    /// // (this is the default mapping)
    /// let soundex = Soundex::try_from("01360240043788015936020505")?;
    ///
    /// assert_eq!(soundex.encode("jumped"), "J816");
    /// #    Ok(())
    /// # }
    /// ```
    fn try_from(mapping: &str) -> Result<Self, Self::Error> {
        let mapping: [char; 26] = mapping.chars().collect::<Vec<char>>().try_into()?;
        Self::try_from(mapping)
    }
}

impl TryFrom<String> for Soundex {
    type Error = Vec<char>;

    /// Construct a [Soundex] from the mapping in parameter. This [String] will
    /// be converted into an array of 26 chars, so `mapping`'s length must be 26.
    ///
    /// Mapping can contains `-` for silent. See [DEFAULT_US_ENGLISH_GENEALOGY_MAPPING_SOUNDEX].
    ///
    /// # Parameters
    ///
    /// * `mapping` : str that contains the corresponding code for each character.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Vec<char>> {
    /// use rphonetic::{Encoder, Soundex};
    ///
    /// // Construct an encoder with 'A' coded into '0', 'B' into '1', 'C' into '3', 'D' into '6', 'E' into '0', ...etc
    /// // (this is the default mapping)
    /// let soundex = Soundex::try_from("01360240043788015936020505")?;
    ///
    /// assert_eq!(soundex.encode("jumped"), "J816");
    /// #    Ok(())
    /// # }
    /// ```
    fn try_from(mapping: String) -> Result<Self, Self::Error> {
        Self::try_from(mapping.as_str())
    }
}

impl Encoder for Soundex {
    fn encode(&self, value: &str) -> String {
        let value = Self::soundex_clean(value);
        if value.is_empty() {
            return value;
        }

        let mut code: [char; 4] = ['0', '0', '0', '0'];
        code[0] = value.chars().next().unwrap();
        let mut count = 1;
        let mut previous = self.get_mapping_code(code[0]);
        let mut iterator = value.chars().skip(1);
        while count < code.len() {
            match iterator.next() {
                None => break,
                Some(ch) => {
                    if self.special_case_h_w && (ch == 'H' || ch == 'W') {
                        continue;
                    }
                    let digit = self.get_mapping_code(ch);
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

        code.iter().collect()
    }
}

impl SoundexUtils for Soundex {}

impl SoundexCommons for Soundex {}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_encoding(data: Vec<&str>, expected: &str) {
        let soundex = Soundex::default();

        for v in data {
            assert_eq!(
                soundex.encode(v),
                expected,
                "Encoding {} should return {}",
                v,
                expected
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

        assert_eq!(soundex.encode("HOL>MES"), "H452");
    }

    #[test]
    fn test_difference() {
        let soundex = Soundex::default();

        assert_eq!(soundex.difference(" ", " "), 0);
        assert_eq!(soundex.difference("Smith", "Smythe"), 4);
        assert_eq!(soundex.difference("Ann", "Andrew"), 2);
        assert_eq!(soundex.difference("Margaret", "Andrew"), 1);
        assert_eq!(soundex.difference("Janet", "Margaret"), 0);
        assert_eq!(soundex.difference("Green", "Greene"), 4);
        assert_eq!(soundex.difference("Blotchet-Halls", "Greene"), 0);
        assert_eq!(soundex.difference("Smith", "Smythe"), 4);
        assert_eq!(soundex.difference("Smithers", "Smythers"), 4);
        assert_eq!(soundex.difference("Anothers", "Brothers"), 2);
    }

    #[test]
    fn test_encode_basic() {
        let soundex = Soundex::default();

        assert_eq!(soundex.encode("testing"), "T235");
        assert_eq!(soundex.encode("The"), "T000");
        assert_eq!(soundex.encode("quick"), "Q200");
        assert_eq!(soundex.encode("brown"), "B650");
        assert_eq!(soundex.encode("fox"), "F200");
        assert_eq!(soundex.encode("jumped"), "J513");
        assert_eq!(soundex.encode("over"), "O160");
        assert_eq!(soundex.encode("the"), "T000");
        assert_eq!(soundex.encode("lazy"), "L200");
        assert_eq!(soundex.encode("dogs"), "D200");
    }

    #[test]
    fn test_encode_batch2() {
        let soundex = Soundex::default();

        assert_eq!(soundex.encode("Allricht"), "A462");
        assert_eq!(soundex.encode("Eberhard"), "E166");
        assert_eq!(soundex.encode("Engebrethson"), "E521");
        assert_eq!(soundex.encode("Heimbach"), "H512");
        assert_eq!(soundex.encode("Hanselmann"), "H524");
        assert_eq!(soundex.encode("Hildebrand"), "H431");
        assert_eq!(soundex.encode("Kavanagh"), "K152");
        assert_eq!(soundex.encode("Lind"), "L530");
        assert_eq!(soundex.encode("Lukaschowsky"), "L222");
        assert_eq!(soundex.encode("McDonnell"), "M235");
        assert_eq!(soundex.encode("McGee"), "M200");
        assert_eq!(soundex.encode("Opnian"), "O155");
        assert_eq!(soundex.encode("Oppenheimer"), "O155");
        assert_eq!(soundex.encode("Riedemanas"), "R355");
        assert_eq!(soundex.encode("Zita"), "Z300");
        assert_eq!(soundex.encode("Zitzmeinn"), "Z325");
    }

    #[test]
    fn test_encode_batch3() {
        let soundex = Soundex::default();

        assert_eq!(soundex.encode("Washington"), "W252");
        assert_eq!(soundex.encode("Lee"), "L000");
        assert_eq!(soundex.encode("Gutierrez"), "G362");
        assert_eq!(soundex.encode("Pfister"), "P236");
        assert_eq!(soundex.encode("Jackson"), "J250");
        assert_eq!(soundex.encode("Tymczak"), "T522");
        assert_eq!(soundex.encode("VanDeusen"), "V532");
    }

    #[test]
    fn test_encode_batch4() {
        let soundex = Soundex::default();

        assert_eq!(soundex.encode("HOLMES"), "H452");
        assert_eq!(soundex.encode("ADOMOMI"), "A355");
        assert_eq!(soundex.encode("VONDERLEHR"), "V536");
        assert_eq!(soundex.encode("BALL"), "B400");
        assert_eq!(soundex.encode("SHAW"), "S000");
        assert_eq!(soundex.encode("JACKSON"), "J250");
        assert_eq!(soundex.encode("SCANLON"), "S545");
        assert_eq!(soundex.encode("SAINTJOHN"), "S532");
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

        assert_eq!(soundex.encode(" \t\n\r Washington \t\n\r "), "W252");
    }

    #[test]
    fn test_hw_rule_ex1() {
        let soundex = Soundex::default();

        assert_eq!(soundex.encode("Ashcraft"), "A261");
        assert_eq!(soundex.encode("Ashcroft"), "A261");
        assert_eq!(soundex.encode("yehudit"), "Y330");
        assert_eq!(soundex.encode("yhwdyt"), "Y330");
    }

    #[test]
    fn test_hw_rule_ex2() {
        let soundex = Soundex::default();

        assert_eq!(soundex.encode("BOOTHDAVIS"), "B312");
        assert_eq!(soundex.encode("BOOTH-DAVIS"), "B312");
    }

    #[test]
    fn test_hw_rule_ex3() {
        let soundex = Soundex::default();

        assert_eq!(soundex.encode("Sgler"), "S460");
        assert_eq!(soundex.encode("Swhgler"), "S460");

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

        assert_eq!(soundex.encode("Smith"), "S530");
        assert_eq!(soundex.encode("Smythe"), "S530");
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

        assert_eq!(soundex.encode("Ann"), "A500");
        assert_eq!(soundex.encode("Andrew"), "A536");
        assert_eq!(soundex.encode("Janet"), "J530");
        assert_eq!(soundex.encode("Margaret"), "M626");
        assert_eq!(soundex.encode("Steven"), "S315");
        assert_eq!(soundex.encode("Michael"), "M240");
        assert_eq!(soundex.encode("Robert"), "R163");
        assert_eq!(soundex.encode("Laura"), "L600");
        assert_eq!(soundex.encode("Anne"), "A500");
    }

    #[test]
    fn test_wikipedia_american_soundex() {
        let soundex = Soundex::default();

        assert_eq!(soundex.encode("Robert"), "R163");
        assert_eq!(soundex.encode("Rupert"), "R163");
        assert_eq!(soundex.encode("Ashcraft"), "A261");
        assert_eq!(soundex.encode("Ashcroft"), "A261");
        assert_eq!(soundex.encode("Tymczak"), "T522");
        assert_eq!(soundex.encode("Pfister"), "P236");
    }

    #[test]
    fn test_genealogy() -> Result<(), Vec<char>> {
        let soundex = Soundex::try_from(DEFAULT_US_ENGLISH_GENEALOGY_MAPPING_SOUNDEX)?;

        assert_eq!(soundex.encode("Heggenburger"), "H251");
        assert_eq!(soundex.encode("Blackman"), "B425");
        assert_eq!(soundex.encode("Schmidt"), "S530");
        assert_eq!(soundex.encode("Lippmann"), "L150");
        assert_eq!(soundex.encode("Dodds"), "D200");
        assert_eq!(soundex.encode("Dhdds"), "D200");
        assert_eq!(soundex.encode("Dwdds"), "D200");

        Ok(())
    }

    #[test]
    fn test_simplified_soundex() {
        let soundex = Soundex::new(DEFAULT_US_ENGLISH_MAPPING_SOUNDEX, false);

        assert_eq!(soundex.encode("WILLIAMS"), "W452");
        assert_eq!(soundex.encode("BARAGWANATH"), "B625");
        assert_eq!(soundex.encode("DONNELL"), "D540");
        assert_eq!(soundex.encode("LLOYD"), "L300");
        assert_eq!(soundex.encode("WOOLCOCK"), "W422");
        assert_eq!(soundex.encode("Dodds"), "D320");
        assert_eq!(soundex.encode("Dhdds"), "D320");
        assert_eq!(soundex.encode("Dwdds"), "D320");
    }

    #[test]
    fn test_try_from_str() -> Result<(), Vec<char>> {
        let result = Soundex::try_from("01230120022455012623010202")?;
        assert_eq!(result, Soundex::default());

        Ok(())
    }
}
