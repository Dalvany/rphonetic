use serde::{Deserialize, Serialize};

use crate::helper::is_vowel;
use crate::Encoder;

/// he plain letter equivalent of the accented letters.
const PLAIN_ASCII: [char; 60] = [
    'A', 'a', 'E', 'e', 'I', 'i', 'O', 'o', 'U', 'u', 'A', 'a', 'E', 'e', 'I', 'i', 'O', 'o', 'U',
    'u', 'Y', 'y', 'A', 'a', 'E', 'e', 'I', 'i', 'O', 'o', 'U', 'u', 'Y', 'y', 'A', 'a', 'O', 'o',
    'N', 'n', 'A', 'a', 'E', 'e', 'I', 'i', 'O', 'o', 'U', 'u', 'Y', 'y', 'A', 'a', 'C', 'c', 'O',
    'o', 'U', 'u',
];

/// Unicode characters corresponding to various accented letters. For example: \u{00DA} is U acute etc...
const UNICODE: [char; 60] = [
    '\u{00C0}', '\u{00E0}', '\u{00C8}', '\u{00E8}', '\u{00CC}', '\u{00EC}', '\u{00D2}', '\u{00F2}',
    '\u{00D9}', '\u{00F9}', '\u{00C1}', '\u{00E1}', '\u{00C9}', '\u{00E9}', '\u{00CD}', '\u{00ED}',
    '\u{00D3}', '\u{00F3}', '\u{00DA}', '\u{00FA}', '\u{00DD}', '\u{00FD}', '\u{00C2}', '\u{00E2}',
    '\u{00CA}', '\u{00EA}', '\u{00CE}', '\u{00EE}', '\u{00D4}', '\u{00F4}', '\u{00DB}', '\u{00FB}',
    '\u{0176}', '\u{0177}', '\u{00C3}', '\u{00E3}', '\u{00D5}', '\u{00F5}', '\u{00D1}', '\u{00F1}',
    '\u{00C4}', '\u{00E4}', '\u{00CB}', '\u{00EB}', '\u{00CF}', '\u{00EF}', '\u{00D6}', '\u{00F6}',
    '\u{00DC}', '\u{00FC}', '\u{0178}', '\u{00FF}', '\u{00C5}', '\u{00E5}', '\u{00C7}', '\u{00E7}',
    '\u{0150}', '\u{0151}', '\u{0170}', '\u{0171}',
];

const DOUBLE_CONSONANT: [(&str, &str); 21] = [
    ("BB", "B"),
    ("CC", "C"),
    ("DD", "D"),
    ("FF", "F"),
    ("GG", "G"),
    ("HH", "H"),
    ("JJ", "J"),
    ("KK", "K"),
    ("LL", "L"),
    ("MM", "M"),
    ("NN", "N"),
    ("PP", "P"),
    ("QQ", "Q"),
    ("RR", "R"),
    ("SS", "S"),
    ("TT", "T"),
    ("VV", "V"),
    ("WW", "W"),
    ("XX", "X"),
    ("YY", "Y"),
    ("ZZ", "Z"),
];

const CHAR_TO_TRIM: [char; 5] = ['-', '&', '\'', '.', ','];

/// This the [match rating approach](https://en.wikipedia.org/wiki/Match_rating_approach) [Encoder].
///
/// # Example
///
/// ```rust
/// use rphonetic::{Encoder, MatchRatingApproach};
///
/// let match_rating = MatchRatingApproach;
/// assert_eq!(match_rating.encode("Smith"), "SMTH");
/// // This is a match
/// assert!(match_rating.is_encoded_equals("Franciszek", "Frances"));
/// // This does not match
/// assert!(!match_rating.is_encoded_equals("Karl", "Alessandro"));
/// ```
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct MatchRatingApproach;

impl MatchRatingApproach {
    fn clean_name(value: &str) -> String {
        let result = value.to_uppercase();

        let result = result
            .chars()
            .filter(|c| !CHAR_TO_TRIM.contains(c))
            .filter(|c| !c.is_whitespace())
            .collect();

        MatchRatingApproach::remove_accent(result)
    }

    fn remove_accent(value: String) -> String {
        value
            .chars()
            .map(|c| {
                let position = UNICODE.iter().position(|ch| ch == &c);
                match position {
                    Some(index) => PLAIN_ASCII[index],
                    None => c,
                }
            })
            .collect()
    }

