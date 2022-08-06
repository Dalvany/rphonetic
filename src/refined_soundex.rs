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

use crate::{Encoder, SoundexCommons, SoundexUtils};

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
/// use rphonetic::{Encoder, RefinedSoundex};
/// let refined_soundex = RefinedSoundex::default();
///
/// assert_eq!(refined_soundex.encode("jumped"), "J408106");
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
    /// * `mapping` : mapping array. It contains, for each letter its corresponding code. Index 0 is the code for `A`, index 1
    /// is for `B`and so on for each letter of the latin alphabet.
    pub fn new(mapping: [char; 26]) -> Self {
        Self { mapping }
    }

    fn get_mapping_code(&self, ch: char) -> char {
        self.mapping[ch as usize - 65]
    }
}

impl FromStr for RefinedSoundex {
    type Err = Vec<char>;

    /// Construct a [RefinedSoundex] from the mapping in parameter. This [str] will
    /// be converted into an array of 26 chars, so `mapping`'s length must be 26.
    ///
    /// # Parameters
    ///
    /// * `mapping` : str that contains the corresponding code for each character.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Vec<char>> {
    /// use rphonetic::{Encoder, RefinedSoundex};
    ///
    /// // Construct an encoder with 'A' coded into '0', 'B' into '1', 'C' into '3', 'D' into '6', 'E' into '0', ...etc
    /// // (this is the default mapping)
    /// let refined_soundex = "01360240043788015936020505".parse::<RefinedSoundex>()?;
    ///
    /// assert_eq!(refined_soundex.encode("jumped"), "J408106");
    /// #    Ok(())
    /// # }
    /// ```
    fn from_str(mapping: &str) -> Result<Self, Self::Err> {
        let mapping: [char; 26] = mapping.chars().collect::<Vec<char>>().try_into()?;
        Ok(Self { mapping })
    }
}

impl TryFrom<&str> for RefinedSoundex {
    type Error = Vec<char>;

    /// Construct a [RefinedSoundex] from the mapping in parameter. This [str] will
    /// be converted into an array of 26 chars, so `mapping`'s length must be 26.
    ///
    /// # Parameters
    ///
    /// * `mapping` : str that contains the corresponding code for each character.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Vec<char>> {
    /// use rphonetic::{Encoder, RefinedSoundex};
    ///
    /// // Construct an encoder with 'A' coded into '0', 'B' into '1', 'C' into '3', 'D' into '6', 'E' into '0', ...etc
    /// // (this is the default mapping)
    /// let refined_soundex = RefinedSoundex::try_from("01360240043788015936020505")?;
    ///
    /// assert_eq!(refined_soundex.encode("jumped"), "J408106");
    /// #    Ok(())
    /// # }
    /// ```
    fn try_from(mapping: &str) -> Result<Self, Self::Error> {
        let mapping: [char; 26] = mapping.chars().collect::<Vec<char>>().try_into()?;
        Ok(Self { mapping })
    }
}

impl TryFrom<String> for RefinedSoundex {
    type Error = Vec<char>;

    /// Construct a [RefinedSoundex] from the mapping in parameter. This [String] will
    /// be converted into an array of 26 chars, so `mapping`'s length must be 26.
    ///
    /// # Parameters
    ///
    /// * `mapping` : str that contains the corresponding code for each character.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), Vec<char>> {
    /// use rphonetic::{Encoder, RefinedSoundex};
    ///
    /// // Construct an encoder with 'A' coded into '0', 'B' into '1', 'C' into '3', 'D' into '6', 'E' into '0', ...etc
    /// // (this is the default mapping)
    /// let refined_soundex = RefinedSoundex::try_from("01360240043788015936020505".to_string())?;
    ///
    /// assert_eq!(refined_soundex.encode("jumped"), "J408106");
    /// #    Ok(())
    /// # }
    /// ```
    fn try_from(mapping: String) -> Result<Self, Self::Error> {
        mapping.as_str().parse::<RefinedSoundex>()
    }
}

impl Default for RefinedSoundex {
    fn default() -> Self {
        Self {
            mapping: ENGLISH_MAPPING,
        }
    }
}

impl Encoder for RefinedSoundex {
    fn encode(&self, value: &str) -> String {
        let value = Self::soundex_clean(value);
        if value.is_empty() {
            return value;
        }

        let mut code = String::with_capacity(value.len() + 1);
        code.push(value.chars().next().unwrap());

        let mut previous: Option<char> = None;

        for ch in value.chars() {
            let code_value = self.get_mapping_code(ch);
            if Some(code_value) != previous {
                code.push(code_value);
            }
            previous = Some(code_value);
        }

        code
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

        assert_eq!(refined_soundex.difference("", ""), 0);
        assert_eq!(refined_soundex.difference(" ", " "), 0);
        assert_eq!(refined_soundex.difference("Smith", "Smythe"), 6);
        assert_eq!(refined_soundex.difference("Ann", "Andrew"), 3);
        assert_eq!(refined_soundex.difference("Margaret", "Andrew"), 1);
        assert_eq!(refined_soundex.difference("Janet", "Margaret"), 1);
        assert_eq!(refined_soundex.difference("Green", "Greene"), 5);
        assert_eq!(refined_soundex.difference("Blotchet-Halls", "Greene"), 1);
        assert_eq!(refined_soundex.difference("Smith", "Smythe"), 6);
        assert_eq!(refined_soundex.difference("Smithers", "Smythers"), 8);
        assert_eq!(refined_soundex.difference("Anothers", "Brothers"), 5);
    }

    #[test]
    fn test_encode() {
        let refined_soundex = RefinedSoundex::default();

        assert_eq!(refined_soundex.encode("testing"), "T6036084");
        assert_eq!(refined_soundex.encode("TESTING"), "T6036084");
        assert_eq!(refined_soundex.encode("The"), "T60");
        assert_eq!(refined_soundex.encode("quick"), "Q503");
        assert_eq!(refined_soundex.encode("brown"), "B1908");
        assert_eq!(refined_soundex.encode("fox"), "F205");
        assert_eq!(refined_soundex.encode("jumped"), "J408106");
        assert_eq!(refined_soundex.encode("over"), "O0209");
        assert_eq!(refined_soundex.encode("the"), "T60");
        assert_eq!(refined_soundex.encode("lazy"), "L7050");
        assert_eq!(refined_soundex.encode("dogs"), "D6043");
    }

    #[test]
    fn test_new() {
        assert_eq!(
            RefinedSoundex::new(ENGLISH_MAPPING),
            RefinedSoundex::default()
        );
    }

    #[test]
    fn test_try_from_str() -> Result<(), Vec<char>> {
        let refined_soundex = RefinedSoundex::try_from("01360240043788015936020505")?;
        assert_eq!(refined_soundex, RefinedSoundex::default());

        Ok(())
    }

    #[test]
    fn test_try_from_string() -> Result<(), Vec<char>> {
        let refined_soundex = RefinedSoundex::try_from("01360240043788015936020505".to_string())?;
        assert_eq!(refined_soundex, RefinedSoundex::default());

        Ok(())
    }
}
