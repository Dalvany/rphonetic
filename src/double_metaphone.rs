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
use std::fmt::{Display, Formatter};
use std::iter::Peekable;
use std::str::CharIndices;

use serde::{Deserialize, Serialize};

use crate::helper::is_vowel;
use crate::Encoder;

const SILENT_START: &[&str; 5] = &["GN", "KN", "PN", "WR", "PS"];
const L_R_N_M_B_H_F_V_W_SPACE: &[&str; 10] = &["L", "R", "N", "M", "B", "H", "F", "V", "W", " "];
const ES_EP_EB_EL_EY_IB_IL_IN_IE_EI_ER: &[&str; 11] = &[
    "ES", "EP", "EB", "EL", "EY", "IB", "IL", "IN", "IE", "EI", "ER",
];
const L_T_K_S_N_M_B_Z: &[&str; 8] = &["L", "T", "K", "S", "N", "M", "B", "Z"];

/// THis struct represent a double metaphone result. It contains both `primary` and
/// `alternate` code.
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct DoubleMetaphoneResult {
    primary: String,
    alternate: String,
    max_length: usize,
}

impl Display for DoubleMetaphoneResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[primary={}, alternate={}]",
            self.primary, self.alternate
        )
    }
}

/// This is representing a [DoubleMetaphone] result.
///
/// It contains both `primary` and `alternate` codes.
impl DoubleMetaphoneResult {
    fn new(max_length: usize) -> Self {
        Self {
            primary: String::with_capacity(max_length),
            alternate: String::with_capacity(max_length),
            max_length,
        }
    }

    /// Return the `primary` code.
    pub fn primary(&self) -> String {
        self.primary.clone()
    }

    /// Return the `alternate` code.
    pub fn alternate(&self) -> String {
        self.alternate.clone()
    }

    fn append_char(&mut self, ch: char, alternate: Option<char>) {
        self.append_char_primary(ch);
        self.append_char_alternate(alternate.unwrap_or(ch));
    }

    fn append_char_primary(&mut self, ch: char) {
        if self.primary.len() < self.max_length {
            self.primary.push(ch);
        }
    }

    fn append_char_alternate(&mut self, ch: char) {
        if self.alternate.len() < self.max_length {
            self.alternate.push(ch);
        }
    }

    fn append_str(&mut self, value: &str, alternate: Option<&str>) {
        self.append_str_primary(value);
        self.append_str_alternate(alternate.unwrap_or(value));
    }

    fn append_str_primary(&mut self, value: &str) {
        let length_remaining = self.max_length - self.primary.len();
        if value.len() <= length_remaining {
            self.primary.push_str(value);
        } else {
            self.primary.push_str(&value[0..length_remaining]);
        }
    }

    fn append_str_alternate(&mut self, value: &str) {
        let length_remaining = self.max_length - self.alternate.len();
        if value.len() <= length_remaining {
            self.alternate.push_str(value);
        } else {
            self.alternate.push_str(&value[0..length_remaining]);
        }
    }

    fn is_complete(&self) -> bool {
        self.primary.len() >= self.max_length && self.alternate.len() >= self.max_length
    }
}

/// This is the [Double Metaphone](https://en.wikipedia.org/wiki/Metaphone#Double_Metaphone) implementation.
///
/// The [Default] implementation have a maximum code length of 4. Use `new()` contructor to override.
///
/// Double Metaphone can generate two codes :  `primary` and `alternate`.
/// [Encoder] implementation return the primary code while `encode_alternate()` returns `alternate` code.
///
/// # Example
///
/// ```rust
/// use rphonetic::{DoubleMetaphone, Encoder};
///
/// let double_metaphone = DoubleMetaphone::default();
///
/// assert_eq!(double_metaphone.encode("jumped"), "JMPT");
/// assert_eq!(double_metaphone.encode_alternate("jumped"), "AMPT");
/// ```
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct DoubleMetaphone {
    max_code_length: usize,
}

impl Default for DoubleMetaphone {
    /// Construct a new [DoubleMetaphone] with a maximum code length of 4.
    fn default() -> Self {
        Self { max_code_length: 4 }
    }
}

impl DoubleMetaphone {
    /// Construct a new [DoubleMetaphone] with the maximum code length provided.
    ///
    /// # Parameter
    ///
    /// * `max_code_length : the maximum code length.
    pub fn new(max_code_length: usize) -> Self {
        Self { max_code_length }
    }

    /// This method encode and return the alternate code.
    ///
    /// # Parameter
    ///
    /// * `value` : value to encode.
    ///
    /// # Result
    ///
    /// Alternate value's code.
    pub fn encode_alternate(&self, value: &str) -> String {
        self.double_metaphone(value).alternate
    }

    /// This method check if code generated by `value1` and `value2` are equals.
    ///
    /// # Parameters
    ///
    /// * `value1` and `value2` : values to check.
    /// *  `alternate` : if  ̀false` then `primary` codes are checked, otherwise it is the alternate codes that are compared.
    ///
    /// # Result
    ///
    /// Return `true` if both codes are equals.
    pub fn is_double_metaphone_equal(&self, value1: &str, value2: &str, alternate: bool) -> bool {
        let result1 = self.double_metaphone(value1);
        let result2 = self.double_metaphone(value2);
        if alternate {
            result1.alternate == result2.alternate
        } else {
            result1.primary == result2.primary
        }
    }

    fn is_slavo_germanic(value: &str) -> bool {
        value.chars().any(|c| c == 'W' || c == 'K')
            || value.contains("CZ")
            || value.contains("WITZ")
    }

    fn contains(value: &str, start: usize, length: usize, criteria: Vec<&str>) -> bool {
        let result = false;

        if start + length <= value.len() {
            let target: &str = &value[start..start + length];
            return criteria.contains(&target);
        }

        result
    }

    fn contains_array(value: &str, start: usize, length: usize, criteria: &[&str]) -> bool {
        let result = false;

        if start + length <= value.len() {
            let target: &str = &value[start..start + length];
            return criteria.contains(&target);
        }

        result
    }

    fn char_at(value: &str, index: usize) -> Option<char> {
        if index < value.len() {
            return value[index..].chars().next();
        }

        None
    }

    /// Encode `value` and return the code. If  ̀alternate` is `false` then `primary` code
    /// is return, otherwise it will be the `alternate` code.
    ///
    /// # Parameter
    ///
    /// * `value` : value to encode.
    ///
    /// # Result
    ///
    /// A [DoubleMetaphone] that contains both `primary` and `alternate` code.
    pub fn double_metaphone(&self, value: &str) -> DoubleMetaphoneResult {
        let mut result = DoubleMetaphoneResult::new(self.max_code_length);
        let value = value.trim();
        if value.is_empty() {
            return result;
        }

        let value = &value.to_uppercase();

        let slavo_germanic = Self::is_slavo_germanic(value);

        let mut iterator: Peekable<CharIndices<'_>> = value.char_indices().peekable();
        let mut char_index: Option<(usize, char)> = iterator.next();
        if SILENT_START.iter().any(|sl| value.starts_with(sl)) {
            char_index = iterator.next();
        }
        while !result.is_complete() && char_index.is_some() {
            let (index, ch) = char_index.unwrap();

            let skip = match ch {
                'A' | 'E' | 'I' | 'O' | 'U' | 'Y' => {
                    if index == 0 {
                        result.append_char('A', None);
                    }
                    0
                }
                'B' => {
                    result.append_char('P', None);
                    if Self::char_at(value, index + 1) == Some('B') {
                        1
                    } else {
                        0
                    }
                }
                'Ç' => {
                    result.append_char('S', None);
                    0
                }
                'C' => Self::handle_c(value, &mut result, index),
                'D' => Self::handle_d(value, &mut result, index),
                'F' => {
                    result.append_char('F', None);
                    if Self::char_at(value, index + 1) == Some('F') {
                        1
                    } else {
                        0
                    }
                }
                'G' => Self::handle_g(value, &mut result, index, slavo_germanic),
                'H' => Self::handle_h(value, &mut result, index),
                'J' => Self::handle_j(value, &mut result, index, slavo_germanic),
                'K' => {
                    result.append_char('K', None);
                    if Self::char_at(value, index + 1) == Some('K') {
                        1
                    } else {
                        0
                    }
                }
                'L' => Self::handle_l(value, &mut result, index),
                'M' => {
                    result.append_char('M', None);
                    if Self::condition_m0(value, index) {
                        1
                    } else {
                        0
                    }
                }
                'N' => {
                    result.append_char('N', None);
                    if Self::char_at(value, index + 1) == Some('N') {
                        1
                    } else {
                        0
                    }
                }
                'Ñ' => {
                    result.append_char('N', None);
                    0
                }
                'P' => Self::handle_p(value, &mut result, index),
                'Q' => {
                    result.append_char('K', None);
                    if Self::char_at(value, index + 1) == Some('Q') {
                        1
                    } else {
                        0
                    }
                }
                'R' => Self::handle_r(value, &mut result, index, slavo_germanic),
                'S' => Self::handle_s(value, &mut result, index, slavo_germanic),
                'T' => Self::handle_t(value, &mut result, index),
                'V' => {
                    result.append_char('F', None);
                    if Self::char_at(value, index + 1) == Some('V') {
                        1
                    } else {
                        0
                    }
                }
                'W' => Self::handle_w(value, &mut result, index),
                'X' => Self::handle_x(value, &mut result, index),
                'Z' => Self::handle_z(value, &mut result, index, slavo_germanic),
                _ => 0,
            };

            char_index = iterator.nth(skip);
        }

        result
    }

    fn handle_c(value: &str, result: &mut DoubleMetaphoneResult, index: usize) -> usize {
        if Self::condition_c0(value, index) {
            result.append_char('K', None);
            1
        } else if index == 0 && Self::contains(value, index, 6, vec!["CAESAR"]) {
            result.append_char('S', None);
            1
        } else if Self::contains(value, index, 2, vec!["CH"]) {
            Self::handle_ch(value, result, index)
        } else if Self::contains(value, index, 2, vec!["CZ"])
            && (index < 2 || !Self::contains(value, index - 2, 4, vec!["WICZ"]))
        {
            //-- "Czerny" --//
            result.append_char('S', Some('X'));
            1
        } else if Self::contains(value, index + 1, 3, vec!["CIA"]) {
            //-- "focaccia" --//
            result.append_char('X', None);
            2
        } else if Self::contains(value, index, 2, vec!["CC"])
            && !(index == 1 && Self::char_at(value, 0) == Some('M'))
        {
            //-- double "cc" but not "McClelland" --//
            Self::handle_cc(value, result, index)
        } else if Self::contains(value, index, 2, vec!["CK", "CG", "CQ"]) {
            result.append_char('K', None);
            1
        } else if Self::contains(value, index, 2, vec!["CI", "CE", "CY"]) {
            //-- Italian vs. English --//
            if Self::contains(value, index, 3, vec!["CIO", "CIE", "CIA"]) {
                result.append_char('S', Some('X'));
            } else {
                result.append_char('S', None);
            }
            1
        } else {
            result.append_char('K', None);
            if Self::contains(value, index + 1, 2, vec![" C", " Q", " G"]) {
                //-- Mac Caffrey, Mac Gregor --//
                2
            } else if Self::contains(value, index + 1, 1, vec!["C", "K", "Q"])
                && !Self::contains(value, index + 1, 2, vec!["CE", "CI"])
            {
                1
            } else {
                0
            }
        }
    }

    fn condition_c0(value: &str, index: usize) -> bool {
        if Self::contains(value, index, 4, vec!["CHIA"]) {
            return true;
        }
        if index < 1 {
            return false;
        }
        if index < 2
            || Self::char_at(value, index - 2).map_or(false, |ch| {
                is_vowel(Some(ch).map(|c| c.to_ascii_lowercase()), true)
            })
        {
            return false;
        }

        if index > 0 && !Self::contains(value, index - 1, 3, vec!["ACH"]) {
            return false;
        }

        let ch = Self::char_at(value, index + 2);
        if index < 2 {
            false
        } else {
            ch.map_or(true, |c| c != 'I' && c != 'E')
                || Self::contains(value, index - 2, 6, vec!["BACHER", "MACHER"])
        }
    }

    fn handle_ch(value: &str, result: &mut DoubleMetaphoneResult, index: usize) -> usize {
        if index > 0 && Self::contains(value, index, 4, vec!["CHAE"]) {
            // Michael
            result.append_char('K', Some('X'));
        } else if Self::condition_ch0(value, index) || Self::condition_ch1(value, index) {
            //-- Greek roots ("chemistry", "chorus", etc.) --//
            //-- Germanic, Greek, or otherwise 'ch' for 'kh' sound --//
            result.append_char('K', None);
        } else if index > 0 {
            if Self::contains(value, 0, 2, vec!["MC"]) {
                result.append_char('K', None);
            } else {
                result.append_char('X', Some('K'));
            }
        } else {
            result.append_char('X', None);
        }

        1
    }

    fn condition_ch0(value: &str, index: usize) -> bool {
        if index != 0 {
            return false;
        }

        if !Self::contains(value, index + 1, 5, vec!["HARAC", "HARIS"])
            && !Self::contains(value, index + 1, 3, vec!["HOR", "HYM", "HIA", "HEM"])
        {
            return false;
        }

        !Self::contains(value, 0, 5, vec!["CHORE"])
    }

    fn condition_ch1(value: &str, index: usize) -> bool {
        (Self::contains(value, 0, 4, vec!["VAN", "VON"])
            || Self::contains(value, 0, 3, vec!["SCH"]))
            || (index > 1
                && Self::contains(value, index - 2, 6, vec!["ORCHES", "ARCHIT", "ORCHID"]))
            || (index > 1 && Self::contains(value, index + 2, 1, vec!["T", "S"]))
            || ((index == 0 || Self::contains(value, index - 1, 1, vec!["A", "O", "U", "E"]))
                && (Self::contains_array(value, index + 2, 1, L_R_N_M_B_H_F_V_W_SPACE)
                    || index + 1 == value.len() - 1))
    }

    fn handle_cc(value: &str, result: &mut DoubleMetaphoneResult, index: usize) -> usize {
        if Self::contains(value, index + 2, 1, vec!["I", "E", "H"])
            && !Self::contains(value, index + 2, 2, vec!["HU"])
        {
            //-- "bellocchio" but not "bacchus" --//
            if (index == 1 && Self::char_at(value, index - 1) == Some('A'))
                || Self::contains(value, index - 1, 5, vec!["UCCEE", "UCCES"])
            {
                //-- "accident", "accede", "succeed" --//
                result.append_str("KS", None);
            } else {
                //-- "bacci", "bertucci", other Italian --//
                result.append_char('X', None);
            }
            2
        } else {
            // Pierce's rule
            result.append_char('K', None);
            1
        }
    }

    fn handle_d(value: &str, result: &mut DoubleMetaphoneResult, index: usize) -> usize {
        if Self::contains(value, index, 2, vec!["DG"]) {
            if Self::contains(value, index + 2, 1, vec!["I", "E", "Y"]) {
                result.append_char('J', None);
                2
            } else {
                result.append_str("TK", None);
                1
            }
        } else if Self::contains(value, index, 2, vec!["DT", "DD"]) {
            result.append_char('T', None);
            1
        } else {
            result.append_char('T', None);
            0
        }
    }

    fn handle_g(
        value: &str,
        result: &mut DoubleMetaphoneResult,
        index: usize,
        slavo_germanic: bool,
    ) -> usize {
        if Self::char_at(value, index + 1) == Some('H') {
            Self::handle_gh(value, result, index)
        } else if Self::char_at(value, index + 1) == Some('N') {
            if index == 1
                && is_vowel(
                    Self::char_at(value, 0).map(|c| c.to_ascii_lowercase()),
                    true,
                )
                && !slavo_germanic
            {
                result.append_str("KN", Some("N"));
            } else if !Self::contains(value, index + 2, 2, vec!["EY"])
                && Self::char_at(value, index + 1) != Some('Y')
                && !slavo_germanic
            {
                result.append_str("N", Some("KN"));
            } else {
                result.append_str("KN", None);
            }
            1
        } else if Self::contains(value, index + 1, 2, vec!["LI"]) && !slavo_germanic {
            result.append_str("KL", Some("L"));
            1
        } else if (index == 0
            && (Self::char_at(value, index + 1) == Some('Y')
                || Self::contains_array(value, index + 1, 2, ES_EP_EB_EL_EY_IB_IL_IN_IE_EI_ER)))
            || (Self::contains(value, index + 1, 2, vec!["ER"])
                || Self::char_at(value, index + 1) == Some('Y'))
                && !Self::contains(value, 0, 6, vec!["DANGER", "RANGER", "MANGER"])
                && (index == 0 || !Self::contains(value, index - 1, 1, vec!["E", "I"]))
                && (index == 0 || !Self::contains(value, index - 1, 3, vec!["RGY", "OGY"]))
        {
            //-- -ger-, -gy- --//
            //-- -ges-, -gep-, -gel-, -gie- at beginning --//
            result.append_char('K', Some('J'));
            1
        } else if Self::contains(value, index + 1, 1, vec!["E", "I", "Y"])
            || (index > 0 && Self::contains(value, index - 1, 4, vec!["AGGI", "OGGI"]))
        {
            //-- Italian "biaggi" --//
            if Self::contains(value, 0, 4, vec!["VAN ", "VON "])
                || Self::contains(value, 0, 3, vec!["SCH"])
                || Self::contains(value, index + 1, 2, vec!["ET"])
            {
                //-- obvious germanic --//
                result.append_char('K', None);
            } else if Self::contains(value, index + 1, 3, vec!["IER"]) {
                result.append_char('J', None);
            } else {
                result.append_char('J', Some('K'));
            }
            1
        } else if Self::char_at(value, index + 1) == Some('G') {
            result.append_char('K', None);
            1
        } else {
            result.append_char('K', None);
            0
        }
    }

