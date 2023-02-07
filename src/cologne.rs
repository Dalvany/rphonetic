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

use crate::Encoder;

const CHAR_IGNORE: char = '-';
const AEIJOUY: [char; 7] = ['A', 'E', 'I', 'J', 'O', 'U', 'Y'];
const CSZ: [char; 3] = ['C', 'S', 'Z'];
const FPVW: [char; 4] = ['F', 'P', 'V', 'W'];
const GKQ: [char; 3] = ['G', 'K', 'Q'];
const CKQ: [char; 3] = ['C', 'K', 'Q'];
const AHKLOQRUX: [char; 9] = ['A', 'H', 'K', 'L', 'O', 'Q', 'R', 'U', 'X'];
const SZ: [char; 2] = ['S', 'Z'];
const AHKOQUX: [char; 7] = ['A', 'H', 'K', 'O', 'Q', 'U', 'X'];
const DTX: [char; 3] = ['D', 'T', 'X'];

struct CologneOutput {
    last_char: char,
    buffer: String,
}

impl CologneOutput {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            last_char: '/',
            buffer: String::with_capacity(capacity),
        }
    }

    fn push(&mut self, ch: char) {
        if ch != CHAR_IGNORE && self.last_char != ch && (ch != '0' || self.buffer.is_empty()) {
            self.buffer.push(ch);
        }

        self.last_char = ch;
    }
}

/// This a [Cologne](https://en.wikipedia.org/wiki/Cologne_phonetics) encoder.
///
/// # Example :
///
/// ```rust
/// use rphonetic::{Cologne, Encoder};
///
/// let cologne = Cologne;
///
/// assert_eq!(cologne.encode("m\u{00FC}ller"), "657");
/// ```
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Cologne;

impl Encoder for Cologne {
    fn encode(&self, s: &str) -> String {
        let mut output = CologneOutput::with_capacity(s.len());

        // Uppercase and aumlaut transcription
        let mut tmp = s.to_uppercase();
        tmp = tmp.replace('Ä', "A");
        tmp = tmp.replace('Ü', "U");
        tmp = tmp.replace('Ö', "O");

        let mut last_char = CHAR_IGNORE;

        let mut iterator = tmp.chars().peekable();

        while let Some(ch) = iterator.next() {
            if !ch.is_ascii_uppercase() {
                continue;
            }

            let next_char = iterator.peek().unwrap_or(&CHAR_IGNORE);

            if AEIJOUY.contains(&ch) {
                output.push('0');
            } else if ch == 'B' || (ch == 'P' && *next_char != 'H') {
                output.push('1');
            } else if (ch == 'D' || ch == 'T') && !CSZ.contains(next_char) {
                output.push('2');
            } else if FPVW.contains(&ch) {
                output.push('3');
            } else if GKQ.contains(&ch) {
                output.push('4');
            } else if ch == 'X' && !CKQ.contains(&last_char) {
                output.push('4');
                output.push('8');
            } else if ch == 'S' || ch == 'Z' {
                output.push('8');
            } else if ch == 'C' {
                if output.buffer.is_empty() {
                    if AHKLOQRUX.contains(next_char) {
                        output.push('4');
                    } else {
                        output.push('8');
                    }
                } else if SZ.contains(&last_char) || !AHKOQUX.contains(next_char) {
                    output.push('8');
                } else {
                    output.push('4');
                }
            } else if DTX.contains(&ch) {
                output.push('8')
            } else {
                match ch {
                    'R' => output.push('7'),
                    'L' => output.push('5'),
                    'M' | 'N' => output.push('6'),
                    'H' => output.push(CHAR_IGNORE),
                    _ => (),
                }
            }

            last_char = ch;
        }

        output.buffer
    }
}

#[cfg(test)]
mod tests {
    use crate::cologne::Cologne;
    use crate::Encoder;

    #[test]
    fn test_aabjoe() {
        let result = Cologne.encode("Aabjoe");

        assert_eq!(result, "01");
    }

    #[test]
    fn test_aaclan() {
        let result = Cologne.encode("Aaclan");

        assert_eq!(result, "0856");
    }

    #[test]
    fn test_aychlmajr_for_codec122() {
        let result = Cologne.encode("Aychlmajr");

        assert_eq!(result, "04567");
    }