    fn remove_vowels(value: String) -> String {
        // I drop the Java "name = name.replaceAll("\\s{2,}\\b", SPACE);" of remove_vowels(...) because
        // clean name removes any whitespace.
        value
            .char_indices()
            .filter(|(index, ch)| index == &0 || !is_vowel(Some(ch.to_ascii_lowercase()), false))
            .filter(|(_, c)| !CHAR_TO_TRIM.contains(c) && !c.is_whitespace())
            .map(|(_, ch)| ch)
            .collect()
    }

    fn remove_double_consonants(value: String) -> String {
        let mut result = value.to_uppercase();

        for (double, replacement) in DOUBLE_CONSONANT {
            result = result.replace(double, replacement);
        }

        result
    }

    fn get_first3_last3(value: String) -> String {
        if value.len() > 6 {
            format!("{}{}", &value[0..3], &value[value.len() - 3..])
        } else {
            value
        }
    }

    fn get_minimum_rating(sum_length: usize) -> usize {
        match sum_length {
            0..=4 => 5,
            5..=7 => 4,
            8..=11 => 3,
            12 => 2,
            _ => 1,
        }
    }

    fn left_to_right_then_right_to_left_processing(name1: String, name2: String) -> usize {
        let mut n1: Vec<char> = name1.chars().collect();
        let mut n2: Vec<char> = name2.chars().collect();

        let n1len = n1.len() - 1;
        let n2len = n2.len() - 1;

        for i in 0..n1.len() {
            if i > n2len {
                break;
            }

            let c1: &char = n1.get(i).unwrap();
            let c2: &char = n2.get(i).unwrap();
            if c1 == c2 {
                n1[i] = ' ';
                n2[i] = ' ';
            }

            let c1: &char = n1.get(n1.len() - (i + 1)).unwrap();
            let c2: &char = n2.get(n2.len() - (i + 1)).unwrap();
            if c1 == c2 {
                n1[n1len - i] = ' ';
                n2[n2len - i] = ' ';
            }
        }

        let r1: String = n1.iter().filter(|c| c != &&' ').collect();
        let r2: String = n2.iter().filter(|c| c != &&' ').collect();

        if r1.len() > r2.len() {
            6usize.abs_diff(r1.len())
        } else {
            6usize.abs_diff(r2.len())
        }
    }
}

impl Encoder for MatchRatingApproach {
    fn encode(&self, value: &str) -> String {
        if value.trim().is_empty() || value.trim().len() == 1 {
            return String::new();
        }

        // We can do clean_name and remove_vowels in one pass, but I keep for the
        // moment the same as commons-codec.
        let value = MatchRatingApproach::clean_name(value);
        let value = MatchRatingApproach::remove_vowels(value);
        let value = MatchRatingApproach::remove_double_consonants(value);
        MatchRatingApproach::get_first3_last3(value)
    }

