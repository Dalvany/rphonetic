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
/// paper ["An assessment of name matching algorithms"](https://citeseerx.ist.psu.edu/viewdoc/download;jsessionid=E3997DC51F2046A95EE6459F2B997029?doi=10.1.1.453.4046&rep=rep1&type=pdf) in the Technical Report
/// Series published by the University of Newcastle Upon Tyne Computing Science.
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
    /// * `max_code_length`: the maximum code length.
    pub fn new(max_code_length: usize) -> Self {
        Self { max_code_length }
    }

    fn preprocess(&self, value: &str) -> String {
        let mut input = Self::soundex_clean(value);

        // 1. Remove all trailing 'S' characters
        while input.ends_with('S') {
            input.pop();
        }

        // 2. Replace leading letter pairs as follows:
        //    KN -> NN, PH -> FH, WR -> RR
        let first_two = input.chars().take(2).collect::<String>();
        match first_two.as_str() {
            "KN" => input.replace_range(..1, "N"),
            "PH" => input.replace_range(..1, "F"),
            "WR" => input.replace_range(..1, "R"),
            _ => (),
        };

        // Replace first characters as follows:
        //    H -> Remove
        let first = input.chars().next();
        if first == Some('H') {
            input.remove(0);
        }

        // Replace first characters as follows:
        //    E, I, O, U, Y -> A
        //    P -> B
        //    V -> F
        //    K, Q -> C
        //    J -> G
        //    Z -> S
        let first = input.chars().next();
        match first {
            Some('E') | Some('I') | Some('O') | Some('U') | Some('Y') => {
                input.replace_range(..1, "A")
            }
            Some('P') => input.replace_range(..1, "B"),
            Some('V') => input.replace_range(..1, "F"),
            Some('K') | Some('Q') => input.replace_range(..1, "C"),
            Some('J') => input.replace_range(..1, "G"),
            Some('Z') => input.replace_range(..1, "S"),
            _ => (),
        };

        input
    }

    fn is_vowel(c: Option<char>) -> bool {
        match c {
            Some(c) => is_vowel(Some(c.to_ascii_lowercase()), true),
            _ => false,
        }
    }

    fn transcode(
        &self,
        curr: char,
        next: Option<char>,
        is_last_char: bool,
    ) -> (Option<char>, bool) {
        let mut skip_next_char = false;

        let code: Option<char> = match curr {
            'B' | 'P' | 'F' | 'V' => Some('1'),
            'C' | 'S' | 'K' | 'G' | 'J' | 'Q' | 'X' | 'Z' => Some('2'),
            'D' | 'T' => match next {
                Some('C') => None,
                _ => Some('3'),
            },
            'L' => {
                if Phonex::is_vowel(next) || is_last_char {
                    Some('4')
                } else {
                    None
                }
            }
            'M' | 'N' => {
                skip_next_char = matches!(next, Some('D') | Some('G'));
                Some('5')
            }
            'R' => {
                if Phonex::is_vowel(next) || is_last_char {
                    Some('6')
                } else {
                    None
                }
            }
            _ => Some('0'),
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

        let mut chars = input.chars().enumerate().peekable();

        // Use directly the return type, with the right allocated capacity (if we're
        // only using ASCII).
        let mut result = String::with_capacity(self.max_code_length);

        let mut code = '0';
        let mut last = '0';
        let mut last_push = '0';

        'char_iter: while let Some((mut i, curr)) = chars.next() {
            // We reach max_code_length we stop here.
            if result.len() == self.max_code_length {
                break 'char_iter;
            }

            // We always add the first (sanitized, preprocessed) character.
            if i == 0 {
                result.push(curr);
                last_push = curr;
            }

            let next = chars.peek().map(|(_, ch)| ch).copied();

            // If `next` is None, it means that `curr` is last char. It also worked with previous
            // implementation.
            let (new_code, skip_next_char) = self.transcode(curr, next, next.is_none());

            // We only care if we have an actual value
            if let Some(c) = new_code {
                code = c
            }

            if skip_next_char {
                // Consume iterator
                let _ = chars.next();
                // Since `next()` return an Option, and we don't really
                // want to deal with the `None` case (what value to set `i` with ?)
                // we directly increment `i` for the remaining loop code.
                // Note that it will be reset with the right value in the next iteration
                i += 1;
            }

            if last != code && code != '0' && i != 0 {
                result.push(code);
                last_push = code;
            }

            last = last_push;

            if i == 0 {
                last = code;
            }
        }

        // Pad to ensure we meet `max_code_length`
        while result.len() < self.max_code_length {
            result.push('0');
        }

        result
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

    #[allow(clippy::type_complexity)]
    fn transcode(values: Vec<(char, Option<char>, bool, Option<char>, bool)>) {
        let phonex = Phonex::default();
        for (curr, next, is_last_char, e_code, e_skip_next_char) in values {
            let (code, skip_next_char) = phonex.transcode(curr, next, is_last_char);

            let expected = match code {
                Some(c) => c.to_string(),
                _ => String::from("None"),
            };

            let actual = match e_code {
                Some(c) => c.to_string(),
                _ => String::from("None"),
            };

            assert_eq!(
                code, e_code,
                "expected code to be {expected} but got {actual}"
            );

            assert_eq!(
                skip_next_char, e_skip_next_char,
                "expected skip_next_char to be {e_skip_next_char} but got {skip_next_char}"
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
            ("TESTSSS", String::from("TEST")),
            ("SSS", String::from("")),
            ("KNUTH", String::from("NNUTH")),
            ("PHONETIC", String::from("FHONETIC")),
            ("WRIGHT", String::from("RRIGHT")),
            ("HARRINGTON", String::from("ARRINGTON")),
            ("EIGER", String::from("AIGER")),
            ("PERCIVAL", String::from("BERCIVAL")),
            ("VERTIGAN", String::from("FERTIGAN")),
            ("KELVIN", String::from("CELVIN")),
            ("JONES", String::from("GONE")),
            ("ZEPHYR", String::from("SEPHYR")),
        ])
    }

    #[test]
    fn test_transcode() {
        transcode(vec![
            ('B', None, false, Some('1'), false),
            ('P', None, false, Some('1'), false),
            ('F', None, false, Some('1'), false),
            ('V', None, false, Some('1'), false),
            ('C', None, false, Some('2'), false),
            ('S', None, false, Some('2'), false),
            ('K', None, false, Some('2'), false),
            ('G', None, false, Some('2'), false),
            ('J', None, false, Some('2'), false),
            ('Q', None, false, Some('2'), false),
            ('X', None, false, Some('2'), false),
            ('Z', None, false, Some('2'), false),
            ('D', None, false, Some('3'), false),
            ('T', None, false, Some('3'), false),
            ('D', Some('C'), false, None, false),
            ('T', Some('C'), false, None, false),
            ('L', Some('A'), false, Some('4'), false),
            ('L', Some('B'), true, Some('4'), false),
            ('L', Some('B'), false, None, false),
            ('M', None, false, Some('5'), false),
            ('N', None, false, Some('5'), false),
            ('M', Some('D'), false, Some('5'), true),
            ('M', Some('G'), false, Some('5'), true),
            ('R', Some('A'), false, Some('6'), false),
            ('R', None, true, Some('6'), false),
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
            ("Wright", "R623"),
            ("Phonic", "F520"),
            ("Quickening", "C250"),
            ("Kuickening", "C250"),
            ("Joben", "G150"),
            ("Zelda", "S300"),
        ]);
    }

    #[test]
    fn test_encode_number() {
        let encoder = Phonex::default();

        assert_eq!(encoder.encode("123456789"), "0000");
    }

    #[test]
    fn test_encode_empty_string() {
        let encoder = Phonex::default();

        assert_eq!(encoder.encode(""), "0000");
    }
}