    fn handle_gh(value: &str, result: &mut DoubleMetaphoneResult, index: usize) -> usize {
        // Unwrap is safe in the first if because index > 0
        if index > 0
            && !(is_vowel(
                Self::char_at(value, index - 1).map(|c| c.to_ascii_lowercase()),
                true,
            ))
        {
            result.append_char('K', None);
            1
        } else if index == 0 {
            if Self::char_at(value, index + 2) == Some('I') {
                result.append_char('J', None);
            } else {
                result.append_char('K', None);
            }
            1
        } else if (index > 1 && Self::contains(value, index - 2, 1, vec!["B", "H", "D"]))
            || (index > 2 && Self::contains(value, index - 3, 1, vec!["B", "H", "D"]))
            || (index > 3 && Self::contains(value, index - 4, 1, vec!["B", "H"]))
        {
            //-- Parker's rule (with some further refinements) - "hugh"
            1
        } else {
            if index > 2
                && Self::char_at(value, index - 1) == Some('U')
                && Self::contains(value, index - 3, 1, vec!["C", "G", "L", "R", "T"])
            {
                //-- "laugh", "McLaughlin", "cough", "gough", "rough", "tough"
                result.append_char('F', None);
            } else if index > 0 && Self::char_at(value, index - 1) != Some('I') {
                result.append_char('K', None);
            }
            1
        }
    }

    fn handle_h(value: &str, result: &mut DoubleMetaphoneResult, index: usize) -> usize {
        //-- only keep if first & before vowel or between 2 vowels --//
        if (index == 0
            || is_vowel(
                Self::char_at(value, index - 1).map(|c| c.to_ascii_lowercase()),
                true,
            ))
            && is_vowel(
                Self::char_at(value, index + 1).map(|c| c.to_ascii_lowercase()),
                true,
            )
        {
            result.append_char('H', None);
            1
            //-- also takes car of "HH" --//
        } else {
            0
        }
    }

    fn handle_j(
        value: &str,
        result: &mut DoubleMetaphoneResult,
        index: usize,
        slavo_germanic: bool,
    ) -> usize {
        if Self::contains(value, index, 4, vec!["JOSE"])
            || Self::contains(value, 0, 4, vec!["SAN "])
        {
            //-- obvious Spanish, "Jose", "San Jacinto" --//
            if (index == 0 && (Self::char_at(value, index + 4) == Some(' ')) || value.len() == 4)
                || Self::contains(value, 0, 4, vec!["SAN "])
            {
                result.append_char('H', None);
            } else {
                result.append_char('J', Some('H'));
            }
            0
        } else {
            if index == 0 && !Self::contains(value, index, 4, vec!["JOSE"]) {
                result.append_char('J', Some('A'));
            } else if index > 0
                && is_vowel(
                    Self::char_at(value, index - 1).map(|c| c.to_ascii_lowercase()),
                    true,
                )
                && !slavo_germanic
                && (Self::char_at(value, index + 1) == Some('A')
                    || Self::char_at(value, index + 1) == Some('O'))
            {
                result.append_char('J', Some('H'));
            } else if index == value.len() - 1 {
                result.append_char('J', Some(' '));
            } else if !Self::contains_array(value, index + 1, 1, L_T_K_S_N_M_B_Z)
                && (index == 0 || !Self::contains(value, index - 1, 1, vec!["S", "K", "L"]))
            {
                result.append_char('J', None);
            }

            if Self::char_at(value, index + 1) == Some('J') {
                1
            } else {
                0
            }
        }
    }

    fn handle_l(value: &str, result: &mut DoubleMetaphoneResult, index: usize) -> usize {
        if Self::char_at(value, index + 1) == Some('L') {
            if Self::condition_l0(value, index) {
                result.append_char_primary('L');
            } else {
                result.append_char('L', None);
            }
            1
        } else {
            result.append_char('L', None);
            0
        }
    }

    fn condition_l0(value: &str, index: usize) -> bool {
        if index == value.len() - 3
            && index > 0
            && Self::contains(value, index - 1, 4, vec!["ILLO", "ILLA", "ALLE"])
        {
            return true;
        }

        ((value.len() > 1 && Self::contains(value, value.len() - 2, 2, vec!["AS", "OS"]))
            || (!value.is_empty() && Self::contains(value, value.len() - 1, 1, vec!["A", "O"])))
            && !value.is_empty()
            && Self::contains(value, index - 1, 4, vec!["ALLE"])
    }

    fn condition_m0(value: &str, index: usize) -> bool {
        if Self::char_at(value, index + 1) == Some('M') {
            return true;
        }

        index > 0
            && Self::contains(value, index - 1, 3, vec!["UMB"])
            && ((index + 1) == value.len() - 1 || Self::contains(value, index + 2, 2, vec!["ER"]))
    }

    fn handle_p(value: &str, result: &mut DoubleMetaphoneResult, index: usize) -> usize {
        if Self::char_at(value, index + 1) == Some('H') {
            result.append_char('F', None);
            1
        } else {
            result.append_char('P', None);
            if Self::contains(value, index + 1, 1, vec!["P", "B"]) {
                1
            } else {
                0
            }
        }
    }

    fn handle_r(
        value: &str,
        result: &mut DoubleMetaphoneResult,
        index: usize,
        slavo_germanic: bool,
    ) -> usize {
        if index > 3
            && index == value.len() - 1
            && !slavo_germanic
            && Self::contains(value, index - 2, 2, vec!["IE"])
            && !Self::contains(value, index - 4, 2, vec!["ME", "MA"])
        {
            result.append_char_alternate('R');
        } else {
            result.append_char('R', None);
        }
        if Self::char_at(value, index + 1) == Some('R') {
            1
        } else {
            0
        }
    }

    fn handle_s(
        value: &str,
        result: &mut DoubleMetaphoneResult,
        index: usize,
        slavo_germanic: bool,
    ) -> usize {
        if index > 0 && Self::contains(value, index - 1, 3, vec!["ISL", "YSL"]) {
            //-- special cases "island", "isle", "carlisle", "carlysle" --//
            0
        } else if index == 0 && Self::contains(value, index, 5, vec!["SUGAR"]) {
            //-- special case "sugar-" --//
            result.append_char('X', Some('S'));
            0
        } else if Self::contains(value, index, 2, vec!["SH"]) {
            if Self::contains(value, index + 1, 4, vec!["HEIM", "HOEK", "HOLM", "HOLZ"]) {
                //-- germanic --//
                result.append_char('S', None);
            } else {
                result.append_char('X', None);
            }
            1
        } else if Self::contains(value, index, 3, vec!["SIO", "SIA"])
            || Self::contains(value, index, 4, vec!["SIAN"])
        {
            //-- Italian and Armenian --//
            if slavo_germanic {
                result.append_char('S', None);
            } else {
                result.append_char('S', Some('X'));
            }
            2
        } else if (index == 0 && Self::contains(value, index + 1, 1, vec!["M", "N", "L", "W"]))
            || Self::contains(value, index + 1, 1, vec!["Z"])
        {
            //-- german & anglicisations, e.g. "smith" match "schmidt" //
            // "snider" match "schneider" --//
            //-- also, -sz- in slavic language although in hungarian it //
            //   is pronounced "s" --//
            result.append_char('S', Some('X'));
            if Self::contains(value, index + 1, 1, vec!["Z"]) {
                1
            } else {
                0
            }
        } else if Self::contains(value, index, 2, vec!["SC"]) {
            Self::handle_sc(value, result, index)
        } else {
            if index > 1
                && index == value.len() - 1
                && Self::contains(value, index - 2, 2, vec!["AI", "OI"])
            {
                //-- french e.g. "resnais", "artois" --//
                result.append_char_alternate('S');
            } else {
                result.append_char('S', None);
            }
            if Self::contains(value, index + 1, 1, vec!["S", "Z"]) {
                1
            } else {
                0
            }
        }
    }

    fn handle_sc(value: &str, result: &mut DoubleMetaphoneResult, index: usize) -> usize {
        if Self::char_at(value, index + 2) == Some('H') {
            //-- Schlesinger's rule --//
            if Self::contains(
                value,
                index + 3,
                2,
                vec!["OO", "ER", "EN", "UY", "ED", "EM"],
            ) {
                //-- Dutch origin, e.g. "school", "schooner" --//
                if Self::contains(value, index + 3, 2, vec!["ER", "EN"]) {
                    //-- "schermerhorn", "schenker" --//
                    result.append_str("X", Some("SK"));
                } else {
                    result.append_str("SK", None);
                }
            } else if index == 0
                && !is_vowel(
                    Self::char_at(value, 3).map(|c| c.to_ascii_lowercase()),
                    true,
                )
                && Self::char_at(value, 3) != Some('W')
            {
                result.append_char('X', Some('S'));
            } else {
                result.append_char('X', None);
            }
        } else if Self::contains(value, index + 2, 1, vec!["I", "E", "Y"]) {
            result.append_char('S', None);
        } else {
            result.append_str("SK", None);
        }
        2
    }

    fn handle_t(value: &str, result: &mut DoubleMetaphoneResult, index: usize) -> usize {
        if Self::contains(value, index, 4, vec!["TION"])
            || Self::contains(value, index, 3, vec!["TIA", "TCH"])
        {
            result.append_char('X', None);
            2
        } else if Self::contains(value, index, 2, vec!["TH"])
            || Self::contains(value, index, 3, vec!["TTH"])
        {
            if Self::contains(value, index + 2, 2, vec!["OM", "AM"]) ||
                //-- special case "thomas", "thames" or germanic --//
                Self::contains(value, 0, 4, vec!["VAN ", "VON "]) ||
                Self::contains(value, 0, 3, vec!["SCH"])
            {
                result.append_char('T', None);
            } else {
                result.append_char('0', Some('T'));
            }
            1
        } else {
            result.append_char('T', None);
            if Self::contains(value, index + 1, 1, vec!["T", "D"]) {
                1
            } else {
                0
            }
        }
    }

    fn handle_w(value: &str, result: &mut DoubleMetaphoneResult, index: usize) -> usize {
        if Self::contains(value, index, 2, vec!["WR"]) {
            //-- can also be in middle of word --//
            result.append_char('R', None);
            1
        } else if index == 0
            && (is_vowel(
                Self::char_at(value, index + 1).map(|c| c.to_ascii_lowercase()),
                true,
            ) || Self::contains(value, index, 2, vec!["WH"]))
        {
            if is_vowel(
                Self::char_at(value, index + 1).map(|c| c.to_ascii_lowercase()),
                true,
            ) {
                //-- Wasserman should match Vasserman --//
                result.append_char('A', Some('F'));
            } else {
                //-- need Uomo to match Womo --//
                result.append_char('A', None);
            }
            0
        } else if (index > 0
            && index == value.len() - 1
            && is_vowel(
                Self::char_at(value, index - 1).map(|c| c.to_ascii_lowercase()),
                true,
            ))
            || (index > 0
                && Self::contains(
                    value,
                    index - 1,
                    5,
                    vec!["EWSKI", "EWSKY", "OWSKI", "OWSKY"],
                ))
            || Self::contains(value, 0, 3, vec!["SCH"])
        {
            //-- Arnow should match Arnoff --//
            result.append_char_alternate('F');
            0
        } else if Self::contains(value, index, 4, vec!["WICZ", "WITZ"]) {
            //-- Polish e.g. "filipowicz" --//
            result.append_str("TS", Some("FX"));
            3
        } else {
            0
        }
    }

    fn handle_x(value: &str, result: &mut DoubleMetaphoneResult, index: usize) -> usize {
        if index == 0 {
            result.append_char('S', None);
            0
        } else {
            if !((index == value.len() - 1)
                && ((index > 2 && Self::contains(value, index - 3, 3, vec!["IAU", "EAU"]))
                    || (index > 1 && Self::contains(value, index - 2, 2, vec!["AU", "OU"]))))
            {
                //-- French e.g. breaux --//
                result.append_str("KS", None);
            }
            if Self::contains(value, index + 1, 1, vec!["C", "X"]) {
                1
            } else {
                0
            }
        }
    }

    fn handle_z(
        value: &str,
        result: &mut DoubleMetaphoneResult,
        index: usize,
        slavo_germanic: bool,
    ) -> usize {
        if Self::char_at(value, index + 1) == Some('H') {
            //-- Chinese pinyin e.g. "zhao" or Angelina "Zhang" --//
            result.append_char('J', None);
            1
        } else {
            if Self::contains(value, index + 1, 2, vec!["ZO", "ZI", "ZA"])
                || (slavo_germanic && (index > 0 && Self::char_at(value, index - 1) != Some('T')))
            {
                result.append_str("S", Some("TS"));
            } else {
                result.append_char('S', None);
            }
            if Self::char_at(value, index + 1) == Some('Z') {
                1
            } else {
                0
            }
        }
    }
}

impl Encoder for DoubleMetaphone {
    /// Encode the `value` and return primary code.
    ///
    /// # Parameter
    ///
    /// * `value` : value to encode.
    ///
    /// # Result
    ///
    /// Returns the value's primary code.
    fn encode(&self, value: &str) -> String {
        self.double_metaphone(value).primary
    }
}

#[cfg(test)]
mod tests {
    use crate::{DoubleMetaphone, Encoder};

