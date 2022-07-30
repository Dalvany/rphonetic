use std::error::Error;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};

pub use rule::RuleType;

use crate::beider_morse::engine::PhoneticEngine;
use crate::beider_morse::lang::Langs;
pub use crate::beider_morse::languages::LanguageSet;
use crate::beider_morse::languages::Languages;
use crate::beider_morse::rule::Rules;
use crate::Encoder;

mod engine;
mod lang;
mod languages;
mod rule;

const ASH: &str = "ash";
const GEN: &str = "gen";
const SEP: &str = "sep";
const DEFAULT_MAX_PHONEMES: usize = 20;

/// Beider-Morse errors.
#[derive(Debug, Clone, PartialEq)]
pub enum BMError {
    /// This error can be raised when parsing a [NameType] that isn't
    /// a variant of the enum or when a filename does not contain
    /// a [NameType] variant.
    UnknownNameType(String),
    /// This error is raised when parsing a [RuleType] that isn't a
    /// variant of the enum.
    UnknownRuleType(String),
    /// This error is raised when a configuration file contains a line
    /// that can't be parsed.
    ParseConfiguration(String),
    /// This error is raised when a rule file is missing.
    WrongFilename(String),
    /// This error is raised when the parser can't parse a phoneme
    /// in a rule file.
    WrongPhoneme(String),
    /// This error is raised when a regex in a rule file is wrong
    BadContextRegex(regex::Error),
    /// This error is raised when trying to parse a boolean in a rule
    /// file. Boolean should be either true or false.
    NotABoolean(String),
    /// This error is raised when a rule is not well-formed.
    BadRule(String),
}

impl Display for BMError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BMError::UnknownNameType(error) => write!(f, "Unknown NameType {}", error),
            BMError::ParseConfiguration(error) => write!(f, "Error reading files {}", error),
            BMError::WrongFilename(error) => write!(f, "Wrong file name : {}", error),
            BMError::WrongPhoneme(error) => write!(f, "{}", error),
            BMError::BadContextRegex(error) => write!(f, "{}", error),
            BMError::NotABoolean(error) => write!(f, "{}", error),
            BMError::BadRule(error) => write!(f, "{}", error),
            BMError::UnknownRuleType(error) => write!(f, "Unknown RuleType {}", error),
        }
    }
}

impl From<std::io::Error> for BMError {
    fn from(error: std::io::Error) -> Self {
        Self::ParseConfiguration(error.to_string())
    }
}

impl From<regex::Error> for BMError {
    fn from(error: regex::Error) -> Self {
        Self::BadContextRegex(error)
    }
}

impl Error for BMError {}

/// Supported type of names. Unless you are matching particular family name, use [generic variant](NameType#Generic)
/// as it should work reasonably well for non-name words. The other variant are specifically tune for family name
/// and may not work well for general text.
#[derive(
    Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize, Sequence,
)]
pub enum NameType {
    /// Ashkenazi family name.
    #[serde(rename = "ash")]
    Ashkenazi,
    /// Generic names and words.
    #[serde(rename = "gen")]
    Generic,
    /// Sephardic family names.
    #[serde(rename = "sep")]
    Sephardic,
}

impl NameType {
    fn language_filename(&self) -> String {
        format!("{}_languages.txt", self)
    }
}

impl Display for NameType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let r = match self {
            Self::Ashkenazi => ASH,
            Self::Generic => GEN,
            Self::Sephardic => SEP,
        };
        write!(f, "{}", r)
    }
}

impl TryFrom<&str> for NameType {
    type Error = BMError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            ASH => Ok(Self::Ashkenazi),
            GEN => Ok(Self::Generic),
            SEP => Ok(Self::Sephardic),
            _ => Err(BMError::UnknownNameType(value.to_string())),
        }
    }
}

impl TryFrom<OsString> for NameType {
    type Error = BMError;

    fn try_from(value: OsString) -> Result<Self, Self::Error> {
        if value == OsString::from(NameType::Ashkenazi.language_filename()) {
            Ok(NameType::Ashkenazi)
        } else if value == OsString::from(NameType::Generic.language_filename()) {
            Ok(NameType::Generic)
        } else if value == OsString::from(NameType::Sephardic.language_filename()) {
            Ok(NameType::Sephardic)
        } else {
            Err(BMError::UnknownNameType(
                value.to_string_lossy().to_string(),
            ))
        }
    }
}

/// This structures contains languages set, rules and language guessing rules.
/// It avoids parsing files multiple time and should be thread-safe.
///
/// If `embedded_bm` feature is enable, then there is a [Default] implementation
/// that only support `any` and `common` languages rules for each variant of
/// [NameType]. It is provided as a convenience but as files are embedded into
/// code, it can result in a significant increase of binary size. The preferred
/// way is to construct a new [ConfigFiles] with a [path to files](ConfigFiles#new).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "embedded_bm", derive(Default))]
pub struct ConfigFiles {
    langs: Langs,
    rules: Rules,
}

