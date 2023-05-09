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

/// Phonex is a modification of the venerable Soundex algorithm. It accounts
/// for a few more letter combinations to improve accuracy on some data sets.
/// It was created by A.J. Lait and Brian Randell in 1996, described in their
/// paper "An assessment of name matching algorithms" in the Technical Report
/// Series published by University of Newcastle Upon Tyne Computing Science.
///
/// ```rust
/// use rphonetic::{Phonex, Encoder};
///
/// let phonex = Phonex::default();
/// assert_eq!(phonex.encode("KNUTH"),"N300");
/// ```
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Phonex {
    max_code_length: usize,
}

impl Phonex {
    /// Construct a new [Phonex] with the maximum code length provided.
    ///
    /// # Parameter
    ///
    /// * `max_code_length : the maximum code length.
    pub fn new(max_code_length: usize) -> Self {
        Self { max_code_length }
    }

    fn preprocess(&self, value: &str) -> String {
        let mut input = Self::soundex_clean(value);

        // 1. Remove all trailing 'S' characters
        while input.ends_with('S') {
            input.pop();
        }

        // 2. Convert leading letter pairs as follows:
        //    KN -> N, PH -> F, WR -> R
        let first_two = input.chars().take(2).collect::<String>();
        match first_two.as_str() {
            "KN" => input.replace_range(..2, "N"),
            "PH" => input.replace_range(..2, "F"),
            "WR" => input.replace_range(..2, "R"),
            _ => (),
        };

        // Replace first characters as follows:
        //    H -> Remove
        let first = input.chars().take(1).collect::<String>();
        match first.as_str() {
            "H" => {
                input.remove(0);
            }
            _ => (),
        }

        // Replace first characters as follows:
        //    E, I, O, U, Y -> A
        //    P -> B
        //    V -> F
        //    K, Q -> C
        //    J -> G
        //    Z -> S
        let first = input.chars().take(1).collect::<String>();
        match first.as_str() {
            "E" | "I" | "O" | "U" | "Y" => input.replace_range(..1, "A"),
            "P" => input.replace_range(..1, "B"),
            "V" => input.replace_range(..1, "F"),
            "K" | "Q" => input.replace_range(..1, "C"),
            "J" => input.replace_range(..1, "G"),
            "Z" => input.replace_range(..1, "S"),
            _ => (),
        };

        input
    }

    fn is_vowel(c: Option<&char>) -> bool {
        match c {
            Some(c) => is_vowel(Some(c.to_ascii_lowercase()), true),
            _ => false,
        }
    }

    fn transcode(&self, curr: &char, next: Option<&char>, is_last_char: bool) -> (char, bool) {
        let mut skip_next_char = false;

        let code = match curr {
            'B' | 'P' | 'F' | 'V' => '1',
            'C' | 'S' | 'K' | 'G' | 'J' | 'Q' | 'X' | 'Z' => '2',
            'D' | 'T' => match next {
                Some('C') => '0',
                _ => '3',
            },
            'L' => {
                if Phonex::is_vowel(next) || is_last_char {
                    '4'
                } else {
                    '0'
                }
            }
            'M' | 'N' => {
                skip_next_char = match next {
                    Some('D') | Some('G') => true,
                    _ => false,
                };
                '5'
            }
            'R' => {
                if Phonex::is_vowel(next) || is_last_char {
                    '6'
                } else {
                    '0'
                }
            }
            _ => '0',
        };

        (code, skip_next_char)
    }
}

impl Default for Phonex {
    fn default() -> Self {
        Self { max_code_length: 4 }
    }
}

impl SoundexUtils for Phonex {}

impl Encoder for Phonex {
    fn encode(&self, value: &str) -> String {
        let input = self.preprocess(value);
        println!("preprocessed: {input}");
        let chars: Vec<_> = input.chars().collect();

        let mut result = vec![];

        let mut last = '0';
        let mut i = 0;

        while i < chars.len() && result.len() < self.max_code_length {
            let curr = chars[i];
            let next = chars.get(i + 1);
            let is_last_char = i == (chars.len() - 1);

            let (code, skip_next_char) = self.transcode(&curr, next, is_last_char);
            if skip_next_char {
                i += 1;
            }

            if last != code && code != '0' && i != 0 {
                result.push(code);
            }

            if i == 0 {
                result.push(curr);
                last = code;
            } else {
                last = result[result.len() - 1]
            }

            i += 1;
        }

        // Pad to ensure we meet `max_code_length`
        while result.len() < self.max_code_length {
            result.push('0');
        }

        result.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Encoder, Phonex};

    fn preprocess(values: Vec<(&str, String)>) {
        let phonex = Phonex::default();
        for (input, expected) in values {
            let actual = phonex.preprocess(input);

            assert_eq!(
                actual, expected,
                "expected input {input} to be preprocessed to {expected}, but instead got {actual}"
            );
        }
    }

