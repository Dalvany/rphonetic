//! This library contains a set of phonetic algorithms from [Apache commons-codec](https://commons.apache.org/proper/commons-codec/)
//! written in Rust.
//!
//! It currently implements :
//!
//! * [Caverphone1] : see [Wikipedia](https://en.wikipedia.org/wiki/Caverphone).
//! * [Caverphone2] : see [Wikipedia](https://en.wikipedia.org/wiki/Caverphone).
//! * [Cologne] : see [Wikipedia](https://en.wikipedia.org/wiki/Cologne_phonetics).
//! * [Daitch-Motokoff soundex] : see [Wikipedia](https://en.wikipedia.org/wiki/Daitch%E2%80%93Mokotoff_Soundex)
//! * [Double Metaphone] : see [Wikipedia](https://en.wikipedia.org/wiki/Metaphone#Double_Metaphone)
#[macro_use]
extern crate lazy_static;

use std::fmt;
use std::fmt::Formatter;

use regex::Regex;

pub use crate::caverphone::Caverphone1;
pub use crate::caverphone::Caverphone2;
pub use crate::cologne::Cologne;
pub use crate::daitch_mokotoff::{DaitchMokotoffSoundex, DaitchMokotoffSoundexBuilder};
pub use crate::double_metaphone::DoubleMetaphone;

mod caverphone;
mod cologne;
mod daitch_mokotoff;
mod double_metaphone;
mod helper;

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
    /// let caverphone = Caverphone1::new();
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
    /// let caverphone = Caverphone1::new();
    /// assert!(!caverphone.is_encoded_equals("Peter", "Stevenson"));
    /// assert!(caverphone.is_encoded_equals("Peter", "Peady"));
    /// ```
    fn is_encoded_equals(&self, first: &str, second: &str) -> bool {
        let f = self.encode(first);
        let s = self.encode(second);

        f == s
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