    /**
     * Test data from http://aspell.net/test/orig/batch0.tab.
     *
     * "Copyright (C) 2002 Kevin Atkinson (kevina@gnu.org). Verbatim copying
     * and distribution of this entire article is permitted in any medium,
     * provided this notice is preserved."
     *
     * Massaged the test data in the array below.
     */
    const FIXTURE: [(&str, &str); 547] = [
        ("Accosinly", "Occasionally"),
        ("Ciculer", "Circler"),
        ("Circue", "Circle"),
        ("Maddness", "Madness"),
        ("Occusionaly", "Occasionally"),
        ("Steffen", "Stephen"),
        ("Thw", "The"),
        ("Unformanlly", "Unfortunately"),
        ("Unfortally", "Unfortunately"),
        ("abilitey", "ability"),
        ("abouy", "about"),
        ("absorbtion", "absorption"),
        ("accidently", "accidentally"),
        ("accomodate", "accommodate"),
        ("acommadate", "accommodate"),
        ("acord", "accord"),
        ("adultry", "adultery"),
        ("aggresive", "aggressive"),
        ("alchohol", "alcohol"),
        ("alchoholic", "alcoholic"),
        ("allieve", "alive"),
        ("alot", "a lot"),
        ("alright", "all right"),
        ("amature", "amateur"),
        ("ambivilant", "ambivalent"),
        ("amification", "amplification"),
        ("amourfous", "amorphous"),
        ("annoint", "anoint"),
        ("annonsment", "announcement"),
        ("annoyting", "anting"),
        ("annuncio", "announce"),
        ("anonomy", "anatomy"),
        ("anotomy", "anatomy"),
        (
            "antidesestablishmentarianism",
            "antidisestablishmentarianism",
        ),
        ("antidisestablishmentarism", "antidisestablishmentarianism"),
        ("anynomous", "anonymous"),
        ("appelet", "applet"),
        ("appreceiated", "appreciated"),
        ("appresteate", "appreciate"),
        ("aquantance", "acquaintance"),
        ("aratictature", "architecture"),
        ("archeype", "archetype"),
        ("aricticure", "architecture"),
        ("artic", "arctic"),
        ("asentote", "asymptote"),
        ("ast", "at"),
        ("asterick", "asterisk"),
        ("asymetric", "asymmetric"),
        ("atentively", "attentively"),
        ("autoamlly", "automatically"),
        ("bankrot", "bankrupt"),
        ("basicly", "basically"),
        ("batallion", "battalion"),
        ("bbrose", "browse"),
        ("beauro", "bureau"),
        ("beaurocracy", "bureaucracy"),
        ("beggining", "beginning"),
        ("beging", "beginning"),
        ("behaviour", "behavior"),
        ("beleive", "believe"),
        ("belive", "believe"),
        ("benidifs", "benefits"),
        ("bigginging", "beginning"),
        ("blait", "bleat"),
        ("bouyant", "buoyant"),
        ("boygot", "boycott"),
        ("brocolli", "broccoli"),
        ("buch", "bush"),
        ("buder", "butter"),
        ("budr", "butter"),
        ("budter", "butter"),
        ("buracracy", "bureaucracy"),
        ("burracracy", "bureaucracy"),
        ("buton", "button"),
        ("byby", "by by"),
        ("cauler", "caller"),
        ("ceasar", "caesar"),
        ("cemetary", "cemetery"),
        ("changeing", "changing"),
        ("cheet", "cheat"),
        ("cicle", "circle"),
        ("cimplicity", "simplicity"),
        ("circumstaces", "circumstances"),
        ("clob", "club"),
        ("coaln", "colon"),
        ("cocamena", "cockamamie"),
        ("colleaque", "colleague"),
        ("colloquilism", "colloquialism"),
        ("columne", "column"),
        ("comiler", "compiler"),
        ("comitmment", "commitment"),
        ("comitte", "committee"),
        ("comittmen", "commitment"),
        ("comittmend", "commitment"),
        ("commerciasl", "commercials"),
        ("commited", "committed"),
        ("commitee", "committee"),
        ("companys", "companies"),
        ("compicated", "complicated"),
        ("comupter", "computer"),
        ("concensus", "consensus"),
        ("confusionism", "confucianism"),
        ("congradulations", "congratulations"),
        ("conibation", "contribution"),
        ("consident", "consistent"),
        ("consident", "consonant"),
        ("contast", "constant"),
        ("contastant", "constant"),
        ("contunie", "continue"),
        ("cooly", "coolly"),
        ("copping", "coping"),
        ("cosmoplyton", "cosmopolitan"),
        ("courst", "court"),
        ("crasy", "crazy"),
        ("cravets", "caveats"),
        ("credetability", "credibility"),
        ("criqitue", "critique"),
        ("croke", "croak"),
        ("crucifiction", "crucifixion"),
        ("crusifed", "crucified"),
        ("ctitique", "critique"),
        ("cumba", "combo"),
        ("custamisation", "customization"),
        ("dag", "dog"),
        ("daly", "daily"),
        ("danguages", "dangerous"),
        ("deaft", "draft"),
        ("defence", "defense"),
        ("defenly", "defiantly"),
        ("definate", "definite"),
        ("definately", "definitely"),
        ("dependeble", "dependable"),
        ("descrption", "description"),
        ("descrptn", "description"),
        ("desparate", "desperate"),
        ("dessicate", "desiccate"),
        ("destint", "distant"),
        ("develepment", "developments"),
        ("developement", "development"),
        ("develpond", "development"),
        ("devulge", "divulge"),
        ("diagree", "disagree"),
        ("dieties", "deities"),
        ("dinasaur", "dinosaur"),
        ("dinasour", "dinosaur"),
        ("direcyly", "directly"),
        ("discuess", "discuss"),
        ("disect", "dissect"),
        ("disippate", "dissipate"),
        ("disition", "decision"),
        ("dispair", "despair"),
        ("disssicion", "discussion"),
        ("distarct", "distract"),
        ("distart", "distort"),
        ("distroy", "destroy"),
        ("documtations", "documentation"),
        ("doenload", "download"),
        ("dongle", "dangle"),
        ("doog", "dog"),
        ("dramaticly", "dramatically"),
        ("drunkeness", "drunkenness"),
        ("ductioneery", "dictionary"),
        ("dur", "due"),
        ("duren", "during"),
        ("dymatic", "dynamic"),
        ("dynaic", "dynamic"),
        ("ecstacy", "ecstasy"),
        ("efficat", "efficient"),
        ("efficity", "efficacy"),
        ("effots", "efforts"),
        ("egsistence", "existence"),
        ("eitiology", "etiology"),
        ("elagent", "elegant"),
        ("elligit", "elegant"),
        ("embarass", "embarrass"),
        ("embarassment", "embarrassment"),
        ("embaress", "embarrass"),
        ("encapsualtion", "encapsulation"),
        ("encyclapidia", "encyclopedia"),
        ("encyclopia", "encyclopedia"),
        ("engins", "engine"),
        ("enhence", "enhance"),
        ("enligtment", "Enlightenment"),
        ("ennuui", "ennui"),
        ("enought", "enough"),
        ("enventions", "inventions"),
        ("envireminakl", "environmental"),
        ("enviroment", "environment"),
        ("epitomy", "epitome"),
        ("equire", "acquire"),
        ("errara", "error"),
        ("erro", "error"),
        ("evaualtion", "evaluation"),
        ("evething", "everything"),
        ("evtually", "eventually"),
        ("excede", "exceed"),
        ("excercise", "exercise"),
        ("excpt", "except"),
        ("excution", "execution"),
        ("exhileration", "exhilaration"),
        ("existance", "existence"),
        ("expleyly", "explicitly"),
        ("explity", "explicitly"),
        ("expresso", "espresso"),
        ("exspidient", "expedient"),
        ("extions", "extensions"),
        ("factontion", "factorization"),
        ("failer", "failure"),
        ("famdasy", "fantasy"),
        ("faver", "favor"),
        ("faxe", "fax"),
        ("febuary", "february"),
        ("firey", "fiery"),
        ("fistival", "festival"),
        ("flatterring", "flattering"),
        ("fluk", "flux"),
        ("flukse", "flux"),
        ("fone", "phone"),
        ("forsee", "foresee"),
        ("frustartaion", "frustrating"),
        ("fuction", "function"),
        ("funetik", "phonetic"),
        ("futs", "guts"),
        ("gamne", "came"),
        ("gaurd", "guard"),
        ("generly", "generally"),
        ("ghandi", "gandhi"),
        ("goberment", "government"),
        ("gobernement", "government"),
        ("gobernment", "government"),
        ("gotton", "gotten"),
        ("gracefull", "graceful"),
        ("gradualy", "gradually"),
        ("grammer", "grammar"),
        ("hallo", "hello"),
        ("hapily", "happily"),
        ("harrass", "harass"),
        ("havne", "have"),
        ("heellp", "help"),
        ("heighth", "height"),
        ("hellp", "help"),
        ("helo", "hello"),
        ("herlo", "hello"),
        ("hifin", "hyphen"),
        ("hifine", "hyphen"),
        ("higer", "higher"),
        ("hiphine", "hyphen"),
        ("hippie", "hippy"),
        ("hippopotamous", "hippopotamus"),
        ("hlp", "help"),
        ("hourse", "horse"),
        ("houssing", "housing"),
        ("howaver", "however"),
        ("howver", "however"),
        ("humaniti", "humanity"),
        ("hyfin", "hyphen"),
        ("hypotathes", "hypothesis"),
        ("hypotathese", "hypothesis"),
        ("hystrical", "hysterical"),
        ("ident", "indent"),
        ("illegitament", "illegitimate"),
        ("imbed", "embed"),
        ("imediaetly", "immediately"),
        ("imfamy", "infamy"),
        ("immenant", "immanent"),
        ("implemtes", "implements"),
        ("inadvertant", "inadvertent"),
        ("incase", "in case"),
        ("incedious", "insidious"),
        ("incompleet", "incomplete"),
        ("incomplot", "incomplete"),
        ("inconvenant", "inconvenient"),
        ("inconvience", "inconvenience"),
        ("independant", "independent"),
        ("independenent", "independent"),
        ("indepnends", "independent"),
        ("indepth", "in depth"),
        ("indispensible", "indispensable"),
        ("inefficite", "inefficient"),
        ("inerface", "interface"),
        ("infact", "in fact"),
        ("influencial", "influential"),
        ("inital", "initial"),
        ("initinized", "initialized"),
        ("initized", "initialized"),
        ("innoculate", "inoculate"),
        ("insistant", "insistent"),
        ("insistenet", "insistent"),
        ("instulation", "installation"),
        ("intealignt", "intelligent"),
        ("intejilent", "intelligent"),
        ("intelegent", "intelligent"),
        ("intelegnent", "intelligent"),
        ("intelejent", "intelligent"),
        ("inteligent", "intelligent"),
        ("intelignt", "intelligent"),
        ("intellagant", "intelligent"),
        ("intellegent", "intelligent"),
        ("intellegint", "intelligent"),
        ("intellgnt", "intelligent"),
        ("intensionality", "intensionally"),
        ("interate", "iterate"),
        ("internation", "international"),
        ("interpretate", "interpret"),
        ("interpretter", "interpreter"),
        ("intertes", "interested"),
        ("intertesd", "interested"),
        ("invermeantial", "environmental"),
        ("irregardless", "regardless"),
        ("irresistable", "irresistible"),
        ("irritible", "irritable"),
        ("islams", "muslims"),
        ("isotrop", "isotope"),
        ("isreal", "israel"),
        ("johhn", "john"),
        ("judgement", "judgment"),
        ("kippur", "kipper"),
        ("knawing", "knowing"),
        ("latext", "latest"),
        ("leasve", "leave"),
        ("lesure", "leisure"),
        ("liasion", "lesion"),
        ("liason", "liaison"),
        ("libary", "library"),
        ("likly", "likely"),
        ("lilometer", "kilometer"),
        ("liquify", "liquefy"),
        ("lloyer", "layer"),
        ("lossing", "losing"),
        ("luser", "laser"),
        ("maintanence", "maintenance"),
        ("majaerly", "majority"),
        ("majoraly", "majority"),
        ("maks", "masks"),
        ("mandelbrot", "Mandelbrot"),
        ("mant", "want"),
        ("marshall", "marshal"),
        ("maxium", "maximum"),
        ("meory", "memory"),
        ("metter", "better"),
        ("mic", "mike"),
        ("midia", "media"),
        ("millenium", "millennium"),
        ("miniscule", "minuscule"),
        ("minkay", "monkey"),
        ("minum", "minimum"),
        ("mischievious", "mischievous"),
        ("misilous", "miscellaneous"),
        ("momento", "memento"),
        ("monkay", "monkey"),
        ("mosaik", "mosaic"),
        ("mostlikely", "most likely"),
        ("mousr", "mouser"),
        ("mroe", "more"),
        ("neccessary", "necessary"),
        ("necesary", "necessary"),
        ("necesser", "necessary"),
        ("neice", "niece"),
        ("neighbour", "neighbor"),
        ("nemonic", "pneumonic"),
        ("nevade", "Nevada"),
        ("nickleodeon", "nickelodeon"),
        ("nieve", "naive"),
        ("noone", "no one"),
        ("noticably", "noticeably"),
        ("notin", "not in"),
        ("nozled", "nuzzled"),
        ("objectsion", "objects"),
        ("obsfuscate", "obfuscate"),
        ("ocassion", "occasion"),
        ("occuppied", "occupied"),
        ("occurence", "occurrence"),
        ("octagenarian", "octogenarian"),
        ("olf", "old"),
        ("opposim", "opossum"),
        ("organise", "organize"),
        ("organiz", "organize"),
        ("orientate", "orient"),
        ("oscilascope", "oscilloscope"),
        ("oving", "moving"),
        ("paramers", "parameters"),
        ("parametic", "parameter"),
        ("paranets", "parameters"),
        ("partrucal", "particular"),
        ("pataphysical", "metaphysical"),
        ("patten", "pattern"),
        ("permissable", "permissible"),
        ("permition", "permission"),
        ("permmasivie", "permissive"),
        ("perogative", "prerogative"),
        ("persue", "pursue"),
        ("phantasia", "fantasia"),
        ("phenominal", "phenomenal"),
        ("picaresque", "picturesque"),
        ("playwrite", "playwright"),
        ("poeses", "poesies"),
        ("polation", "politician"),
        ("poligamy", "polygamy"),
        ("politict", "politic"),
        ("pollice", "police"),
        ("polypropalene", "polypropylene"),
        ("pompom", "pompon"),
        ("possable", "possible"),
        ("practicle", "practical"),
        ("pragmaticism", "pragmatism"),
        ("preceeding", "preceding"),
        ("precion", "precision"),
        ("precios", "precision"),
        ("preemptory", "peremptory"),
        ("prefices", "prefixes"),
        ("prefixt", "prefixed"),
        ("presbyterian", "Presbyterian"),
        ("presue", "pursue"),
        ("presued", "pursued"),
        ("privielage", "privilege"),
        ("priviledge", "privilege"),
        ("proceedures", "procedures"),
        ("pronensiation", "pronunciation"),
        ("pronisation", "pronunciation"),
        ("pronounciation", "pronunciation"),
        ("properally", "properly"),
        ("proplematic", "problematic"),
        ("protray", "portray"),
        ("pscolgst", "psychologist"),
        ("psicolagest", "psychologist"),
        ("psycolagest", "psychologist"),
        ("quoz", "quiz"),
        ("radious", "radius"),
        ("ramplily", "rampantly"),
        ("reccomend", "recommend"),
        ("reccona", "raccoon"),
        ("recieve", "receive"),
        ("reconise", "recognize"),
        ("rectangeles", "rectangle"),
        ("redign", "redesign"),
        ("reoccurring", "recurring"),
        ("repitition", "repetition"),
        ("replasments", "replacement"),
        ("reposable", "responsible"),
        ("reseblence", "resemblance"),
        ("respct", "respect"),
        ("respecally", "respectfully"),
        ("roon", "room"),
        ("rought", "roughly"),
        ("rsx", "RSX"),
        ("rudemtry", "rudimentary"),
        ("runnung", "running"),
        ("sacreligious", "sacrilegious"),
        ("saftly", "safely"),
        ("salut", "salute"),
        ("satifly", "satisfy"),
        ("scrabdle", "scrabble"),
        ("searcheable", "searchable"),
        ("secion", "section"),
        ("seferal", "several"),
        ("segements", "segments"),
        ("sence", "sense"),
        ("seperate", "separate"),
        ("sherbert", "sherbet"),
        ("sicolagest", "psychologist"),
        ("sieze", "seize"),
        ("simpfilty", "simplicity"),
        ("simplye", "simply"),
        ("singal", "signal"),
        ("sitte", "site"),
        ("situration", "situation"),
        ("slyph", "sylph"),
        ("smil", "smile"),
        ("snuck", "sneaked"),
        ("sometmes", "sometimes"),
        ("soonec", "sonic"),
        ("specificialy", "specifically"),
        ("spel", "spell"),
        ("spoak", "spoke"),
        ("sponsered", "sponsored"),
        ("stering", "steering"),
        ("straightjacket", "straitjacket"),
        ("stumach", "stomach"),
        ("stutent", "student"),
        ("styleguide", "style guide"),
        ("subisitions", "substitutions"),
        ("subjecribed", "subscribed"),
        ("subpena", "subpoena"),
        ("substations", "substitutions"),
        ("suger", "sugar"),
        ("supercede", "supersede"),
        ("superfulous", "superfluous"),
        ("susan", "Susan"),
        ("swimwear", "swim wear"),
        ("syncorization", "synchronization"),
        ("taff", "tough"),
        ("taht", "that"),
        ("tattos", "tattoos"),
        ("techniquely", "technically"),
        ("teh", "the"),
        ("tem", "team"),
        ("teo", "two"),
        ("teridical", "theoretical"),
        ("tesst", "test"),
        ("tets", "tests"),
        ("thanot", "than or"),
        ("theirselves", "themselves"),
        ("theridically", "theoretical"),
        ("thredically", "theoretically"),
        ("thruout", "throughout"),
        ("ths", "this"),
        ("titalate", "titillate"),
        ("tobagan", "tobaggon"),
        ("tommorrow", "tomorrow"),
        ("tomorow", "tomorrow"),
        ("tradegy", "tragedy"),
        ("trubbel", "trouble"),
        ("ttest", "test"),
        ("tunnellike", "tunnel like"),
        ("tured", "turned"),
        ("tyrrany", "tyranny"),
        ("unatourral", "unnatural"),
        ("unaturral", "unnatural"),
        ("unconisitional", "unconstitutional"),
        ("unconscience", "unconscious"),
        ("underladder", "under ladder"),
        ("unentelegible", "unintelligible"),
        ("unfortunently", "unfortunately"),
        ("unnaturral", "unnatural"),
        ("upcast", "up cast"),
        ("upmost", "utmost"),
        ("uranisium", "uranium"),
        ("verison", "version"),
        ("vinagarette", "vinaigrette"),
        ("volumptuous", "voluptuous"),
        ("volunteerism", "voluntarism"),
        ("volye", "volley"),
        ("wadting", "wasting"),
        ("waite", "wait"),
        ("wan't", "won't"),
        ("warloord", "warlord"),
        ("whaaat", "what"),
        ("whard", "ward"),
        ("whimp", "wimp"),
        ("wicken", "weaken"),
        ("wierd", "weird"),
        ("wrank", "rank"),
        ("writeen", "righten"),
        ("writting", "writing"),
        ("wundeews", "windows"),
        ("yeild", "yield"),
        ("youe", "your"),
    ];

