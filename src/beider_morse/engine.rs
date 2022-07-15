use crate::beider_morse::lang::Lang;
use crate::beider_morse::rule::PrivateRuleType;
use crate::{NameType, RuleType};

const DEFAULT_MAX_PHONEMES: usize = 20;

pub struct PhoneticEngine {
    lang: Lang,
    name_type: NameType,
    rule_type: PrivateRuleType,
    concat: bool,
    max_phonemes: usize,
}
