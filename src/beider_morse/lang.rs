use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::str::FromStr;

use enum_iterator::all;
use regex::Regex;

use crate::beider_morse::{LanguageSet, Languages};
use crate::constants::{
    MULTI_LINE_COMMENT_END, MULTI_LINE_COMMENT_START, RULE_LANG_LINE, SINGLE_LINE_COMMENT,
};
use crate::{BMError, NameType};

#[derive(Clone, Debug)]
struct LangRule {
    accept_on_match: bool,
    languages: BTreeSet<String>,
    pattern: Regex,
}

impl LangRule {
    pub fn matches(&self, value: &str) -> bool {
        self.pattern.is_match(value)
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "embedded", derive(Default))]
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

#[cfg(feature = "embedded")]
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
    pub fn new(directory: &Path, languages: &Languages) -> Result<Self, BMError> {
        build_langs(directory, languages)
    }

    pub fn get(&self, name_type: &NameType) -> Option<&Lang> {
        self.langs.get(name_type)
    }
}

fn parse_lang(content: String, languages: &BTreeSet<String>) -> Result<Lang, BMError> {
    let mut rules: Vec<LangRule> = Vec::new();
    let mut multiline_comment = false;
    for mut line in content.split('\n') {
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

        match RULE_LANG_LINE.captures(line) {
            None => {
                return Err(BMError::BadRule(format!("Wrong line {}", line)));
            }
            Some(matcher) => {
                let pattern = matcher.get(1).unwrap().as_str();
                let pattern: Regex = Regex::new(pattern)?;
                let langs = matcher.get(2).unwrap().as_str();
                let langs: BTreeSet<String> =
                    BTreeSet::from_iter(langs.split('+').map(|v| v.to_string()));
                let accept_on_match = matcher.get(3).unwrap().as_str();
                let accept_on_match: bool = bool::from_str(accept_on_match).map_err(|_error| {
                    BMError::NotABoolean(format!(
                        "{} is not a boolean. Should be 'true' or 'false'",
                        accept_on_match
                    ))
                })?;
                rules.push(LangRule {
                    accept_on_match,
                    languages: langs,
                    pattern,
                });
            }
        }
    }

    Ok(Lang {
        languages: languages.clone(),
        rules,
    })
}

fn build_langs(directory: &Path, languages_set: &Languages) -> Result<Langs, BMError> {
    let mut langs: BTreeMap<NameType, Lang> = BTreeMap::new();

    for name_type in all::<NameType>() {
        let languages = languages_set.get(&name_type).unwrap();
        let filename = directory.join(format!("{}_lang.txt", name_type));
        let content = std::fs::read_to_string(filename)?;
        let lang = parse_lang(content, languages)?;
        langs.insert(name_type, lang);
    }

    Ok(Langs { langs })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_langs() -> Result<(), BMError> {
        let path = &PathBuf::from("./test_assets/cc-rules/");
        let langs = Langs::new(path, &Languages::try_from(path)?)?;

        assert!(!langs.langs.is_empty());
        Ok(())
    }

    #[test]
    fn test_language_guessing() -> Result<(), BMError> {
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
            assert_eq!(result, expected,);
        }
        Ok(())
    }
}