    const MATCHES: [(&str, &str); 406] = [
        ("Accosinly", "Occasionally"),
        ("Maddness", "Madness"),
        ("Occusionaly", "Occasionally"),
        ("Steffen", "Stephen"),
        ("Thw", "The"),
        ("Unformanlly", "Unfortunately"),
        ("Unfortally", "Unfortunately"),
        ("abilitey", "ability"),
        ("absorbtion", "absorption"),
        ("accidently", "accidentally"),
        ("accomodate", "accommodate"),
        ("acommadate", "accommodate"),
        ("acord", "accord"),
        ("adultry", "adultery"),
        ("aggresive", "aggressive"),
        ("alchohol", "alcohol"),
        ("alchoholic", "alcoholic"),
        ("allieve", "alive"),
        ("alot", "a lot"),
        ("alright", "all right"),
        ("amature", "amateur"),
        ("ambivilant", "ambivalent"),
        ("amourfous", "amorphous"),
        ("annoint", "anoint"),
        ("annonsment", "announcement"),
        ("annoyting", "anting"),
        ("annuncio", "announce"),
        ("anotomy", "anatomy"),
        (
            "antidesestablishmentarianism",
            "antidisestablishmentarianism",
        ),
        ("antidisestablishmentarism", "antidisestablishmentarianism"),
        ("anynomous", "anonymous"),
        ("appelet", "applet"),
        ("appreceiated", "appreciated"),
        ("appresteate", "appreciate"),
        ("aquantance", "acquaintance"),
        ("aricticure", "architecture"),
        ("asterick", "asterisk"),
        ("asymetric", "asymmetric"),
        ("atentively", "attentively"),
        ("bankrot", "bankrupt"),
        ("basicly", "basically"),
        ("batallion", "battalion"),
        ("bbrose", "browse"),
        ("beauro", "bureau"),
        ("beaurocracy", "bureaucracy"),
        ("beggining", "beginning"),
        ("behaviour", "behavior"),
        ("beleive", "believe"),
        ("belive", "believe"),
        ("blait", "bleat"),
        ("bouyant", "buoyant"),
        ("boygot", "boycott"),
        ("brocolli", "broccoli"),
        ("buder", "butter"),
        ("budr", "butter"),
        ("budter", "butter"),
        ("buracracy", "bureaucracy"),
        ("burracracy", "bureaucracy"),
        ("buton", "button"),
        ("byby", "by by"),
        ("cauler", "caller"),
        ("ceasar", "caesar"),
        ("cemetary", "cemetery"),
        ("changeing", "changing"),
        ("cheet", "cheat"),
        ("cimplicity", "simplicity"),
        ("circumstaces", "circumstances"),
        ("clob", "club"),
        ("coaln", "colon"),
        ("colleaque", "colleague"),
        ("colloquilism", "colloquialism"),
        ("columne", "column"),
        ("comitmment", "commitment"),
        ("comitte", "committee"),
        ("comittmen", "commitment"),
        ("comittmend", "commitment"),
        ("commerciasl", "commercials"),
        ("commited", "committed"),
        ("commitee", "committee"),
        ("companys", "companies"),
        ("comupter", "computer"),
        ("concensus", "consensus"),
        ("confusionism", "confucianism"),
        ("congradulations", "congratulations"),
        ("contunie", "continue"),
        ("cooly", "coolly"),
        ("copping", "coping"),
        ("cosmoplyton", "cosmopolitan"),
        ("crasy", "crazy"),
        ("croke", "croak"),
        ("crucifiction", "crucifixion"),
        ("crusifed", "crucified"),
        ("cumba", "combo"),
        ("custamisation", "customization"),
        ("dag", "dog"),
        ("daly", "daily"),
        ("defence", "defense"),
        ("definate", "definite"),
        ("definately", "definitely"),
        ("dependeble", "dependable"),
        ("descrption", "description"),
        ("descrptn", "description"),
        ("desparate", "desperate"),
        ("dessicate", "desiccate"),
        ("destint", "distant"),
        ("develepment", "developments"),
        ("developement", "development"),
        ("develpond", "development"),
        ("devulge", "divulge"),
        ("dieties", "deities"),
        ("dinasaur", "dinosaur"),
        ("dinasour", "dinosaur"),
        ("discuess", "discuss"),
        ("disect", "dissect"),
        ("disippate", "dissipate"),
        ("disition", "decision"),
        ("dispair", "despair"),
        ("distarct", "distract"),
        ("distart", "distort"),
        ("distroy", "destroy"),
        ("doenload", "download"),
        ("dongle", "dangle"),
        ("doog", "dog"),
        ("dramaticly", "dramatically"),
        ("drunkeness", "drunkenness"),
        ("ductioneery", "dictionary"),
        ("ecstacy", "ecstasy"),
        ("egsistence", "existence"),
        ("eitiology", "etiology"),
        ("elagent", "elegant"),
        ("embarass", "embarrass"),
        ("embarassment", "embarrassment"),
        ("embaress", "embarrass"),
        ("encapsualtion", "encapsulation"),
        ("encyclapidia", "encyclopedia"),
        ("encyclopia", "encyclopedia"),
        ("engins", "engine"),
        ("enhence", "enhance"),
        ("ennuui", "ennui"),
        ("enventions", "inventions"),
        ("envireminakl", "environmental"),
        ("enviroment", "environment"),
        ("epitomy", "epitome"),
        ("equire", "acquire"),
        ("errara", "error"),
        ("evaualtion", "evaluation"),
        ("excede", "exceed"),
        ("excercise", "exercise"),
        ("excpt", "except"),
        ("exhileration", "exhilaration"),
        ("existance", "existence"),
        ("expleyly", "explicitly"),
        ("explity", "explicitly"),
        ("failer", "failure"),
        ("faver", "favor"),
        ("faxe", "fax"),
        ("firey", "fiery"),
        ("fistival", "festival"),
        ("flatterring", "flattering"),
        ("flukse", "flux"),
        ("fone", "phone"),
        ("forsee", "foresee"),
        ("frustartaion", "frustrating"),
        ("funetik", "phonetic"),
        ("gaurd", "guard"),
        ("generly", "generally"),
        ("ghandi", "gandhi"),
        ("gotton", "gotten"),
        ("gracefull", "graceful"),
        ("gradualy", "gradually"),
        ("grammer", "grammar"),
        ("hallo", "hello"),
        ("hapily", "happily"),
        ("harrass", "harass"),
        ("heellp", "help"),
        ("heighth", "height"),
        ("hellp", "help"),
        ("helo", "hello"),
        ("hifin", "hyphen"),
        ("hifine", "hyphen"),
        ("hiphine", "hyphen"),
        ("hippie", "hippy"),
        ("hippopotamous", "hippopotamus"),
        ("hourse", "horse"),
        ("houssing", "housing"),
        ("howaver", "however"),
        ("howver", "however"),
        ("humaniti", "humanity"),
        ("hyfin", "hyphen"),
        ("hystrical", "hysterical"),
        ("illegitament", "illegitimate"),
        ("imbed", "embed"),
        ("imediaetly", "immediately"),
        ("immenant", "immanent"),
        ("implemtes", "implements"),
        ("inadvertant", "inadvertent"),
        ("incase", "in case"),
        ("incedious", "insidious"),
        ("incompleet", "incomplete"),
        ("incomplot", "incomplete"),
        ("inconvenant", "inconvenient"),
        ("inconvience", "inconvenience"),
        ("independant", "independent"),
        ("independenent", "independent"),
        ("indepnends", "independent"),
        ("indepth", "in depth"),
        ("indispensible", "indispensable"),
        ("inefficite", "inefficient"),
        ("infact", "in fact"),
        ("influencial", "influential"),
        ("innoculate", "inoculate"),
        ("insistant", "insistent"),
        ("insistenet", "insistent"),
        ("instulation", "installation"),
        ("intealignt", "intelligent"),
        ("intelegent", "intelligent"),
        ("intelegnent", "intelligent"),
        ("intelejent", "intelligent"),
        ("inteligent", "intelligent"),
        ("intelignt", "intelligent"),
        ("intellagant", "intelligent"),
        ("intellegent", "intelligent"),
        ("intellegint", "intelligent"),
        ("intellgnt", "intelligent"),
        ("intensionality", "intensionally"),
        ("internation", "international"),
        ("interpretate", "interpret"),
        ("interpretter", "interpreter"),
        ("intertes", "interested"),
        ("intertesd", "interested"),
        ("invermeantial", "environmental"),
        ("irresistable", "irresistible"),
        ("irritible", "irritable"),
        ("isreal", "israel"),
        ("johhn", "john"),
        ("kippur", "kipper"),
        ("knawing", "knowing"),
        ("lesure", "leisure"),
        ("liasion", "lesion"),
        ("liason", "liaison"),
        ("likly", "likely"),
        ("liquify", "liquefy"),
        ("lloyer", "layer"),
        ("lossing", "losing"),
        ("luser", "laser"),
        ("maintanence", "maintenance"),
        ("mandelbrot", "Mandelbrot"),
        ("marshall", "marshal"),
        ("maxium", "maximum"),
        ("mic", "mike"),
        ("midia", "media"),
        ("millenium", "millennium"),
        ("miniscule", "minuscule"),
        ("minkay", "monkey"),
        ("mischievious", "mischievous"),
        ("momento", "memento"),
        ("monkay", "monkey"),
        ("mosaik", "mosaic"),
        ("mostlikely", "most likely"),
        ("mousr", "mouser"),
        ("mroe", "more"),
        ("necesary", "necessary"),
        ("necesser", "necessary"),
        ("neice", "niece"),
        ("neighbour", "neighbor"),
        ("nemonic", "pneumonic"),
        ("nevade", "Nevada"),
        ("nickleodeon", "nickelodeon"),
        ("nieve", "naive"),
        ("noone", "no one"),
        ("notin", "not in"),
        ("nozled", "nuzzled"),
        ("objectsion", "objects"),
        ("ocassion", "occasion"),
        ("occuppied", "occupied"),
        ("occurence", "occurrence"),
        ("octagenarian", "octogenarian"),
        ("opposim", "opossum"),
        ("organise", "organize"),
        ("organiz", "organize"),
        ("orientate", "orient"),
        ("oscilascope", "oscilloscope"),
        ("parametic", "parameter"),
        ("permissable", "permissible"),
        ("permmasivie", "permissive"),
        ("persue", "pursue"),
        ("phantasia", "fantasia"),
        ("phenominal", "phenomenal"),
        ("playwrite", "playwright"),
        ("poeses", "poesies"),
        ("poligamy", "polygamy"),
        ("politict", "politic"),
        ("pollice", "police"),
        ("polypropalene", "polypropylene"),
        ("possable", "possible"),
        ("practicle", "practical"),
        ("pragmaticism", "pragmatism"),
        ("preceeding", "preceding"),
        ("precios", "precision"),
        ("preemptory", "peremptory"),
        ("prefixt", "prefixed"),
        ("presbyterian", "Presbyterian"),
        ("presue", "pursue"),
        ("presued", "pursued"),
        ("privielage", "privilege"),
        ("priviledge", "privilege"),
        ("proceedures", "procedures"),
        ("pronensiation", "pronunciation"),
        ("pronounciation", "pronunciation"),
        ("properally", "properly"),
        ("proplematic", "problematic"),
        ("protray", "portray"),
        ("pscolgst", "psychologist"),
        ("psicolagest", "psychologist"),
        ("psycolagest", "psychologist"),
        ("quoz", "quiz"),
        ("radious", "radius"),
        ("reccomend", "recommend"),
        ("reccona", "raccoon"),
        ("recieve", "receive"),
        ("reconise", "recognize"),
        ("rectangeles", "rectangle"),
        ("reoccurring", "recurring"),
        ("repitition", "repetition"),
        ("replasments", "replacement"),
        ("respct", "respect"),
        ("respecally", "respectfully"),
        ("rsx", "RSX"),
        ("runnung", "running"),
        ("sacreligious", "sacrilegious"),
        ("salut", "salute"),
        ("searcheable", "searchable"),
        ("seferal", "several"),
        ("segements", "segments"),
        ("sence", "sense"),
        ("seperate", "separate"),
        ("sicolagest", "psychologist"),
        ("sieze", "seize"),
        ("simplye", "simply"),
        ("sitte", "site"),
        ("slyph", "sylph"),
        ("smil", "smile"),
        ("sometmes", "sometimes"),
        ("soonec", "sonic"),
        ("specificialy", "specifically"),
        ("spel", "spell"),
        ("spoak", "spoke"),
        ("sponsered", "sponsored"),
        ("stering", "steering"),
        ("straightjacket", "straitjacket"),
        ("stumach", "stomach"),
        ("stutent", "student"),
        ("styleguide", "style guide"),
        ("subpena", "subpoena"),
        ("substations", "substitutions"),
        ("supercede", "supersede"),
        ("superfulous", "superfluous"),
        ("susan", "Susan"),
        ("swimwear", "swim wear"),
        ("syncorization", "synchronization"),
        ("taff", "tough"),
        ("taht", "that"),
        ("tattos", "tattoos"),
        ("techniquely", "technically"),
        ("teh", "the"),
        ("tem", "team"),
        ("teo", "two"),
        ("teridical", "theoretical"),
        ("tesst", "test"),
        ("theridically", "theoretical"),
        ("thredically", "theoretically"),
        ("thruout", "throughout"),
        ("ths", "this"),
        ("titalate", "titillate"),
        ("tobagan", "tobaggon"),
        ("tommorrow", "tomorrow"),
        ("tomorow", "tomorrow"),
        ("trubbel", "trouble"),
        ("ttest", "test"),
        ("tyrrany", "tyranny"),
        ("unatourral", "unnatural"),
        ("unaturral", "unnatural"),
        ("unconisitional", "unconstitutional"),
        ("unconscience", "unconscious"),
        ("underladder", "under ladder"),
        ("unentelegible", "unintelligible"),
        ("unfortunently", "unfortunately"),
        ("unnaturral", "unnatural"),
        ("upcast", "up cast"),
        ("verison", "version"),
        ("vinagarette", "vinaigrette"),
        ("volunteerism", "voluntarism"),
        ("volye", "volley"),
        ("waite", "wait"),
        ("wan't", "won't"),
        ("warloord", "warlord"),
        ("whaaat", "what"),
        ("whard", "ward"),
        ("whimp", "wimp"),
        ("wicken", "weaken"),
        ("wierd", "weird"),
        ("wrank", "rank"),
        ("writeen", "righten"),
        ("writting", "writing"),
        ("wundeews", "windows"),
        ("yeild", "yield"),
    ];

    fn assert_double_metaphone(expected: &str, source: &str) {
        let encoder = DoubleMetaphone::default();
        assert_eq!(encoder.encode(source), expected);
    }

    fn assert_double_metaphone_alternate(expected: &str, source: &str) {
        let encoder = DoubleMetaphone::default();
        assert_eq!(encoder.encode_alternate(source), expected);
    }

    fn double_metaphone_equal_test(data: &[(&str, &str)], use_alternate: bool) {
        let encoder = DoubleMetaphone::default();
        for (v1, v2) in data.iter() {
            if use_alternate {
                assert!(
                    encoder.is_double_metaphone_equal(v1, v2, use_alternate),
                    "{v1} not equals to {v2} (use_alternate {use_alternate})"
                );
                assert!(
                    encoder.is_double_metaphone_equal(v1, v2, use_alternate),
                    "{v2} not equals to {v1} (use_alternate {use_alternate})"
                );
            } else {
                assert!(
                    encoder.is_encoded_equals(v1, v2),
                    "{v1} not equals to {v2} (use_alternate {use_alternate})"
                );
                assert!(
                    encoder.is_encoded_equals(v1, v2),
                    "{v2} not equals to {v1} (use_alternate {use_alternate})"
                );
            }
        }
    }

    fn double_metaphone_not_equal_test(alternate: bool) {
        let encoder = DoubleMetaphone::default();

        assert!(!encoder.is_double_metaphone_equal("Brain", "Band", alternate));
        assert!(!encoder.is_double_metaphone_equal("Band", "Brain", alternate));
    }

    #[test]
    fn test_c_cedilla() {
        let encoder = DoubleMetaphone::default();

        assert!(encoder.is_encoded_equals("\u{00e7}", "S"))
    }

    #[test]
    fn test_codec184() {
        let encoder = DoubleMetaphone::default();

        assert!(encoder.is_double_metaphone_equal("", "", false));
        assert!(encoder.is_double_metaphone_equal("", "", true));
        assert!(!encoder.is_double_metaphone_equal("aa", "", false));
        assert!(!encoder.is_double_metaphone_equal("aa", "", true));
        assert!(!encoder.is_double_metaphone_equal("", "aa", false));
        assert!(!encoder.is_double_metaphone_equal("", "aa", true));
    }

