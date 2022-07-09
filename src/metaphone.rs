use serde::{Deserialize, Serialize};

use crate::helper::is_vowel;
use crate::Encoder;

const FRONTV: &str = "EIY";
const VARSON: &str = "CSPTG";

/// This the [Metaphone] implementation of [Encoder].
///
/// It takes a maximum code length for the `new` constructor and has
/// a [Default] implementation with a maximum code length of 4.
///
/// # Example
///
/// ```rust
/// use rphonetic::{Encoder, Metaphone};
/// let metaphone = Metaphone::default();
///
/// assert_eq!(metaphone.encode("Joanne"), "JN");
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Metaphone {
    max_code_length: usize,
}

impl Metaphone {
    /// Construct a new [Metaphone] with the maximum code length provided.
    ///
    /// # Parameter
    ///
    /// * `max_code_length : the maximum code length.
    pub fn new(max_code_length: usize) -> Self {
        Self { max_code_length }
    }

    fn is_vowel(text: &str, index: usize) -> bool {
        let ch = text.chars().nth(index).map(|c| c.to_ascii_lowercase());
        is_vowel(ch, false)
    }

    fn is_previous_char(text: &str, index: usize, ch: char) -> bool {
        index > 0 && text.chars().nth(index - 1) == Some(ch)
    }

    fn is_next_char(text: &str, index: usize, ch: char) -> bool {
        text.chars().nth(index + 1) == Some(ch)
    }

    fn region_match(text: &str, index: usize, test: &str) -> bool {
        index + test.len() - 1 < text.len() && text[index..].contains(test)
    }

    fn is_last_char(wdsz: usize, n: usize) -> bool {
        n + 1 == wdsz
    }
}

/// [Default] implementation with a `max_code_length`of 4.
impl Default for Metaphone {
    fn default() -> Self {
        Self { max_code_length: 4 }
    }
}

