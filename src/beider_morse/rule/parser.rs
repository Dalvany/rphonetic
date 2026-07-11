use std::collections::BTreeMap;

use either::Either;
use enum_iterator::all;
use nom::Parser as _;
use regex::Regex;

use crate::beider_morse::languages::Languages;
use crate::beider_morse::rule::{Phoneme, PhonemeList, PrivateRuleType, Resolver, Rule, Rules};
use crate::beider_morse::{OptimizedRegex, ParseBmError};
use crate::nom::{end_of_line, include, multiline_comment, quadruplet};
use crate::{build_parse_error, LanguageSet, NameType};

fn parse_phoneme(phoneme: &str) -> Result<Phoneme, ParseBmError> {
    let index: Option<(usize, _)> = phoneme.char_indices().find(|(_, c)| c == &'[');
    if let Some((index, _)) = index {
        if !phoneme.ends_with(']') {
            return Err(ParseBmError::UnclosedPhonemeExpression(phoneme.to_owned()));
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

fn parse_phoneme_expr(phoneme_rule: &str) -> Result<PhonemeList, ParseBmError> {
    if phoneme_rule.starts_with('(') {
        if !phoneme_rule.ends_with(')') {
            return Err(ParseBmError::UnclosedPhonemeRule(phoneme_rule.to_owned()));
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
        Ok(PhonemeList { phonemes: phs })
    } else {
        let phoneme = parse_phoneme(phoneme_rule)?;
        Ok(PhonemeList {
            phonemes: vec![phoneme],
        })
    }
}

fn parse_rule(
    resolver: &Resolver,
    filename: &str,
) -> Result<BTreeMap<char, Vec<Rule>>, ParseBmError> {
    let content = resolver.resolve(filename)?;
    let mut result: BTreeMap<char, Vec<Rule>> = BTreeMap::new();
    let mut remains = content.as_str();
    let mut line_number: usize = 0;

    while !remains.is_empty() {
        line_number += 1;

        // Parrsing test from more probable to less probable.
        // Try quadruplet rule
        if let Ok((rm, (pattern, left_context, right_context, phoneme_expr))) =
            quadruplet().parse(remains)
        {
            remains = rm;
            let pattern_length_char = pattern.chars().count();
            let left_context = format!("{left_context}$");
            let left_context: Either<Regex, OptimizedRegex> =
                match &left_context.parse::<OptimizedRegex>() {
                    Ok(optimized) => Either::Right(optimized.clone()),
                    Err(_) => Either::Left(Regex::new(&left_context)?),
                };
            let right_context = format!("^{right_context}");
            let right_context: Either<Regex, OptimizedRegex> =
                match &right_context.parse::<OptimizedRegex>() {
                    Ok(optimized) => Either::Right(optimized.clone()),
                    Err(_) => Either::Left(Regex::new(&right_context)?),
                };
            let phoneme = parse_phoneme_expr(phoneme_expr)?;
            let rule = Rule {
                location: filename.to_string(),
                line: line_number,
                left_context,
                pattern: pattern.to_string(),
                pattern_length_char,
                right_context,
                phoneme,
            };
            let ch = pattern.chars().next().ok_or(ParseBmError::EmptyPattern)?;
            result.entry(ch).or_default().push(rule);
            continue;
        }

        // Try single line comment
        if let Ok((rm, _)) = end_of_line().parse(remains) {
            remains = rm;
            continue;
        }

        // Try includes file
        if let Ok((rm, include_filename)) = include().parse(remains) {
            remains = rm;
            let rules = parse_rule(resolver, include_filename)?;
            result.extend(rules);
            continue;
        }

        // Try multiline comment
        if let Ok((rm, ln)) = multiline_comment().parse(remains) {
            line_number += ln - 1;
            remains = rm;
            continue;
        }

        // Everything fails, then return an error...
        return Err(build_parse_error(
            line_number,
            Some(filename.to_string()),
            remains,
            "Can't parse line".to_string(),
        )
        .into());
    }

    Ok(result)
}

pub(crate) fn build_rules(
    resolver: Resolver,
    languages: &Languages,
) -> Result<Rules, ParseBmError> {
    let mut rules: BTreeMap<(NameType, PrivateRuleType, String), BTreeMap<char, Vec<Rule>>> =
        BTreeMap::new();

    for name_type in all::<NameType>() {
        let l = languages
            .get(&name_type)
            .ok_or(ParseBmError::UnknownNameType(name_type))?;
        for rule_type in all::<PrivateRuleType>() {
            for language in l {
                let filename = format!("{name_type}_{rule_type}_{language}");
                let r = parse_rule(&resolver, &filename)?;
                rules.insert((name_type, rule_type, language.clone()), r);
            }
            if PrivateRuleType::Rules != rule_type {
                let filename = format!("{name_type}_{rule_type}_common");
                let r = parse_rule(&resolver, &filename)?;
                rules.insert((name_type, rule_type, String::from("common")), r);
            }
        }
    }

    Ok(Rules { rules })
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use super::*;

    fn make_phonemes() -> Vec<Vec<Phoneme>> {
        let mut result = Vec::new();

        let data: Vec<Phoneme> = [
            "rinD", "rinDlt", "rina", "rinalt", "rino", "rinolt", "rinu", "rinult",
        ]
        .iter()
        .map(|v| Phoneme {
            phoneme_text: v.to_string(),
            languages: LanguageSet::NoLanguages,
        })
        .collect();
        result.push(data);

        let data: Vec<Phoneme> = ["dortlaj", "dortlej", "ortlaj", "ortlej", "ortlej-dortlaj"]
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
    fn test_phoneme_compared_to_later_is_less() {
        let data = make_phonemes();
        for (set, phonemes) in data.iter().enumerate() {
            for (index, phoneme1) in phonemes.iter().enumerate() {
                for phoneme2 in phonemes.iter().skip(index + 1) {
                    assert_eq!(
                        phoneme1.cmp(phoneme2),
                        Ordering::Less,
                        "Error for data ({set}, {index}) : {phoneme1} should be 'less' than {phoneme2}"
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
                    "Error for data ({set}, {index}) : {phoneme1} should be 'equals' to itself"
                );
            }
        }
    }

    #[test]
    fn test_parse_rule_include() {
        let resolver = Resolver {
            path: Some(PathBuf::from("./test_assets/test-include/")),
        };
        let tmp = parse_rule(&resolver, "gen_exact_german").unwrap();
        let mut result: BTreeSet<String> = BTreeSet::new();
        for v in tmp.values() {
            for r in v {
                result.insert(r.pattern.clone());
            }
        }

        let expected = BTreeSet::from(["included".to_string(), "original".to_string()]);

        assert_eq!(result, expected);
    }
}