    #[test]
    fn test_double_metaphone() {
        assert_double_metaphone("TSTN", "testing");
        assert_double_metaphone("0", "The");
        assert_double_metaphone("KK", "quick");
        assert_double_metaphone("PRN", "brown");
        assert_double_metaphone("FKS", "fox");
        assert_double_metaphone("JMPT", "jumped");
        assert_double_metaphone("AFR", "over");
        assert_double_metaphone("0", "the");
        assert_double_metaphone("LS", "lazy");
        assert_double_metaphone("TKS", "dogs");
        assert_double_metaphone("MKFR", "MacCafferey");
        assert_double_metaphone("STFN", "Stephan");
        assert_double_metaphone("KSSK", "Kuczewski");
        assert_double_metaphone("MKLL", "McClelland");
        assert_double_metaphone("SNHS", "san jose");
        assert_double_metaphone("SNFP", "xenophobia");

        assert_double_metaphone_alternate("TSTN", "testing");
        assert_double_metaphone_alternate("T", "The");
        assert_double_metaphone_alternate("KK", "quick");
        assert_double_metaphone_alternate("PRN", "brown");
        assert_double_metaphone_alternate("FKS", "fox");
        assert_double_metaphone_alternate("AMPT", "jumped");
        assert_double_metaphone_alternate("AFR", "over");
        assert_double_metaphone_alternate("T", "the");
        assert_double_metaphone_alternate("LS", "lazy");
        assert_double_metaphone_alternate("TKS", "dogs");
        assert_double_metaphone_alternate("MKFR", "MacCafferey");
        assert_double_metaphone_alternate("STFN", "Stephan");
        assert_double_metaphone_alternate("KXFS", "Kutchefski");
        assert_double_metaphone_alternate("MKLL", "McClelland");
        assert_double_metaphone_alternate("SNHS", "san jose");
        assert_double_metaphone_alternate("SNFP", "xenophobia");
        assert_double_metaphone_alternate("FKR", "Fokker");
        assert_double_metaphone_alternate("AK", "Joqqi");
        assert_double_metaphone_alternate("HF", "Hovvi");
        assert_double_metaphone_alternate("XRN", "Czerny");
    }

    #[test]
    fn test_empty() {
        let encoder = DoubleMetaphone::default();

        assert_eq!(encoder.encode(""), "");
        assert_eq!(encoder.encode(" "), "");
        assert_eq!(encoder.encode("\t\n\r "), "");
    }

    #[test]
    fn test_is_double_metaphone_equal_basic() {
        let fixture = [
            ("", ""),
            ("Case", "case"),
            ("CASE", "Case"),
            ("caSe", "cAsE"),
            ("cookie", "quick"),
            ("quick", "cookie"),
            ("Brian", "Bryan"),
            ("Auto", "Otto"),
            ("Steven", "Stefan"),
            ("Philipowitz", "Filipowicz"),
        ];

        double_metaphone_equal_test(&fixture, false);
        double_metaphone_equal_test(&fixture, true);
    }

    #[test]
    fn test_is_double_metaphone_equal_extended2() {
        let fixture = [("Jablonski", "Yablonsky")];
        double_metaphone_equal_test(&fixture, true);
    }

    // This test isn't really enable in commons-codec 1.15
    #[test]
    #[ignore]
    fn test_is_double_metaphone_equal_extended3() {
        let encoder = DoubleMetaphone::default();

        let mut count = 0;
        let mut error = String::new();
        for (i, (data1, data2)) in FIXTURE.into_iter().enumerate() {
            let match1 = encoder.is_double_metaphone_equal(data1, data2, false);
            let match2 = encoder.is_double_metaphone_equal(data1, data2, true);
            if !match1 && !match2 {
                error.push_str(format!("[{i}] {data1} and {data2}\n").as_str());
                count += 1;
            }
        }
        if count > 0 {
            panic!("{}", error);
        }
    }

    #[test]
    fn test_is_double_metaphone_equal_with_matches() {
        let encoder = DoubleMetaphone::default();
        for (i, (data1, data2)) in MATCHES.into_iter().enumerate() {
            let match1 = encoder.is_double_metaphone_equal(data1, data2, false);
            let match2 = encoder.is_double_metaphone_equal(data1, data2, true);
            assert!(match1 || match2, "Expected match [{i}] {data1} and {data2}");
        }
    }

    #[test]
    fn test_is_double_metaphone_not_equal() {
        double_metaphone_not_equal_test(true);
        double_metaphone_not_equal_test(false);
    }

    #[test]
    fn test_n_tilde() {
        let encoder = DoubleMetaphone::default();

        assert!(encoder.is_encoded_equals("\u{00f1}", "N"));
    }

    #[test]
    fn test_set_max_code_length() {
        let value = "jumped";

        let encoder = DoubleMetaphone::default();
        assert_eq!(encoder.max_code_length, 4);
        assert_eq!(encoder.encode(value), "JMPT");
        assert_eq!(encoder.encode_alternate(value), "AMPT");

        let encoder = DoubleMetaphone::new(3);
        assert_eq!(encoder.max_code_length, 3);
        assert_eq!(encoder.encode(value), "JMP");
        assert_eq!(encoder.encode_alternate(value), "AMP");
    }

    // This test is for debugging purpose
    #[test]
    #[ignore]
    fn test_debug() {
        let encoder = DoubleMetaphone::default();

        let (data1, data2) = ("playwrite", "playwright");
        let encode_primary1 = encoder.encode(data1);
        let encode_primary2 = encoder.encode(data2);
        let encode_alternate1 = encoder.encode_alternate(data1);
        let encode_alternate2 = encoder.encode_alternate(data2);

        assert_eq!(
            (encode_primary1, encode_alternate1),
            (encode_primary2, encode_alternate2)
        );
    }

