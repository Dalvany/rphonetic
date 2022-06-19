//! This library contains a set of phonetic algorithms from [Apache commons-codec](https://commons.apache.org/proper/commons-codec/)
//! written in Rust.
//!
//! It currently implements :
//!
//! * [Caverphone1] : see [Wikipedia](https://en.wikipedia.org/wiki/Caverphone).
//! * [Caverphone2] : see [Wikipedia](https://en.wikipedia.org/wiki/Caverphone).
pub use crate::caverphone::Caverphone1;
pub use crate::caverphone::Caverphone2;

mod caverphone;
mod helper;

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