    fn is_encoded_equals(&self, first: &str, second: &str) -> bool {
        if first.trim().is_empty() || second.trim().is_empty() {
            return false;
        }

        if first.trim().len() == 1 || second.trim().len() == 1 {
            return false;
        }

        if first == second {
            return true;
        }

        let name1 = self.encode(first);
        let name2 = self.encode(second);

        if name1.len().abs_diff(name2.len()) >= 3 {
            return false;
        }

        let sum_length = name1.len() + name2.len();

        let min_rating = MatchRatingApproach::get_minimum_rating(sum_length);
        let count = MatchRatingApproach::left_to_right_then_right_to_left_processing(name1, name2);

        count >= min_rating
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accent_removal_all_lower_successfully_removed() {
        assert_eq!(
            MatchRatingApproach::remove_accent("áéíóú".to_string()),
            "aeiou".to_string()
        );
    }

    #[test]
    fn test_accent_removal_with_spaces_successfully_removed_and_spaces_invariant() {
        assert_eq!(
            MatchRatingApproach::remove_accent("áé íó  ú".to_string()),
            "ae io  u".to_string()
        );
    }

    #[test]
    fn test_accent_removal_upper_and_lower_successfully_removed_and_case_invariant() {
        assert_eq!(
            MatchRatingApproach::remove_accent("ÁeíÓuu".to_string()),
            "AeiOuu".to_string()
        );
    }

    #[test]
    fn test_accent_removal_mixed_with_unusual_chars_successfully_removed_and_unusual_characters_invariant(
    ) {
        assert_eq!(
            MatchRatingApproach::remove_accent("Á-e'í.,ó&ú".to_string()),
            "A-e'i.,o&u".to_string()
        );
    }

    #[test]
    fn test_accent_removal_ger_span_fren_mix_successfully_removed() {
        assert_eq!(
            MatchRatingApproach::remove_accent("äëöüßÄËÖÜñÑà".to_string()),
            "aeoußAEOUnNa".to_string()
        );
    }

    #[test]
    fn test_accent_removal_comprehensive_accent_mix_all_successfully_removed() {
        assert_eq!(
            MatchRatingApproach::remove_accent(
                "È,É,Ê,Ë,Û,Ù,Ï,Î,À,Â,Ô,è,é,ê,ë,û,ù,ï,î,à,â,ô,ç".to_string()
            ),
            "E,E,E,E,U,U,I,I,A,A,O,e,e,e,e,u,u,i,i,a,a,o,c".to_string()
        );
    }

    #[test]
    fn test_accent_removal_normal_string_no_change() {
        assert_eq!(
            MatchRatingApproach::remove_accent("Colorless green ideas sleep furiously".to_string()),
            "Colorless green ideas sleep furiously".to_string()
        );
    }

    #[test]
    fn test_accent_removal_nino_no_change() {
        assert_eq!(
            MatchRatingApproach::remove_accent("".to_string()),
            "".to_string()
        );
    }

    #[test]
    fn test_remove_single_double_consonants_buble_removed_successfully() {
        assert_eq!(
            MatchRatingApproach::remove_double_consonants("BUBBLE".to_string()),
            "BUBLE".to_string()
        );
    }

    #[test]
    fn test_remove_double_consonants_mississippi_removed_successfully() {
        assert_eq!(
            MatchRatingApproach::remove_double_consonants("MISSISSIPPI".to_string()),
            "MISISIPI".to_string()
        );
    }

    #[test]
    fn test_remove_double_double_vowel_beetle_not_removed() {
        assert_eq!(
            MatchRatingApproach::remove_double_consonants("BEETLE".to_string()),
            "BEETLE".to_string()
        );
    }

    #[test]
    fn test_remove_vowel_alessandra_returns_alssndr() {
        assert_eq!(
            MatchRatingApproach::remove_vowels("ALESSANDRA".to_string()),
            "ALSSNDR".to_string()
        );
    }

    #[test]
    fn test_remove_vowel_aidan_returns_adn() {
        assert_eq!(
            MatchRatingApproach::remove_vowels("AIDAN".to_string()),
            "ADN".to_string()
        );
    }

    #[test]
    fn test_remove_vowel_declan_returns_dcln() {
        assert_eq!(
            MatchRatingApproach::remove_vowels("DECLAN".to_string()),
            "DCLN".to_string()
        );
    }

    #[test]
    fn test_get_first3_last3_alexander_returns_aleder() {
        assert_eq!(
            MatchRatingApproach::get_first3_last3("Alexzander".to_string()),
            "Aleder".to_string()
        );
    }

    #[test]
    fn test_get_first3_last3_pete_returns_pete() {
        assert_eq!(
            MatchRatingApproach::get_first3_last3("PETE".to_string()),
            "PETE".to_string()
        );
    }

    #[test]
    fn test_left_to_right_then_right_to_left_alexander_alexandra_returns_4() {
        assert_eq!(
            MatchRatingApproach::left_to_right_then_right_to_left_processing(
                "ALEXANDER".to_string(),
                "ALEXANDRA".to_string()
            ),
            4
        );
    }

    #[test]
    fn test_left_to_right_then_right_to_left_einstein_michaela_returns_0() {
        assert_eq!(
            MatchRatingApproach::left_to_right_then_right_to_left_processing(
                "EINSTEIN".to_string(),
                "MICHAELA".to_string()
            ),
            0
        );
    }

    #[test]
    fn test_get_min_rating_7_return_4_successfully() {
        assert_eq!(MatchRatingApproach::get_minimum_rating(7), 4);
    }

    #[test]
    fn test_get_min_rating_1_returns_5_successfully() {
        assert_eq!(MatchRatingApproach::get_minimum_rating(1), 5);
    }

    #[test]
    fn test_get_min_rating_2_returns_5_successfully() {
        assert_eq!(MatchRatingApproach::get_minimum_rating(2), 5);
    }

    #[test]
    fn test_get_min_rating_5_returns_4_successfully() {
        assert_eq!(MatchRatingApproach::get_minimum_rating(5), 4);
    }

    #[test]
    fn test_get_min_rating_6_returns_4_successfully() {
        assert_eq!(MatchRatingApproach::get_minimum_rating(6), 4);
    }

    #[test]
    fn test_get_min_rating_7_returns_4_successfully() {
        assert_eq!(MatchRatingApproach::get_minimum_rating(7), 4);
    }

    #[test]
    fn test_get_min_rating_8_returns_3_successfully() {
        assert_eq!(MatchRatingApproach::get_minimum_rating(8), 3);
    }

    #[test]
    fn test_get_min_rating_10_returns_3_successfully() {
        assert_eq!(MatchRatingApproach::get_minimum_rating(10), 3);
    }

    #[test]
    fn test_get_min_rating_13_returns_1_successfully() {
        assert_eq!(MatchRatingApproach::get_minimum_rating(13), 1);
    }

    #[test]
    fn test_clean_name_successfully_clean() {
        assert_eq!(
            MatchRatingApproach::clean_name("This-ís   a t.,es &t"),
            "THISISATEST"
        );
    }

    #[test]
    fn test_is_encode_equals_corner_case_second_name_nothing_returns_false() {
        let encoder = MatchRatingApproach;
        assert!(!encoder.is_encoded_equals("test", ""));
    }

    #[test]
    fn test_is_encode_equals_corner_case_first_name_nothing_returns_false() {
        let encoder = MatchRatingApproach;
        assert!(!encoder.is_encoded_equals("", "test"));
    }

    #[test]
    fn test_is_encode_equals_corner_case_second_name_just_space_returns_false() {
        let encoder = MatchRatingApproach;
        assert!(!encoder.is_encoded_equals("test", " "));
    }

    #[test]
    fn test_is_encode_equals_corner_case_first_name_just_space_returns_false() {
        let encoder = MatchRatingApproach;
        assert!(!encoder.is_encoded_equals(" ", "test"));
    }

    #[test]
    fn test_is_encode_equals_corner_case_first_name_just_1_letter_returns_false() {
        let encoder = MatchRatingApproach;
        assert!(!encoder.is_encoded_equals("t", "test"));
    }

    #[test]
    fn test_is_encode_equals_second_name_just_1_letter_returns_false() {
        let encoder = MatchRatingApproach;
        assert!(!encoder.is_encoded_equals("test", "t"));
    }

    #[test]
    fn test_get_encoding_harper_hrpr() {
        let encoder = MatchRatingApproach;
        assert_eq!(encoder.encode("HARPER"), "HRPR");
    }

    #[test]
    fn test_get_encoding_smith_to_smth() {
        let encoder = MatchRatingApproach;
        assert_eq!(encoder.encode("Smith"), "SMTH");
    }

    #[test]
    fn test_get_encoding_smyth_to_smyth() {
        let encoder = MatchRatingApproach;
        assert_eq!(encoder.encode("Smyth"), "SMYTH");
    }

    #[test]
    fn test_get_encoding_space_to_nothing() {
        let encoder = MatchRatingApproach;
        assert_eq!(encoder.encode(" "), "");
    }

    #[test]
    fn test_get_encoding_no_space_to_nothing() {
        let encoder = MatchRatingApproach;
        assert_eq!(encoder.encode(""), "");
    }

    #[test]
    fn test_get_encoding_one_letter_to_nothing() {
        let encoder = MatchRatingApproach;
        assert_eq!(encoder.encode("E"), "");
    }

    #[test]
    fn test_compare_name_same_names_returns_false_successfully() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("John", "John"));
    }

    #[test]
    fn test_compare_smith_smyth_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("smith", "smyth"));
    }

    #[test]
    fn test_compare_burns_bourne_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Burns", "Bourne"));
    }

    #[test]
    fn test_compare_short_names_al_ed_works_but_no_match() {
        let encoder = MatchRatingApproach;
        assert!(!encoder.is_encoded_equals("Al", "Ed"));
    }

    #[test]
    fn test_compare_catherine_kathryn_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Catherine", "Kathryn"));
    }

    #[test]
    fn test_compare_brian_bryan_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Brian", "Bryan"));
    }

    #[test]
    fn test_compare_sean_shaun_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Séan", "Shaun"));
    }

    #[test]
    fn test_compare_colm_colin_with_accents_and_symbols_and_spaces_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Cólm", "C-olín"));
    }

    #[test]
    fn test_compare_stephen_steven_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Stephen", "Steven"));
    }

    #[test]
    fn test_compare_steven_stefan_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Steven", "Stefan"));
    }

    #[test]
    fn test_compare_stephen_stefan_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Stephen", "Stefan"));
    }

    #[test]
    fn test_compare_sam_samuel_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Sam", "Samuel"));
    }

    #[test]
    fn test_compare_micky_michael_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Micky", "Michael"));
    }

    #[test]
    fn test_compare_oona_oonagh_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Oona", "Oonagh"));
    }

    #[test]
    fn test_compare_sophie_sofia_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Sophie", "Sofia"));
    }

    #[test]
    fn test_compare_franciszek_frances_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Franciszek", "Frances"));
    }

    #[test]
    fn test_compare_tomasz_tom_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Tomasz", "tom"));
    }

    #[test]
    fn test_compare_small_input_cark_kl_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Kl", "Karl"));
    }

    #[test]
    fn test_compare_name_to_single_letter_karl_c_does_not_match() {
        let encoder = MatchRatingApproach;
        assert!(!encoder.is_encoded_equals("Karl", "C"));
    }

    #[test]
    fn test_compare_zach_zakaria_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Zach", "Zacharia"));
    }

    #[test]
    fn test_compare_karl_alessandro_does_not_match() {
        let encoder = MatchRatingApproach;
        assert!(!encoder.is_encoded_equals("Karl", "Alessandro"));
    }

    #[test]
    fn test_compare_forenames_una_oonagh_should_successfully_match_but_does_not() {
        let encoder = MatchRatingApproach;
        assert!(!encoder.is_encoded_equals("Úna", "Oonagh"));
    }

    #[test]
    fn test_compare_surname_osullivan_osuilleabhain_successful_match() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("O'Sullivan", "Ó ' Súilleabháin"));
    }

    #[test]
    fn test_compare_long_surnames_moriarty_omuircheartaigh_does_not_successful_match() {
        let encoder = MatchRatingApproach;
        assert!(!encoder.is_encoded_equals("Moriarty", "OMuircheartaigh"));
    }

    #[test]
    fn test_compare_long_surnames_omuircheartaigh_omireadhaigh_successful_match() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("o'muireadhaigh", "Ó 'Muircheartaigh "));
    }

    #[test]
    fn test_compare_surname_cooperflynn_superlyn_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Cooper-Flynn", "Super-Lyn"));
    }

    #[test]
    fn test_compare_surname_hailey_halley_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Hailey", "Halley"));
    }

    #[test]
    fn test_compare_surname_auerbach_uhrbach_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Auerbach", "Uhrbach"));
    }

    #[test]
    fn test_compare_surname_moskowitz_moskovitz_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Moskowitz", "Moskovitz"));
    }

    #[test]
    fn test_compare_surname_lipshitz_lippszyc_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("LIPSHITZ", "LIPPSZYC"));
    }

    #[test]
    fn test_compare_surname_lewinsky_levinski_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("LEWINSKY", "LEVINSKI"));
    }

    #[test]
    fn test_compare_surname_szlamawicz_shlamovitz_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("SZLAMAWICZ", "SHLAMOVITZ"));
    }

    #[test]
    fn test_compare_surname_rosochowaciec_rosokhovatsets_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("R o s o ch o w a c ie c", " R o s o k ho v a ts e ts"));
    }

    #[test]
    fn test_compare_surname_przemysl_pshemeshil_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals(" P rz e m y s l", " P sh e m e sh i l"));
    }

    #[test]
    fn test_compare_peterson_peters_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Peterson", "Peters"));
    }

    #[test]
    fn test_compare_mcgowan_mcgeoghegan_successfully_matched() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("McGowan", "Mc Geoghegan"));
    }

    #[test]
    fn test_compare_surnames_corner_case_murphy_space_no_match() {
        let encoder = MatchRatingApproach;
        assert!(!encoder.is_encoded_equals("Murphy", " "));
    }

    #[test]
    fn test_compare_surnames_corner_case_murphy_no_space_no_match() {
        let encoder = MatchRatingApproach;
        assert!(!encoder.is_encoded_equals("Murphy", ""));
    }

    #[test]
    fn test_compare_surnames_murphy_lynch_no_match_expected() {
        let encoder = MatchRatingApproach;
        assert!(!encoder.is_encoded_equals("Murphy", "Lynch"));
    }

    #[test]
    fn test_compare_forenames_sean_john_match_expected() {
        let encoder = MatchRatingApproach;
        assert!(encoder.is_encoded_equals("Sean", "John"));
    }

    #[test]
    fn test_compare_forenames_sean_pete_no_match_expected() {
        let encoder = MatchRatingApproach;
        assert!(!encoder.is_encoded_equals("Sean", "Pete"));
    }
}