    const TEST_DATA: [(&str, &str, &str); 1221] = [
        ("ALLERTON", "ALRT", "ALRT"),
        ("Acton", "AKTN", "AKTN"),
        ("Adams", "ATMS", "ATMS"),
        ("Aggar", "AKR", "AKR"),
        ("Ahl", "AL", "AL"),
        ("Aiken", "AKN", "AKN"),
        ("Alan", "ALN", "ALN"),
        ("Alcock", "ALKK", "ALKK"),
        ("Alden", "ALTN", "ALTN"),
        ("Aldham", "ALTM", "ALTM"),
        ("Allen", "ALN", "ALN"),
        ("Allerton", "ALRT", "ALRT"),
        ("Alsop", "ALSP", "ALSP"),
        ("Alwein", "ALN", "ALN"),
        ("Ambler", "AMPL", "AMPL"),
        ("Andevill", "ANTF", "ANTF"),
        ("Andrews", "ANTR", "ANTR"),
        ("Andreyco", "ANTR", "ANTR"),
        ("Andriesse", "ANTR", "ANTR"),
        ("Angier", "ANJ", "ANJR"),
        ("Annabel", "ANPL", "ANPL"),
        ("Anne", "AN", "AN"),
        ("Anstye", "ANST", "ANST"),
        ("Appling", "APLN", "APLN"),
        ("Apuke", "APK", "APK"),
        ("Arnold", "ARNL", "ARNL"),
        ("Ashby", "AXP", "AXP"),
        ("Astwood", "ASTT", "ASTT"),
        ("Atkinson", "ATKN", "ATKN"),
        ("Audley", "ATL", "ATL"),
        ("Austin", "ASTN", "ASTN"),
        ("Avenal", "AFNL", "AFNL"),
        ("Ayer", "AR", "AR"),
        ("Ayot", "AT", "AT"),
        ("Babbitt", "PPT", "PPT"),
        ("Bachelor", "PXLR", "PKLR"),
        ("Bachelour", "PXLR", "PKLR"),
        ("Bailey", "PL", "PL"),
        ("Baivel", "PFL", "PFL"),
        ("Baker", "PKR", "PKR"),
        ("Baldwin", "PLTN", "PLTN"),
        ("Balsley", "PLSL", "PLSL"),
        ("Barber", "PRPR", "PRPR"),
        ("Barker", "PRKR", "PRKR"),
        ("Barlow", "PRL", "PRLF"),
        ("Barnard", "PRNR", "PRNR"),
        ("Barnes", "PRNS", "PRNS"),
        ("Barnsley", "PRNS", "PRNS"),
        ("Barouxis", "PRKS", "PRKS"),
        ("Bartlet", "PRTL", "PRTL"),
        ("Basley", "PSL", "PSL"),
        ("Basset", "PST", "PST"),
        ("Bassett", "PST", "PST"),
        ("Batchlor", "PXLR", "PXLR"),
        ("Bates", "PTS", "PTS"),
        ("Batson", "PTSN", "PTSN"),
        ("Bayes", "PS", "PS"),
        ("Bayley", "PL", "PL"),
        ("Beale", "PL", "PL"),
        ("Beauchamp", "PXMP", "PKMP"),
        ("Beauclerc", "PKLR", "PKLR"),
        ("Beech", "PK", "PK"),
        ("Beers", "PRS", "PRS"),
        ("Beke", "PK", "PK"),
        ("Belcher", "PLXR", "PLKR"),
        ("benign", "PNN", "PNKN"),
        ("Benjamin", "PNJM", "PNJM"),
        ("Benningham", "PNNK", "PNNK"),
        ("Bereford", "PRFR", "PRFR"),
        ("Bergen", "PRJN", "PRKN"),
        ("Berkeley", "PRKL", "PRKL"),
        ("Berry", "PR", "PR"),
        ("Besse", "PS", "PS"),
        ("Bessey", "PS", "PS"),
        ("Bessiles", "PSLS", "PSLS"),
        ("Bigelow", "PJL", "PKLF"),
        ("Bigg", "PK", "PK"),
        ("Bigod", "PKT", "PKT"),
        ("Billings", "PLNK", "PLNK"),
        ("Bimper", "PMPR", "PMPR"),
        ("Binker", "PNKR", "PNKR"),
        ("Birdsill", "PRTS", "PRTS"),
        ("Bishop", "PXP", "PXP"),
        ("Black", "PLK", "PLK"),
        ("Blagge", "PLK", "PLK"),
        ("Blake", "PLK", "PLK"),
        ("Blanck", "PLNK", "PLNK"),
        ("Bledsoe", "PLTS", "PLTS"),
        ("Blennerhasset", "PLNR", "PLNR"),
        ("Blessing", "PLSN", "PLSN"),
        ("Blewett", "PLT", "PLT"),
        ("Bloctgoed", "PLKT", "PLKT"),
        ("Bloetgoet", "PLTK", "PLTK"),
        ("Bloodgood", "PLTK", "PLTK"),
        ("Blossom", "PLSM", "PLSM"),
        ("Blount", "PLNT", "PLNT"),
        ("Bodine", "PTN", "PTN"),
        ("Bodman", "PTMN", "PTMN"),
        ("BonCoeur", "PNKR", "PNKR"),
        ("Bond", "PNT", "PNT"),
        ("Boscawen", "PSKN", "PSKN"),
        ("Bosworth", "PSR0", "PSRT"),
        ("Bouchier", "PX", "PKR"),
        ("Bowne", "PN", "PN"),
        ("Bradbury", "PRTP", "PRTP"),
        ("Bradder", "PRTR", "PRTR"),
        ("Bradford", "PRTF", "PRTF"),
        ("Bradstreet", "PRTS", "PRTS"),
        ("Braham", "PRHM", "PRHM"),
        ("Brailsford", "PRLS", "PRLS"),
        ("Brainard", "PRNR", "PRNR"),
        ("Brandish", "PRNT", "PRNT"),
        ("Braun", "PRN", "PRN"),
        ("Brecc", "PRK", "PRK"),
        ("Brent", "PRNT", "PRNT"),
        ("Brenton", "PRNT", "PRNT"),
        ("Briggs", "PRKS", "PRKS"),
        ("Brigham", "PRM", "PRM"),
        ("Brobst", "PRPS", "PRPS"),
        ("Brome", "PRM", "PRM"),
        ("Bronson", "PRNS", "PRNS"),
        ("Brooks", "PRKS", "PRKS"),
        ("Brouillard", "PRLR", "PRLR"),
        ("Brown", "PRN", "PRN"),
        ("Browne", "PRN", "PRN"),
        ("Brownell", "PRNL", "PRNL"),
        ("Bruley", "PRL", "PRL"),
        ("Bryant", "PRNT", "PRNT"),
        ("Brzozowski", "PRSS", "PRTS"),
        ("Buide", "PT", "PT"),
        ("Bulmer", "PLMR", "PLMR"),
        ("Bunker", "PNKR", "PNKR"),
        ("Burden", "PRTN", "PRTN"),
        ("Burge", "PRJ", "PRK"),
        ("Burgoyne", "PRKN", "PRKN"),
        ("Burke", "PRK", "PRK"),
        ("Burnett", "PRNT", "PRNT"),
        ("Burpee", "PRP", "PRP"),
        ("Bursley", "PRSL", "PRSL"),
        ("Burton", "PRTN", "PRTN"),
        ("Bushnell", "PXNL", "PXNL"),
        ("Buss", "PS", "PS"),
        ("Buswell", "PSL", "PSL"),
        ("Butler", "PTLR", "PTLR"),
        ("Calkin", "KLKN", "KLKN"),
        ("Canada", "KNT", "KNT"),
        ("Canmore", "KNMR", "KNMR"),
        ("Canney", "KN", "KN"),
        ("Capet", "KPT", "KPT"),
        ("Card", "KRT", "KRT"),
        ("Carman", "KRMN", "KRMN"),
        ("Carpenter", "KRPN", "KRPN"),
        ("Cartwright", "KRTR", "KRTR"),
        ("Casey", "KS", "KS"),
        ("Catterfield", "KTRF", "KTRF"),
        ("Ceeley", "SL", "SL"),
        ("Chambers", "XMPR", "XMPR"),
        ("Champion", "XMPN", "XMPN"),
        ("Chapman", "XPMN", "XPMN"),
        ("Chase", "XS", "XS"),
        ("Cheney", "XN", "XN"),
        ("Chetwynd", "XTNT", "XTNT"),
        ("Chevalier", "XFL", "XFLR"),
        ("Chillingsworth", "XLNK", "XLNK"),
        ("Christie", "KRST", "KRST"),
        ("Chubbuck", "XPK", "XPK"),
        ("Church", "XRX", "XRK"),
        ("Clark", "KLRK", "KLRK"),
        ("Clarke", "KLRK", "KLRK"),
        ("Cleare", "KLR", "KLR"),
        ("Clement", "KLMN", "KLMN"),
        ("Clerke", "KLRK", "KLRK"),
        ("Clibben", "KLPN", "KLPN"),
        ("Clifford", "KLFR", "KLFR"),
        ("Clivedon", "KLFT", "KLFT"),
        ("Close", "KLS", "KLS"),
        ("Clothilde", "KL0L", "KLTL"),
        ("Cobb", "KP", "KP"),
        ("Coburn", "KPRN", "KPRN"),
        ("Coburne", "KPRN", "KPRN"),
        ("Cocke", "KK", "KK"),
        ("Coffin", "KFN", "KFN"),
        ("Coffyn", "KFN", "KFN"),
        ("Colborne", "KLPR", "KLPR"),
        ("Colby", "KLP", "KLP"),
        ("Cole", "KL", "KL"),
        ("Coleman", "KLMN", "KLMN"),
        ("Collier", "KL", "KLR"),
        ("Compton", "KMPT", "KMPT"),
        ("Cone", "KN", "KN"),
        ("Cook", "KK", "KK"),
        ("Cooke", "KK", "KK"),
        ("Cooper", "KPR", "KPR"),
        ("Copperthwaite", "KPR0", "KPRT"),
        ("Corbet", "KRPT", "KRPT"),
        ("Corell", "KRL", "KRL"),
        ("Corey", "KR", "KR"),
        ("Corlies", "KRLS", "KRLS"),
        ("Corneliszen", "KRNL", "KRNL"),
        ("Cornelius", "KRNL", "KRNL"),
        ("Cornwallis", "KRNL", "KRNL"),
        ("Cosgrove", "KSKR", "KSKR"),
        ("Count of Brionne", "KNTF", "KNTF"),
        ("Covill", "KFL", "KFL"),
        ("Cowperthwaite", "KPR0", "KPRT"),
        ("Cowperwaite", "KPRT", "KPRT"),
        ("Crane", "KRN", "KRN"),
        ("Creagmile", "KRKM", "KRKM"),
        ("Crew", "KR", "KRF"),
        ("Crispin", "KRSP", "KRSP"),
        ("Crocker", "KRKR", "KRKR"),
        ("Crockett", "KRKT", "KRKT"),
        ("Crosby", "KRSP", "KRSP"),
        ("Crump", "KRMP", "KRMP"),
        ("Cunningham", "KNNK", "KNNK"),
        ("Curtis", "KRTS", "KRTS"),
        ("Cutha", "K0", "KT"),
        ("Cutter", "KTR", "KTR"),
        ("D'Aubigny", "TPN", "TPKN"),
        ("DAVIS", "TFS", "TFS"),
        ("Dabinott", "TPNT", "TPNT"),
        ("Dacre", "TKR", "TKR"),
        ("Daggett", "TKT", "TKT"),
        ("Danvers", "TNFR", "TNFR"),
        ("Darcy", "TRS", "TRS"),
        ("Davis", "TFS", "TFS"),
        ("Dawn", "TN", "TN"),
        ("Dawson", "TSN", "TSN"),
        ("Day", "T", "T"),
        ("Daye", "T", "T"),
        ("DeGrenier", "TKRN", "TKRN"),
        ("Dean", "TN", "TN"),
        ("Deekindaugh", "TKNT", "TKNT"),
        ("Dennis", "TNS", "TNS"),
        ("Denny", "TN", "TN"),
        ("Denton", "TNTN", "TNTN"),
        ("Desborough", "TSPR", "TSPR"),
        ("Despenser", "TSPN", "TSPN"),
        ("Deverill", "TFRL", "TFRL"),
        ("Devine", "TFN", "TFN"),
        ("Dexter", "TKST", "TKST"),
        ("Dillaway", "TL", "TL"),
        ("Dimmick", "TMK", "TMK"),
        ("Dinan", "TNN", "TNN"),
        ("Dix", "TKS", "TKS"),
        ("Doggett", "TKT", "TKT"),
        ("Donahue", "TNH", "TNH"),
        ("Dorfman", "TRFM", "TRFM"),
        ("Dorris", "TRS", "TRS"),
        ("Dow", "T", "TF"),
        ("Downey", "TN", "TN"),
        ("Downing", "TNNK", "TNNK"),
        ("Dowsett", "TST", "TST"),
        ("Duck?", "TK", "TK"),
        ("Dudley", "TTL", "TTL"),
        ("Duffy", "TF", "TF"),
        ("Dunn", "TN", "TN"),
        ("Dunsterville", "TNST", "TNST"),
        ("Durrant", "TRNT", "TRNT"),
        ("Durrin", "TRN", "TRN"),
        ("Dustin", "TSTN", "TSTN"),
        ("Duston", "TSTN", "TSTN"),
        ("Eames", "AMS", "AMS"),
        ("Early", "ARL", "ARL"),
        ("Easty", "AST", "AST"),
        ("Ebbett", "APT", "APT"),
        ("Eberbach", "APRP", "APRP"),
        ("Eberhard", "APRR", "APRR"),
        ("Eddy", "AT", "AT"),
        ("Edenden", "ATNT", "ATNT"),
        ("Edwards", "ATRT", "ATRT"),
        ("Eglinton", "AKLN", "ALNT"),
        ("Eliot", "ALT", "ALT"),
        ("Elizabeth", "ALSP", "ALSP"),
        ("Ellis", "ALS", "ALS"),
        ("Ellison", "ALSN", "ALSN"),
        ("Ellot", "ALT", "ALT"),
        ("Elny", "ALN", "ALN"),
        ("Elsner", "ALSN", "ALSN"),
        ("Emerson", "AMRS", "AMRS"),
        ("Empson", "AMPS", "AMPS"),
        ("Est", "AST", "AST"),
        ("Estabrook", "ASTP", "ASTP"),
        ("Estes", "ASTS", "ASTS"),
        ("Estey", "AST", "AST"),
        ("Evans", "AFNS", "AFNS"),
        ("Fallowell", "FLL", "FLL"),
        ("Farnsworth", "FRNS", "FRNS"),
        ("Feake", "FK", "FK"),
        ("Feke", "FK", "FK"),
        ("Fellows", "FLS", "FLS"),
        ("Fettiplace", "FTPL", "FTPL"),
        ("Finney", "FN", "FN"),
        ("Fischer", "FXR", "FSKR"),
        ("Fisher", "FXR", "FXR"),
        ("Fisk", "FSK", "FSK"),
        ("Fiske", "FSK", "FSK"),
        ("Fletcher", "FLXR", "FLXR"),
        ("Folger", "FLKR", "FLJR"),
        ("Foliot", "FLT", "FLT"),
        ("Folyot", "FLT", "FLT"),
        ("Fones", "FNS", "FNS"),
        ("Fordham", "FRTM", "FRTM"),
        ("Forstner", "FRST", "FRST"),
        ("Fosten", "FSTN", "FSTN"),
        ("Foster", "FSTR", "FSTR"),
        ("Foulke", "FLK", "FLK"),
        ("Fowler", "FLR", "FLR"),
        ("Foxwell", "FKSL", "FKSL"),
        ("Fraley", "FRL", "FRL"),
        ("Franceys", "FRNS", "FRNS"),
        ("Franke", "FRNK", "FRNK"),
        ("Frascella", "FRSL", "FRSL"),
        ("Frazer", "FRSR", "FRSR"),
        ("Fredd", "FRT", "FRT"),
        ("Freeman", "FRMN", "FRMN"),
        ("French", "FRNX", "FRNK"),
        ("Freville", "FRFL", "FRFL"),
        ("Frey", "FR", "FR"),
        ("Frick", "FRK", "FRK"),
        ("Frier", "FR", "FRR"),
        ("Froe", "FR", "FR"),
        ("Frorer", "FRRR", "FRRR"),
        ("Frost", "FRST", "FRST"),
        ("Frothingham", "FR0N", "FRTN"),
        ("Fry", "FR", "FR"),
        ("Gaffney", "KFN", "KFN"),
        ("Gage", "KJ", "KK"),
        ("Gallion", "KLN", "KLN"),
        ("Gallishan", "KLXN", "KLXN"),
        ("Gamble", "KMPL", "KMPL"),
        ("garage", "KRJ", "KRK"),
        ("Garbrand", "KRPR", "KRPR"),
        ("Gardner", "KRTN", "KRTN"),
        ("Garrett", "KRT", "KRT"),
        ("Gassner", "KSNR", "KSNR"),
        ("Gater", "KTR", "KTR"),
        ("Gaunt", "KNT", "KNT"),
        ("Gayer", "KR", "KR"),
        ("George", "JRJ", "KRK"),
        ("Gerken", "KRKN", "JRKN"),
        ("Gerritsen", "KRTS", "JRTS"),
        ("Gibbs", "KPS", "JPS"),
        ("Giffard", "JFRT", "KFRT"),
        ("Gilbert", "KLPR", "JLPR"),
        ("Gill", "KL", "JL"),
        ("Gilman", "KLMN", "JLMN"),
        ("Glass", "KLS", "KLS"),
        ("Goddard\\Gifford", "KTRT", "KTRT"),
        ("Godfrey", "KTFR", "KTFR"),
        ("Godwin", "KTN", "KTN"),
        ("Goodale", "KTL", "KTL"),
        ("Goodnow", "KTN", "KTNF"),
        ("Gorham", "KRM", "KRM"),
        ("Goseline", "KSLN", "KSLN"),
        ("Gott", "KT", "KT"),
        ("Gould", "KLT", "KLT"),
        ("Grafton", "KRFT", "KRFT"),
        ("Grant", "KRNT", "KRNT"),
        ("Gray", "KR", "KR"),
        ("Green", "KRN", "KRN"),
        ("Griffin", "KRFN", "KRFN"),
        ("Grill", "KRL", "KRL"),
        ("Grim", "KRM", "KRM"),
        ("Grisgonelle", "KRSK", "KRSK"),
        ("Gross", "KRS", "KRS"),
        ("Guba", "KP", "KP"),
        ("Gybbes", "KPS", "JPS"),
        ("Haburne", "HPRN", "HPRN"),
        ("Hackburne", "HKPR", "HKPR"),
        ("Haddon?", "HTN", "HTN"),
        ("Haines", "HNS", "HNS"),
        ("Hale", "HL", "HL"),
        ("Hall", "HL", "HL"),
        ("Hallet", "HLT", "HLT"),
        ("Hallock", "HLK", "HLK"),
        ("Halstead", "HLST", "HLST"),
        ("Hammond", "HMNT", "HMNT"),
        ("Hance", "HNS", "HNS"),
        ("Handy", "HNT", "HNT"),
        ("Hanson", "HNSN", "HNSN"),
        ("Harasek", "HRSK", "HRSK"),
        ("Harcourt", "HRKR", "HRKR"),
        ("Hardy", "HRT", "HRT"),
        ("Harlock", "HRLK", "HRLK"),
        ("Harris", "HRS", "HRS"),
        ("Hartley", "HRTL", "HRTL"),
        ("Harvey", "HRF", "HRF"),
        ("Harvie", "HRF", "HRF"),
        ("Harwood", "HRT", "HRT"),
        ("Hathaway", "H0", "HT"),
        ("Haukeness", "HKNS", "HKNS"),
        ("Hawkes", "HKS", "HKS"),
        ("Hawkhurst", "HKRS", "HKRS"),
        ("Hawkins", "HKNS", "HKNS"),
        ("Hawley", "HL", "HL"),
        ("Heald", "HLT", "HLT"),
        ("Helsdon", "HLST", "HLST"),
        ("Hemenway", "HMN", "HMN"),
        ("Hemmenway", "HMN", "HMN"),
        ("Henck", "HNK", "HNK"),
        ("Henderson", "HNTR", "HNTR"),
        ("Hendricks", "HNTR", "HNTR"),
        ("Hersey", "HRS", "HRS"),
        ("Hewes", "HS", "HS"),
        ("Heyman", "HMN", "HMN"),
        ("Hicks", "HKS", "HKS"),
        ("Hidden", "HTN", "HTN"),
        ("Higgs", "HKS", "HKS"),
        ("Hill", "HL", "HL"),
        ("Hills", "HLS", "HLS"),
        ("Hinckley", "HNKL", "HNKL"),
        ("Hipwell", "HPL", "HPL"),
        ("Hobart", "HPRT", "HPRT"),
        ("Hoben", "HPN", "HPN"),
        ("Hoffmann", "HFMN", "HFMN"),
        ("Hogan", "HKN", "HKN"),
        ("Holmes", "HLMS", "HLMS"),
        ("Hoo", "H", "H"),
        ("Hooker", "HKR", "HKR"),
        ("Hopcott", "HPKT", "HPKT"),
        ("Hopkins", "HPKN", "HPKN"),
        ("Hopkinson", "HPKN", "HPKN"),
        ("Hornsey", "HRNS", "HRNS"),
        ("Houckgeest", "HKJS", "HKKS"),
        ("Hough", "H", "H"),
        ("Houstin", "HSTN", "HSTN"),
        ("How", "H", "HF"),
        ("Howe", "H", "H"),
        ("Howland", "HLNT", "HLNT"),
        ("Hubner", "HPNR", "HPNR"),
        ("Hudnut", "HTNT", "HTNT"),
        ("Hughes", "HS", "HS"),
        ("Hull", "HL", "HL"),
        ("Hulme", "HLM", "HLM"),
        ("Hume", "HM", "HM"),
        ("Hundertumark", "HNTR", "HNTR"),
        ("Hundley", "HNTL", "HNTL"),
        ("Hungerford", "HNKR", "HNJR"),
        ("Hunt", "HNT", "HNT"),
        ("Hurst", "HRST", "HRST"),
        ("Husbands", "HSPN", "HSPN"),
        ("Hussey", "HS", "HS"),
        ("Husted", "HSTT", "HSTT"),
        ("Hutchins", "HXNS", "HXNS"),
        ("Hutchinson", "HXNS", "HXNS"),
        ("Huttinger", "HTNK", "HTNJ"),
        ("Huybertsen", "HPRT", "HPRT"),
        ("Iddenden", "ATNT", "ATNT"),
        ("Ingraham", "ANKR", "ANKR"),
        ("Ives", "AFS", "AFS"),
        ("Jackson", "JKSN", "AKSN"),
        ("Jacob", "JKP", "AKP"),
        ("Jans", "JNS", "ANS"),
        ("Jenkins", "JNKN", "ANKN"),
        ("Jewett", "JT", "AT"),
        ("Jewitt", "JT", "AT"),
        ("Johnson", "JNSN", "ANSN"),
        ("Jones", "JNS", "ANS"),
        ("Josephine", "JSFN", "HSFN"),
        ("Judd", "JT", "AT"),
        ("June", "JN", "AN"),
        ("Kamarowska", "KMRS", "KMRS"),
        ("Kay", "K", "K"),
        ("Kelley", "KL", "KL"),
        ("Kelly", "KL", "KL"),
        ("Keymber", "KMPR", "KMPR"),
        ("Keynes", "KNS", "KNS"),
        ("Kilham", "KLM", "KLM"),
        ("Kim", "KM", "KM"),
        ("Kimball", "KMPL", "KMPL"),
        ("King", "KNK", "KNK"),
        ("Kinsey", "KNS", "KNS"),
        ("Kirk", "KRK", "KRK"),
        ("Kirton", "KRTN", "KRTN"),
        ("Kistler", "KSTL", "KSTL"),
        ("Kitchen", "KXN", "KXN"),
        ("Kitson", "KTSN", "KTSN"),
        ("Klett", "KLT", "KLT"),
        ("Kline", "KLN", "KLN"),
        ("Knapp", "NP", "NP"),
        ("Knight", "NT", "NT"),
        ("Knote", "NT", "NT"),
        ("Knott", "NT", "NT"),
        ("Knox", "NKS", "NKS"),
        ("Koeller", "KLR", "KLR"),
        ("La Pointe", "LPNT", "LPNT"),
        ("LaPlante", "LPLN", "LPLN"),
        ("Laimbeer", "LMPR", "LMPR"),
        ("Lamb", "LMP", "LMP"),
        ("Lambertson", "LMPR", "LMPR"),
        ("Lancto", "LNKT", "LNKT"),
        ("Landry", "LNTR", "LNTR"),
        ("Lane", "LN", "LN"),
        ("Langendyck", "LNJN", "LNKN"),
        ("Langer", "LNKR", "LNJR"),
        ("Langford", "LNKF", "LNKF"),
        ("Lantersee", "LNTR", "LNTR"),
        ("Laquer", "LKR", "LKR"),
        ("Larkin", "LRKN", "LRKN"),
        ("Latham", "LTM", "LTM"),
        ("Lathrop", "L0RP", "LTRP"),
        ("Lauter", "LTR", "LTR"),
        ("Lawrence", "LRNS", "LRNS"),
        ("Leach", "LK", "LK"),
        ("Leager", "LKR", "LJR"),
        ("Learned", "LRNT", "LRNT"),
        ("Leavitt", "LFT", "LFT"),
        ("Lee", "L", "L"),
        ("Leete", "LT", "LT"),
        ("Leggett", "LKT", "LKT"),
        ("Leland", "LLNT", "LLNT"),
        ("Leonard", "LNRT", "LNRT"),
        ("Lester", "LSTR", "LSTR"),
        ("Lestrange", "LSTR", "LSTR"),
        ("Lethem", "L0M", "LTM"),
        ("Levine", "LFN", "LFN"),
        ("Lewes", "LS", "LS"),
        ("Lewis", "LS", "LS"),
        ("Lincoln", "LNKL", "LNKL"),
        ("Lindsey", "LNTS", "LNTS"),
        ("Linher", "LNR", "LNR"),
        ("Lippet", "LPT", "LPT"),
        ("Lippincott", "LPNK", "LPNK"),
        ("Lockwood", "LKT", "LKT"),
        ("Loines", "LNS", "LNS"),
        ("Lombard", "LMPR", "LMPR"),
        ("Long", "LNK", "LNK"),
        ("Longespee", "LNJS", "LNKS"),
        ("Look", "LK", "LK"),
        ("Lounsberry", "LNSP", "LNSP"),
        ("Lounsbury", "LNSP", "LNSP"),
        ("Louthe", "L0", "LT"),
        ("Loveyne", "LFN", "LFN"),
        ("Lowe", "L", "L"),
        ("Ludlam", "LTLM", "LTLM"),
        ("Lumbard", "LMPR", "LMPR"),
        ("Lund", "LNT", "LNT"),
        ("Luno", "LN", "LN"),
        ("Lutz", "LTS", "LTS"),
        ("Lydia", "LT", "LT"),
        ("Lynne", "LN", "LN"),
        ("Lyon", "LN", "LN"),
        ("MacAlpin", "MKLP", "MKLP"),
        ("MacBricc", "MKPR", "MKPR"),
        ("MacCrinan", "MKRN", "MKRN"),
        ("MacKenneth", "MKN0", "MKNT"),
        ("MacMael nam Bo", "MKML", "MKML"),
        ("MacMurchada", "MKMR", "MKMR"),
        ("Macomber", "MKMP", "MKMP"),
        ("Macy", "MS", "MS"),
        ("Magnus", "MNS", "MKNS"),
        ("Mahien", "MHN", "MHN"),
        ("Malmains", "MLMN", "MLMN"),
        ("Malory", "MLR", "MLR"),
        ("Mancinelli", "MNSN", "MNSN"),
        ("Mancini", "MNSN", "MNSN"),
        ("Mann", "MN", "MN"),
        ("Manning", "MNNK", "MNNK"),
        ("Manter", "MNTR", "MNTR"),
        ("Marion", "MRN", "MRN"),
        ("Marley", "MRL", "MRL"),
        ("Marmion", "MRMN", "MRMN"),
        ("Marquart", "MRKR", "MRKR"),
        ("Marsh", "MRX", "MRX"),
        ("Marshal", "MRXL", "MRXL"),
        ("Marshall", "MRXL", "MRXL"),
        ("Martel", "MRTL", "MRTL"),
        ("Martha", "MR0", "MRT"),
        ("Martin", "MRTN", "MRTN"),
        ("Marturano", "MRTR", "MRTR"),
        ("Marvin", "MRFN", "MRFN"),
        ("Mary", "MR", "MR"),
        ("Mason", "MSN", "MSN"),
        ("Maxwell", "MKSL", "MKSL"),
        ("Mayhew", "MH", "MHF"),
        ("McAllaster", "MKLS", "MKLS"),
        ("McAllister", "MKLS", "MKLS"),
        ("McConnell", "MKNL", "MKNL"),
        ("McFarland", "MKFR", "MKFR"),
        ("McIlroy", "MSLR", "MSLR"),
        ("McNair", "MKNR", "MKNR"),
        ("McNair-Landry", "MKNR", "MKNR"),
        ("McRaven", "MKRF", "MKRF"),
        ("Mead", "MT", "MT"),
        ("Meade", "MT", "MT"),
        ("Meck", "MK", "MK"),
        ("Melton", "MLTN", "MLTN"),
        ("Mendenhall", "MNTN", "MNTN"),
        ("Mering", "MRNK", "MRNK"),
        ("Merrick", "MRK", "MRK"),
        ("Merry", "MR", "MR"),
        ("Mighill", "ML", "ML"),
        ("Miller", "MLR", "MLR"),
        ("Milton", "MLTN", "MLTN"),
        ("Mohun", "MHN", "MHN"),
        ("Montague", "MNTK", "MNTK"),
        ("Montboucher", "MNTP", "MNTP"),
        ("Moore", "MR", "MR"),
        ("Morrel", "MRL", "MRL"),
        ("Morrill", "MRL", "MRL"),
        ("Morris", "MRS", "MRS"),
        ("Morton", "MRTN", "MRTN"),
        ("Moton", "MTN", "MTN"),
        ("Muir", "MR", "MR"),
        ("Mulferd", "MLFR", "MLFR"),
        ("Mullins", "MLNS", "MLNS"),
        ("Mulso", "MLS", "MLS"),
        ("Munger", "MNKR", "MNJR"),
        ("Munt", "MNT", "MNT"),
        ("Murchad", "MRXT", "MRKT"),
        ("Murdock", "MRTK", "MRTK"),
        ("Murray", "MR", "MR"),
        ("Muskett", "MSKT", "MSKT"),
        ("Myers", "MRS", "MRS"),
        ("Myrick", "MRK", "MRK"),
        ("NORRIS", "NRS", "NRS"),
        ("Nayle", "NL", "NL"),
        ("Newcomb", "NKMP", "NKMP"),
        ("Newcomb(e)", "NKMP", "NKMP"),
        ("Newkirk", "NKRK", "NKRK"),
        ("Newton", "NTN", "NTN"),
        ("Niles", "NLS", "NLS"),
        ("Noble", "NPL", "NPL"),
        ("Noel", "NL", "NL"),
        ("Northend", "NR0N", "NRTN"),
        ("Norton", "NRTN", "NRTN"),
        ("Nutter", "NTR", "NTR"),
        ("Odding", "ATNK", "ATNK"),
        ("Odenbaugh", "ATNP", "ATNP"),
        ("Ogborn", "AKPR", "AKPR"),
        ("Oppenheimer", "APNM", "APNM"),
        ("Otis", "ATS", "ATS"),
        ("Oviatt", "AFT", "AFT"),
        ("PRUST?", "PRST", "PRST"),
        ("Paddock", "PTK", "PTK"),
        ("Page", "PJ", "PK"),
        ("Paine", "PN", "PN"),
        ("Paist", "PST", "PST"),
        ("Palmer", "PLMR", "PLMR"),
        ("Park", "PRK", "PRK"),
        ("Parker", "PRKR", "PRKR"),
        ("Parkhurst", "PRKR", "PRKR"),
        ("Parrat", "PRT", "PRT"),
        ("Parsons", "PRSN", "PRSN"),
        ("Partridge", "PRTR", "PRTR"),
        ("Pashley", "PXL", "PXL"),
        ("Pasley", "PSL", "PSL"),
        ("Patrick", "PTRK", "PTRK"),
        ("Pattee", "PT", "PT"),
        ("Patten", "PTN", "PTN"),
        ("Pawley", "PL", "PL"),
        ("Payne", "PN", "PN"),
        ("Peabody", "PPT", "PPT"),
        ("Peake", "PK", "PK"),
        ("Pearson", "PRSN", "PRSN"),
        ("Peat", "PT", "PT"),
        ("Pedersen", "PTRS", "PTRS"),
        ("Percy", "PRS", "PRS"),
        ("Perkins", "PRKN", "PRKN"),
        ("Perrine", "PRN", "PRN"),
        ("Perry", "PR", "PR"),
        ("Peson", "PSN", "PSN"),
        ("Peterson", "PTRS", "PTRS"),
        ("Peyton", "PTN", "PTN"),
        ("Phinney", "FN", "FN"),
        ("Pickard", "PKRT", "PKRT"),
        ("Pierce", "PRS", "PRS"),
        ("Pierrepont", "PRPN", "PRPN"),
        ("Pike", "PK", "PK"),
        ("Pinkham", "PNKM", "PNKM"),
        ("Pitman", "PTMN", "PTMN"),
        ("Pitt", "PT", "PT"),
        ("Pitts", "PTS", "PTS"),
        ("Plantagenet", "PLNT", "PLNT"),
        ("Platt", "PLT", "PLT"),
        ("Platts", "PLTS", "PLTS"),
        ("Pleis", "PLS", "PLS"),
        ("Pleiss", "PLS", "PLS"),
        ("Plisko", "PLSK", "PLSK"),
        ("Pliskovitch", "PLSK", "PLSK"),
        ("Plum", "PLM", "PLM"),
        ("Plume", "PLM", "PLM"),
        ("Poitou", "PT", "PT"),
        ("Pomeroy", "PMR", "PMR"),
        ("Poretiers", "PRTR", "PRTR"),
        ("Pote", "PT", "PT"),
        ("Potter", "PTR", "PTR"),
        ("Potts", "PTS", "PTS"),
        ("Powell", "PL", "PL"),
        ("Pratt", "PRT", "PRT"),
        ("Presbury", "PRSP", "PRSP"),
        ("Priest", "PRST", "PRST"),
        ("Prindle", "PRNT", "PRNT"),
        ("Prior", "PRR", "PRR"),
        ("Profumo", "PRFM", "PRFM"),
        ("Purdy", "PRT", "PRT"),
        ("Purefoy", "PRF", "PRF"),
        ("Pury", "PR", "PR"),
        ("Quinter", "KNTR", "KNTR"),
        ("Rachel", "RXL", "RKL"),
        ("Rand", "RNT", "RNT"),
        ("Rankin", "RNKN", "RNKN"),
        ("Ravenscroft", "RFNS", "RFNS"),
        ("Raynsford", "RNSF", "RNSF"),
        ("Reakirt", "RKRT", "RKRT"),
        ("Reaves", "RFS", "RFS"),
        ("Reeves", "RFS", "RFS"),
        ("Reichert", "RXRT", "RKRT"),
        ("Remmele", "RML", "RML"),
        ("Reynolds", "RNLT", "RNLT"),
        ("Rhodes", "RTS", "RTS"),
        ("Richards", "RXRT", "RKRT"),
        ("Richardson", "RXRT", "RKRT"),
        ("Ring", "RNK", "RNK"),
        ("Roberts", "RPRT", "RPRT"),
        ("Robertson", "RPRT", "RPRT"),
        ("Robson", "RPSN", "RPSN"),
        ("Rodie", "RT", "RT"),
        ("Rody", "RT", "RT"),
        ("Rogers", "RKRS", "RJRS"),
        ("Ross", "RS", "RS"),
        ("Rosslevin", "RSLF", "RSLF"),
        ("Rowland", "RLNT", "RLNT"),
        ("Ruehl", "RL", "RL"),
        ("Russell", "RSL", "RSL"),
        ("Ruth", "R0", "RT"),
        ("Ryan", "RN", "RN"),
        ("Rysse", "RS", "RS"),
        ("Sadler", "STLR", "STLR"),
        ("Salmon", "SLMN", "SLMN"),
        ("Salter", "SLTR", "SLTR"),
        ("Salvatore", "SLFT", "SLFT"),
        ("Sanders", "SNTR", "SNTR"),
        ("Sands", "SNTS", "SNTS"),
        ("Sanford", "SNFR", "SNFR"),
        ("Sanger", "SNKR", "SNJR"),
        ("Sargent", "SRJN", "SRKN"),
        ("Saunders", "SNTR", "SNTR"),
        ("Schilling", "XLNK", "XLNK"),
        ("Schlegel", "XLKL", "SLKL"),
        ("Scott", "SKT", "SKT"),
        ("Sears", "SRS", "SRS"),
        ("Segersall", "SJRS", "SKRS"),
        ("Senecal", "SNKL", "SNKL"),
        ("Sergeaux", "SRJ", "SRK"),
        ("Severance", "SFRN", "SFRN"),
        ("Sharp", "XRP", "XRP"),
        ("Sharpe", "XRP", "XRP"),
        ("Sharply", "XRPL", "XRPL"),
        ("Shatswell", "XTSL", "XTSL"),
        ("Shattack", "XTK", "XTK"),
        ("Shattock", "XTK", "XTK"),
        ("Shattuck", "XTK", "XTK"),
        ("Shaw", "X", "XF"),
        ("Sheldon", "XLTN", "XLTN"),
        ("Sherman", "XRMN", "XRMN"),
        ("Shinn", "XN", "XN"),
        ("Shirford", "XRFR", "XRFR"),
        ("Shirley", "XRL", "XRL"),
        ("Shively", "XFL", "XFL"),
        ("Shoemaker", "XMKR", "XMKR"),
        ("Short", "XRT", "XRT"),
        ("Shotwell", "XTL", "XTL"),
        ("Shute", "XT", "XT"),
        ("Sibley", "SPL", "SPL"),
        ("Silver", "SLFR", "SLFR"),
        ("Simes", "SMS", "SMS"),
        ("Sinken", "SNKN", "SNKN"),
        ("Sinn", "SN", "SN"),
        ("Skelton", "SKLT", "SKLT"),
        ("Skiffe", "SKF", "SKF"),
        ("Skotkonung", "SKTK", "SKTK"),
        ("Slade", "SLT", "XLT"),
        ("Slye", "SL", "XL"),
        ("Smedley", "SMTL", "XMTL"),
        ("Smith", "SM0", "XMT"),
        ("Smythe", "SM0", "XMT"),
        ("Snow", "SN", "XNF"),
        ("Soole", "SL", "SL"),
        ("Soule", "SL", "SL"),
        ("Southworth", "S0R0", "STRT"),
        ("Sowles", "SLS", "SLS"),
        ("Spalding", "SPLT", "SPLT"),
        ("Spark", "SPRK", "SPRK"),
        ("Spencer", "SPNS", "SPNS"),
        ("Sperry", "SPR", "SPR"),
        ("Spofford", "SPFR", "SPFR"),
        ("Spooner", "SPNR", "SPNR"),
        ("Sprague", "SPRK", "SPRK"),
        ("Springer", "SPRN", "SPRN"),
        ("St. Clair", "STKL", "STKL"),
        ("St. Claire", "STKL", "STKL"),
        ("St. Leger", "STLJ", "STLK"),
        ("St. Omer", "STMR", "STMR"),
        ("Stafferton", "STFR", "STFR"),
        ("Stafford", "STFR", "STFR"),
        ("Stalham", "STLM", "STLM"),
        ("Stanford", "STNF", "STNF"),
        ("Stanton", "STNT", "STNT"),
        ("Star", "STR", "STR"),
        ("Starbuck", "STRP", "STRP"),
        ("Starkey", "STRK", "STRK"),
        ("Starkweather", "STRK", "STRK"),
        ("Stearns", "STRN", "STRN"),
        ("Stebbins", "STPN", "STPN"),
        ("Steele", "STL", "STL"),
        ("Stephenson", "STFN", "STFN"),
        ("Stevens", "STFN", "STFN"),
        ("Stoddard", "STTR", "STTR"),
        ("Stodder", "STTR", "STTR"),
        ("Stone", "STN", "STN"),
        ("Storey", "STR", "STR"),
        ("Storrada", "STRT", "STRT"),
        ("Story", "STR", "STR"),
        ("Stoughton", "STFT", "STFT"),
        ("Stout", "STT", "STT"),
        ("Stow", "ST", "STF"),
        ("Strong", "STRN", "STRN"),
        ("Strutt", "STRT", "STRT"),
        ("Stryker", "STRK", "STRK"),
        ("Stuckeley", "STKL", "STKL"),
        ("Sturges", "STRJ", "STRK"),
        ("Sturgess", "STRJ", "STRK"),
        ("Sturgis", "STRJ", "STRK"),
        ("Suevain", "SFN", "SFN"),
        ("Sulyard", "SLRT", "SLRT"),
        ("Sutton", "STN", "STN"),
        ("Swain", "SN", "XN"),
        ("Swayne", "SN", "XN"),
        ("Swayze", "SS", "XTS"),
        ("Swift", "SFT", "XFT"),
        ("Taber", "TPR", "TPR"),
        ("Talcott", "TLKT", "TLKT"),
        ("Tarne", "TRN", "TRN"),
        ("Tatum", "TTM", "TTM"),
        ("Taverner", "TFRN", "TFRN"),
        ("Taylor", "TLR", "TLR"),
        ("Tenney", "TN", "TN"),
        ("Thayer", "0R", "TR"),
        ("Thember", "0MPR", "TMPR"),
        ("Thomas", "TMS", "TMS"),
        ("Thompson", "TMPS", "TMPS"),
        ("Thorne", "0RN", "TRN"),
        ("Thornycraft", "0RNK", "TRNK"),
        ("Threlkeld", "0RLK", "TRLK"),
        ("Throckmorton", "0RKM", "TRKM"),
        ("Thwaits", "0TS", "TTS"),
        ("Tibbetts", "TPTS", "TPTS"),
        ("Tidd", "TT", "TT"),
        ("Tierney", "TRN", "TRN"),
        ("Tilley", "TL", "TL"),
        ("Tillieres", "TLRS", "TLRS"),
        ("Tilly", "TL", "TL"),
        ("Tisdale", "TSTL", "TSTL"),
        ("Titus", "TTS", "TTS"),
        ("Tobey", "TP", "TP"),
        ("Tooker", "TKR", "TKR"),
        ("Towle", "TL", "TL"),
        ("Towne", "TN", "TN"),
        ("Townsend", "TNSN", "TNSN"),
        ("Treadway", "TRT", "TRT"),
        ("Trelawney", "TRLN", "TRLN"),
        ("Trinder", "TRNT", "TRNT"),
        ("Tripp", "TRP", "TRP"),
        ("Trippe", "TRP", "TRP"),
        ("Trott", "TRT", "TRT"),
        ("True", "TR", "TR"),
        ("Trussebut", "TRSP", "TRSP"),
        ("Tucker", "TKR", "TKR"),
        ("Turgeon", "TRJN", "TRKN"),
        ("Turner", "TRNR", "TRNR"),
        ("Tuttle", "TTL", "TTL"),
        ("Tyler", "TLR", "TLR"),
        ("Tylle", "TL", "TL"),
        ("Tyrrel", "TRL", "TRL"),
        ("Ua Tuathail", "AT0L", "ATTL"),
        ("Ulrich", "ALRX", "ALRK"),
        ("Underhill", "ANTR", "ANTR"),
        ("Underwood", "ANTR", "ANTR"),
        ("Unknown", "ANKN", "ANKN"),
        ("Valentine", "FLNT", "FLNT"),
        ("Van Egmond", "FNKM", "FNKM"),
        ("Van der Beek", "FNTR", "FNTR"),
        ("Vaughan", "FKN", "FKN"),
        ("Vermenlen", "FRMN", "FRMN"),
        ("Vincent", "FNSN", "FNSN"),
        ("Volentine", "FLNT", "FLNT"),
        ("Wagner", "AKNR", "FKNR"),
        ("Waite", "AT", "FT"),
        ("Walker", "ALKR", "FLKR"),
        ("Walter", "ALTR", "FLTR"),
        ("Wandell", "ANTL", "FNTL"),
        ("Wandesford", "ANTS", "FNTS"),
        ("Warbleton", "ARPL", "FRPL"),
        ("Ward", "ART", "FRT"),
        ("Warde", "ART", "FRT"),
        ("Ware", "AR", "FR"),
        ("Wareham", "ARHM", "FRHM"),
        ("Warner", "ARNR", "FRNR"),
        ("Warren", "ARN", "FRN"),
        ("Washburne", "AXPR", "FXPR"),
        ("Waterbury", "ATRP", "FTRP"),
        ("Watson", "ATSN", "FTSN"),
        ("WatsonEllithorpe", "ATSN", "FTSN"),
        ("Watts", "ATS", "FTS"),
        ("Wayne", "AN", "FN"),
        ("Webb", "AP", "FP"),
        ("Weber", "APR", "FPR"),
        ("Webster", "APST", "FPST"),
        ("Weed", "AT", "FT"),
        ("Weeks", "AKS", "FKS"),
        ("Wells", "ALS", "FLS"),
        ("Wenzell", "ANSL", "FNTS"),
        ("West", "AST", "FST"),
        ("Westbury", "ASTP", "FSTP"),
        ("Whatlocke", "ATLK", "ATLK"),
        ("Wheeler", "ALR", "ALR"),
        ("Whiston", "ASTN", "ASTN"),
        ("White", "AT", "AT"),
        ("Whitman", "ATMN", "ATMN"),
        ("Whiton", "ATN", "ATN"),
        ("Whitson", "ATSN", "ATSN"),
        ("Wickes", "AKS", "FKS"),
        ("Wilbur", "ALPR", "FLPR"),
        ("Wilcotes", "ALKT", "FLKT"),
        ("Wilkinson", "ALKN", "FLKN"),
        ("Willets", "ALTS", "FLTS"),
        ("Willett", "ALT", "FLT"),
        ("Willey", "AL", "FL"),
        ("Williams", "ALMS", "FLMS"),
        ("Williston", "ALST", "FLST"),
        ("Wilson", "ALSN", "FLSN"),
        ("Wimes", "AMS", "FMS"),
        ("Winch", "ANX", "FNK"),
        ("Winegar", "ANKR", "FNKR"),
        ("Wing", "ANK", "FNK"),
        ("Winsley", "ANSL", "FNSL"),
        ("Winslow", "ANSL", "FNSL"),
        ("Winthrop", "AN0R", "FNTR"),
        ("Wise", "AS", "FS"),
        ("Wood", "AT", "FT"),
        ("Woodbridge", "ATPR", "FTPR"),
        ("Woodward", "ATRT", "FTRT"),
        ("Wooley", "AL", "FL"),
        ("Woolley", "AL", "FL"),
        ("Worth", "AR0", "FRT"),
        ("Worthen", "AR0N", "FRTN"),
        ("Worthley", "AR0L", "FRTL"),
        ("Wright", "RT", "RT"),
        ("Wyer", "AR", "FR"),
        ("Wyere", "AR", "FR"),
        ("Wynkoop", "ANKP", "FNKP"),
        ("Yarnall", "ARNL", "ARNL"),
        ("Yeoman", "AMN", "AMN"),
        ("Yorke", "ARK", "ARK"),
        ("Young", "ANK", "ANK"),
        ("ab Wennonwen", "APNN", "APNN"),
        ("ap Llewellyn", "APLL", "APLL"),
        ("ap Lorwerth", "APLR", "APLR"),
        ("d'Angouleme", "TNKL", "TNKL"),
        ("de Audeham", "TTHM", "TTHM"),
        ("de Bavant", "TPFN", "TPFN"),
        ("de Beauchamp", "TPXM", "TPKM"),
        ("de Beaumont", "TPMN", "TPMN"),
        ("de Bolbec", "TPLP", "TPLP"),
        ("de Braiose", "TPRS", "TPRS"),
        ("de Braose", "TPRS", "TPRS"),
        ("de Briwere", "TPRR", "TPRR"),
        ("de Cantelou", "TKNT", "TKNT"),
        ("de Cherelton", "TXRL", "TKRL"),
        ("de Cherleton", "TXRL", "TKRL"),
        ("de Clare", "TKLR", "TKLR"),
        ("de Claremont", "TKLR", "TKLR"),
        ("de Clifford", "TKLF", "TKLF"),
        ("de Colville", "TKLF", "TKLF"),
        ("de Courtenay", "TKRT", "TKRT"),
        ("de Fauconberg", "TFKN", "TFKN"),
        ("de Forest", "TFRS", "TFRS"),
        ("de Gai", "TK", "TK"),
        ("de Grey", "TKR", "TKR"),
        ("de Guernons", "TKRN", "TKRN"),
        ("de Haia", "T", "T"),
        ("de Harcourt", "TRKR", "TRKR"),
        ("de Hastings", "TSTN", "TSTN"),
        ("de Hoke", "TK", "TK"),
        ("de Hooch", "TK", "TK"),
        ("de Hugelville", "TJLF", "TKLF"),
        ("de Huntingdon", "TNTN", "TNTN"),
        ("de Insula", "TNSL", "TNSL"),
        ("de Keynes", "TKNS", "TKNS"),
        ("de Lacy", "TLS", "TLS"),
        ("de Lexington", "TLKS", "TLKS"),
        ("de Lusignan", "TLSN", "TLSK"),
        ("de Manvers", "TMNF", "TMNF"),
        ("de Montagu", "TMNT", "TMNT"),
        ("de Montault", "TMNT", "TMNT"),
        ("de Montfort", "TMNT", "TMNT"),
        ("de Mortimer", "TMRT", "TMRT"),
        ("de Morville", "TMRF", "TMRF"),
        ("de Morvois", "TMRF", "TMRF"),
        ("de Neufmarche", "TNFM", "TNFM"),
        ("de Odingsells", "TTNK", "TTNK"),
        ("de Odyngsells", "TTNK", "TTNK"),
        ("de Percy", "TPRS", "TPRS"),
        ("de Pierrepont", "TPRP", "TPRP"),
        ("de Plessetis", "TPLS", "TPLS"),
        ("de Porhoet", "TPRT", "TPRT"),
        ("de Prouz", "TPRS", "TPRS"),
        ("de Quincy", "TKNS", "TKNS"),
        ("de Ripellis", "TRPL", "TRPL"),
        ("de Ros", "TRS", "TRS"),
        ("de Salisbury", "TSLS", "TSLS"),
        ("de Sanford", "TSNF", "TSNF"),
        ("de Somery", "TSMR", "TSMR"),
        ("de St. Hilary", "TSTL", "TSTL"),
        ("de St. Liz", "TSTL", "TSTL"),
        ("de Sutton", "TSTN", "TSTN"),
        ("de Toeni", "TTN", "TTN"),
        ("de Tony", "TTN", "TTN"),
        ("de Umfreville", "TMFR", "TMFR"),
        ("de Valognes", "TFLN", "TFLK"),
        ("de Vaux", "TF", "TF"),
        ("de Vere", "TFR", "TFR"),
        ("de Vermandois", "TFRM", "TFRM"),
        ("de Vernon", "TFRN", "TFRN"),
        ("de Vexin", "TFKS", "TFKS"),
        ("de Vitre", "TFTR", "TFTR"),
        ("de Wandesford", "TNTS", "TNTS"),
        ("de Warenne", "TRN", "TRN"),
        ("de Westbury", "TSTP", "TSTP"),
        ("di Saluzzo", "TSLS", "TSLT"),
        ("fitz Alan", "FTSL", "FTSL"),
        ("fitz Geoffrey", "FTSJ", "FTSK"),
        ("fitz Herbert", "FTSR", "FTSR"),
        ("fitz John", "FTSJ", "FTSJ"),
        ("fitz Patrick", "FTSP", "FTSP"),
        ("fitz Payn", "FTSP", "FTSP"),
        ("fitz Piers", "FTSP", "FTSP"),
        ("fitz Randolph", "FTSR", "FTSR"),
        ("fitz Richard", "FTSR", "FTSR"),
        ("fitz Robert", "FTSR", "FTSR"),
        ("fitz Roy", "FTSR", "FTSR"),
        ("fitz Scrob", "FTSS", "FTSS"),
        ("fitz Walter", "FTSL", "FTSL"),
        ("fitz Warin", "FTSR", "FTSR"),
        ("fitz Williams", "FTSL", "FTSL"),
        ("la Zouche", "LSX", "LSK"),
        ("le Botiller", "LPTL", "LPTL"),
        ("le Despenser", "LTSP", "LTSP"),
        ("le deSpencer", "LTSP", "LTSP"),
        ("of Allendale", "AFLN", "AFLN"),
        ("of Angouleme", "AFNK", "AFNK"),
        ("of Anjou", "AFNJ", "AFNJ"),
        ("of Aquitaine", "AFKT", "AFKT"),
        ("of Aumale", "AFML", "AFML"),
        ("of Bavaria", "AFPF", "AFPF"),
        ("of Boulogne", "AFPL", "AFPL"),
        ("of Brittany", "AFPR", "AFPR"),
        ("of Brittary", "AFPR", "AFPR"),
        ("of Castile", "AFKS", "AFKS"),
        ("of Chester", "AFXS", "AFKS"),
        ("of Clermont", "AFKL", "AFKL"),
        ("of Cologne", "AFKL", "AFKL"),
        ("of Dinan", "AFTN", "AFTN"),
        ("of Dunbar", "AFTN", "AFTN"),
        ("of England", "AFNK", "AFNK"),
        ("of Essex", "AFSK", "AFSK"),
        ("of Falaise", "AFFL", "AFFL"),
        ("of Flanders", "AFFL", "AFFL"),
        ("of Galloway", "AFKL", "AFKL"),
        ("of Germany", "AFKR", "AFJR"),
        ("of Gloucester", "AFKL", "AFKL"),
        ("of Heristal", "AFRS", "AFRS"),
        ("of Hungary", "AFNK", "AFNK"),
        ("of Huntington", "AFNT", "AFNT"),
        ("of Kiev", "AFKF", "AFKF"),
        ("of Kuno", "AFKN", "AFKN"),
        ("of Landen", "AFLN", "AFLN"),
        ("of Laon", "AFLN", "AFLN"),
        ("of Leinster", "AFLN", "AFLN"),
        ("of Lens", "AFLN", "AFLN"),
        ("of Lorraine", "AFLR", "AFLR"),
        ("of Louvain", "AFLF", "AFLF"),
        ("of Mercia", "AFMR", "AFMR"),
        ("of Metz", "AFMT", "AFMT"),
        ("of Meulan", "AFML", "AFML"),
        ("of Nass", "AFNS", "AFNS"),
        ("of Normandy", "AFNR", "AFNR"),
        ("of Ohningen", "AFNN", "AFNN"),
        ("of Orleans", "AFRL", "AFRL"),
        ("of Poitou", "AFPT", "AFPT"),
        ("of Polotzk", "AFPL", "AFPL"),
        ("of Provence", "AFPR", "AFPR"),
        ("of Ringelheim", "AFRN", "AFRN"),
        ("of Salisbury", "AFSL", "AFSL"),
        ("of Saxony", "AFSK", "AFSK"),
        ("of Scotland", "AFSK", "AFSK"),
        ("of Senlis", "AFSN", "AFSN"),
        ("of Stafford", "AFST", "AFST"),
        ("of Swabia", "AFSP", "AFSP"),
        ("of Tongres", "AFTN", "AFTN"),
        ("of the Tributes", "AF0T", "AFTT"),
        ("unknown", "ANKN", "ANKN"),
        ("van der Gouda", "FNTR", "FNTR"),
        ("von Adenbaugh", "FNTN", "FNTN"),
        ("ARCHITure", "ARKT", "ARKT"),
        ("Arnoff", "ARNF", "ARNF"),
        ("Arnow", "ARN", "ARNF"),
        ("DANGER", "TNJR", "TNKR"),
        ("Jankelowicz", "JNKL", "ANKL"),
        ("MANGER", "MNJR", "MNKR"),
        ("McClellan", "MKLL", "MKLL"),
        ("McHugh", "MK", "MK"),
        ("McLaughlin", "MKLF", "MKLF"),
        ("ORCHEStra", "ARKS", "ARKS"),
        ("ORCHID", "ARKT", "ARKT"),
        ("Pierce", "PRS", "PRS"),
        ("RANGER", "RNJR", "RNKR"),
        ("Schlesinger", "XLSN", "SLSN"),
        ("Uomo", "AM", "AM"),
        ("Vasserman", "FSRM", "FSRM"),
        ("Wasserman", "ASRM", "FSRM"),
        ("Womo", "AM", "FM"),
        ("Yankelovich", "ANKL", "ANKL"),
        ("accede", "AKST", "AKST"),
        ("accident", "AKST", "AKST"),
        ("adelsheim", "ATLS", "ATLS"),
        ("aged", "AJT", "AKT"),
        ("ageless", "AJLS", "AKLS"),
        ("agency", "AJNS", "AKNS"),
        ("aghast", "AKST", "AKST"),
        ("agio", "AJ", "AK"),
        ("agrimony", "AKRM", "AKRM"),
        ("album", "ALPM", "ALPM"),
        ("alcmene", "ALKM", "ALKM"),
        ("alehouse", "ALHS", "ALHS"),
        ("antique", "ANTK", "ANTK"),
        ("artois", "ART", "ARTS"),
        ("automation", "ATMX", "ATMX"),
        ("bacchus", "PKS", "PKS"),
        ("bacci", "PX", "PX"),
        ("bajador", "PJTR", "PHTR"),
        ("bellocchio", "PLX", "PLX"),
        ("bertucci", "PRTX", "PRTX"),
        ("biaggi", "PJ", "PK"),
        ("bough", "P", "P"),
        ("breaux", "PR", "PR"),
        ("broughton", "PRTN", "PRTN"),
        ("cabrillo", "KPRL", "KPR"),
        ("caesar", "SSR", "SSR"),
        ("cagney", "KKN", "KKN"),
        ("campbell", "KMPL", "KMPL"),
        ("carlisle", "KRLL", "KRLL"),
        ("carlysle", "KRLL", "KRLL"),
        ("chemistry", "KMST", "KMST"),
        ("chianti", "KNT", "KNT"),
        ("chorus", "KRS", "KRS"),
        ("cough", "KF", "KF"),
        ("czerny", "SRN", "XRN"),
        ("deffenbacher", "TFNP", "TFNP"),
        ("dumb", "TM", "TM"),
        ("edgar", "ATKR", "ATKR"),
        ("edge", "AJ", "AJ"),
        ("filipowicz", "FLPT", "FLPF"),
        ("focaccia", "FKX", "FKX"),
        ("gallegos", "KLKS", "KKS"),
        ("gambrelli", "KMPR", "KMPR"),
        ("geithain", "K0N", "JTN"),
        ("ghiradelli", "JRTL", "JRTL"),
        ("ghislane", "JLN", "JLN"),
        ("gough", "KF", "KF"),
        ("hartheim", "HR0M", "HRTM"),
        ("heimsheim", "HMSM", "HMSM"),
        ("hochmeier", "HKMR", "HKMR"),
        ("hugh", "H", "H"),
        ("hunger", "HNKR", "HNJR"),
        ("hungry", "HNKR", "HNKR"),
        ("island", "ALNT", "ALNT"),
        ("isle", "AL", "AL"),
        ("jose", "HS", "HS"),
        ("laugh", "LF", "LF"),
        ("mac caffrey", "MKFR", "MKFR"),
        ("mac gregor", "MKRK", "MKRK"),
        ("pegnitz", "PNTS", "PKNT"),
        ("piskowitz", "PSKT", "PSKF"),
        ("queen", "KN", "KN"),
        ("raspberry", "RSPR", "RSPR"),
        ("resnais", "RSN", "RSNS"),
        ("rogier", "RJ", "RJR"),
        ("rough", "RF", "RF"),
        ("san jacinto", "SNHS", "SNHS"),
        ("schenker", "XNKR", "SKNK"),
        ("schermerhorn", "XRMR", "SKRM"),
        ("schmidt", "XMT", "SMT"),
        ("schneider", "XNTR", "SNTR"),
        ("school", "SKL", "SKL"),
        ("schooner", "SKNR", "SKNR"),
        ("schrozberg", "XRSP", "SRSP"),
        ("schulman", "XLMN", "XLMN"),
        ("schwabach", "XPK", "XFPK"),
        ("schwarzach", "XRSK", "XFRT"),
        ("smith", "SM0", "XMT"),
        ("snider", "SNTR", "XNTR"),
        ("succeed", "SKST", "SKST"),
        ("sugarcane", "XKRK", "SKRK"),
        ("svobodka", "SFPT", "SFPT"),
        ("tagliaro", "TKLR", "TLR"),
        ("thames", "TMS", "TMS"),
        ("theilheim", "0LM", "TLM"),
        ("thomas", "TMS", "TMS"),
        ("thumb", "0M", "TM"),
        ("tichner", "TXNR", "TKNR"),
        ("tough", "TF", "TF"),
        ("umbrella", "AMPR", "AMPR"),
        ("vilshofen", "FLXF", "FLXF"),
        ("von schuller", "FNXL", "FNXL"),
        ("wachtler", "AKTL", "FKTL"),
        ("wechsler", "AKSL", "FKSL"),
        ("weikersheim", "AKRS", "FKRS"),
        ("zhao", "J", "J"),
    ];

    #[test]
    fn check_double_metaphone() {
        let encoder = DoubleMetaphone::default();

        for (i, (value, primary, alternate)) in TEST_DATA.iter().enumerate() {
            let result = encoder.encode(value);
            assert_eq!(result, primary.to_string(), "[{i}] primary {value} fail");
            let result = encoder.encode_alternate(value);
            assert_eq!(
                result,
                alternate.to_string(),
                "[{i}] alternate {value} fail"
            );
        }
    }
}
