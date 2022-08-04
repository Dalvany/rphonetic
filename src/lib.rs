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
//! * [BeiderMorse] : see [Wikipedia](https://en.wikipedia.org/wiki/Daitch%E2%80%93Mokotoff_Soundex#Beider%E2%80%93Morse_Phonetic_Name_Matching_Algorithm)
//!
//! Please note that most of these algorithms are design for ASCII, and they are usually design for certain use case (eg.
//! english names, ...etc).
//!
//! # Features
//!
//! There is two features that provide default rules and [Default] implementation for some struct. They are not enabled
//! by default as files are embedded into code, so it might increase binary size. It's best to provide rules by your own.
//!
//! * `embedded` : shorthand for `embedded_bm` and `embedded_dm`.
//!   * `embedded_bm` : Beider-Morse rules. It includes only `any` language and other files that are required. All file can be found in
//! [commons-codec repository](https://github.com/apache/commons-codec/tree/rel/commons-codec-1.15/src/main/resources/org/apache/commons/codec/language/bm)
//!   * `embedded_dm` : Daitch-Mokotoff rules. They can be also found in [commons-codec repository](https://github.com/apache/commons-codec/blob/rel/commons-codec-1.15/src/main/resources/org/apache/commons/codec/language/dmrules.txt)
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

use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use rules_parser::*;

pub use crate::beider_morse::{
    BMError, BeiderMorse, BeiderMorseBuilder, ConfigFiles, LanguageSet, NameType, RuleType,
};
pub use crate::caverphone::Caverphone1;
pub use crate::caverphone::Caverphone2;
pub use crate::cologne::Cologne;
pub use crate::daitch_mokotoff::{DaitchMokotoffSoundex, DaitchMokotoffSoundexBuilder};
pub use crate::double_metaphone::{DoubleMetaphone, DoubleMetaphoneResult};
pub use crate::helper::CharSequence;
pub use crate::match_rating_approach::MatchRatingApproach;
pub use crate::metaphone::Metaphone;
pub use crate::nysiis::Nysiis;
pub use crate::refined_soundex::RefinedSoundex;
pub use crate::soundex::{
    Soundex, DEFAULT_US_ENGLISH_GENEALOGY_MAPPING_SOUNDEX, DEFAULT_US_ENGLISH_MAPPING_SOUNDEX,
};

mod beider_morse;
mod caverphone;
mod cologne;
mod constants;
mod daitch_mokotoff;
mod double_metaphone;
mod helper;
mod match_rating_approach;
mod metaphone;
mod nysiis;
mod refined_soundex;
mod rules_parser;
mod soundex;

/// This represents a parsing error. It contains the
/// line number, the line, and if possible the filename.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ParseError {
    /// Line number
    pub line_number: usize,
    /// Filename
    pub filename: Option<String>,
    /// Wrong line
    pub line_content: String,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{} -> {}",
            self.filename
                .clone()
                .unwrap_or_else(|| "Unknown".to_string()),
            self.line_number,
            self.line_content
        )
    }
}

impl Error for ParseError {}

/// Errors
#[derive(Debug, Clone, PartialEq)]
pub enum PhoneticError {
    /// This variant contains parsing errors.
    ParseRuleError(ParseError),
    /// This error contains errors related to Beider Morse.
    BMError(BMError),
}

impl From<BMError> for PhoneticError {
    fn from(error: BMError) -> Self {
        Self::BMError(error)
    }
}

impl Display for PhoneticError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseRuleError(error) => write!(f, "Error parsing rule file {}", error),
            Self::BMError(error) => write!(f, "Error : {}", error),
        }
    }
}

impl Error for PhoneticError {}

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
    /// Example using [Caverphone1] algorithm.
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
