use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::beider_morse::lang::Lang;
use crate::beider_morse::languages::LanguageSet;
use crate::beider_morse::rule::{Phoneme, PhonemeExpr, PrivateRuleType, Rule, Rules};
use crate::NameType;

lazy_static! {
    static ref NAME_PREFIXES: BTreeMap<NameType, BTreeSet<&'static str>> = BTreeMap::from([
        (
            NameType::Ashkenazi,
            BTreeSet::from(["bar", "ben", "da", "de", "van", "von"])
        ),
        (
            NameType::Generic,
            BTreeSet::from([
                "da", "dal", "de", "del", "dela", "de la", "della", "des", "di", "do", "dos", "du",
                "van", "von"
            ])
        ),
        (
            NameType::Sephardic,
            BTreeSet::from([
                "al", "el", "da", "dal", "de", "del", "dela", "de la", "della", "des", "di", "do",
                "dos", "du", "van", "von"
            ])
        )
    ]);
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
struct PhonemeBuilder {
    phonemes: BTreeSet<Phoneme>,
}

impl PhonemeBuilder {
    fn empty(languages: &LanguageSet) -> Self {
        let phonemes = BTreeSet::from([Phoneme::new("", languages.clone())]);
        Self { phonemes }
    }

    fn append_char(&mut self, value: char) {
        self.append(&value.to_string())
    }

    fn append(&mut self, text: &str) {
        self.phonemes = self
            .phonemes
            .iter()
            .cloned()
            .map(|v| v.append(text))
            .collect();
    }

    fn make_string(&self) -> String {
        self.phonemes
            .iter()
            .map(|v| v.phoneme_text())
            .collect::<Vec<String>>()
            .join("|")
    }

    fn apply(&mut self, phoneme_expr: &Box<dyn PhonemeExpr>, max_phonemes: usize) {
        let mut phonemes: BTreeSet<Phoneme> = BTreeSet::new();

        'outer: for left in self.phonemes.iter() {
            for right in phoneme_expr.phonemes().iter() {
                let languages = left.languages().restrict_to(right.languages());
                if !languages.is_empty() {
                    let phoneme = Phoneme::join(left, right, languages);
                    if phonemes.len() < max_phonemes {
                        phonemes.insert(phoneme);
                    }
                    if phonemes.len() >= max_phonemes {
                        break 'outer;
                    }
                }
            }
        }
        self.phonemes = phonemes;
    }
}

#[derive(Debug)]
struct RulesApplication<'a> {
    rules: &'a BTreeMap<char, Vec<Rule>>,
    input: &'a String,
    phoneme_builder: &'a mut PhonemeBuilder,
    i: usize,
    max_phoneme: usize,
    found: bool,
}

impl<'a> RulesApplication<'a> {
    fn i(&self) -> usize {
        self.i
    }

    fn invoke(mut self) -> Self {
        self.found = false;
        let mut pattern_length: usize = 1;
        let key = self.input.chars().next().unwrap();
        let rules = self.rules.get(&key);
        if let Some(rules) = rules {
            for rule in rules {
                let pattern = rule.pattern();
                pattern_length = pattern.len();
                if rule.pattern_and_context_matches(pattern, pattern_length) {
                    self.phoneme_builder.apply(rule.phoneme(), self.max_phoneme);
                    self.found = true;
                    break;
                }
            }
        }

        if !self.found {
            pattern_length = 1;
        }

        self.i += pattern_length;
        self
    }
}

#[derive(Debug)]
pub(crate) struct PhoneticEngine<'a> {
    pub(crate) rules: &'a Rules,
    pub(crate) lang: &'a Lang,
    pub(crate) name_type: NameType,
    pub(crate) rule_type: PrivateRuleType,
    pub(crate) concat: bool,
    pub(crate) max_phonemes: usize,
}

