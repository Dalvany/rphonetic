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
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::soundex_commons::SoundexUtils;
use crate::{Encoder, SoundexCommons, SoundexConvertError, SoundexEncodeError};

const ENGLISH_MAPPING: [char; 26] = [
    '0', '1', '3', '6', '0', '2', '4', '0', '0', '4', '3', '7', '8', '8', '0', '1', '5', '9', '3',
    '6', '0', '2', '0', '5', '0', '5',
];

/// This the [refined soundex]() implementation of [Encoder].
///
/// It works only with ASCII and contains an array that contains the code for each letter.
///
/// [Default] implementation provides an array for english US.
///
/// ```rust
/// # fn main() -> anyhow::Result<()> {
/// use rphonetic::{Encoder, RefinedSoundex};
/// let refined_soundex = RefinedSoundex::default();
///
/// assert_eq!(refined_soundex.encode("jumped")?, "J408106");
/// #   Ok(())
/// # }
/// ```
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct RefinedSoundex {
    mapping: [char; 26],
}

impl RefinedSoundex {
    /// Use this constructor to provide a custom array.
    ///
    /// There are implementations of [TryFrom] for convenience.
    ///
    /// # Parameter
    ///
    /// * `mapping`: mapping array.
    ///   It contains for each letter its corresponding code.
    ///   Index 0 is the code for `A`, index 1
    ///   is for `B`and so on for each letter of the latin alphabet.
    pub fn new(mapping: [char; 26]) -> Self {
        Self { mapping }
    }
}

impl From<[char; 26]> for RefinedSoundex {
    fn from(mapping: [char; 26]) -> Self {
        Self { mapping }
    }
}

impl TryFrom<&str> for RefinedSoundex {
    type Error = SoundexConvertError;

    /// Construct a [RefinedSoundex] from the mapping in parameter. This [str] will
    /// be converted into an array of 26 chars, so `mapping`'s length must be 26.
    ///
    /// # Parameters
    ///
    /// * `mapping`: str that contains the corresponding code for each character.
    ///
    /// # Errors
    ///
    /// Returns an error the number of [char] in the [str] isn't equals to 26.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> anyhow::Result<()> {
    /// use rphonetic::{Encoder, RefinedSoundex};
    ///
    /// // Construct an encoder with 'A' coded into '0', 'B' into '1', 'C' into '3', 'D' into '6', 'E' into '0', ...etc
    /// // (this is the default mapping)
    /// let refined_soundex = RefinedSoundex::try_from("01360240043788015936020505")?;
    ///
    /// assert_eq!(refined_soundex.encode("jumped")?, "J408106");
    /// #    Ok(())
    /// # }
    /// ```
    fn try_from(mapping: &str) -> Result<Self, Self::Error> {
        let mapping: [char; 26] = mapping
            .chars()
            .collect::<Vec<char>>()
            .try_into()
            .map_err(SoundexConvertError)?;
        Ok(Self::from(mapping))
    }
}

impl FromStr for RefinedSoundex {
    type Err = SoundexConvertError;

    /// Construct a [RefinedSoundex] from the mapping in parameter. This [str] will
    /// be converted into an array of 26 chars, so `mapping`'s length must be 26.
    ///
    /// # Parameters
    ///
    /// * `mapping`: str that contains the corresponding code for each character.
    ///
    /// # Errors
    ///
    /// Returns an error the number of [char] in the [str] isn't equals to 26.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> anyhow::Result<()> {
    /// use rphonetic::{Encoder, RefinedSoundex};
    ///
    /// // Construct an encoder with 'A' coded into '0', 'B' into '1', 'C' into '3', 'D' into '6', 'E' into '0', ...etc
    /// // (this is the default mapping)
    /// let refined_soundex = "01360240043788015936020505".parse::<RefinedSoundex>()?;
    ///
    /// assert_eq!(refined_soundex.encode("jumped")?, "J408106");
    /// #    Ok(())
    /// # }
    /// ```
    fn from_str(mapping: &str) -> Result<Self, Self::Err> {
        Self::try_from(mapping)
    }
}

impl TryFrom<String> for RefinedSoundex {
    type Error = SoundexConvertError;

