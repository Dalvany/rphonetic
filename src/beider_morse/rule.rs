use std::fmt::{Display, Formatter};

use regex::Regex;
use serde::{Deserialize, Serialize};

use super::LanguageSet;

const APPROX: &str = "approx";
const EXACT: &str = "exact";
const RULES: &str = "rules";

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum RuleType {
    Approx,
    Exact,
}

/// This is a copy of [RuleType] but with a variant for `rules` as this variant
/// is for internal use.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum PrivateRuleType {
    Approx,
    Exact,
    Rules,
}

impl From<RuleType> for PrivateRuleType {
    fn from(rule_type: RuleType) -> Self {
        match rule_type {
            RuleType::Approx => Self::Approx,
            RuleType::Exact => Self::Exact,
        }
    }
}

impl Display for PrivateRuleType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let r = match self {
            Self::Approx => APPROX,
            Self::Exact => EXACT,
            Self::Rules => RULES,
        };
        write!(f, "{}", r)
    }
}

trait PhonemeExpr {
    fn get_phonemes(&self) -> Vec<&Phoneme>;
}

#[derive(Clone, Debug, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
struct Phoneme {
    phoneme_text: String,
    languages: LanguageSet,
}

impl PhonemeExpr for Phoneme {
    fn get_phonemes(&self) -> Vec<&Phoneme> {
        vec![self]
    }
}

impl Phoneme {
    pub fn append(mut self, value: &str) -> Self {
        self.phoneme_text.push_str(value);
        self
    }

    pub fn get_phoneme_text(&self) -> String {
        self.phoneme_text.clone()
    }

    pub fn merge_with_language(&self, languages: &LanguageSet) -> Self {
        Self {
            phoneme_text: self.phoneme_text.clone(),
            languages: self.languages.merge(languages),
        }
    }
}

struct PhonemeList {
    phonemes: Vec<Phoneme>,
}

impl PhonemeExpr for PhonemeList {
    fn get_phonemes(&self) -> Vec<&Phoneme> {
        self.phonemes.iter().collect()
    }
}

pub struct Rule {
    left_context: Regex,
    pattern: String,
    right_context: Regex,
}
