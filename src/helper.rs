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
use std::ops::{Index, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

use serde::{Deserialize, Serialize};

/// Replace regex like "s+" by a single char "S".
pub fn replace_compact_all_to_uppercase(string: String, chars: Vec<char>) -> String {
    let mut ret = String::with_capacity(string.len());
    let mut previous: Option<char> = None;

    string.chars().for_each(|ch| {
        if chars.contains(&ch) {
            if let Some(prev) = previous {
                if prev != ch {
                    ret.push(ch.to_ascii_uppercase());
                    previous = Some(ch);
                }
            } else {
                ret.push(ch.to_ascii_uppercase());
                previous = Some(ch);
            }
        } else {
            ret.push(ch);
            previous = None;
        }
    });

    ret
}

/// Test if `string` ends with `pattern` and replace it by `to`.
pub fn replace_end<'a>(mut string: String, pattern: &'a str, to: &'a str) -> String {
    if string.ends_with(pattern) {
        string.replace_range(string.len() - pattern.len().., to);
    }
    string
}

/// Test if a char is a vowel.
pub fn is_vowel(c: Option<char>, include_y: bool) -> bool {
    match c {
        Some(ch) => matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u') || (include_y && ch == 'y'),
        None => false,
    }
}

pub fn replace_char<F>(string: String, f: F) -> String
where
    F: FnMut((usize, char)) -> char,
{
    string
        .chars()
        .into_iter()
        .enumerate()
        .map(f)
        .collect::<String>()
}

pub fn remove_all_nonletter(string: String) -> String {
    string
        .chars()
        .into_iter()
        .filter(|&c| c.is_lowercase())
        .collect::<String>()
}

/// This struct is a wrapper around an `&str` allowing
/// to slice by char.
///
/// It implements [Index], allowing to slice according to
/// [char]. Please note that it is not really efficient as
/// it uses [CharIndices](std::str::CharIndices).
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct CharSequence<'a> {
    inner: &'a str,
    len_in_char: usize,
}

impl<'a> Display for CharSequence<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<'a> CharSequence<'a> {
    /// Return the length of the string in terme of
    /// [char] instead of byte.
    pub fn len(&self) -> usize {
        self.len_in_char
    }

    /// Return `true` if the string contains no `char`.
    pub fn is_empty(&self) -> bool {
        self.len_in_char == 0
    }

    /// Return the inner string.
    pub fn as_str(&self) -> &str {
        self.inner
    }
}

impl<'a> From<&'a str> for CharSequence<'a> {
    fn from(original: &'a str) -> Self {
        let len_in_char = original.chars().count();
        Self {
            inner: original,
            len_in_char,
        }
    }
}

impl<'a> From<CharSequence<'a>> for &'a str {
    fn from(value: CharSequence<'a>) -> Self {
        value.inner
    }
}

impl<'a> Index<Range<usize>> for CharSequence<'a> {
    type Output = str;

    // To make this faster at the cost of an increase of memory usage
    // we could store an array in an array of size chars().count()
    // the index of each char().
    fn index(&self, index: Range<usize>) -> &'a Self::Output {
        let mut iterator = self.inner.char_indices().skip(index.start);

        let start: Option<(usize, _)> = iterator.next();
        let skip = if index.end > index.start {
            index.end - (index.start + 1)
        } else {
            return "";
        };
        let mut iterator = iterator.skip(skip);
        let end: Option<(usize, _)> = iterator.next();

        let start = match start {
            None => return "",
            Some((s, _)) => s,
        };

        match end {
            None => &self.inner[start..],
            Some((s, _)) => &self.inner[start..s],
        }
    }
}

impl<'a> Index<RangeFrom<usize>> for CharSequence<'a> {
    type Output = str;

    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        &self[index.start..self.len_in_char]
    }
}

impl<'a> Index<RangeFull> for CharSequence<'a> {
    type Output = str;

    fn index(&self, _: RangeFull) -> &Self::Output {
        &self[0..self.len_in_char]
    }
}

impl<'a> Index<RangeInclusive<usize>> for CharSequence<'a> {
    type Output = str;

    fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
        &self[*index.start()..*index.end() + 1]
    }
}

impl<'a> Index<RangeTo<usize>> for CharSequence<'a> {
    type Output = str;

    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        &self[0..index.end]
    }
}

impl<'a> Index<RangeToInclusive<usize>> for CharSequence<'a> {
    type Output = str;