    fn transcode(values: Vec<(&char, Option<&char>, bool, char, bool)>) {
        let phonex = Phonex::default();
        for (curr, next, is_last_char, e_code, e_skip_next_char) in values {
            let (code, skip_next_char) = phonex.transcode(curr, next, is_last_char);

            assert_eq!(code, e_code, "code {code} should output {e_code}");

            assert_eq!(
                skip_next_char, e_skip_next_char,
                "skip_next_char {skip_next_char} should output {e_skip_next_char}"
            );
        }
    }

    fn encode(values: Vec<(&str, &str)>) {
        let phonex = Phonex::default();
        for (value, expected) in values {
            assert_eq!(
                phonex.encode(value),
                expected,
                "Encoding {value} should output {expected}"
            );
        }
    }

    #[test]
    fn test_preprocess() {
        preprocess(vec![
            (&"TESTSSS", String::from("TEST")),
            (&"SSS", String::from("")),
            (&"KNUTH", String::from("NUTH")),
            (&"PHONETIC", String::from("FONETIC")),
            (&"WRIGHT", String::from("RIGHT")),
            (&"HARRINGTON", String::from("ARRINGTON")),
            (&"EIGER", String::from("AIGER")),
            (&"PERCIVAL", String::from("BERCIVAL")),
            (&"VERTIGAN", String::from("FERTIGAN")),
            (&"KELVIN", String::from("CELVIN")),
            (&"JONES", String::from("GONE")),
            (&"ZEPHYR", String::from("SEPHYR")),
        ])
    }

    #[test]
    fn test_transcode() {
        transcode(vec![
            (&'B', None, false, '1', false),
            (&'P', None, false, '1', false),
            (&'F', None, false, '1', false),
            (&'V', None, false, '1', false),
            (&'C', None, false, '2', false),
            (&'S', None, false, '2', false),
            (&'K', None, false, '2', false),
            (&'G', None, false, '2', false),
            (&'J', None, false, '2', false),
            (&'Q', None, false, '2', false),
            (&'X', None, false, '2', false),
            (&'Z', None, false, '2', false),
            (&'D', None, false, '3', false),
            (&'T', None, false, '3', false),
            (&'D', Some(&'C'), false, '0', false),
            (&'T', Some(&'C'), false, '0', false),
            (&'L', Some(&'A'), false, '4', false),
            (&'L', Some(&'B'), true, '4', false),
            (&'L', Some(&'B'), false, '0', false),
            (&'M', None, false, '5', false),
            (&'N', None, false, '5', false),
            (&'M', Some(&'D'), false, '5', true),
            (&'M', Some(&'G'), false, '5', true),
            (&'R', Some(&'A'), false, '6', false),
            (&'R', None, true, '6', false),
        ]);
    }

    #[test]
    fn test_encode() {
        encode(vec![
            ("123 testsss", "T230"),
            ("24/7 test", "T230"),
            ("A", "A000"),
            ("Ashcraft", "A261"),
            ("Lee", "L000"),
            ("Kuhne", "C500"),
            ("Meyer-Lansky", "M452"),
            ("Oepping", "A150"),
            ("Daley", "D400"),
            ("Dalitz", "D432"),
            ("Duhlitz", "D432"),
            ("Dull", "D400"),
            ("De Ledes", "D430"),
            ("Sandemann", "S500"),
            ("Schmidt", "S530"),
            ("Sinatra", "S536"),
            ("Heinrich", "A562"),
            ("Hammerschlag", "A524"),
            ("Williams", "W450"),
            ("Wilms", "W500"),
            ("Wilson", "W250"),
            ("Worms", "W500"),
            ("Zedlitz", "S343"),
            ("Zotteldecke", "S320"),
            ("ZYX test", "S232"),
            ("Scherman", "S500"),
            ("Schurman", "S500"),
            ("Sherman", "S500"),
            ("Shermansss", "S500"),
            ("Shireman", "S650"),
            ("Shurman", "S500"),
            ("Euler", "A460"),
            ("Ellery", "A460"),
            ("Hilbert", "A130"),
            ("Heilbronn", "A165"),
            ("Gauss", "G000"),
            ("Ghosh", "G200"),
            ("Knuth", "N300"),
            ("Kant", "C530"),
            ("Lloyd", "L430"),
            ("Ladd", "L300"),
            ("Lukasiewicz", "L200"),
            ("Lissajous", "L200"),
            ("Philip", "F410"),
            ("Fripp", "F610"),
            ("Czarkowska", "C200"),
            ("Hornblower", "A514"),
            ("Looser", "L260"),
            ("Wright", "R230"),
            ("Phonic", "F520"),
            ("Quickening", "C250"),
            ("Kuickening", "C250"),
            ("Joben", "G150"),
            ("Zelda", "S300"),
        ]);
    }
}
