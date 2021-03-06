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
//! This library contains a set of phonetic algorithms from [Apache commons-codec](https://commons.apache.org/proper/commons-codec/)
//! written in Rust.
//!
//! It currently implements :
//!
//! * [Caverphone1] : see [Wikipedia](https://en.wikipedia.org/wiki/Caverphone).
//! * [Caverphone2] : see [Wikipedia](https://en.wikipedia.org/wiki/Caverphone).
//! * [Cologne] : see [Wikipedia](https://en.wikipedia.org/wiki/Cologne_phonetics).
//! * [DaitchMokotoffSoundex] : see [Wikipedia](https://en.wikipedia.org/wiki/Daitch%E2%80%93Mokotoff_Soundex)
//! * [DoubleMetaphone] : see [Wikipedia](https://en.wikipedia.org/wiki/Metaphone#Double_Metaphone)
//! * [MatchRatingApproach] : see [Wikipedia](https://en.wikipedia.org/wiki/Match_rating_approach)
//! * [Metaphone] : see [Wikipedia](https://en.wikipedia.org/wiki/Metaphone)
//! * [Nysiis] : see [Wikipedia](https://en.wikipedia.org/wiki/New_York_State_Identification_and_Intelligence_System)
//! * [RefinedSoundex] : see [Wikipedia](https://en.wikipedia.org/wiki/Soundex)
//! * [Soundex] : see [Wikipedia](https://en.wikipedia.org/wiki/Soundex)
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    trivial_numeric_casts,
    unsafe_code,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications
)]
#[macro_use]
extern crate lazy_static;

use std::fmt;
use std::fmt::Formatter;

use regex::Regex;

pub use crate::caverphone::Caverphone1;
pub use crate::caverphone::Caverphone2;
pub use crate::cologne::Cologne;
pub use crate::daitch_mokotoff::{DaitchMokotoffSoundex, DaitchMokotoffSoundexBuilder};
pub use crate::double_metaphone::{DoubleMetaphone, DoubleMetaphoneResult};
pub use crate::match_rating_approach::MatchRatingApproach;
pub use crate::metaphone::Metaphone;
pub use crate::nysiis::Nysiis;
pub use crate::refined_soundex::RefinedSoundex;
pub use crate::soundex::{
    Soundex, DEFAULT_US_ENGLISH_GENEALOGY_MAPPING_SOUNDEX, DEFAULT_US_ENGLISH_MAPPING_SOUNDEX,
};

mod caverphone;
mod cologne;
mod daitch_mokotoff;
mod double_metaphone;
mod helper;
mod match_rating_approach;
mod metaphone;
mod nysiis;
mod refined_soundex;
mod soundex;

lazy_static! {
    static ref RULE_LINE: Regex = Regex::new(
        r"\s*\x22(.+?)\x22\s+\x22(.*?)\x22\s+\x22(.*?)\x22\s+\x22(.*?)\x22\s*(//.*){0,1}$"
    )
    .unwrap();
}

/// Errors
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum PhoneticError {
    /// This variant is raised when there is an error in the rule
    /// file of Daitch Mokotoff soundex or Beider Morse.
    ParseRuleError(String),
}

impl fmt::Display for PhoneticError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseRuleError(error) => write!(f, "Error parsing rule file : {}", error),
        }
    }
}

/// This trait represents a phonetic algorithm.
pub trait Encoder {
    /// This method convert a string into its code.
    ///
    /// # Parameter
    ///
    /// * `s` : string to encode.
    ///
    /// # Return
    ///
    /// String encoded.
    ///
    /// # Example
    ///
    /// Example using [Caverphone 1] algorithm.
    ///
    /// ```rust
    /// use rphonetic::{Caverphone1, Encoder};
    ///
    /// let caverphone = Caverphone1;
    ///
    /// assert_eq!(caverphone.encode("Thompson"), "TMPSN1");
    /// ```
    fn encode(&self, s: &str) -> String;

    /// This method check that two strings have the same code.
    ///
    /// # Parameters
    ///
    /// * `first` : first string.
    /// * `second` : second string.
    ///
    /// # Return
    ///
    /// Return `true` if both strings have the same code, false otherwise.
    ///
    /// # Example
    ///
    /// Example with [Caverphone1]
    ///
    /// ```rust
    /// use rphonetic::{Encoder, Caverphone1};
    ///
    /// let caverphone = Caverphone1;
    /// assert!(!caverphone.is_encoded_equals("Peter", "Stevenson"));
    /// assert!(caverphone.is_encoded_equals("Peter", "Peady"));
    /// ```
    fn is_encoded_equals(&self, first: &str, second: &str) -> bool {
        let f = self.encode(first);
        let s = self.encode(second);

        f == s
    }
}

