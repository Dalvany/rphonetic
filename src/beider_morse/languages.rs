use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::beider_morse::{BMError, NameType};
use crate::constants::{
    DM_LANGUAGE_LINE, MULTI_LINE_COMMENT_END, MULTI_LINE_COMMENT_START, SINGLE_LINE_COMMENT,
};

/// This represent a set of languages.
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
    pub fn any(&self) -> Option<String> {
        match self {
            LanguageSet::Any => None,
            LanguageSet::NoLanguages => None,
            LanguageSet::SomeLanguages(languages) => languages.iter().next().cloned(),
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

#[cfg(feature = "embedded")]
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

impl TryFrom<&PathBuf> for Languages {
    type Error = BMError;

    fn try_from(directory: &PathBuf) -> Result<Self, Self::Error> {
        let mut map: BTreeMap<NameType, BTreeSet<String>> = BTreeMap::new();
        let paths = std::fs::read_dir(directory)?;

        for path in paths {
            let path = path?;
            if let Ok(name_type) = NameType::try_from(path.file_name()) {
                let content = std::fs::read_to_string(path.path())?;
                let languages = parse_liste(content);
                map.insert(name_type, languages);
            }
        }

        Ok(Self { languages: map })
    }
}

fn parse_liste(list: String) -> BTreeSet<String> {
    let mut result = BTreeSet::new();
    let mut multiline_comment = false;
    for mut line in list.split('\n') {
        line = line.trim();

        // Start to test multiline comment ends, thus we can collapse some 'if'.
        if line.ends_with(MULTI_LINE_COMMENT_END) {
            multiline_comment = false;
            continue;
        } else if line.is_empty() || line.starts_with(SINGLE_LINE_COMMENT) || multiline_comment {
            continue;
        } else if line.starts_with(MULTI_LINE_COMMENT_START) {
            multiline_comment = true;
            continue;
        }

        match DM_LANGUAGE_LINE.captures(line) {
            None => {
                result.insert(line.to_string());
            }
            Some(capture) => {
                let tmp = capture.get(1).unwrap().as_str();
                result.insert(tmp.to_string());
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "embedded")]
    fn test_default() {
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

    #[test]
    fn test_from_path() -> Result<(), BMError> {
        let path = PathBuf::from("./test_assets/cc-rules/");
        let result = Languages::try_from(&path)?;
        let languages = BTreeMap::from([
            (
                NameType::Ashkenazi,
                BTreeSet::from([
                    "any".to_string(),
                    "cyrillic".to_string(),
                    "english".to_string(),
                    "french".to_string(),
                    "german".to_string(),
                    "hebrew".to_string(),
                    "hungarian".to_string(),
                    "polish".to_string(),
                    "romanian".to_string(),
                    "russian".to_string(),
                    "spanish".to_string(),
                ]),
            ),
            (
                NameType::Generic,
                BTreeSet::from([
                    "any".to_string(),
                    "arabic".to_string(),
                    "cyrillic".to_string(),
                    "czech".to_string(),
                    "dutch".to_string(),
                    "english".to_string(),
                    "french".to_string(),
                    "german".to_string(),
                    "greek".to_string(),
                    "greeklatin".to_string(),
                    "hebrew".to_string(),
                    "hungarian".to_string(),
                    "italian".to_string(),
                    "polish".to_string(),
                    "portuguese".to_string(),
                    "romanian".to_string(),
                    "russian".to_string(),
                    "spanish".to_string(),
                    "turkish".to_string(),
                ]),
            ),
            (
                NameType::Sephardic,
                BTreeSet::from([
                    "any".to_string(),
                    "french".to_string(),
                    "hebrew".to_string(),
                    "italian".to_string(),
                    "portuguese".to_string(),
                    "spanish".to_string(),
                ]),
            ),
        ]);
        let expected = Languages { languages };

        assert_eq!(result, expected);
        Ok(())
    }
}