impl Encoder for Metaphone {
    fn encode(&self, value: &str) -> String {
        let inwd = value.to_uppercase();

        if inwd.len() == 1 {
            return inwd;
        }

        let mut local = String::with_capacity(40);
        let mut code = String::with_capacity(10);

        let mut iterator = inwd.chars().peekable();
        match iterator.next() {
            Some('K' | 'G' | 'P') => {
                if iterator.peek() == Some(&'N') {
                    local.push_str(&inwd[1..]);
                } else {
                    local.push_str(&inwd);
                }
            }
            Some('A') => {
                if iterator.peek() == Some(&'E') {
                    local.push_str(&inwd[1..]);
                } else {
                    local.push_str(&inwd);
                }
            }
            Some('W') => match iterator.peek() {
                Some('R') => local.push_str(&inwd[1..]),
                Some('H') => {
                    local.push('W');
                    local.push_str(&inwd[2..]);
                }
                _ => local.push_str(&inwd),
            },
            Some('X') => {
                local.push('S');
                local.push_str(&inwd[1..]);
            }
            _ => local.push_str(&inwd),
        }

        let wdsz = local.len();

        let mut skip = 0;
        for (index, symb) in local.chars().enumerate() {
            if skip == 0 {
                if code.len() == self.max_code_length {
                    break;
                }
                if symb == 'C' || !Metaphone::is_previous_char(&local, index, symb) {
                    match symb {
                        'A' | 'E' | 'I' | 'O' | 'U' => {
                            if index == 0 {
                                code.push(symb);
                            }
                        }
                        'B' => {
                            if !Metaphone::is_previous_char(&local, index, 'M')
                                || !Metaphone::is_last_char(wdsz, index)
                            {
                                code.push(symb);
                            }
                        }
                        'C' => {
                            let next = local.chars().nth(index + 1);
                            if Metaphone::is_previous_char(&local, index, 'S')
                                && !Metaphone::is_last_char(wdsz, index)
                                && next.is_some()
                                && FRONTV.contains(next.unwrap())
                            {
                                // Doing nothing
                            } else if Metaphone::region_match(&local, index, "CIA") {
                                code.push('X');
                            } else if !Metaphone::is_last_char(wdsz, index)
                                && next.is_some()
                                && FRONTV.contains(next.unwrap())
                            {
                                code.push('S');
                            } else if Metaphone::is_previous_char(&local, index, 'S')
                                && Metaphone::is_next_char(&local, index, 'H')
                            {
                                code.push('K');
                            } else if Metaphone::is_next_char(&local, index, 'H') {
                                if index == 0 && wdsz > 3 && Metaphone::is_vowel(&local, 2) {
                                    code.push('K');
                                } else {
                                    code.push('X');
                                }
                            } else {
                                code.push('K');
                            }
                        }
                        'D' => {
                            if !Metaphone::is_last_char(wdsz, index + 1)
                                && Metaphone::is_next_char(&local, index, 'G')
                                && FRONTV.contains(local.chars().nth(index + 2).unwrap())
                            {
                                code.push('J');
                                skip = 2;
                            } else {
                                code.push('T');
                            }
                        }
                        'G' => {
                            if (Metaphone::is_last_char(wdsz, index + 1)
                                && Metaphone::is_next_char(&local, index, 'H'))
                                || (!Metaphone::is_last_char(wdsz, index + 1)
                                    && Metaphone::is_next_char(&local, index, 'H')
                                    && !Metaphone::is_vowel(&local, index + 2))
                                || (index > 0
                                    && (Metaphone::region_match(&local, index, "GN")
                                        || Metaphone::region_match(&local, index, "GNED")))
                            {
                                // Doing nothing
                            } else {
                                let hard = Metaphone::is_previous_char(&local, index, 'G');
                                if !Metaphone::is_last_char(wdsz, index)
                                    && FRONTV.contains(local.chars().nth(index + 1).unwrap())
                                    && !hard
                                {
                                    code.push('J');
                                } else {
                                    code.push('K');
                                }
                            }
                        }
                        'H' => {
                            if Metaphone::is_last_char(wdsz, index)
                                || (index > 0
                                    && VARSON.contains(local.chars().nth(index - 1).unwrap()))
                            {
                                // Doing nothing
                            } else if Metaphone::is_vowel(&local, index + 1) {
                                code.push('H');
                            }
                        }
                        'F' | 'J' | 'L' | 'M' | 'N' | 'R' => code.push(symb),
                        'K' => {
                            if index == 0 || !Metaphone::is_previous_char(&local, index, 'C') {
                                code.push(symb);
                            }
                        }
                        'P' => {
                            if Metaphone::is_next_char(&local, index, 'H') {
                                code.push('F');
                            } else {
                                code.push(symb);
                            }
                        }
                        'Q' => code.push('K'),
                        'S' => {
                            if Metaphone::region_match(&local, index, "SH")
                                || Metaphone::region_match(&local, index, "SIO")
                                || Metaphone::region_match(&local, index, "SIA")
                            {
                                code.push('X');
                            } else {
                                code.push('S');
                            }
                        }
                        'T' => {
                            if Metaphone::region_match(&local, index, "TIA")
                                || Metaphone::region_match(&local, index, "TIO")
                            {
                                code.push('X');
                            } else if Metaphone::region_match(&local, index, "TCH") {
                                // Doing nothing
                            } else if Metaphone::region_match(&local, index, "TH") {
                                code.push('0');
                            } else {
                                code.push('T');
                            }
                        }
                        'V' => code.push('F'),
                        'W' | 'Y' => {
                            if !Metaphone::is_last_char(wdsz, index)
                                && Metaphone::is_vowel(&local, index + 1)
                            {
                                code.push(symb);
                            }
                        }
                        'X' => {
                            code.push('K');
                            code.push('S');
                        }
                        'Z' => code.push('S'),
                        _ => {
                            // Doing nothing
                        }
                    }
                }
                if code.len() > self.max_code_length {
                    code = code[..self.max_code_length].to_string();
                }
            } else {
                skip -= 1;
            }
        }

        code
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_metaphone_equal1() {
        let metaphone = Metaphone::default();
        let data: Vec<(&str, &str)> = vec![
            ("Case", "case"),
            ("CASE", "Case"),
            ("caSe", "cAsE"),
            ("quick", "cookie"),
        ];

        for (v1, v2) in data.iter() {
            assert!(
                metaphone.is_encoded_equals(v1, v2),
                "{} should be equals to {}",
                v1,
                v2
            );
        }
    }

    #[test]
    fn test_is_metaphone_equal2() {
        let metaphone = Metaphone::default();
        let data: Vec<(&str, &str)> = vec![("Lawrence", "Lorenza"), ("Gary", "Cahra")];

        for (v1, v2) in data.iter() {
            assert!(
                metaphone.is_encoded_equals(v1, v2),
                "{} should be equals to {}",
                v1,
                v2
            );
        }
    }

    #[test]
    fn test_is_metaphone_equal_aero() {
        let metaphone = Metaphone::default();

        let v1 = "Aero";
        let data: Vec<&str> = vec!["Eure"];
        for v2 in data.iter() {
            assert!(
                metaphone.is_encoded_equals(v1, v2),
                "{} should be equals to {}",
                v1,
                v2
            );
        }
    }

    #[test]
    fn test_is_metaphone_equal_white() {
        let metaphone = Metaphone::default();

        let v1 = "White";
        let data: Vec<&str> = vec![
            "Wade", "Wait", "Waite", "Wat", "Whit", "Wiatt", "Wit", "Wittie", "Witty", "Wood",
            "Woodie", "Woody",
        ];
        for v2 in data.iter() {
            assert!(
                metaphone.is_encoded_equals(v1, v2),
                "{} should be equals to {}",
                v1,
                v2
            );
        }
    }

    #[test]
    fn test_is_metaphone_equal_albert() {
        let metaphone = Metaphone::default();

        let v1 = "Albert";
        let data: Vec<&str> = vec!["Ailbert", "Alberik", "Albert", "Alberto", "Albrecht"];
        for v2 in data.iter() {
            assert!(
                metaphone.is_encoded_equals(v1, v2),
                "{} should be equals to {}",
                v1,
                v2
            );
        }
    }

    #[test]
    fn test_is_metaphone_equal_gary() {
        let metaphone = Metaphone::default();

        let v1 = "Gary";
        let data: Vec<&str> = vec![
            "Cahra", "Cara", "Carey", "Cari", "Caria", "Carie", "Caro", "Carree", "Carri",
            "Carrie", "Carry", "Cary", "Cora", "Corey", "Cori", "Corie", "Correy", "Corri",
            "Corrie", "Corry", "Cory", "Gray", "Kara", "Kare", "Karee", "Kari", "Karia", "Karie",
            "Karrah", "Karrie", "Karry", "Kary", "Keri", "Kerri", "Kerrie", "Kerry", "Kira",
            "Kiri", "Kora", "Kore", "Kori", "Korie", "Korrie", "Korry",
        ];
        for v2 in data.iter() {
            assert!(
                metaphone.is_encoded_equals(v1, v2),
                "{} should be equals to {}",
                v1,
                v2
            );
        }
    }

    #[test]
    fn test_is_metaphone_equal_john() {
        let metaphone = Metaphone::default();

        let v1 = "John";
        let data: Vec<&str> = vec![
            "Gena", "Gene", "Genia", "Genna", "Genni", "Gennie", "Genny", "Giana", "Gianna",
            "Gina", "Ginni", "Ginnie", "Ginny", "Jaine", "Jan", "Jana", "Jane", "Janey", "Jania",
            "Janie", "Janna", "Jany", "Jayne", "Jean", "Jeana", "Jeane", "Jeanie", "Jeanna",
            "Jeanne", "Jeannie", "Jen", "Jena", "Jeni", "Jenn", "Jenna", "Jennee", "Jenni",
            "Jennie", "Jenny", "Jinny", "Jo Ann", "Jo-Ann", "Jo-Anne", "Joan", "Joana", "Joane",
            "Joanie", "Joann", "Joanna", "Joanne", "Joeann", "Johna", "Johnna", "Joni", "Jonie",
            "Juana", "June", "Junia", "Junie",
        ];
        for v2 in data.iter() {
            assert!(
                metaphone.is_encoded_equals(v1, v2),
                "{} should be equals to {}",
                v1,
                v2
            );
        }
    }

    #[test]
    fn test_is_metaphone_equal_knight() {
        let metaphone = Metaphone::default();

        let v1 = "Knight";
        let data: Vec<&str> = vec![
            "Hynda", "Nada", "Nadia", "Nady", "Nat", "Nata", "Natty", "Neda", "Nedda", "Nedi",
            "Netta", "Netti", "Nettie", "Netty", "Nita", "Nydia",
        ];
        for v2 in data.iter() {
            assert!(
                metaphone.is_encoded_equals(v1, v2),
                "{} should be equals to {}",
                v1,
                v2
            );
        }
    }

    #[test]
    fn test_is_metaphone_equal_mary() {
        let metaphone = Metaphone::default();

        let v1 = "Mary";
        let data: Vec<&str> = vec![
            "Mair", "Maire", "Mara", "Mareah", "Mari", "Maria", "Marie", "Mary", "Maura", "Maure",
            "Meara", "Merrie", "Merry", "Mira", "Moira", "Mora", "Moria", "Moyra", "Muire", "Myra",
            "Myrah",
        ];
        for v2 in data.iter() {
            assert!(
                metaphone.is_encoded_equals(v1, v2),
                "{} should be equals to {}",
                v1,
                v2
            );
        }
    }

    #[test]
    fn test_is_metaphone_equal_paris() {
        let metaphone = Metaphone::default();

        let v1 = "Paris";
        let data: Vec<&str> = vec!["Pearcy", "Perris", "Piercy", "Pierz", "Pryse"];
        for v2 in data.iter() {
            assert!(
                metaphone.is_encoded_equals(v1, v2),
                "{} should be equals to {}",
                v1,
                v2
            );
        }
    }

    #[test]
    fn test_is_metaphone_equal_peter() {
        let metaphone = Metaphone::default();

        let v1 = "Peter";
        let data: Vec<&str> = vec![
            "Peadar", "Peder", "Pedro", "Peter", "Petr", "Peyter", "Pieter", "Pietro", "Piotr",
        ];
        for v2 in data.iter() {
            assert!(
                metaphone.is_encoded_equals(v1, v2),
                "{} should be equals to {}",
                v1,
                v2
            );
        }
    }

    #[test]
    fn test_is_metaphone_equal_ray() {
        let metaphone = Metaphone::default();

        let v1 = "Ray";
        let data: Vec<&str> = vec!["Ray", "Rey", "Roi", "Roy", "Ruy"];
        for v2 in data.iter() {
            assert!(
                metaphone.is_encoded_equals(v1, v2),
                "{} should be equals to {}",
                v1,
                v2
            );
        }
    }

    #[test]
    fn test_is_metaphone_equal_susan() {
        let metaphone = Metaphone::default();

        let v1 = "Susan";
        let data: Vec<&str> = vec![
            "Siusan", "Sosanna", "Susan", "Susana", "Susann", "Susanna", "Susannah", "Susanne",
            "Suzann", "Suzanna", "Suzanne", "Zuzana",
        ];
        for v2 in data.iter() {
            assert!(
                metaphone.is_encoded_equals(v1, v2),
                "{} should be equals to {}",
                v1,
                v2
            );
        }
    }

    #[test]
    fn test_is_metaphone_equal_wright() {
        let metaphone = Metaphone::default();

        let v1 = "Wright";
        let data: Vec<&str> = vec!["Rota", "Rudd", "Ryde"];
        for v2 in data.iter() {
            assert!(
                metaphone.is_encoded_equals(v1, v2),
                "{} should be equals to {}",
                v1,
                v2
            );
        }
    }

    #[test]
    fn test_is_metaphone_equal_xalan() {
        let metaphone = Metaphone::default();

        let v1 = "Xalan";
        let data: Vec<&str> = vec![
            "Celene", "Celina", "Celine", "Selena", "Selene", "Selina", "Seline", "Suellen",
            "Xylina",
        ];
        for v2 in data.iter() {
            assert!(
                metaphone.is_encoded_equals(v1, v2),
                "{} should be equals to {}",
                v1,
                v2
            );
        }
    }

    #[test]
    fn test_metaphone() {
        let metaphone = Metaphone::default();

        assert_eq!(metaphone.encode("howl"), "HL");
        assert_eq!(metaphone.encode("testing"), "TSTN");
        assert_eq!(metaphone.encode("The"), "0");
        assert_eq!(metaphone.encode("quick"), "KK");
        assert_eq!(metaphone.encode("brown"), "BRN");
        assert_eq!(metaphone.encode("fox"), "FKS");
        assert_eq!(metaphone.encode("jumped"), "JMPT");
        assert_eq!(metaphone.encode("over"), "OFR");
        assert_eq!(metaphone.encode("the"), "0");
        assert_eq!(metaphone.encode("lazy"), "LS");
        assert_eq!(metaphone.encode("dogs"), "TKS");
    }

    #[test]
    fn test_word_ending_in_mb() {
        let metaphone = Metaphone::default();

        assert_eq!(metaphone.encode("COMB"), "KM");
        assert_eq!(metaphone.encode("TOMB"), "TM");
        assert_eq!(metaphone.encode("WOMB"), "WM");
    }

    #[test]
    fn test_discard_of_sce_or_sci_or_scy() {
        let metaphone = Metaphone::default();

        assert_eq!(metaphone.encode("SCIENCE"), "SNS");
        assert_eq!(metaphone.encode("SCENE"), "SN");
        assert_eq!(metaphone.encode("SCY"), "S");
    }

    #[test]
    fn test_why() {
        let metaphone = Metaphone::default();

        assert_eq!(metaphone.encode("WHY"), "");
    }

    #[test]
    fn test_words_with_cia() {
        let metaphone = Metaphone::default();

        assert_eq!(metaphone.encode("CIAPO"), "XP");
    }

    #[test]
    fn test_translate_of_sch_and_ch() {
        let metaphone = Metaphone::default();

        assert_eq!(metaphone.encode("SCHEDULE"), "SKTL");
        assert_eq!(metaphone.encode("SCHEMATIC"), "SKMT");
        assert_eq!(metaphone.encode("CHARACTER"), "KRKT");
        assert_eq!(metaphone.encode("TEACH"), "TX");
    }

    #[test]
    fn test_translate_to_j_of_dge_or_dgi_or_dgy() {
        let metaphone = Metaphone::default();

        assert_eq!(metaphone.encode("DODGY"), "TJ");
        assert_eq!(metaphone.encode("DODGE"), "TJ");
        assert_eq!(metaphone.encode("ADGIEMTI"), "AJMT");
    }

    #[test]
    fn test_discard_of_silent_h_after_g() {
        let metaphone = Metaphone::default();

        assert_eq!(metaphone.encode("GHENT"), "KNT");
        assert_eq!(metaphone.encode("BAUGH"), "B");
    }

    #[test]
    fn test_discard_of_silent_gn() {
        let metaphone = Metaphone::default();

        assert_eq!(metaphone.encode("GNU"), "N");
        assert_eq!(metaphone.encode("SIGNED"), "SNT");
    }

    #[test]
    fn test_ph_to_f() {
        let metaphone = Metaphone::default();

        assert_eq!(metaphone.encode("PHISH"), "FX");
    }

    #[test]
    fn test_sh_and_sio_and_sia_to_x() {
        let metaphone = Metaphone::default();

        assert_eq!(metaphone.encode("SHOT"), "XT");
        assert_eq!(metaphone.encode("ODSIAN"), "OTXN");
        assert_eq!(metaphone.encode("PULSION"), "PLXN");
    }

    #[test]
    fn test_tio_and_tia_to_x() {
        let metaphone = Metaphone::default();

        assert_eq!(metaphone.encode("OTIA"), "OX");
        assert_eq!(metaphone.encode("PORTION"), "PRXN");
    }

    #[test]
    fn test_tch() {
        let metaphone = Metaphone::default();

        assert_eq!(metaphone.encode("RETCH"), "RX");
        assert_eq!(metaphone.encode("WATCH"), "WX");
    }

    #[test]
    fn test_exceed_length() {
        let metaphone = Metaphone::default();

        assert_eq!(metaphone.encode("AXEAXE"), "AKSK");
    }

    #[test]
    fn test_set_max_length_with_truncation() {
        let metaphone = Metaphone::new(6);

        assert_eq!(metaphone.encode("AXEAXEAXE"), "AKSKSK");
    }
}