trait SoundexUtils {
    fn soundex_clean(value: &str) -> String {
        value
            .chars()
            .filter(|c| c.is_alphabetic())
            .map(|c| c.to_uppercase().collect::<String>())
            .collect()
    }
}

/// This trait represent a soundex algorithm (except for [Nysiis]).
///
/// It has a method, [difference(value1, value2)](Soundex::difference) that returns
/// the number of letter that are at the same place in both encoded strings.
pub trait SoundexCommons: Encoder {
    /// This methode compute the number of characters thar are at the same place
    /// in both encoded strings.
    ///
    /// It calls [encode(value)](Encoder::encode).
    ///
    ///
    /// # Parameters
    ///
    /// * `value1` : first value
    /// * `value2` : second value
    ///
    /// # Return
    ///
    /// The number of characters at the same position. 0 indicates no similarities, while 4 (out of 4)
    /// indicates strong similarity. Please note that [RefinedSoundex] difference can be greater than 4.
    ///
    /// # Examples
    ///
    /// An example with [RefinedSoundex] :
    ///
    /// ```rust
    /// use rphonetic::{RefinedSoundex, Soundex, SoundexCommons};
    ///
    /// let refined_soundex = RefinedSoundex::default();
    ///
    /// // Low similarity
    /// assert_eq!(refined_soundex.difference("Margaret", "Andrew"), 1);
    ///
    /// // High similarity
    /// assert_eq!(refined_soundex.difference("Smithers", "Smythers"), 8);
    /// ```
    ///
    /// With [Soundex], maximum proximity will be 4 as values are coded with 4 characters :
    ///
    /// ```rust
    /// use rphonetic::{Soundex, SoundexCommons};
    ///
    /// let soundex = Soundex::default();
    ///
    /// // Low similarity
    /// assert_eq!(soundex.difference("Margaret", "Andrew"), 1);
    ///
    /// // High similarity
    /// assert_eq!(soundex.difference("Smithers", "Smythers"), 4);
    /// ```
    fn difference(&self, value1: &str, value2: &str) -> usize {
        let value1 = self.encode(value1);
        let value2 = self.encode(value2);

        if value1.is_empty() || value2.is_empty() {
            return 0;
        }

        let mut result: usize = 0;
        for (ch1, ch2) in value1.chars().zip(value2.chars()) {
            if ch1 == ch2 {
                result += 1;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regexp() {
        let data = "  \"part1\"   \"part2\" \"part3\"\t\"part4\"";
        assert!(RULE_LINE.is_match(data));
        for cap in RULE_LINE.captures_iter(data) {
            assert_eq!(&cap[1], "part1");
            assert_eq!(&cap[2], "part2");
            assert_eq!(&cap[3], "part3");
            assert_eq!(&cap[4], "part4");
        }
    }

    #[test]
    fn test_regexp_with_one_line_comment() {
        let data =
            "  \"part1\"   \"part2\"\t \"part3\"\t\"part4\"\t\t // This is a one line comment";
        assert!(RULE_LINE.is_match(data));
        for cap in RULE_LINE.captures_iter(data) {
            assert_eq!(&cap[1], "part1");
            assert_eq!(&cap[2], "part2");
            assert_eq!(&cap[3], "part3");
            assert_eq!(&cap[4], "part4");
        }
    }

    #[test]
    fn test_regexp_with_empty_parts() {
        let data = "  \"part1\"   \"part2\"\t \"\"\t\"\"\t\t";
        assert!(RULE_LINE.is_match(data));
        for cap in RULE_LINE.captures_iter(data) {
            assert_eq!(&cap[1], "part1");
            assert_eq!(&cap[2], "part2");
            assert_eq!(&cap[3], "");
            assert_eq!(&cap[4], "");
        }
    }

    #[test]
    fn test_regexp_no_match() {
        let data = "  \"part1\"   \t \"part3\"\t\"part4\"\t\t // This is not a match, missing a part \"test\"";
        assert!(!RULE_LINE.is_match(data));
    }

    #[test]
    fn test_regexp_whatever() {
        let data = "This is not a match";
        assert!(!RULE_LINE.is_match(data));
    }
}
