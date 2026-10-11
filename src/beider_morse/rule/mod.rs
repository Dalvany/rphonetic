use std::cmp::Ordering;
use std::collections::btree_map::BTreeMap;
use std::fmt::{Debug, Display, Formatter};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use either::Either;
use enum_iterator::Sequence;
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::{IsMatch, LanguageSet};
use crate::beider_morse::regex_optim::OptimizedRegex;
use crate::beider_morse::rule::parser::build_rules;
use crate::beider_morse::{Languages, ParseBmError};
use crate::helper::CharSequence;
use crate::{BMError, NameType};

mod parser;

const APPROX: &str = "approx";
const EXACT: &str = "exact";
const RULES: &str = "rules";

/// Type of rules.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum RuleType {
    /// Approximate rules. It will lead to the largest number phonetic interpretation.
    Approx,
    /// Exact rules. It will lead to the minimum number phonetic interpretation.
    Exact,
}

/// This is a copy of [RuleType] but with a variant for `rules` as this variant
/// is for internal use.
#[derive(
    Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Sequence,
)]
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
        write!(f, "{r}")
    }
}

impl FromStr for PrivateRuleType {
    type Err = BMError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            APPROX => Ok(Self::Approx),
            EXACT => Ok(Self::Exact),
            RULES => Ok(Self::Rules),
            other => Err(BMError::UnknownRuleType(other.to_string())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Phoneme {
    phoneme_text: String,
    languages: LanguageSet,
}

impl PartialOrd<Self> for Phoneme {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Phoneme {
    fn cmp(&self, other: &Self) -> Ordering {
        let iterator = self.phoneme_text.chars().zip(other.phoneme_text.chars());
        for (ch1, ch2) in iterator {
            if ch1 != ch2 {
                return ch1.cmp(&ch2);
            }
        }

        let o1length = self.phoneme_text.len();
        let o2length = other.phoneme_text.len();

        o1length.cmp(&o2length)
    }
}

impl Display for Phoneme {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{}]", self.phoneme_text, self.languages)
    }
}

impl Phoneme {
    pub fn new(phoneme_text: &str, languages: LanguageSet) -> Self {
        Self {
            phoneme_text: phoneme_text.to_string(),
            languages,
        }
    }

    pub fn join(phoneme1: &Phoneme, phoneme2: &Phoneme, languages: LanguageSet) -> Self {
        let mut phoneme_text =
            String::with_capacity(phoneme1.phoneme_text.len() + phoneme2.phoneme_text.len());
        phoneme_text.push_str(&phoneme1.phoneme_text);
        phoneme_text.push_str(&phoneme2.phoneme_text);
        Self {
            phoneme_text,
            languages,
        }
    }

    pub fn append(mut self, value: &str) -> Self {
        self.phoneme_text.push_str(value);
        self
    }

    pub fn phoneme_text(&self) -> String {
        self.phoneme_text.clone()
    }

    pub fn merge_with_language(&self, languages: &LanguageSet) -> Self {
        Self {
            phoneme_text: self.phoneme_text.clone(),
            languages: self.languages.merge(languages),
        }
    }

    pub fn languages(&self) -> &LanguageSet {
        &self.languages
    }
}

#[derive(Clone, Debug, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PhonemeList {
    phonemes: Vec<Phoneme>,
}

impl PhonemeList {
    pub(crate) fn phonemes(&self) -> Vec<&Phoneme> {
        self.phonemes.iter().collect()
    }
}

struct Resolver {
    path: Option<PathBuf>,
}