impl ConfigFiles {
    /// Construct a new [ConfigFiles].
    ///
    /// # Parameter :
    /// * `directory` : this directory must contain all rules files. You can get them
    /// from [commons-codec](https://github.com/apache/commons-codec/tree/rel/commons-codec-1.15/src/main/resources/org/apache/commons/codec/language/bm)
    /// repository.
    ///
    /// # Errors :
    /// Returns a [BMError] if it misses some files or some rules are not well-formed.
    pub fn new(directory: &PathBuf) -> Result<Self, BMError> {
        let languages = Languages::try_from(directory)?;
        let langs = Langs::new(directory, &languages)?;
        let rules = Rules::new(directory, &languages)?;

        Ok(Self { langs, rules })
    }
}

/// This is the Beider-Morse encoder.
/// It needs rules to work, you can get them
/// from [commons-codec](https://github.com/apache/commons-codec/tree/rel/commons-codec-1.15/src/main/resources/org/apache/commons/codec/language/bm).
/// If feature `embedded_bm`, the default rules will be included in binary, it contains only `any` and `common`
/// rules from commons-codec.
///
/// # Encoding result format
///
/// From [commons-codec Javadoc](https://javadoc.io/static/commons-codec/commons-codec/1.15/org/apache/commons/codec/language/bm/BeiderMorseEncoder.html) :
///
/// Individual phonetic spellings of an input word are represented in upper- and lower-case roman characters. Where there are multiple possible
/// phonetic representations, these are joined with a pipe (`|`) character. If multiple hyphenated words where found, or if the word may contain a
/// name prefix, each encoded word is placed in elipses and these blocks are then joined with hyphens. For example, `d'ortley` has a possible prefix.
/// The form without prefix encodes to `ortlaj|ortlej`, while the form with prefix encodes to `dortlaj|dortlej`. Thus, the full, combined encoding
/// is `(ortlaj|ortlej)-(dortlaj|dortlej)`.
///
/// The encoded forms are often quite a bit longer than the input strings. This is because a single input may have many potential phonetic
/// interpretations. For example, `Renault` encodes to `rYnDlt|rYnalt|rYnult|rinDlt|rinalt|rinult`. The [APPROX](RuleType::Approx) rules will tend to
/// produce larger encodings as they consider a wider range of possible, approximate phonetic interpretations of the original word. Down-stream
/// applications may wish to further process the encoding for indexing or lookup purposes, for example, by splitting on pipe (`|`) and indexing
/// under each of these alternatives.
///
/// # Example
///
/// ```rust
/// # fn main() -> Result<(), rphonetic::PhoneticError> {
/// use std::path::PathBuf;
/// use rphonetic::{BeiderMorseBuilder, ConfigFiles, Encoder};
///
/// let config_files = ConfigFiles::new(&PathBuf::from("./test_assets/cc-rules/"))?;
/// let builder = BeiderMorseBuilder::new(&config_files);
/// let beider_morse = builder.build();
///
/// assert_eq!(beider_morse.encode("Van Helsing"),"(Ylznk|ilzn|ilznk|xilzn|xilznk)-(banilznk|bonilznk|fYnYlznk|fYnilznk|fanYlznk|fanilznk|fonYlznk|fonilznk|vYnYlznk|vYnilznk|vanYlznk|vaniilznk|vanilzn|vanilznk|vonYlznk|voniilznk|vonilzn|vonilznk)");
/// #   Ok(())
/// # }
/// ```
///
/// If you know the language, you can skip language detection using [encode_with_languages](BeiderMorse::encode_with_languages)
#[derive(Debug, Clone)]
pub struct BeiderMorse<'a> {
    engine: PhoneticEngine<'a>,
}

impl<'a> BeiderMorse<'a> {
    /// Encode a value with the provided [LanguageSet]. Using this method will avoid language detection.
    ///
    /// # Parameters
    ///
    /// * `value` : value to encode.
    /// * `languages` : languages to use.
    ///
    /// # Example
    ///
    /// ```rust
    /// # fn main() -> Result<(), rphonetic::PhoneticError> {
    /// use std::path::PathBuf;
    /// use rphonetic::{BeiderMorseBuilder, ConfigFiles, Encoder, LanguageSet, RuleType};
    ///
    /// let config_files = ConfigFiles::new(&PathBuf::from("./test_assets/cc-rules/"))?;
    /// let builder = BeiderMorseBuilder::new(&config_files).rule_type(RuleType::Exact);
    /// let beider_morse = builder.build();
    ///
    /// assert_eq!(beider_morse.encode("Angelo"),"anZelo|andZelo|angelo|anhelo|anjelo|anxelo");
    ///
    /// let language_set = LanguageSet::from(vec!["italian", "greek", "spanish"]);
    /// assert_eq!(beider_morse.encode_with_languages("Angelo", &language_set),"andZelo|angelo|anxelo");
    ///
    /// let language_set = LanguageSet::from(vec!["italian"]);
    /// assert_eq!(beider_morse.encode_with_languages("Angelo", &language_set),"andZelo");
    ///
    /// #   Ok(())
    /// # }
    /// ```
    pub fn encode_with_languages(&self, value: &str, languages: &LanguageSet) -> String {
        self.engine.encode_with_language_set(value, languages)
    }
}

