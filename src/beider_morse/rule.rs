use std::cmp::Ordering;
use std::collections::btree_map::BTreeMap;
use std::fmt::{Debug, Display, Formatter};
use std::path::{Path, PathBuf};

use enum_iterator::{all, Sequence};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::beider_morse::Languages;
use crate::constants::{
    BM_INCLUDE_LINE, MULTI_LINE_COMMENT_END, MULTI_LINE_COMMENT_START, RULE_LINE,
    SINGLE_LINE_COMMENT,
};
use crate::{BMError, NameType};

use super::LanguageSet;

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
        write!(f, "{}", r)
    }
}

pub trait PhonemeExpr: Debug {
    fn phonemes(&self) -> Vec<&Phoneme>;
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Phoneme {
    phoneme_text: String,
    languages: LanguageSet,
}

impl PartialOrd<Self> for Phoneme {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let iterator = self.phoneme_text.chars().zip(other.phoneme_text.chars());
        for (ch1, ch2) in iterator {
            if ch1 != ch2 {
                return ch1.partial_cmp(&ch2);
            }
        }

        let o1length = self.phoneme_text.len();
        let o2length = other.phoneme_text.len();

        o1length.partial_cmp(&o2length)
    }
}

impl Ord for Phoneme {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PhonemeExpr for Phoneme {
    fn phonemes(&self) -> Vec<&Phoneme> {
        vec![self]
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
        let phoneme_text = format!("{}{}", phoneme1.phoneme_text, phoneme2.phoneme_text);
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
struct PhonemeList {
    phonemes: Vec<Phoneme>,
}

impl PhonemeExpr for PhonemeList {
    fn phonemes(&self) -> Vec<&Phoneme> {
        self.phonemes.iter().collect()
    }
}

fn parse_phoneme(phoneme: &str) -> Result<Phoneme, BMError> {
    let index: Option<(usize, _)> = phoneme.char_indices().find(|(_, c)| c == &'[');
    if let Some((index, _)) = index {
        if !phoneme.ends_with(']') {
            return Err(BMError::WrongPhoneme(format!(
                "Phoneme expression {} has a '[' but doesn't ends with an ']'",
                phoneme
            )));
        }
        let before = &phoneme[0..index];
        let after = &phoneme[index + 1..phoneme.len() - 1];
        let languages: Vec<&str> = after.split('+').collect();
        Ok(Phoneme {
            phoneme_text: before.to_string(),
            languages: LanguageSet::from(languages),
        })
    } else {
        Ok(Phoneme {
            phoneme_text: phoneme.to_string(),
            languages: LanguageSet::Any,
        })
    }
}

fn parse_phoneme_expr(phoneme_rule: &str) -> Result<Box<dyn PhonemeExpr>, BMError> {
    if phoneme_rule.starts_with('(') {
        if !phoneme_rule.ends_with(')') {
            return Err(BMError::WrongPhoneme(format!(
                "Wrong phoneme rule {}",
                phoneme_rule
            )));
        }
        let mut phs: Vec<Phoneme> = Vec::new();
        let phoneme_rule = &phoneme_rule[1..phoneme_rule.len() - 1];
        for phoneme in phoneme_rule.split('|') {
            phs.push(parse_phoneme(phoneme)?)
        }
        if phoneme_rule.starts_with('|') || phoneme_rule.ends_with('|') {
            phs.push(Phoneme {
                phoneme_text: "".to_string(),
                languages: LanguageSet::Any,
            })
        }
        Ok(Box::new(PhonemeList { phonemes: phs }))
    } else {
        Ok(Box::new(parse_phoneme(phoneme_rule)?))
    }
}

fn parse_rule(resolver: &Resolver, filename: &str) -> Result<BTreeMap<char, Vec<Rule>>, BMError> {
    let content = resolver.resolve(filename)?;
    let mut multiline_comment = false;

    let mut result: BTreeMap<char, Vec<Rule>> = BTreeMap::new();

    for (current_line, mut line) in content.split('\n').enumerate() {
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

        let include_match = BM_INCLUDE_LINE.captures(line);
        if let Some(include) = include_match {
            let include_file = include.get(1);
            let included_file = include_file.unwrap().as_str();
            let include_rule = parse_rule(resolver, included_file);
            match include_rule {
                Ok(rules) => {
                    result.extend(rules);
                }
                Err(error) => {
                    return Err(BMError::WrongFilename(format!(
                        "Can't include file {} in {:?} at line {} : {}",
                        included_file, filename, current_line, error
                    )));
                }
            }
        }
        let rules_line_match = RULE_LINE.captures(line);
        if let Some(cap) = rules_line_match {
            let pattern = cap.get(1).unwrap().as_str();
            let left_context = cap.get(2).unwrap().as_str();
            let left_context = Regex::new(left_context)?;
            let right_context = cap.get(3).unwrap().as_str();
            let right_context = Regex::new(right_context)?;
            let phoneme_expr = cap.get(4).unwrap().as_str();
            let phoneme = parse_phoneme_expr(phoneme_expr)?;
            let rule = Rule {
                location: filename.to_string(),
                line: current_line,
                left_context,
                pattern: pattern.to_string(),
                right_context,
                phoneme,
            };
            let ch = pattern.chars().next().unwrap();
            result.entry(ch).or_insert_with(Vec::new);
            let rules = result.get_mut(&ch).unwrap();
            rules.push(rule);
        }
    }

    Ok(result)
}

fn build_rules(resolver: Resolver, languages: &Languages) -> Result<Rules, BMError> {
    let mut rules: BTreeMap<(NameType, PrivateRuleType, String), BTreeMap<char, Vec<Rule>>> =
        BTreeMap::new();

    for name_type in all::<NameType>() {
        let l = languages
            .get(&name_type)
            .ok_or_else(|| BMError::UnknownNameType(name_type.language_filename()))?;
        for rule_type in all::<PrivateRuleType>() {
            for language in l {
                let filename = format!("{}_{}_{}", name_type, rule_type, language);
                let r = parse_rule(&resolver, &filename)?;
                rules.insert((name_type, rule_type, language.clone()), r);
            }
            if PrivateRuleType::Rules != rule_type {
                let filename = format!("{}_{}_common", name_type, rule_type);
                let r = parse_rule(&resolver, &filename)?;
                rules.insert((name_type, rule_type, String::from("common")), r);
            }
        }
    }

    Ok(Rules { rules })
}

struct Resolver {
    path: Option<PathBuf>,
}

impl Resolver {
    fn resolve(&self, filename: &str) -> Result<String, BMError> {
        match &self.path {
            Some(folder) => {
                let f = folder.join(format!("{}.txt", filename));
                std::fs::read_to_string(f).map_err(|_| {
                    BMError::WrongFilename(format!("Can't find file for {} rules", filename))
                })
            }
            #[cfg(feature = "embedded")]
            None => embedded::EMBEDDED_RULES
                .get(filename)
                .map(|v| v.to_string())
                .ok_or_else(|| {
                    BMError::WrongFilename(format!("Missing embedded rule {}", filename))
                }),
            #[cfg(not(feature = "embedded"))]
            None => Err(BMError::WrongFilename(
                "Missing embedded configuration. Use corresponding feature".to_string(),
            )),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Rule {
    location: String,
    line: usize,
    left_context: Regex,
    pattern: String,
    right_context: Regex,
    phoneme: Box<dyn PhonemeExpr>,
}

impl Rule {
    pub(crate) fn pattern_and_context_matches(&self, input: &str, index: usize) -> bool {
        let ipl = input.len() + self.pattern.len();
        if ipl > input.len()
            || input[index..ipl] != self.pattern
            || !self.right_context.is_match(&input[ipl..])
        {
            false
        } else {
            self.left_context.is_match(&input[..index])
        }
    }

    pub(crate) fn pattern(&self) -> &String {
        &self.pattern
    }

    pub(crate) fn phoneme(&self) -> &Box<dyn PhonemeExpr> {
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

#[derive(Debug)]
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

    pub fn new(rules_folder: &Path, languages: &Languages) -> Result<Self, BMError> {
        let resolver = Resolver {
            path: Some(rules_folder.to_path_buf()),
        };
        build_rules(resolver, languages)
    }
}

/// Module that contains default rules (any and commons) and [Default] implementation
/// for [Rules] for convenience with features.
#[cfg(feature = "embedded")]
mod embedded {
    use std::collections::BTreeMap;

    use super::*;

    const ASH_APPROX_ANY: &str = include_str!("../../rules/bm/ash_approx_any.txt");
    const ASH_APPROX_COMMON: &str = include_str!("../../rules/bm/ash_approx_common.txt");
    const ASH_EXACT_ANY: &str = include_str!("../../rules/bm/ash_exact_any.txt");
    const ASH_EXACT_COMMON: &str = include_str!("../../rules/bm/ash_exact_common.txt");
    const ASH_RULES_ANY: &str = include_str!("../../rules/bm/ash_rules_any.txt");

    const GEN_APPROX_ANY: &str = include_str!("../../rules/bm/gen_approx_any.txt");
    const GEN_APPROX_COMMON: &str = include_str!("../../rules/bm/gen_approx_common.txt");
    const GEN_EXACT_ANY: &str = include_str!("../../rules/bm/gen_exact_any.txt");
    const GEN_EXACT_COMMON: &str = include_str!("../../rules/bm/gen_exact_common.txt");
    const GEN_RULES_ANY: &str = include_str!("../../rules/bm/gen_rules_any.txt");

    const SEP_APPROX_ANY: &str = include_str!("../../rules/bm/sep_approx_any.txt");
    const SEP_APPROX_COMMON: &str = include_str!("../../rules/bm/sep_approx_common.txt");
    const SEP_EXACT_ANY: &str = include_str!("../../rules/bm/sep_exact_any.txt");
    const SEP_EXACT_COMMON: &str = include_str!("../../rules/bm/sep_exact_common.txt");
    const SEP_RULES_ANY: &str = include_str!("../../rules/bm/sep_rules_any.txt");

    lazy_static! {
        pub static ref EMBEDDED_RULES: BTreeMap<&'static str, &'static str> = BTreeMap::from([
            ("ash_approx_any", ASH_APPROX_ANY),
            ("ash_approx_common", ASH_APPROX_COMMON),
            ("ash_exact_any", ASH_EXACT_ANY),
            ("ash_exact_common", ASH_EXACT_COMMON),
            ("ash_rules_any", ASH_RULES_ANY),
            ("gen_approx_any", GEN_APPROX_ANY),
            ("gen_approx_common", GEN_APPROX_COMMON),
            ("gen_exact_any", GEN_EXACT_ANY),
            ("gen_exact_common", GEN_EXACT_COMMON),
            ("gen_rules_any", GEN_RULES_ANY),
            ("sep_approx_any", SEP_APPROX_ANY),
            ("sep_approx_common", SEP_APPROX_COMMON),
            ("sep_exact_any", SEP_EXACT_ANY),
            ("sep_exact_common", SEP_EXACT_COMMON),
            ("sep_rules_any", SEP_RULES_ANY),
        ]);
    }

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

    fn make_phonemes() -> Vec<Vec<Phoneme>> {
        let mut result = Vec::new();

        let data: Vec<Phoneme> = vec![
            "rinD", "rinDlt", "rina", "rinalt", "rino", "rinolt", "rinu", "rinult",
        ]
        .iter()
        .map(|v| Phoneme {
            phoneme_text: v.to_string(),
            languages: LanguageSet::NoLanguages,
        })
        .collect();
        result.push(data);

        let data: Vec<Phoneme> = vec!["dortlaj", "dortlej", "ortlaj", "ortlej", "ortlej-dortlaj"]
            .iter()
            .map(|v| Phoneme {
                phoneme_text: v.to_string(),
                languages: LanguageSet::NoLanguages,
            })
            .collect();
        result.push(data);

        result
    }

    #[test]
    #[cfg(feature = "embedded")]
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
    #[cfg(feature = "embedded")]
    fn test_default_unknown_language() {
        let rules = Rules::default();

        let r = rules.rules(NameType::Generic, PrivateRuleType::Exact, "english");
        assert!(r.is_none());
    }

    #[test]
    fn test_with_path() -> Result<(), BMError> {
        let path = &PathBuf::from("./test_assets/");
        let rules = Rules::new(path, &Languages::try_from(path)?)?;

        assert!(!rules.rules.is_empty());

        Ok(())
    }

    #[test]
    fn test_phoneme_compared_to_later_is_less() {
        let data = make_phonemes();
        for (set, phonemes) in data.iter().enumerate() {
            for (index, phoneme1) in phonemes.iter().enumerate() {
                for phoneme2 in phonemes.iter().skip(index + 1) {
                    assert_eq!(
                        phoneme1.cmp(phoneme2),
                        Ordering::Less,
                        "Error for data ({}, {}) : {} should be 'less' than {}",
                        set,
                        index,
                        phoneme1,
                        phoneme2
                    );
                }
            }
        }
    }

    #[test]
    fn test_phoneme_compared_to_self_is_equals() {
        let data = make_phonemes();
        for (set, phonemes) in data.iter().enumerate() {
            for (index, phoneme1) in phonemes.iter().enumerate() {
                assert_eq!(
                    phoneme1.cmp(phoneme1),
                    Ordering::Equal,
                    "Error for data ({}, {}) : {} should be 'equals' to itself",
                    set,
                    index,
                    phoneme1
                );
            }
        }
    }
}