    #[test]
    fn test_edge_cases() {
        let data: Vec<(&str, &str)> = vec![
            ("a", "0"),
            ("e", "0"),
            ("i", "0"),
            ("o", "0"),
            ("u", "0"),
            ("\u{00E4}", "0"), // a-umlaut
            ("\u{00F6}", "0"), // o-umlaut
            ("\u{00FC}", "0"), // u-umlaut
            ("\u{00DF}", "8"), // small sharp s
            ("aa", "0"),
            ("ha", "0"),
            ("h", ""),
            ("aha", "0"),
            ("b", "1"),
            ("p", "1"),
            ("ph", "3"),
            ("f", "3"),
            ("v", "3"),
            ("w", "3"),
            ("g", "4"),
            ("k", "4"),
            ("q", "4"),
            ("x", "48"),
            ("ax", "048"),
            ("cx", "48"),
            ("l", "5"),
            ("cl", "45"),
            ("acl", "085"),
            ("mn", "6"),
            ("{mn}", "6"), // test chars above Z
            ("r", "7"),
        ];

        for (test, expected) in data {
            let result = Cologne.encode(test);
            assert_eq!(result, expected, "Wrong for {test}");
        }
    }

    #[test]
    fn test_examples() {
        let data: Vec<(&str, &str)> = vec![
            ("m\u{00DC}ller", "657"), // mÜller - why upper case U-umlaut?
            ("m\u{00FC}ller", "657"), // müller - add equivalent lower-case
            ("schmidt", "862"),
            ("schneider", "8627"),
            ("fischer", "387"),
            ("weber", "317"),
            ("wagner", "3467"),
            ("becker", "147"),
            ("hoffmann", "0366"),
            ("sch\u{00C4}fer", "837"), // schÄfer - why upper case A-umlaut ?
            ("sch\u{00e4}fer", "837"), // schäfer - add equivalent lower-case
            ("Breschnew", "17863"),
            ("Wikipedia", "3412"),
            ("peter", "127"),
            ("pharma", "376"),
            ("m\u{00f6}nchengladbach", "664645214"), // mönchengladbach
            ("deutsch", "28"),
            ("deutz", "28"),
            ("hamburg", "06174"),
            ("hannover", "0637"),
            ("christstollen", "478256"),
            ("Xanthippe", "48621"),
            ("Zacharias", "8478"),
            ("Holzbau", "0581"),
            ("matsch", "68"),
            ("matz", "68"),
            ("Arbeitsamt", "071862"),
            ("Eberhard", "01772"),
            ("Eberhardt", "01772"),
            ("Celsius", "8588"),
            ("Ace", "08"),
            ("shch", "84"), // CODEC-254
            ("xch", "484"), // CODEC-255
            ("heithabu", "021"),
        ];

        for (test, expected) in data {
            let result = Cologne.encode(test);
            assert_eq!(result, expected, "Wrong for {test}");
        }
    }

    #[test]
    fn test_hyphen() {
        let data: Vec<(&str, &str)> = vec![
            ("bergisch-gladbach", "174845214"),
            ("M\u{00fc}ller-L\u{00fc}denscheidt", "65752682"),
        ];

        for (test, expected) in data {
            let result = Cologne.encode(test);
            assert_eq!(result, expected, "Wrong for {test}");
        }
    }

    #[test]
    fn test_is_encode_equals() {
        let data: Vec<(&str, &str)> = vec![
            ("Muller", "M\u{00fc}ller"), // Müller
            ("Meyer", "Mayr"),
            ("house", "house"),
            ("House", "house"),
            ("Haus", "house"),
            ("ganz", "Gans"),
            ("ganz", "G\u{00e4}nse"), // Gänse
            ("Miyagi", "Miyako"),
        ];

        for (a, b) in data {
            let result = Cologne.is_encoded_equals(a, b);
            assert!(result, "Encoding {a} and {b} gives a different result");
        }
    }

    #[test]
    fn test_variations_mella() {
        let data: Vec<&str> = vec!["mella", "milah", "moulla", "mellah", "muehle", "mule"];

        for text in data {
            let result = Cologne.encode(text);
            assert_eq!(result, "65");
        }
    }

    #[test]
    fn test_variations_meyer() {
        let data: Vec<&str> = vec!["Meier", "Maier", "Mair", "Meyer", "Meyr", "Mejer", "Major"];

        for text in data {
            let result = Cologne.encode(text);
            assert_eq!(result, "67");
        }
    }

    #[test]
    fn test_special_chars_between_same_letters() {
        let data: Vec<&str> = vec![
            "Test test",
            "Testtest",
            "Test-test",
            "TesT#Test",
            "TesT?test",
        ];

        for text in data {
            let result = Cologne.encode(text);
            assert_eq!(result, "28282");
        }
    }
}