    /// Construct a [RefinedSoundex] from the mapping in parameter. This [String] will
    /// be converted into an array of 26 chars, so `mapping`'s length must be 26.
    ///
    /// # Parameters
    ///
    /// * `mapping`: str that contains the corresponding code for each character.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> anyhow::Result<()> {
    /// use rphonetic::{Encoder, RefinedSoundex};
    ///
    /// // Construct an encoder with 'A' coded into '0', 'B' into '1', 'C' into '3', 'D' into '6', 'E' into '0', ...etc
    /// // (this is the default mapping)
    /// let refined_soundex = RefinedSoundex::try_from("01360240043788015936020505".to_string())?;
    ///
    /// assert_eq!(refined_soundex.encode("jumped")?, "J408106");
    /// #    Ok(())
    /// # }
    /// ```
    fn try_from(mapping: String) -> Result<Self, Self::Error> {
        Self::try_from(mapping.as_str())
    }
}

impl Default for RefinedSoundex {
    fn default() -> Self {
        Self {
            mapping: ENGLISH_MAPPING,
        }
    }
}

/// [Encoder] implementation.
///
/// Note that it should be safe to use the `unchecked` method
/// for this algorithm because non ASCII letters are removed. Then
/// all `get(...)` calls on slice must be safe.
impl Encoder for RefinedSoundex {
    type Error = SoundexEncodeError;

    fn encode(&self, value: &str) -> Result<String, Self::Error> {
        let value = Self::soundex_clean(value);

        let mut code = match value.chars().next() {
            Some(ch) => {
                let mut code = String::with_capacity(value.len() + 1);
                code.push(ch);
                code
            }
            None => return Ok(value),
        };

        let mut previous: Option<char> = None;

        for ch in value.chars() {
            let code_value = crate::soundex_commons::get_mapping_code(&self.mapping, ch)?;
            if Some(code_value) != previous {
                code.push(code_value);
            }
            previous = Some(code_value);
        }

        Ok(code)
    }
}

impl SoundexUtils for RefinedSoundex {}

impl SoundexCommons for RefinedSoundex {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difference() {
        let refined_soundex = RefinedSoundex::default();

        assert_eq!(refined_soundex.difference("", ""), Ok(0));
        assert_eq!(refined_soundex.difference(" ", " "), Ok(0));
        assert_eq!(refined_soundex.difference("Smith", "Smythe"), Ok(6));
        assert_eq!(refined_soundex.difference("Ann", "Andrew"), Ok(3));
        assert_eq!(refined_soundex.difference("Margaret", "Andrew"), Ok(1));
        assert_eq!(refined_soundex.difference("Janet", "Margaret"), Ok(1));
        assert_eq!(refined_soundex.difference("Green", "Greene"), Ok(5));
        assert_eq!(
            refined_soundex.difference("Blotchet-Halls", "Greene"),
            Ok(1)
        );
        assert_eq!(refined_soundex.difference("Smith", "Smythe"), Ok(6));
        assert_eq!(refined_soundex.difference("Smithers", "Smythers"), Ok(8));
        assert_eq!(refined_soundex.difference("Anothers", "Brothers"), Ok(5));
    }

    #[test]
    fn test_encode() {
        let refined_soundex = RefinedSoundex::default();

        assert_eq!(
            refined_soundex.encode("testing"),
            Ok("T6036084".to_string())
        );
        assert_eq!(
            refined_soundex.encode("TESTING"),
            Ok("T6036084".to_string())
        );
        assert_eq!(refined_soundex.encode("The"), Ok("T60".to_string()));
        assert_eq!(refined_soundex.encode("quick"), Ok("Q503".to_string()));
        assert_eq!(refined_soundex.encode("brown"), Ok("B1908".to_string()));
        assert_eq!(refined_soundex.encode("fox"), Ok("F205".to_string()));
        assert_eq!(refined_soundex.encode("jumped"), Ok("J408106".to_string()));
        assert_eq!(refined_soundex.encode("over"), Ok("O0209".to_string()));
        assert_eq!(refined_soundex.encode("the"), Ok("T60".to_string()));
        assert_eq!(refined_soundex.encode("lazy"), Ok("L7050".to_string()));
        assert_eq!(refined_soundex.encode("dogs"), Ok("D6043".to_string()));
    }

    #[test]
    fn test_new() {
        assert_eq!(
            RefinedSoundex::new(ENGLISH_MAPPING),
            RefinedSoundex::default()
        );
    }

    #[test]
    fn test_try_from_str() {
        let refined_soundex = RefinedSoundex::try_from("01360240043788015936020505");
        assert_eq!(refined_soundex, Ok(RefinedSoundex::default()));
    }

    #[test]
    fn test_try_from_string() {
        let refined_soundex = RefinedSoundex::try_from("01360240043788015936020505".to_string());
        assert_eq!(refined_soundex, Ok(RefinedSoundex::default()));
    }
}
