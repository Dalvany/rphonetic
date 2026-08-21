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
//! * [Phonex] see [paper](https://citeseerx.ist.psu.edu/viewdoc/download;jsessionid=E3997DC51F2046A95EE6459F2B997029?doi=10.1.1.453.4046&rep=rep1&type=pdf)
//!
//! Please note that most of these algorithms are design for ASCII, and they are usually design for certain use case (eg.
//! english names, ...etc).
//!
//! ## Feature flags
#![doc = document_features::document_features!()]
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
#![cfg_attr(docsrs, feature(doc_cfg))]

use nom::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use crate::beider_morse::{
    BMError, BeiderMorse, BeiderMorseBuilder, ConfigFiles, LanguageSet, NameType, ParseBmError,
    RuleType,
};
pub use crate::caverphone::{Caverphone1, Caverphone2};
pub use crate::cologne::Cologne;
pub use crate::daitch_mokotoff::{DaitchMokotoffSoundex, DaitchMokotoffSoundexBuilder};
pub use crate::double_metaphone::{DoubleMetaphone, DoubleMetaphoneResult};
pub use crate::helper::CharSequence;
pub use crate::match_rating_approach::MatchRatingApproach;
pub use crate::metaphone::Metaphone;
pub use crate::nysiis::Nysiis;
pub use crate::phonex::Phonex;
pub use crate::refined_soundex::RefinedSoundex;
pub use crate::soundex::{
    Soundex, DEFAULT_US_ENGLISH_GENEALOGY_MAPPING_SOUNDEX, DEFAULT_US_ENGLISH_MAPPING_SOUNDEX,
};
pub use crate::soundex_commons::{SoundexCommons, SoundexConvertError, SoundexEncodeError};

mod beider_morse;
mod caverphone;
mod cologne;
mod daitch_mokotoff;
mod double_metaphone;
mod helper;
mod match_rating_approach;
mod metaphone;
mod nom;
mod nysiis;
mod phonex;
mod refined_soundex;
mod soundex;
pub(crate) mod soundex_commons;

/// This represents a parsing error. It contains the
/// line number, the line, and if possible the filename.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Error, Serialize, Deserialize)]
#[error("{}:{line_number} {description} : {line_content}", filename.clone().unwrap_or_else(|| "Unknown".to_string()))]
pub struct ParseError {
    /// Line number
    pub line_number: usize,
    /// Filename
    pub filename: Option<String>,
    /// Wrong line
    pub line_content: String,
    /// Description
    pub description: String,
}

/// Errors
#[derive(Debug, Clone, PartialEq, Error)]
pub enum PhoneticError {
    /// This variant contains parsing errors.
    #[error("Error parsing rule file {0}")]
    ParseRuleError(#[from] ParseError),
    /// This error contains errors related to Beider Morse.
    #[error("Error : {0}")]
    BMError(#[from] BMError),
}

impl From<std::io::Error> for PhoneticError {
    fn from(error: std::io::Error) -> Self {
        Self::BMError(BMError::from(error))
    }
}

impl From<regex::Error> for PhoneticError {
    fn from(error: regex::Error) -> Self {
        Self::BMError(BMError::from(error))
    }
}

fn build_parse_error(
    line_number: usize,
    filename: Option<String>,
    remains: &str,
    description: String,
) -> ParseError {
    let eol = remains.find('\n');
    let line_content = match eol {
        None => remains,
        Some(index) => &remains[..index],
    }
    .to_string();

    ParseError {
        line_number,
        filename,
        line_content,
        description,
    }
}

/// This trait represents a phonetic algorithm.
pub trait Encoder {
    type Error: std::error::Error;

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
    /// # fn main() -> anyhow::Result<()> {
    /// use rphonetic::{Caverphone1, Encoder};
    ///
    /// let caverphone = Caverphone1;
    ///
    /// assert_eq!(caverphone.encode("Thompson")?, "TMPSN1");
    /// #   Ok(())
    /// # }
    /// ```
    fn encode(&self, s: &str) -> Result<String, Self::Error>;

    /// Call [encode](Self::encode) but unwrap the result.
    ///
    /// # Panic
    ///
    /// This method panic if the underlying `encode` call
    /// returns an error.
    fn encode_unchecked(&self, s: &str) -> String {
        #[allow(clippy::unwrap_in_result)]
        self.encode(s).unwrap()
    }

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
    /// # fn main() -> anyhow::Result<()> {
    /// use rphonetic::{Encoder, Caverphone1};
    ///
    /// let caverphone = Caverphone1;
    /// assert!(!caverphone.is_encoded_equals("Peter", "Stevenson")?);
    /// assert!(caverphone.is_encoded_equals("Peter", "Peady")?);
    /// #   Ok(())
    /// # }
    /// ```
    fn is_encoded_equals(&self, first: &str, second: &str) -> Result<bool, Self::Error> {
        let f = self.encode(first)?;
        let s = self.encode(second)?;

        Ok(f == s)
    }

    /// Call [is_encoded_equals](Self::is_encoded_equals) but unwrap the result.
    ///
    /// # Panic
    ///
    /// This method panic if the underlying `is_encoded_equals` call
    /// returns an error.
    fn is_encoded_equals_unchecked(&self, first: &str, second: &str) -> bool {
        #[allow(clippy::unwrap_in_result)]
        self.is_encoded_equals(first, second).unwrap()
    }
}
