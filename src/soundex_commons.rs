use std::char::TryFromCharError;

use thiserror::Error;

use crate::Encoder;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
#[error("Error building soundex, the number of char must be 26 (0:?)")]
pub struct SoundexConvertError(pub Vec<char>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SoundexEncodeError {
    #[error(transparent)]
    TryFromChar(#[from] TryFromCharError),
    #[error("Found non ASCII character '{0}'. Soundex only support ASCII letter (ranging from 'a' to 'z') without regard to the casing.")]
    NonASCIICharacterUnsupported(char),
}

pub(crate) fn get_mapping_code(mapping: &[char; 26], ch: char) -> Result<char, SoundexEncodeError> {
    let number: usize = ch.try_into()?;
    let index = number
        .checked_sub(65)
        .ok_or(SoundexEncodeError::NonASCIICharacterUnsupported(ch))?;

    let ch = *mapping
        .get(index)
        .ok_or(SoundexEncodeError::NonASCIICharacterUnsupported(ch))?;

    Ok(ch)
}

pub(crate) trait SoundexUtils {
    fn soundex_clean(value: &str) -> String {
        value
            .chars()
            .filter(|c| c.is_ascii_alphabetic())
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
    /// # fn main() -> anyhow::Result<()> {
    /// use rphonetic::{RefinedSoundex, Soundex, SoundexCommons};
    ///
    /// let refined_soundex = RefinedSoundex::default();
    ///
    /// // Low similarity
    /// assert_eq!(refined_soundex.difference("Margaret", "Andrew")?, 1);
    ///
    /// // High similarity
    /// assert_eq!(refined_soundex.difference("Smithers", "Smythers")?, 8);
    /// #   Ok(())
    /// # }
    /// ```
    ///
    /// With [Soundex], maximum proximity will be 4 as values are coded with 4 characters :
    ///
    /// ```rust
    /// # fn main() -> anyhow::Result<()> {
    /// use rphonetic::{Soundex, SoundexCommons};
    ///
    /// let soundex = Soundex::default();
    ///
    /// // Low similarity
    /// assert_eq!(soundex.difference("Margaret", "Andrew")?, 1);
    ///
    /// // High similarity
    /// assert_eq!(soundex.difference("Smithers", "Smythers")?, 4);
    /// #   Ok(())
    /// # }
    /// ```
    fn difference(&self, value1: &str, value2: &str) -> Result<usize, Self::Error> {
        let value1 = self.encode(value1)?;
        let value2 = self.encode(value2)?;

        if value1.is_empty() || value2.is_empty() {
            return Ok(0);
        }

        let mut result: usize = 0;
        for (ch1, ch2) in value1.chars().zip(value2.chars()) {
            if ch1 == ch2 {
                result += 1;
            }
        }

        Ok(result)
    }
}
