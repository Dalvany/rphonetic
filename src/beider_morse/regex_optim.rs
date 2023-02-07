use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::beider_morse::IsMatch;

#[derive(Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq, Deserialize, Serialize)]
pub(super) enum OptimizedRegex {
    AllStringsMatcher,
    Equals(String),
    IsEmpty,
    StartsWith(String),
    EndsWith(String),
    EqualsChar(String, bool),
    StartsWithChar(String, bool),
    EndsWithChar(String, bool),
}

impl Display for OptimizedRegex {
    /// Reconstruct the regex string.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AllStringsMatcher => write!(f, "\"\""),
            Self::IsEmpty => write!(f, "\"^$\""),
            Self::Equals(pattern) => write!(f, "\"{pattern}\""),
            Self::StartsWith(pattern) => write!(f, "\"^{pattern}\""),
            Self::EndsWith(pattern) => write!(f, "\"{pattern}$\""),
            Self::EqualsChar(pattern, should_match) => {
                let negate = match should_match {
                    true => "",
                    false => "^",
                };
                write!(f, "\"^[{negate}{pattern}]$\"")
            }
            Self::StartsWithChar(pattern, should_match) => {
                let negate = match should_match {
                    true => "",
                    false => "^",
                };
                write!(f, "\"^[{negate}{pattern}]\"")
            }
            Self::EndsWithChar(pattern, should_match) => {
                let negate = match should_match {
                    true => "",
                    false => "^",
                };
                write!(f, "\"[{negate}{pattern}]$\"")
            }
        }
    }
}

/// Apply the pattern
impl IsMatch for OptimizedRegex {
    fn is_match(&self, input: &str) -> bool {
        match self {
            Self::AllStringsMatcher => true,
            Self::IsEmpty => input.is_empty(),
            Self::Equals(pattern) => pattern == input,
            Self::StartsWith(prefix) => {
                if prefix.len() > input.len() {
                    false
                } else {
                    input.starts_with(prefix)
                }
            }
            Self::EndsWith(suffix) => {
                if suffix.len() > input.len() {
                    false
                } else {
                    input.ends_with(suffix)
                }
            }
            Self::EqualsChar(char_list, should_match) => {
                // Slicing won't work well since it slices on byte
                // so the trick is to use chars, I think it should be cheap here
                let mut iterator = input.chars();
                let first = iterator.next();
                let second = iterator.next();
                // commons-codec check that length of string is exactly one
                first.is_some()
                    && second.is_none()
                    && char_list.contains(first.unwrap()) == *should_match
            }
            Self::StartsWithChar(char_list, should_match) => {
                let char = input.chars().next();
                char.is_some() && char_list.contains(char.unwrap()) == *should_match
            }
            Self::EndsWithChar(char_list, should_match) => {
                let char = input.chars().rev().next();
                char.is_some() && char_list.contains(char.unwrap()) == *should_match
            }
        }
    }
}

impl FromStr for OptimizedRegex {
    type Err = ();

    fn from_str(regex: &str) -> Result<Self, Self::Err> {
        let starts_with = regex.starts_with('^');
        let ends_with = regex.ends_with('$');
        let content = match (starts_with, ends_with) {
            (false, false) => regex,
            (true, false) => &regex[1..],
            (false, true) => &regex[..regex.len() - 1],
            (true, true) => &regex[1..regex.len() - 1],
        }
        .to_string();
        let boxes = regex.find('[').is_some();

        if !boxes {
            if starts_with && ends_with {
                if content.is_empty() {
                    return Ok(Self::IsEmpty);
                }

                return Ok(Self::Equals(content));
            }
            if (starts_with || ends_with) && content.is_empty() {
                return Ok(Self::AllStringsMatcher);
            }

            if starts_with {
                return Ok(Self::StartsWith(content));
            }

            if ends_with {
                return Ok(Self::EndsWith(content));
            }
        } else {
            let starts_with_box = content.starts_with('[');
            let ends_with_box = content.ends_with(']');
            if starts_with_box && ends_with_box {
                let mut content = content[1..content.len() - 1].to_string();
                if !content.contains('[') {
                    let negate = content.starts_with('^');
                    if negate {
                        content = content[1..].to_string();
                    }
                    let should_match = !negate;
                    if starts_with && ends_with {
                        return Ok(Self::EqualsChar(content, should_match));
                    }
                    if starts_with {
                        return Ok(Self::StartsWithChar(content, should_match));
                    }
                    if ends_with {
                        return Ok(Self::EndsWithChar(content, should_match));
                    }
                }
            }
        }

        Err(())
    }
}