impl Resolver {
    fn resolve(&self, filename: &str) -> Result<String, ParseBmError> {
        match &self.path {
            Some(folder) => {
                let f = folder.join(format!("{filename}.txt"));
                Ok(fs_err::read_to_string(f)?)
            }
            #[cfg(feature = "embedded_bm")]
            None => embedded::EMBEDDED_RULES
                .get(filename)
                .map(|v| v.to_string())
                .ok_or_else(|| ParseBmError::MissingEmbedded(filename.to_owned())),
            #[cfg(not(feature = "embedded_bm"))]
            None => Err(ParseBmError::FeatureDisabled),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Rule {
    location: String,
    line: usize,
    left_context: Either<Regex, OptimizedRegex>,
    pattern: String,
    pattern_length_char: usize,
    right_context: Either<Regex, OptimizedRegex>,
    phoneme: PhonemeList,
}

impl Rule {
    pub(crate) fn pattern_and_context_matches(
        &self,
        input: &CharSequence<'_>,
        index: usize,
    ) -> bool {
        let ipl = index + self.pattern_length_char;
        if ipl > input.len()
            || input[index..ipl] != self.pattern
            || !self.right_context.is_match(&input[ipl..])
        {
            false
        } else {
            self.left_context.is_match(&input[..index])
        }
    }

    pub(crate) fn pattern_len_char(&self) -> usize {
        self.pattern_length_char
    }

    pub(crate) fn phoneme(&self) -> &PhonemeList {
        &self.phoneme
    }
}

impl Display for Rule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "left context = {}, pattern = {}, right context = {} ({}:{}) - phonemes : {:?}",
            self.left_context,
            self.pattern,
            self.right_context,
            self.location,
            self.line,
            self.phoneme
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Rules {
    rules: BTreeMap<(NameType, PrivateRuleType, String), BTreeMap<char, Vec<Rule>>>,
}

impl Rules {
    pub fn rules(
        &self,
        name_type: NameType,
        rule_type: PrivateRuleType,
        language: &str,
    ) -> Option<&BTreeMap<char, Vec<Rule>>> {
        self.rules
            .get(&(name_type, rule_type, language.to_string()))
    }

    pub fn new(rules_folder: &Path, languages: &Languages) -> Result<Self, ParseBmError> {
        let resolver = Resolver {
            path: Some(rules_folder.to_path_buf()),
        };
        build_rules(resolver, languages)
    }
}

/// Module that contains default rules (any and commons) and [Default] implementation
/// for [Rules] for convenience with features.
#[cfg(feature = "embedded_bm")]
mod embedded {
    use std::collections::BTreeMap;
    use std::sync::LazyLock;

    use super::*;

    const ASH_EXACT_APPROX_COMMON: &str =
        include_str!("../../../rules/bm/ash_exact_approx_common.txt");
    const ASH_APPROX_ANY: &str = include_str!("../../../rules/bm/ash_approx_any.txt");
    const ASH_APPROX_COMMON: &str = include_str!("../../../rules/bm/ash_approx_common.txt");
    const ASH_EXACT_ANY: &str = include_str!("../../../rules/bm/ash_exact_any.txt");
    const ASH_EXACT_COMMON: &str = include_str!("../../../rules/bm/ash_exact_common.txt");
    const ASH_RULES_ANY: &str = include_str!("../../../rules/bm/ash_rules_any.txt");

    const GEN_EXACT_APPROX_COMMON: &str =
        include_str!("../../../rules/bm/gen_exact_approx_common.txt");
    const GEN_APPROX_ANY: &str = include_str!("../../../rules/bm/gen_approx_any.txt");
    const GEN_APPROX_COMMON: &str = include_str!("../../../rules/bm/gen_approx_common.txt");
    const GEN_EXACT_ANY: &str = include_str!("../../../rules/bm/gen_exact_any.txt");
    const GEN_EXACT_COMMON: &str = include_str!("../../../rules/bm/gen_exact_common.txt");
    const GEN_RULES_ANY: &str = include_str!("../../../rules/bm/gen_rules_any.txt");

    const SEP_EXACT_APPROX_COMMON: &str =
        include_str!("../../../rules/bm/sep_exact_approx_common.txt");
    const SEP_APPROX_ANY: &str = include_str!("../../../rules/bm/sep_approx_any.txt");
    const SEP_APPROX_COMMON: &str = include_str!("../../../rules/bm/sep_approx_common.txt");
    const SEP_EXACT_ANY: &str = include_str!("../../../rules/bm/sep_exact_any.txt");
    const SEP_EXACT_COMMON: &str = include_str!("../../../rules/bm/sep_exact_common.txt");
    const SEP_RULES_ANY: &str = include_str!("../../../rules/bm/sep_rules_any.txt");

    pub static EMBEDDED_RULES: LazyLock<BTreeMap<&'static str, &'static str>> =
        LazyLock::new(|| {
            BTreeMap::from([
                ("ash_exact_approx_common", ASH_EXACT_APPROX_COMMON),
                ("ash_approx_any", ASH_APPROX_ANY),
                ("ash_approx_common", ASH_APPROX_COMMON),
                ("ash_exact_any", ASH_EXACT_ANY),
                ("ash_exact_common", ASH_EXACT_COMMON),
                ("ash_rules_any", ASH_RULES_ANY),
                ("gen_exact_approx_common", GEN_EXACT_APPROX_COMMON),
                ("gen_approx_any", GEN_APPROX_ANY),
                ("gen_approx_common", GEN_APPROX_COMMON),
                ("gen_exact_any", GEN_EXACT_ANY),
                ("gen_exact_common", GEN_EXACT_COMMON),
                ("gen_rules_any", GEN_RULES_ANY),
                ("sep_exact_approx_common", SEP_EXACT_APPROX_COMMON),
                ("sep_approx_any", SEP_APPROX_ANY),
                ("sep_approx_common", SEP_APPROX_COMMON),
                ("sep_exact_any", SEP_EXACT_ANY),
                ("sep_exact_common", SEP_EXACT_COMMON),
                ("sep_rules_any", SEP_RULES_ANY),
            ])
        });

    impl Default for Rules {
        fn default() -> Self {
            let resolver = Resolver { path: None };
            build_rules(resolver, &Languages::default()).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "embedded_bm")]
    fn test_default() {
        let rules = Rules::default();

        let r = rules.rules(NameType::Ashkenazi, PrivateRuleType::Exact, "any");
        assert!(r.is_some());
        assert!(!r.unwrap().is_empty());

        let r = rules.rules(NameType::Ashkenazi, PrivateRuleType::Exact, "common");
        assert!(r.is_some());
        assert!(!r.unwrap().is_empty());

        let r = rules.rules(NameType::Ashkenazi, PrivateRuleType::Approx, "any");
        assert!(r.is_some());
        assert!(!r.unwrap().is_empty());

        let r = rules.rules(NameType::Ashkenazi, PrivateRuleType::Approx, "common");
        assert!(r.is_some());
        assert!(!r.unwrap().is_empty());

        let r = rules.rules(NameType::Ashkenazi, PrivateRuleType::Rules, "any");
        assert!(r.is_some());
        assert!(!r.unwrap().is_empty());

        let r = rules.rules(NameType::Generic, PrivateRuleType::Exact, "any");
        assert!(r.is_some());
        assert!(!r.unwrap().is_empty());

        let r = rules.rules(NameType::Generic, PrivateRuleType::Exact, "common");
        assert!(r.is_some());
        assert!(!r.unwrap().is_empty());

        let r = rules.rules(NameType::Generic, PrivateRuleType::Approx, "any");
        assert!(r.is_some());
        assert!(!r.unwrap().is_empty());

        let r = rules.rules(NameType::Generic, PrivateRuleType::Approx, "common");
        assert!(r.is_some());
        assert!(!r.unwrap().is_empty());

        let r = rules.rules(NameType::Generic, PrivateRuleType::Rules, "any");
        assert!(r.is_some());
        assert!(!r.unwrap().is_empty());

        let r = rules.rules(NameType::Sephardic, PrivateRuleType::Exact, "any");
        assert!(r.is_some());
        assert!(!r.unwrap().is_empty());

        let r = rules.rules(NameType::Sephardic, PrivateRuleType::Exact, "common");
        assert!(r.is_some());
        assert!(!r.unwrap().is_empty());

        let r = rules.rules(NameType::Sephardic, PrivateRuleType::Approx, "any");
        assert!(r.is_some());
        assert!(!r.unwrap().is_empty());

        let r = rules.rules(NameType::Sephardic, PrivateRuleType::Approx, "common");
        assert!(r.is_some());
        assert!(!r.unwrap().is_empty());

        let r = rules.rules(NameType::Sephardic, PrivateRuleType::Rules, "any");
        assert!(r.is_some());
        assert!(!r.unwrap().is_empty());
    }

    #[test]
    #[cfg(feature = "embedded_bm")]
    fn test_default_unknown_language() {
        let rules = Rules::default();

        let r = rules.rules(NameType::Generic, PrivateRuleType::Exact, "english");
        assert!(r.is_none());
    }

    #[test]
    fn test_with_path() {
        let path = &PathBuf::from("./test_assets/cc-rules/");
        let rules = Rules::new(path, &Languages::try_from(path).unwrap()).unwrap();

        assert!(!rules.rules.is_empty());
    }
}
