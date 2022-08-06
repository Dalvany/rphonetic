use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::path::Path;

use enum_iterator::all;
use regex::Regex;

use crate::beider_morse::{LanguageSet, Languages};
use crate::{build_error, end_of_line, lang, multiline_comment, BMError, NameType, PhoneticError};

#[derive(Clone, Debug)]
struct LangRule {
    line_number: usize,
    accept_on_match: bool,
    languages: BTreeSet<String>,
    pattern: Regex,
}

impl Display for LangRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} : \"{}\" {:?} {} ",
            self.line_number, self.pattern, self.languages, self.accept_on_match
        )
    }
}

impl LangRule {
    pub fn matches(&self, value: &str) -> bool {
        self.pattern.is_match(value)
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "embedded_bm", derive(Default))]
pub struct Lang {
    languages: BTreeSet<String>,
    rules: Vec<LangRule>,
}

impl Lang {
    pub fn guess_languages(&self, input: &str) -> LanguageSet {
        let input = input.to_lowercase();

        let mut langs: BTreeSet<String> = BTreeSet::from_iter(self.languages.iter().cloned());
        for rule in &self.rules {
            if rule.matches(&input) {
                if rule.accept_on_match {
                    langs = langs.intersection(&rule.languages).cloned().collect();
                } else {
                    langs = langs.difference(&rule.languages).cloned().collect();
                }
            }
        }

        let result = LanguageSet::from(langs);
        match &result {
            LanguageSet::NoLanguages => LanguageSet::Any,
            _ => result,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Langs {
    langs: BTreeMap<NameType, Lang>,
}

#[cfg(feature = "embedded_bm")]
impl Default for Langs {
    fn default() -> Self {
        let mut langs: BTreeMap<NameType, Lang> = BTreeMap::new();
        for name_type in all::<NameType>() {
            langs.insert(name_type, Lang::default());
        }

        Self { langs }
    }
}

impl Langs {
    pub fn new(directory: &Path, languages: &Languages) -> Result<Self, PhoneticError> {
        build_langs(directory, languages)
    }

    pub fn get(&self, name_type: &NameType) -> Option<&Lang> {
        self.langs.get(name_type)
    }
}

fn parse_lang(
    filename: Option<String>,
    content: String,
    languages: &BTreeSet<String>,
) -> Result<Lang, PhoneticError> {
    let mut rules: Vec<LangRule> = Vec::new();
    let mut remains = content.as_str();
    let mut line_number: usize = 0;

    while !remains.is_empty() {
        line_number += 1;

        // Since parts are not delimited we try first to parse comment either single line
        // or multiline.

        // Try single line comment
        if let Ok((rm, _)) = end_of_line()(remains) {
            remains = rm;
            continue;
        }

        // Try multiline comment
        if let Ok((rm, ln)) = multiline_comment()(remains) {
            line_number += ln - 1;
            remains = rm;
            continue;
        }

        if let Ok((rm, (pattern, langs, accept_on_match))) = lang()(remains) {
            remains = rm;

            let pattern: Regex = Regex::new(pattern).map_err(|error| {
                build_error(line_number, filename.clone(), remains, error.to_string())
            })?;
            let langs: BTreeSet<String> =
                BTreeSet::from_iter(langs.split('+').map(|v| v.to_string()));
            rules.push(LangRule {
                line_number,
                accept_on_match,
                languages: langs,
                pattern,
            });
            continue;
        }

        // Everything fails, then return an error...
        return Err(build_error(
            line_number,
            None,
            remains,
            "Can't parse line for language detection".to_string(),
        ));
    }

    Ok(Lang {
        languages: languages.clone(),
        rules,
    })
}

fn build_langs(directory: &Path, languages_set: &Languages) -> Result<Langs, PhoneticError> {
    let mut langs: BTreeMap<NameType, Lang> = BTreeMap::new();

    for name_type in all::<NameType>() {
        let languages = languages_set.get(&name_type).unwrap();
        let filename = directory.join(format!("{}_lang.txt", name_type));
        let content = std::fs::read_to_string(filename.clone())
            .map_err(|error| PhoneticError::BMError(BMError::from(error)))?;
        let filename = filename.to_str().map(|v| v.to_string());
        let lang = parse_lang(filename, content, languages)?;
        langs.insert(name_type, lang);
    }

    Ok(Langs { langs })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_langs() -> Result<(), PhoneticError> {
        let path = &PathBuf::from("./test_assets/cc-rules/");
        let langs = Langs::new(path, &Languages::try_from(path)?)?;

        assert!(!langs.langs.is_empty());
        Ok(())
    }

    #[test]
    fn test_language_guessing() -> Result<(), PhoneticError> {
        let path = &PathBuf::from("./test_assets/cc-rules/");
        let langs = Langs::new(path, &Languages::try_from(path)?)?;
        let langs = langs.get(&NameType::Generic).unwrap();

        let data = vec![
            ("Renault", LanguageSet::from(vec!["french"])),
            ("Mickiewicz", LanguageSet::from(vec!["polish"])),
            ("Thompson", LanguageSet::from(vec!["greeklatin", "english"])),
            ("Nu\u{00f1}ez", LanguageSet::from(vec!["spanish"])),
            ("Carvalho", LanguageSet::from(vec!["portuguese"])),
            ("\u{010c}apek", LanguageSet::from(vec!["czech"])),
            ("Sjneijder", LanguageSet::from(vec!["dutch"])),
            ("Klausewitz", LanguageSet::from(vec!["german"])),
            ("K\u{00fc}\u{00e7}\u{00fc}k", LanguageSet::from(vec!["turkish"])),
            ("Giacometti", LanguageSet::from(vec!["italian"])),
            ("Nagy", LanguageSet::from(vec!["hungarian"])),
            ("Ceau\u{015f}escu", LanguageSet::from(vec!["romanian"])),
            ("Angelopoulos", LanguageSet::from(vec!["greeklatin"])),
            ("\u{0391}\u{03b3}\u{03b3}\u{03b5}\u{03bb}\u{03cc}\u{03c0}\u{03bf}\u{03c5}\u{03bb}\u{03bf}\u{03c2}", LanguageSet::from(vec!["greek"])),
            ("\u{041f}\u{0443}\u{0448}\u{043a}\u{0438}\u{043d}", LanguageSet::from(vec!["cyrillic"])),
            ("\u{05db}\u{05d4}\u{05df}", LanguageSet::from(vec!["hebrew"])),
            ("\u{00e1}cz", LanguageSet::Any),
            ("\u{00e1}tz", LanguageSet::Any),
        ];

        for (input, expected) in data {
            let result = langs.guess_languages(input);
            assert_eq!(result, expected, "Error for {}", input);
        }
        Ok(())
    }
}