impl<'a> PhoneticEngine<'a> {
    fn apply_final_rule(
        &self,
        phoneme_builder: PhonemeBuilder,
        final_rules: &BTreeMap<char, Vec<Rule>>,
    ) -> PhonemeBuilder {
        if final_rules.is_empty() {
            return phoneme_builder;
        }

        let mut phonemes: BTreeSet<Phoneme> = BTreeSet::new();
        for phoneme in phoneme_builder.phonemes {
            let mut sub_builder = PhonemeBuilder::empty(phoneme.languages());
            let phoneme_text = phoneme.phoneme_text();

            let mut i = 0;
            let len = phoneme_text.len();
            while i < len {
                let rules_application = RulesApplication {
                    rules: final_rules,
                    input: &phoneme_text,
                    phoneme_builder: &mut sub_builder,
                    i,
                    max_phoneme: self.max_phonemes,
                    found: false,
                }
                .invoke();
                let new_i = rules_application.i();

                if !rules_application.found {
                    let txt = phoneme_text.chars().nth(i).unwrap();
                    sub_builder.append_char(txt);
                }

                i = new_i;
            }

            for new_phoneme in sub_builder.phonemes {
                if phonemes.contains(&new_phoneme) {
                    let old_phoneme = phonemes.get(&new_phoneme).unwrap();
                    let merge_phoneme = old_phoneme.merge_with_language(phoneme.languages());
                    // Since equality is on text, replace should work fine
                    phonemes.replace(merge_phoneme);
                } else {
                    phonemes.insert(new_phoneme);
                }
            }
        }

        PhonemeBuilder { phonemes }
    }

    pub fn encode(&self, input: &str) -> String {
        let languages = self.lang.guess_languages(input);
        self.encode_with_language_set(input, &languages)
    }

    pub fn encode_with_language_set(&self, input: &str, languages: &LanguageSet) -> String {
        let l = if languages.is_singleton() {
            languages.any().unwrap()
        } else {
            "any".to_string()
        };
        let rules = self
            .rules
            .rules(self.name_type, PrivateRuleType::Rules, l.as_str())
            .unwrap();
        let final_rules1 = self
            .rules
            .rules(self.name_type, self.rule_type, "common")
            .unwrap();
        let final_rules2 = self
            .rules
            .rules(self.name_type, self.rule_type, l.as_str())
            .unwrap();

        let input = input.to_lowercase().replace('-', " ");

        if self.name_type == NameType::Generic {
            if let Some(remainder) = input.strip_prefix("d'") {
                let combined = self.encode(format!("d{}", remainder).as_str());
                return format!("({})-({})", self.encode(remainder), combined);
            }
            for prefix in NAME_PREFIXES.get(&self.name_type).unwrap() {
                let p = format!("{} ", prefix);
                if let Some(remainder) = input.strip_prefix(p.as_str()) {
                    let combined = self.encode(format!("{}{}", p, remainder).as_str());
                    return format!("({})-({})", self.encode(remainder), combined);
                }
            }
        }

        let words: Vec<&str> = input
            .split_whitespace()
            .map(|v| {
                if self.name_type == NameType::Sephardic {
                    v.split('\'').last().unwrap()
                } else {
                    v
                }
            })
            .filter(|v| {
                self.name_type == NameType::Generic
                    || !NAME_PREFIXES.get(&self.name_type).unwrap().contains(v)
            })
            .collect();

        let input = if self.concat {
            words.join(" ")
        } else if words.len() == 1 {
            words.first().unwrap().to_string()
        } else {
            return words
                .iter()
                .map(|v| self.encode(v))
                .collect::<Vec<String>>()
                .join("-");
        };

        let mut phoneme_builder = &mut PhonemeBuilder::empty(languages);
        let mut i = 0;
        let end = input.len();
        while i < end {
            let rules_application = RulesApplication {
                rules,
                input: &input,
                phoneme_builder,
                i,
                max_phoneme: self.max_phonemes,
                found: false,
            }
            .invoke();
            i = rules_application.i();
            phoneme_builder = rules_application.phoneme_builder;
        }

        // "unmut"
        let phoneme_builder = phoneme_builder.clone();
        let phoneme_builder = self.apply_final_rule(phoneme_builder, final_rules1);
        let phoneme_builder = self.apply_final_rule(phoneme_builder, final_rules2);

        phoneme_builder.make_string()
    }
}
