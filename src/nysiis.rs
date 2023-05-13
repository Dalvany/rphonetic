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

use crate::helper::is_vowel;
use crate::{Encoder, SoundexUtils};

const CHARS_A: &str = "A";
const CHARS_AF: &str = "AF";
const CHARS_C: &str = "C";
const CHARS_FF: &str = "FF";
const CHARS_G: &str = "G";
const CHARS_N: &str = "N";
const CHARS_NN: &str = "NN";
const CHARS_S: &str = "S";
const CHARS_SSS: &str = "SSS";

const START_MAC: &str = "MAC";
const START_KN: &str = "KN";
const START_K: &str = "K";
const START_PH: &str = "PH";
const START_PF: &str = "PF";
const START_SCH: &str = "SCH";
const END_EE: &str = "EE";
const END_IE: &str = "IE";
const END_DT: &str = "DT";
const END_RT: &str = "RT";
const END_RD: &str = "RD";
const END_NT: &str = "NT";
const END_ND: &str = "ND";

const TRUE_LENGTH: usize = 6;

/// This the [Nysiis](https://en.wikipedia.org/wiki/New_York_State_Identification_and_Intelligence_System) algorithm.
///
/// [Default] implementation constructs a strict version of the generated code.
/// That means the code has at most 6 characters.
/// A `new` constructor is provided, allowing code to have more than 6 characters.
///
/// ```rust
/// use rphonetic::{Nysiis, Encoder};
///
/// // Strict
/// let nysiis = Nysiis::default();
/// assert_eq!(nysiis.encode("WESTERLUND"),"WASTAR");
///
/// // Not strict
/// let nysiis = Nysiis::new(false);
/// assert_eq!(nysiis.encode("WESTERLUND"),"WASTARLAD");
/// ```
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Nysiis {
    strict: bool,
}

impl Nysiis {
    /// Use this constructor to disable the maximum code length.
    ///
    /// # Parameter
    ///
    /// * `strict`: if `true` code will have maximum length of 6.
    pub fn new(strict: bool) -> Self {
        Self { strict }
    }

    fn transcode(
        previous: &char,
        current: &char,
        next: Option<&char>,
        next_next: Option<&char>,
    ) -> String {
        if current == &'E' && next == Some(&'V') {
            return CHARS_AF.to_string();
        }

        if is_vowel(Some(current.to_ascii_lowercase()), false) {
            return CHARS_A.to_string();
        }

        match (current, next) {
            (&'Q', _) => return CHARS_G.to_string(),
            (&'Z', _) => return CHARS_S.to_string(),
            (&'M', _) => return CHARS_N.to_string(),
            (&'K', Some(&'N')) => return CHARS_NN.to_string(),
            (&'K', _) => return CHARS_C.to_string(),
            _ => (),
        }

        if current == &'S' && next == Some(&'C') && next_next == Some(&'H') {
            return CHARS_SSS.to_string();
        }

        if current == &'P' && next == Some(&'H') {
            return CHARS_FF.to_string();
        }

        if (current == &'H'
            && (!is_vowel(Some(previous.to_ascii_lowercase()), false)
                || !is_vowel(next.map(|c| c.to_ascii_lowercase()), false)))
            || (current == &'W' && is_vowel(Some(previous.to_ascii_lowercase()), false))
        {
            previous.to_string()
        } else {
            current.to_string()
        }
    }
}

impl Default for Nysiis {
    fn default() -> Self {
        Self { strict: true }
    }
}

impl SoundexUtils for Nysiis {}

