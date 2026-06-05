use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use nom::Parser as _;

use crate::beider_morse::languages::Languages;
use crate::beider_morse::ParseBmError;
use crate::nom::{end_of_line, language, multiline_comment};
use crate::{build_parse_error, NameType};

impl TryFrom<&PathBuf> for Languages {
    type Error = ParseBmError;

    fn try_from(directory: &PathBuf) -> Result<Self, Self::Error> {
        let mut map: BTreeMap<NameType, BTreeSet<String>> = BTreeMap::new();
        let paths = fs_err::read_dir(directory)?;

        for path in paths {
            let path = path?;
            if let Ok(name_type) = NameType::try_from(path.file_name()) {
                let content = fs_err::read_to_string(path.path())?;
                let languages = parse_liste(content)?;
                map.insert(name_type, languages);
            }
        }

        Ok(Self { languages: map })
    }
}

fn parse_liste(list: String) -> Result<BTreeSet<String>, ParseBmError> {
    let mut result = BTreeSet::new();
    let mut remains = list.as_str();
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

        // Try language
        if let Ok((rm, language)) = language().parse(remains) {
            remains = rm;
            result.insert(language.to_string());
            continue;
        }

        // Everything fails, then return an error...
        return Err(build_parse_error(
            line_number,
            None,
            remains,
            "Can't parse line for languages".to_string(),
        )
        .into());
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_path() {
        let path = PathBuf::from("./test_assets/cc-rules/");
        let result = Languages::try_from(&path).unwrap();
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
    }
}
