use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::beider_morse::NameType;

mod parser;

/// This represents a set of languages.
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum LanguageSet {
    /// This represents `any` language.
    Any,
    /// No languages.
    NoLanguages,
    /// Languages provided.
    SomeLanguages(BTreeSet<String>),
}

impl LanguageSet {
    /// Return `true` if this [LanguageSet] contains no language.
    pub fn is_empty(&self) -> bool {
        match self {
            LanguageSet::Any => false,
            LanguageSet::NoLanguages => true,
            LanguageSet::SomeLanguages(languages) => languages.is_empty(),
        }
    }

    /// Return `true` if this [LanguageSet] contains only one language.
    pub fn is_singleton(&self) -> bool {
        match self {
            LanguageSet::Any => false,
            LanguageSet::NoLanguages => false,
            LanguageSet::SomeLanguages(languages) => languages.len() == 1,
        }
    }

    /// Return a new [LanguageSet] that is the intersection between `self` and `other`.
    pub fn restrict_to(&self, other: &Self) -> Self {
        match (self, other) {
            (_, LanguageSet::Any) => self.clone(),
            (_, LanguageSet::NoLanguages) => other.clone(),
            (LanguageSet::SomeLanguages(languages1), LanguageSet::SomeLanguages(languages2)) => {
                let languages = languages1
                    .intersection(languages2)
                    .cloned()
                    .collect::<BTreeSet<String>>();
                Self::SomeLanguages(languages)
            }
            (LanguageSet::Any, _) => other.clone(),
            (LanguageSet::NoLanguages, _) => self.clone(),
        }
    }

    /// Return a new [LanguageSet] that is the union of `self` and `other`.
    pub fn merge(&self, other: &Self) -> Self {
        match (self, other) {
            (_, LanguageSet::Any) => other.clone(),
            (_, LanguageSet::NoLanguages) => self.clone(),
            (LanguageSet::SomeLanguages(languages1), LanguageSet::SomeLanguages(languages2)) => {
                let languages = languages1
                    .union(languages2)
                    .cloned()
                    .collect::<BTreeSet<String>>();
                Self::SomeLanguages(languages)
            }
            (LanguageSet::Any, _) => self.clone(),
            (LanguageSet::NoLanguages, _) => other.clone(),
        }
    }

    /// Return the first language of `self` or [None](Option::None) if
    /// `self` is empty.
    pub fn any(&self) -> Option<&str> {
        match self {
            LanguageSet::Any => None,
            LanguageSet::NoLanguages => None,
            LanguageSet::SomeLanguages(languages) => languages.iter().next().map(|v| v.as_str()),
        }
    }
}

impl From<BTreeSet<String>> for LanguageSet {
    fn from(languages: BTreeSet<String>) -> Self {
        if languages.is_empty() {
            Self::NoLanguages
        } else {
            Self::SomeLanguages(languages)
        }
    }
}

impl From<Vec<&str>> for LanguageSet {
    fn from(languages: Vec<&str>) -> Self {
        Self::SomeLanguages(BTreeSet::from_iter(languages.iter().map(|v| v.to_string())))
    }
}

impl Display for LanguageSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LanguageSet::Any => write!(f, "ANY_LANGUAGE"),
            LanguageSet::NoLanguages => write!(f, "NO_LANGUAGES"),
            LanguageSet::SomeLanguages(languages) => {
                write!(
                    f,
                    "{}",
                    languages.iter().cloned().collect::<Vec<String>>().join(",")
                )
            }
        }
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Languages {
    languages: BTreeMap<NameType, BTreeSet<String>>,
}

impl Languages {
    pub fn get(&self, name_type: &NameType) -> Option<&BTreeSet<String>> {
        self.languages.get(name_type)
    }
}

#[cfg(feature = "embedded_bm")]
impl Default for Languages {
    fn default() -> Self {
        // As we only provide "any" language there's no need to parse a file or anything
        // Just hardcode stuff.
        let languages = BTreeMap::from([
            (NameType::Ashkenazi, BTreeSet::from(["any".to_string()])),
            (NameType::Generic, BTreeSet::from(["any".to_string()])),
            (NameType::Sephardic, BTreeSet::from(["any".to_string()])),
        ]);

        Languages { languages }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "embedded_bm")]
    fn test_default() {
        use super::*;

        let result = Languages::default();

        assert_eq!(
            result.get(&NameType::Ashkenazi),
            Some(&BTreeSet::from(["any".to_string()]))
        );
        assert_eq!(
            result.get(&NameType::Generic),
            Some(&BTreeSet::from(["any".to_string()]))
        );
        assert_eq!(
            result.get(&NameType::Sephardic),
            Some(&BTreeSet::from(["any".to_string()]))
        );
    }
}