impl<'a> Encoder for BeiderMorse<'a> {
    fn encode(&self, value: &str) -> String {
        self.engine.encode(value)
    }
}

/// This is a builder to construct a [BeiderMorse] encoder.
/// By default, it will use [generic name type](NameType#Generic), [approximate rules](RuleType#Approx),
/// it won't concatenate multiple phonetic encoding.
#[derive(Debug, Clone)]
pub struct BeiderMorseBuilder {
    config_files: ConfigFiles,
    name_type: NameType,
    rule_type: RuleType,
    concat: bool,
    max_phonemes: usize,
}

impl BeiderMorseBuilder {
    /// this will instantiate a new builder with the rules provided.
    ///
    /// # Parameter :
    ///
    /// * `config_files` : rules.
    pub fn new(config_files: &ConfigFiles) -> Self {
        Self {
            config_files: config_files.clone(),
            name_type: NameType::Generic,
            rule_type: RuleType::Approx,
            concat: true,
            max_phonemes: DEFAULT_MAX_PHONEMES,
        }
    }

    /// Set the [NameType] to use.
    pub fn name_type(mut self, name_type: NameType) -> Self {
        self.name_type = name_type;
        self
    }

    /// Set the [RuleType] to use.
    pub fn rule_type(mut self, rule_type: RuleType) -> Self {
        self.rule_type = rule_type;
        self
    }

    /// Set if multiple phoneme are combined. If `true` then multiple
    /// phonemes will be concatenated if a `|`.
    pub fn concat(mut self, concat: bool) -> Self {
        self.concat = concat;
        self
    }

    /// Set the maximum number of phonemes that should be considered by
    /// the engine.
    pub fn max_phonemes(mut self, max_phonemes: usize) -> Self {
        self.max_phonemes = max_phonemes;
        self
    }