    fn index(&self, index: RangeToInclusive<usize>) -> &Self::Output {
        &self[0..=index.end]
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_vowel() {
        assert!(is_vowel(Some('a'), false));
        assert!(is_vowel(Some('e'), false));
        assert!(is_vowel(Some('i'), false));
        assert!(is_vowel(Some('o'), false));
        assert!(is_vowel(Some('u'), false));
        assert!(!is_vowel(Some('b'), false));
        assert!(!is_vowel(Some('d'), false));
        assert!(!is_vowel(Some('p'), false));
        assert!(!is_vowel(Some('q'), false));
        assert!(!is_vowel(Some('z'), false));
        assert!(!is_vowel(Some('A'), false));
        assert!(!is_vowel(Some('I'), false));
        assert!(!is_vowel(Some('3'), false));

        assert!(!is_vowel(Some('y'), false));
        assert!(is_vowel(Some('y'), true));

        assert!(!is_vowel(None, false));
    }

    #[test]
    fn test_replace_compact_all_to_uppercase_nothing_to_compact() {
        let result =
            replace_compact_all_to_uppercase("aaaabbbbccccdddd".to_string(), vec!['e', 'f', 'g']);
        assert_eq!(result, "aaaabbbbccccdddd");
    }

    #[test]
    fn test_replace_compact_all_to_uppercase_compact_all() {
        let result = replace_compact_all_to_uppercase(
            "aaaabbbbccccdddd".to_string(),
            vec!['a', 'b', 'c', 'd'],
        );
        assert_eq!(result, "ABCD");
    }

    #[test]
    fn test_replace_compact_all_to_uppercase() {
        let result =
            replace_compact_all_to_uppercase("aaaabbbbccccdddd".to_string(), vec!['b', 'd']);
        assert_eq!(result, "aaaaBccccD");
    }

    #[test]
    fn test_char_sequence_all_char_range() {
        for ch in '\u{0000}'..'\u{ffff}' {
            let data = ch.to_string();
            let data = data.as_str();
            let char_sequence = CharSequence::from(data);
            assert_eq!(char_sequence.len_in_char, 1);
            assert_eq!(&char_sequence[0..1], data);
        }
    }

    #[test]
    fn test_char_sequence_all_char_range_from() {
        for ch in '\u{0000}'..'\u{ffff}' {
            let data = ch.to_string();
            let data = data.as_str();
            let char_sequence = CharSequence::from(data);
            assert_eq!(char_sequence.len_in_char, 1);
            assert_eq!(&char_sequence[0..], data);
        }
    }

    #[test]
    fn test_char_sequence_all_char_range_full() {
        for ch in '\u{0000}'..'\u{ffff}' {
            let data = ch.to_string();
            let data = data.as_str();
            let char_sequence = CharSequence::from(data);
            assert_eq!(char_sequence.len_in_char, 1);
            assert_eq!(&char_sequence[..], data);
        }
    }

    #[test]
    fn test_char_sequence_all_char_range_inclusive() {
        for ch in '\u{0000}'..'\u{ffff}' {
            let data = ch.to_string();
            let data = data.as_str();
            let char_sequence = CharSequence::from(data);
            assert_eq!(char_sequence.len_in_char, 1);
            assert_eq!(&char_sequence[0..=0], data);
        }
    }

    #[test]
    fn test_char_sequence_all_char_range_to() {
        for ch in '\u{0000}'..'\u{ffff}' {
            let data = ch.to_string();
            let data = data.as_str();
            let char_sequence = CharSequence::from(data);
            assert_eq!(char_sequence.len_in_char, 1);
            assert_eq!(&char_sequence[..1], data);
        }
    }

    #[test]
    fn test_char_sequence_all_char_range_to_inclusive() {
        for ch in '\u{0000}'..'\u{ffff}' {
            let data = ch.to_string();
            let data = data.as_str();
            let char_sequence = CharSequence::from(data);
            assert_eq!(char_sequence.len_in_char, 1);
            assert_eq!(&char_sequence[..=0], data);
        }
    }

    #[test]
    fn test_char_sequence_index_with_ascii() {
        let data = "This is the string to test.";
        let char_sequence = CharSequence::from(data);

        assert_eq!(char_sequence[5..7], data[5..7]);
        assert_eq!(char_sequence[5..], data[5..]);
        assert_eq!(char_sequence[..], data[..]);
        assert_eq!(char_sequence[5..=6], data[5..=6]);
        assert_eq!(char_sequence[..6], data[..6]);
        assert_eq!(char_sequence[..=6], data[..=6]);
    }

    #[test]
    fn test_char_sequence_chinese() {
        let data = "每个人都有他的作战策略，直到脸上中了一拳。";
        assert_ne!(data.len(), 21);
        let char_sequence = CharSequence::from(data);
        assert_eq!(char_sequence.len(), 21);

        assert_eq!(&char_sequence[6..9], "的作战");
        assert_eq!(&char_sequence[6..], "的作战策略，直到脸上中了一拳。");
        assert_eq!(
            &char_sequence[..],
            "每个人都有他的作战策略，直到脸上中了一拳。"
        );
        assert_eq!(&char_sequence[6..=9], "的作战策");
        assert_eq!(&char_sequence[..9], "每个人都有他的作战");
        assert_eq!(&char_sequence[..=9], "每个人都有他的作战策");
    }

    #[test]
    fn test_char_sequence_to_0() {
        let data = "azerty";
        let char_sequence = CharSequence::from(data);

        assert_eq!(&char_sequence[..0], "");
    }
}