impl Encoder for Nysiis {
    fn encode(&self, value: &str) -> String {
        let mut tmp = Self::soundex_clean(value);

        if tmp.is_empty() {
            return tmp;
        }

        if tmp.starts_with(START_MAC) {
            tmp.replace_range(..3, "MCC");
        }
        if tmp.starts_with(START_KN) {
            tmp.replace_range(..2, "NN");
        }
        if tmp.starts_with(START_K) {
            tmp.replace_range(..1, "C");
        }
        if tmp.starts_with(START_PH) || tmp.starts_with(START_PF) {
            tmp.replace_range(..2, "FF");
        }
        if tmp.starts_with(START_SCH) {
            tmp.replace_range(..3, "SSS");
        }

        if tmp.ends_with(END_EE) || tmp.ends_with(END_IE) {
            tmp.replace_range(tmp.len() - 2.., "Y")
        }
        if tmp.ends_with(END_DT)
            || tmp.ends_with(END_RT)
            || tmp.ends_with(END_RD)
            || tmp.ends_with(END_NT)
            || tmp.ends_with(END_ND)
        {
            tmp.replace_range(tmp.len() - 2.., "D")
        }

        let mut result = String::with_capacity(tmp.len());
        result.push(tmp.chars().next().unwrap());

        let mut chars: Vec<char> = tmp.chars().collect();
        let len = chars.len();
        let mut index = 1;

        let mut key = String::with_capacity(tmp.len());
        key.push(chars[0]);

        while index < len {
            let next: Option<&char> = chars.get(index + 1);
            let next_next: Option<&char> = chars.get(index + 2);
            let transcode = Nysiis::transcode(&chars[index - 1], &chars[index], next, next_next);

            for (i, c) in transcode.chars().enumerate() {
                chars[index + i] = c;
            }

            if chars[index - 1] != chars[index] {
                key.push(chars[index]);
            }

            index += 1;
        }

        let mut tmp = key.clone();
        let result: String = if tmp.len() > 1 {
            if tmp.ends_with('S') {
                tmp = tmp[..tmp.len() - 1].to_string();
            }

            if tmp.len() > 2 && tmp.ends_with("AY") {
                let mut start: String = tmp[..tmp.len() - 2].to_string();
                start.push_str(&tmp[tmp.len() - 1..]);
                tmp = start;
            }

            if tmp.ends_with('A') {
                tmp = tmp[..tmp.len() - 1].to_string();
            }

            tmp
        } else {
            key
        };

        if self.strict {
            let min = std::cmp::min(result.len(), TRUE_LENGTH);
            result[..min].to_string()
        } else {
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Encoder, Nysiis};

    fn encode_all(values: Vec<&str>, expected: &str) {
        let nysiis = Nysiis::default();
        for v in values {
            assert_eq!(nysiis.encode(v), expected);
        }
    }

    fn encode(values: Vec<(&str, &str)>) {
        let nysiis = Nysiis::new(false);
        for (value, expected) in values {
            assert_eq!(
                nysiis.encode(value),
                expected,
                "Encoding {value} should output {expected}"
            );
        }
    }

    #[test]
    fn test_bran() {
        encode_all(vec!["Brian", "Brown", "Brun"], "BRAN");
    }

    #[test]
    fn test_cap() {
        encode_all(vec!["Capp", "Cope", "Copp", "Kipp"], "CAP");
    }

    #[test]
    fn test_dad() {
        encode_all(vec!["Dent"], "DAD");
    }

    #[test]
    fn test_dan() {
        encode_all(vec!["Dane", "Dean", "Dionne"], "DAN");
    }

    #[test]
    fn test_fal() {
        encode_all(vec!["Phil"], "FAL");
    }

    #[test]
    fn test_drop_by() {
        let values = vec![
            ("MACINTOSH", "MCANT"),
            ("KNUTH", "NAT"),
            ("KOEHN", "CAN"),
            ("PHILLIPSON", "FALAPSAN"),
            ("PFEISTER", "FASTAR"),
            ("SCHOENHOEFT", "SANAFT"),
            ("MCKEE", "MCY"),
            ("MACKIE", "MCY"),
            ("HEITSCHMIDT", "HATSNAD"),
            ("BART", "BAD"),
            ("HURD", "HAD"),
            ("HUNT", "HAD"),
            ("WESTERLUND", "WASTARLAD"),
            ("CASSTEVENS", "CASTAFAN"),
            ("VASQUEZ", "VASG"),
            ("FRAZIER", "FRASAR"),
            ("BOWMAN", "BANAN"),
            ("MCKNIGHT", "MCNAGT"),
            ("RICKERT", "RACAD"),
            ("DEUTSCH", "DAT"),
            ("WESTPHAL", "WASTFAL"),
            ("SHRIVER", "SRAVAR"),
            ("KUHL", "CAL"),
            ("RAWSON", "RASAN"),
            ("JILES", "JAL"),
            ("CARRAWAY", "CARY"),
            ("YAMADA", "YANAD"),
        ];

        encode(values);
    }

    #[test]
    fn test_others() {
        let values = vec![
            ("O'Daniel", "ODANAL"),
            ("O'Donnel", "ODANAL"),
            ("Cory", "CARY"),
            ("Corey", "CARY"),
            ("Kory", "CARY"),
            ("FUZZY", "FASY"),
        ];

        encode(values);
    }

    #[test]
    fn test_rule1() {
        let values = vec![
            ("MACX", "MCX"),
            ("KNX", "NX"),
            ("KX", "CX"),
            ("PHX", "FX"),
            ("PFX", "FX"),
            ("SCHX", "SX"),
        ];

        encode(values);
    }

    #[test]
    fn test_rule2() {
        let values = vec![
            ("XEE", "XY"),
            ("XIE", "XY"),
            ("XDT", "XD"),
            ("XRT", "XD"),
            ("XRD", "XD"),
            ("XNT", "XD"),
            ("XND", "XD"),
        ];

        encode(values);
    }

    #[test]
    fn test_rule4_dot1() {
        let values = vec![
            ("XEV", "XAF"),
            ("XAX", "XAX"),
            ("XEX", "XAX"),
            ("XIX", "XAX"),
            ("XOX", "XAX"),
            ("XUX", "XAX"),
        ];

        encode(values);
    }

    #[test]
    fn test_rule4_dot2() {
        let values = vec![("XQ", "XG"), ("XZ", "X"), ("XM", "XN")];

        encode(values);
    }

    #[test]
    fn test_rule5() {
        let values = vec![("XS", "X"), ("XSS", "X")];

        encode(values);
    }

    #[test]
    fn test_rule6() {
        let values = vec![("XAY", "XY"), ("XAYS", "XY")];

        encode(values);
    }

    #[test]
    fn test_rule7() {
        let values = vec![("XA", "X"), ("XAS", "X")];

        encode(values);
    }

    #[test]
    fn test_snad() {
        encode_all(vec!["Schmidt"], "SNAD");
    }

    #[test]
    fn test_snat() {
        encode_all(vec!["Smith", "Schmit"], "SNAT");
    }

    #[test]
    fn test_special_branches() {
        encode_all(vec!["Kobwick"], "CABWAC");
        encode_all(vec!["Kocher"], "CACAR");
        encode_all(vec!["Fesca"], "FASC");
        encode_all(vec!["Shom"], "SAN");
        encode_all(vec!["Ohlo"], "OL");
        encode_all(vec!["Uhu"], "UH");
        encode_all(vec!["Um"], "UN");
    }

    #[test]
    fn test_tranan() {
        encode_all(vec!["Trueman", "Truman"], "TRANAN");
    }

    #[test]
    fn test_true_variant() {
        let nysiis = Nysiis::default();

        let result = nysiis.encode("WESTERLUND");
        assert!(result.len() <= 6);
        assert_eq!(result, "WASTAR");
    }
}