    /// Build a new [BeiderMorse] encoder.
    pub fn build(&self) -> BeiderMorse {
        let lang = self.config_files.langs.get(&self.name_type).unwrap();
        let rules = &self.config_files.rules;
        let engine = PhoneticEngine {
            rules,
            lang,
            name_type: self.name_type,
            rule_type: self.rule_type.into(),
            concat: self.concat,
            max_phonemes: self.max_phonemes,
        };
        BeiderMorse { engine }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "embedded_bm")]
    use crate::beider_morse::rule::PrivateRuleType;

    lazy_static! {
        static ref CONFIG_FILE: ConfigFiles =
            ConfigFiles::new(&PathBuf::from("./test_assets/cc-rules/")).unwrap();
    }

    #[test]
    fn test_all_chars() -> Result<(), BMError> {
        let builder = BeiderMorseBuilder::new(&CONFIG_FILE);
        let encoder = builder.build();

        for ch in '\u{0000}'..'\u{ffff}' {
            let _ = encoder.encode(ch.to_string().as_str());
        }

        Ok(())
    }

    #[test]
    fn test_oom() -> Result<(), BMError> {
        let input = "200697900'-->&#1913348150;</  bceaeef >aadaabcf\"aedfbff<!--\'-->?>cae\
        cfaaa><?&#<!--</script>&lang&fc;aadeaf?>>&bdquo<    cc =\"abff\"    /></   afe  ><script>\
        <!-- f(';<    cf aefbeef = \"bfabadcf\" ebbfeedd = fccabeb >";

        let builder = BeiderMorseBuilder::new(&CONFIG_FILE)
            .name_type(NameType::Generic)
            .rule_type(RuleType::Exact)
            .max_phonemes(10);
        let encoder = builder.build();

        let result = encoder.encode(input);
        assert!(!result.is_empty());

        let result = result.split('|').count();
        assert!(result <= 10);

        Ok(())
    }

    #[test]
    fn test_ascii_encode_not_empty_1_letter() -> Result<(), BMError> {
        let builder = BeiderMorseBuilder::new(&CONFIG_FILE);
        let encoder = builder.build();
        for ch in 'a'..'z' {
            assert_ne!(encoder.encode(&ch.to_string()), "");
            assert_ne!(encoder.encode(&ch.to_ascii_uppercase().to_string()), "");
        }

        Ok(())
    }

    #[test]
    fn test_ascii_encode_not_empty_2_letters() -> Result<(), BMError> {
        let builder = BeiderMorseBuilder::new(&CONFIG_FILE);
        let encoder = builder.build();
        for ch1 in 'a'..'z' {
            for ch2 in 'a'..'z' {
                let mut string = String::with_capacity(2);
                string.push(ch1);
                string.push(ch2);
                assert_ne!(encoder.encode(&string), "");
                assert_ne!(encoder.encode(&string.to_ascii_uppercase().to_string()), "");
            }
        }

        Ok(())
    }

    #[test]
    fn test_encode_atz_not_empty() -> Result<(), BMError> {
        let data = vec![
            "\u{00e1}cz",
            "\u{00e1}tz",
            "Ign\u{00e1}cz",
            "Ign\u{00e1}tz",
            "Ign\u{00e1}c",
        ];
        let builder = BeiderMorseBuilder::new(&CONFIG_FILE);
        let encoder = builder.build();

        for d in data {
            assert_ne!(encoder.encode(d), "");
        }

        Ok(())
    }

    #[test]
    fn test_encode_gna() -> Result<(), BMError> {
        let builder = BeiderMorseBuilder::new(&CONFIG_FILE);
        let encoder = builder.build();

        assert_ne!(encoder.encode("gna"), "");

        Ok(())
    }

    #[test]
    fn test_longest_english_surname() -> Result<(), BMError> {
        let builder = BeiderMorseBuilder::new(&CONFIG_FILE);
        let encoder = builder.build();

        assert_ne!(encoder.encode("MacGhilleseatheanaich"), "");

        Ok(())
    }

    #[test]
    fn test_speed_check() -> Result<(), BMError> {
        let test_chars: Vec<char> = vec!['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'o', 'u'];

        let builder = BeiderMorseBuilder::new(&CONFIG_FILE);
        let encoder = builder.build();

        let mut string = String::with_capacity(40);

        for i in 0..40 {
            string.push(test_chars[i % test_chars.len()]);
            assert_ne!(encoder.encode(&string), "");
        }

        assert_ne!(
            encoder.encode("ItstheendoftheworldasweknowitandIfeelfine"),
            ""
        );

        Ok(())
    }

    #[test]
    fn test_speed_check_2() -> Result<(), BMError> {
        let builder = BeiderMorseBuilder::new(&CONFIG_FILE);
        let encoder = builder.build();

        assert_ne!(
            encoder.encode("abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz"),
            ""
        );

        Ok(())
    }

    #[test]
    fn test_speed_check_3() -> Result<(), BMError> {
        let builder = BeiderMorseBuilder::new(&CONFIG_FILE);
        let encoder = builder.build();

        assert_ne!(
            encoder.encode("abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz"),
            ""
        );

        Ok(())
    }

    #[test]
    #[cfg(feature = "embedded_bm")]
    /// Basic test checking that it doesn't fail
    fn test_config_file_default() {
        let config_file = ConfigFiles::default();

        let rules = config_file
            .rules
            .rules(NameType::Generic, PrivateRuleType::Exact, "any");
        assert!(rules.is_some());
        assert_eq!(rules.unwrap().len(), 8);

        let rules = config_file
            .rules
            .rules(NameType::Generic, PrivateRuleType::Approx, "any");
        assert!(rules.is_some());
        assert_eq!(rules.unwrap().len(), 22);
    }

    #[test]
    fn test_builder() {
        let builder = BeiderMorseBuilder::new(&CONFIG_FILE);

        assert_eq!(builder.rule_type, RuleType::Approx);
        assert_eq!(builder.name_type, NameType::Generic);
        assert!(builder.concat);
        assert_eq!(builder.max_phonemes, DEFAULT_MAX_PHONEMES);

        let builder = builder.concat(false);

        assert_eq!(builder.rule_type, RuleType::Approx);
        assert_eq!(builder.name_type, NameType::Generic);
        assert!(!builder.concat);
        assert_eq!(builder.max_phonemes, DEFAULT_MAX_PHONEMES);

        let builder = builder.max_phonemes(5);

        assert_eq!(builder.rule_type, RuleType::Approx);
        assert_eq!(builder.name_type, NameType::Generic);
        assert!(!builder.concat);
        assert_eq!(builder.max_phonemes, 5);

        let builder = builder.name_type(NameType::Ashkenazi);

        assert_eq!(builder.rule_type, RuleType::Approx);
        assert_eq!(builder.name_type, NameType::Ashkenazi);
        assert!(!builder.concat);
        assert_eq!(builder.max_phonemes, 5);

        let builder = builder.rule_type(RuleType::Exact);

        assert_eq!(builder.rule_type, RuleType::Exact);
        assert_eq!(builder.name_type, NameType::Ashkenazi);
        assert!(!builder.concat);
        assert_eq!(builder.max_phonemes, 5);
    }
}
