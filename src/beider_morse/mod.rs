use std::error::Error;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};

use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

mod lang;
mod languages;
mod rule;

pub use languages::{LanguageSet, Languages};
pub use rule::RuleType;

const ASH: &str = "ash";
const GEN: &str = "gen";
const SEP: &str = "sep";

#[derive(Debug)]
pub enum BMError {
    UnknownNameType(String),
    ParseConfiguration(std::io::Error),
    WrongFilename(String),
    WrongPhoneme(String),
    BadContextRegex(regex::Error),
    NotABoolean(String),
    BadRule(String),
}

impl Display for BMError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BMError::UnknownNameType(error) => write!(f, "Unknown NameType {}", error),
            BMError::ParseConfiguration(error) => write!(f, "Error reading files {}", error),
            BMError::WrongFilename(error) => write!(f, "Wrong file name : {}", error),
            BMError::WrongPhoneme(error) => write!(f, "{}", error),
            BMError::BadContextRegex(error) => write!(f, "{}", error),
            BMError::NotABoolean(error) => write!(f, "{}", error),
            BMError::BadRule(error) => write!(f, "{}", error),
        }
    }
}

impl From<std::io::Error> for BMError {
    fn from(error: std::io::Error) -> Self {
        Self::ParseConfiguration(error)
    }
}

impl From<regex::Error> for BMError {
    fn from(error: regex::Error) -> Self {
        Self::BadContextRegex(error)
    }
}

impl Error for BMError {}

#[derive(
    Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Sequence,
)]
pub enum NameType {
    #[serde(rename = "ash")]
    Ashkenazi,
    #[serde(rename = "gen")]
    Generic,
    #[serde(rename = "sep")]
    Sephardic,
}

impl NameType {
    pub fn language_filename(&self) -> String {
        format!("{}_languages.txt", self)
    }
}

impl Display for NameType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let r = match self {
            Self::Ashkenazi => ASH,
            Self::Generic => GEN,
            Self::Sephardic => SEP,
        };
        write!(f, "{}", r)
    }
}

impl TryFrom<&str> for NameType {
    type Error = BMError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            ASH => Ok(Self::Ashkenazi),
            GEN => Ok(Self::Generic),
            SEP => Ok(Self::Sephardic),
            _ => Err(BMError::UnknownNameType(value.to_string())),
        }
    }
}

impl TryFrom<OsString> for NameType {
    type Error = BMError;

    fn try_from(value: OsString) -> Result<Self, Self::Error> {
        if value == OsString::from(NameType::Ashkenazi.language_filename()) {
            Ok(NameType::Ashkenazi)
        } else if value == OsString::from(NameType::Generic.language_filename()) {
            Ok(NameType::Generic)
        } else if value == OsString::from(NameType::Sephardic.language_filename()) {
            Ok(NameType::Sephardic)
        } else {
            Err(BMError::UnknownNameType(
                value.to_string_lossy().to_string(),
            ))
        }
    }
}
