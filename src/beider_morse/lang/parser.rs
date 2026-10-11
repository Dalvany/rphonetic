use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use enum_iterator::all;
use nom::Parser as _;
use regex::Regex;

use crate::beider_morse::lang::{Lang, LangRule, Langs};
use crate::beider_morse::languages::Languages;
use crate::beider_morse::ParseBmError;
use crate::nom::{end_of_line, lang, multiline_comment};
use crate::{build_parse_error, NameType};

fn parse_lang(
    filename: Option<String>,
    content: String,
    languages: &BTreeSet<String>,
) -> Result<Lang, ParseBmError> {
    let mut rules: Vec<LangRule> = Vec::new();
    let mut remains = content.as_str();
    let mut line_number: usize = 0;

    while !remains.is_empty() {
        line_number += 1;

        // Since parts are not delimited we try first to parse comment either single line
        // or multiline.

        // Try single line comment
        if let Ok((rm, _)) = end_of_line().parse(remains) {
            remains = rm;
            continue;
        }

        // Try multiline comment
        if let Ok((rm, ln)) = multiline_comment().parse(remains) {
            line_number += ln - 1;
            remains = rm;
            continue;
        }

        if let Ok((rm, (pattern, langs, accept_on_match))) = lang().parse(remains) {
            remains = rm;

            let pattern: Regex = Regex::new(pattern).map_err(|error| {
                build_parse_error(line_number, filename.clone(), remains, error.to_string())
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
        return Err(build_parse_error(
            line_number,
            None,
            remains,
            "Can't parse line for language detection".to_string(),
        )
        .into());
    }

    Ok(Lang {
        languages: languages.clone(),
        rules,
    })
}

pub(crate) fn build_langs(
    directory: &Path,
    languages_set: &Languages,
) -> Result<Langs, ParseBmError> {
    let mut langs: BTreeMap<NameType, Lang> = BTreeMap::new();

    for name_type in all::<NameType>() {
        let languages = languages_set
            .get(&name_type)
            .ok_or(ParseBmError::MissingNameType(name_type))?;
        let filename = directory.join(format!("{name_type}_lang.txt"));
        let content = fs_err::read_to_string(filename.clone())?;
        let filename = filename.to_str().map(|v| v.to_string());
        let lang = parse_lang(filename, content, languages)?;
        langs.insert(name_type, lang);
    }

    Ok(Langs { langs })
}
