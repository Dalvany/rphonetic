use regex::Regex;

use crate::Encoder;

const SIX_1: &str = "111111";
const TEN_1: &str = "1111111111";

pub struct Caverphone1 {
    non_letter: Regex,
    end_mb: Regex,
    start_vowel: Regex,
    vowel: Regex,
    s: Regex,
    t: Regex,
    p: Regex,
    k: Regex,
    f: Regex,
    m: Regex,
    n: Regex,
}

impl Caverphone1 {
    pub fn new() -> Self {
        // unwrap should work otherwise unit tests will fail.
        let non_letter = Regex::new("[^a-z]").unwrap();
        let end_mb = Regex::new("mb$").unwrap();
        let start_vowel = Regex::new("^[aeiou]").unwrap();
        let vowel = Regex::new("[aeiou]").unwrap();
        let s = Regex::new("s+").unwrap();
        let t = Regex::new("t+").unwrap();
        let p = Regex::new("p+").unwrap();
        let k = Regex::new("k+").unwrap();
        let f = Regex::new("f+").unwrap();
        let m = Regex::new("m+").unwrap();
        let n = Regex::new("n+").unwrap();

        Self {
            non_letter,
            end_mb,
            start_vowel,
            vowel,
            s,
            t,
            p,
            k,
            f,
            m,
            n,
        }
    }
}

impl Encoder for Caverphone1 {
    fn encode(&self, s: &str) -> String {
        if s.is_empty() {
            return SIX_1.to_string();
        }

        let txt = s.to_lowercase();

        let txt = self.non_letter.replace_all(&*txt, "").to_string();

        // Avoid regex since it's quite simple
        let txt = if txt.starts_with("cough") {
            txt.replacen("cough", "cou2f", 1)
        } else {
            txt
        };
        let txt = if txt.starts_with("rough") {
            txt.replacen("rough", "rou2f", 1)
        } else {
            txt
        };
        let txt = if txt.starts_with("tough") {
            txt.replacen("tough", "tou2f", 1)
        } else {
            txt
        };
        let txt = if txt.starts_with("enough") {
            txt.replacen("enough", "enou2f", 1)
        } else {
            txt
        };
        let txt = if txt.starts_with("gn") {
            txt.replacen("gn", "2n", 1)
        } else {
            txt
        };

        let txt = self.end_mb.replace_all(&*txt, "m2");

        let txt = txt.replace("cq", "2q");
        let txt = txt.replace("ci", "si");
        let txt = txt.replace("ce", "se");
        let txt = txt.replace("cy", "sy");
        let txt = txt.replace("tch", "2ch");
        let txt = txt.replace("c", "k");
        let txt = txt.replace("q", "k");
        let txt = txt.replace("x", "k");
        let txt = txt.replace("v", "f");
        let txt = txt.replace("dg", "2g");
        let txt = txt.replace("tio", "sio");
        let txt = txt.replace("tia", "sia");
        let txt = txt.replace("d", "t");
        let txt = txt.replace("ph", "fh");
        let txt = txt.replace("b", "p");
        let txt = txt.replace("sh", "s2");
        let txt = txt.replace("z", "s");
        let txt = self.start_vowel.replace_all(&*txt, "A");

        let txt = self.vowel.replace_all(&*txt, "3");
        let txt = txt.replace("3gh3", "3kh3");
        let txt = txt.replace("gh", "22");
        let txt = txt.replace("g", "k");
        let txt = self.s.replace_all(&*txt, "S");
        let txt = self.t.replace_all(&*txt, "T");
        let txt = self.p.replace_all(&*txt, "P");
        let txt = self.k.replace_all(&*txt, "K");
        let txt = self.f.replace_all(&*txt, "F");
        let txt = self.m.replace_all(&*txt, "M");
        let txt = self.n.replace_all(&*txt, "N");
        let txt = txt.replace("w3", "W3");
        let txt = txt.replace("wy", "Wy");
        let txt = txt.replace("wh3", "Wh3");
        let txt = txt.replace("why", "Why");
        let txt = txt.replace("w", "2");
        let txt = if txt.starts_with("h") {
            txt.replacen("h", "A", 1)
        } else {
            txt
        };
        let txt = txt.replace("h", "2");
        let txt = txt.replace("r3", "R3");
        let txt = txt.replace("ry", "Ry");
        let txt = txt.replace("r", "2");
        let txt = txt.replace("l3", "L3");
        let txt = txt.replace("ly", "Ly");
        let txt = txt.replace("l", "2");
        let txt = txt.replace("j", "y");
        let txt = txt.replace("y3", "Y3");
        let txt = txt.replace("y", "2");

        let txt = txt.replace("2", "");
        let txt = txt.replace("3", "");

        let txt = txt + &*SIX_1;

        return txt[0..SIX_1.len()].to_string();
    }
}

pub struct Caverphone2 {
    non_letter: Regex,
    end_e: Regex,
    end_mb: Regex,
    start_vowel: Regex,
    vowel: Regex,
    s: Regex,
    t: Regex,
    p: Regex,
    k: Regex,
    f: Regex,
    m: Regex,
    n: Regex,
    end_w: Regex,
    end_r: Regex,
    end_l: Regex,
    end_3: Regex,
}

impl Caverphone2 {
    pub fn new() -> Self {
        // unwrap should work otherwise unit tests will fail.
        let non_letter = Regex::new("[^a-z]").unwrap();
        let end_e = Regex::new("e$").unwrap();
        let end_mb = Regex::new("mb$").unwrap();
        let start_vowel = Regex::new("^[aeiou]").unwrap();
        let vowel = Regex::new("[aeiou]").unwrap();
        let s = Regex::new("s+").unwrap();
        let t = Regex::new("t+").unwrap();
        let p = Regex::new("p+").unwrap();
        let k = Regex::new("k+").unwrap();
        let f = Regex::new("f+").unwrap();
        let m = Regex::new("m+").unwrap();
        let n = Regex::new("n+").unwrap();
        let end_w = Regex::new("w$").unwrap();
        let end_r = Regex::new("r$").unwrap();
        let end_l = Regex::new("l$").unwrap();
        let end_3 = Regex::new("3$").unwrap();

        Self {
            non_letter,
            end_e,
            end_mb,
            start_vowel,
            vowel,
            s,
            t,
            p,
            k,
            f,
            m,
            n,
            end_w,
            end_r,
            end_l,
            end_3,
        }
    }
}

impl Encoder for Caverphone2 {
    fn encode(&self, s: &str) -> String {
        if s.is_empty() {
            return TEN_1.to_string();
        }

        let txt = s.to_lowercase();

        let txt = self.non_letter.replace_all(&*txt, "");

        let txt = self.end_e.replace_all(&*txt, "").to_string();

        // Avoid regex since it's quite simple
        let txt = if txt.starts_with("cough") {
            txt.replacen("cough", "cou2f", 1)
        } else {
            txt
        };
        let txt = if txt.starts_with("rough") {
            txt.replacen("rough", "rou2f", 1)
        } else {
            txt
        };
        let txt = if txt.starts_with("tough") {
            txt.replacen("tough", "tou2f", 1)
        } else {
            txt
        };
        let txt = if txt.starts_with("enough") {
            txt.replacen("enough", "enou2f", 1)
        } else {
            txt
        };
        let txt = if txt.starts_with("trough") {
            txt.replacen("trough", "trou2f", 1)
        } else {
            txt
        };
        let txt = if txt.starts_with("gn") {
            txt.replacen("gn", "2n", 1)
        } else {
            txt
        };

        let txt = self.end_mb.replace_all(&*txt, "m2");

        let txt = txt.replace("cq", "2q");
        let txt = txt.replace("ci", "si");
        let txt = txt.replace("ce", "se");
        let txt = txt.replace("cy", "sy");
        let txt = txt.replace("tch", "2ch");
        let txt = txt.replace("c", "k");
        let txt = txt.replace("q", "k");
        let txt = txt.replace("x", "k");
        let txt = txt.replace("v", "f");
        let txt = txt.replace("dg", "2g");
        let txt = txt.replace("tio", "sio");
        let txt = txt.replace("tia", "sia");
        let txt = txt.replace("d", "t");
        let txt = txt.replace("ph", "fh");
        let txt = txt.replace("b", "p");
        let txt = txt.replace("sh", "s2");
        let txt = txt.replace("z", "s");
        let txt = self.start_vowel.replace_all(&*txt, "A");

        let txt = self.vowel.replace_all(&*txt, "3");
        let txt = txt.replace("j", "y");
        let txt = if txt.starts_with("y3") {
            txt.replacen("y3", "Y3", 1)
        } else {
            txt
        };
        let txt = if txt.starts_with("y") {
            txt.replacen("y", "A", 1)
        } else {
            txt
        };
        let txt = txt.replace("y", "3");
        let txt = txt.replace("3gh3", "3kh3");
        let txt = txt.replace("gh", "22");
        let txt = txt.replace("g", "k");
        let txt = self.s.replace_all(&*txt, "S");
        let txt = self.t.replace_all(&*txt, "T");
        let txt = self.p.replace_all(&*txt, "P");
        let txt = self.k.replace_all(&*txt, "K");
        let txt = self.f.replace_all(&*txt, "F");
        let txt = self.m.replace_all(&*txt, "M");
        let txt = self.n.replace_all(&*txt, "N");
        let txt = txt.replace("w3", "W3");
        let txt = txt.replace("wh3", "Wh3");
        let txt = self.end_w.replace_all(&*txt, "3");
        let txt = txt.replace("w", "2");
        let txt = if txt.starts_with("h") {
            txt.replacen("h", "A", 1)
        } else {
            txt
        };
        let txt = txt.replace("h", "2");
        let txt = txt.replace("r3", "R3");
        let txt = self.end_r.replace_all(&*txt, "3");
        let txt = txt.replace("r", "2");
        let txt = txt.replace("l3", "L3");
        let txt = self.end_l.replace_all(&*txt, "3");
        let txt = txt.replace("l", "2");

        let txt = txt.replace("2", "");
        let txt = self.end_3.replace_all(&*txt, "A");
        let txt = txt.replace("3", "");

        let txt = txt + &*TEN_1;

        return txt[0..TEN_1.len()].to_string();
    }
}

#[cfg(test)]
mod tests {
    /// These tests are the same as commons-codec.
    use super::*;

    #[test]
    fn test_caverphone1_revisited_common_code_at1111() {
        let caverphone = Caverphone1::new();

        assert_eq!(caverphone.encode("add"), "AT1111");
        assert_eq!(caverphone.encode("aid"), "AT1111");
        assert_eq!(caverphone.encode("at"), "AT1111");
        assert_eq!(caverphone.encode("art"), "AT1111");
        assert_eq!(caverphone.encode("eat"), "AT1111");
        assert_eq!(caverphone.encode("earth"), "AT1111");
        assert_eq!(caverphone.encode("head"), "AT1111");
        assert_eq!(caverphone.encode("hit"), "AT1111");
        assert_eq!(caverphone.encode("hot"), "AT1111");
        assert_eq!(caverphone.encode("hold"), "AT1111");
        assert_eq!(caverphone.encode("hard"), "AT1111");
        assert_eq!(caverphone.encode("heart"), "AT1111");
        assert_eq!(caverphone.encode("it"), "AT1111");
        assert_eq!(caverphone.encode("out"), "AT1111");
        assert_eq!(caverphone.encode("old"), "AT1111");
    }

    #[test]
    fn test_end_mb_caverphone1() {
        let caverphone = Caverphone1::new();

        assert_eq!(caverphone.encode("mb"), "M11111");
        assert_eq!(caverphone.encode("mbmb"), "MPM111");
    }

    #[test]
    fn test_is_caverphone1_equals() {
        let caverphone = Caverphone1::new();

        assert_eq!(caverphone.is_encoded_equals("Peter", "Stevenson"), false);
        assert!(caverphone.is_encoded_equals("Peter", "Peady"));
    }

    #[test]
    fn test_specification_v1examples() {
        let caverphone = Caverphone1::new();

        assert_eq!(caverphone.encode("David"), "TFT111");
        assert_eq!(caverphone.encode("Whittle"), "WTL111");
    }

    #[test]
    fn test_wikipedia_examples() {
        let caverphone = Caverphone1::new();

        assert_eq!(caverphone.encode("Lee"), "L11111");
        assert_eq!(caverphone.encode("Thompson"), "TMPSN1");
    }

    #[test]
    fn test_caverphone_revisited_common_code_at11111111() {
        let caverphone = Caverphone2::new();

        assert_eq!(caverphone.encode("add"), "AT11111111");
        assert_eq!(caverphone.encode("aid"), "AT11111111");
        assert_eq!(caverphone.encode("at"), "AT11111111");
        assert_eq!(caverphone.encode("art"), "AT11111111");
        assert_eq!(caverphone.encode("eat"), "AT11111111");
        assert_eq!(caverphone.encode("earth"), "AT11111111");
        assert_eq!(caverphone.encode("head"), "AT11111111");
        assert_eq!(caverphone.encode("hit"), "AT11111111");
        assert_eq!(caverphone.encode("hot"), "AT11111111");
        assert_eq!(caverphone.encode("hold"), "AT11111111");
        assert_eq!(caverphone.encode("hard"), "AT11111111");
        assert_eq!(caverphone.encode("heart"), "AT11111111");
        assert_eq!(caverphone.encode("it"), "AT11111111");
        assert_eq!(caverphone.encode("out"), "AT11111111");
        assert_eq!(caverphone.encode("old"), "AT11111111");
    }

    #[test]
    fn test_caverphone_revisited_examples() {
        let caverphone = Caverphone2::new();

        assert_eq!(caverphone.encode("Stevenson"), "STFNSN1111");
        assert_eq!(caverphone.encode("Peter"), "PTA1111111");
    }

    #[test]
    fn test_caverphone_revisited_random_name_kln1111111() {
        let caverphone = Caverphone2::new();

        let names = vec![
            "Cailean",
            "Calan",
            "Calen",
            "Callahan",
            "Callan",
            "Callean",
            "Carleen",
            "Carlen",
            "Carlene",
            "Carlin",
            "Carline",
            "Carlyn",
            "Carlynn",
            "Carlynne",
            "Charlean",
            "Charleen",
            "Charlene",
            "Charline",
            "Cherlyn",
            "Chirlin",
            "Clein",
            "Cleon",
            "Cline",
            "Cohleen",
            "Colan",
            "Coleen",
            "Colene",
            "Colin",
            "Colleen",
            "Collen",
            "Collin",
            "Colline",
            "Colon",
            "Cullan",
            "Cullen",
            "Cullin",
            "Gaelan",
            "Galan",
            "Galen",
            "Garlan",
            "Garlen",
            "Gaulin",
            "Gayleen",
            "Gaylene",
            "Giliane",
            "Gillan",
            "Gillian",
            "Glen",
            "Glenn",
            "Glyn",
            "Glynn",
            "Gollin",
            "Gorlin",
            "Kalin",
            "Karlan",
            "Karleen",
            "Karlen",
            "Karlene",
            "Karlin",
            "Karlyn",
            "Kaylyn",
            "Keelin",
            "Kellen",
            "Kellene",
            "Kellyann",
            "Kellyn",
            "Khalin",
            "Kilan",
            "Kilian",
            "Killen",
            "Killian",
            "Killion",
            "Klein",
            "Kleon",
            "Kline",
            "Koerlin",
            "Kylen",
            "Kylynn",
            "Quillan",
            "Quillon",
            "Qulllon",
            "Xylon",
        ];

        for name in names {
            assert_eq!(caverphone.encode(name), "KLN1111111", "{} cause the error", name);
        }
    }

    #[test]
    fn test_caverphone_revisited_random_name_tn11111111() {
        let caverphone = Caverphone2::new();

        let names = vec![
            "Dan",
            "Dane",
            "Dann",
            "Darn",
            "Daune",
            "Dawn",
            "Ddene",
            "Dean",
            "Deane",
            "Deanne",
            "DeeAnn",
            "Deeann",
            "Deeanne",
            "Deeyn",
            "Den",
            "Dene",
            "Denn",
            "Deonne",
            "Diahann",
            "Dian",
            "Diane",
            "Diann",
            "Dianne",
            "Diannne",
            "Dine",
            "Dion",
            "Dione",
            "Dionne",
            "Doane",
            "Doehne",
            "Don",
            "Donn",
            "Doone",
            "Dorn",
            "Down",
            "Downe",
            "Duane",
            "Dun",
            "Dunn",
            "Duyne",
            "Dyan",
            "Dyane",
            "Dyann",
            "Dyanne",
            "Dyun",
            "Tan",
            "Tann",
            "Teahan",
            "Ten",
            "Tenn",
            "Terhune",
            "Thain",
            "Thaine",
            "Thane",
            "Thanh",
            "Thayne",
            "Theone",
            "Thin",
            "Thorn",
            "Thorne",
            "Thun",
            "Thynne",
            "Tien",
            "Tine",
            "Tjon",
            "Town",
            "Towne",
            "Turne",
            "Tyne",
        ];

        for name in names {
            assert_eq!(caverphone.encode(name), "TN11111111", "{} cause the error", name);
        }
    }

    #[test]
    fn test_caverphone_revisited_random_name_tta1111111() {
        let caverphone = Caverphone2::new();

        let names = vec![
            "Darda",
            "Datha",
            "Dedie",
            "Deedee",
            "Deerdre",
            "Deidre",
            "Deirdre",
            "Detta",
            "Didi",
            "Didier",
            "Dido",
            "Dierdre",
            "Dieter",
            "Dita",
            "Ditter",
            "Dodi",
            "Dodie",
            "Dody",
            "Doherty",
            "Dorthea",
            "Dorthy",
            "Doti",
            "Dotti",
            "Dottie",
            "Dotty",
            "Doty",
            "Doughty",
            "Douty",
            "Dowdell",
            "Duthie",
            "Tada",
            "Taddeo",
            "Tadeo",
            "Tadio",
            "Tati",
            "Teador",
            "Tedda",
            "Tedder",
            "Teddi",
            "Teddie",
            "Teddy",
            "Tedi",
            "Tedie",
            "Teeter",
            "Teodoor",
            "Teodor",
            "Terti",
            "Theda",
            "Theodor",
            "Theodore",
            "Theta",
            "Thilda",
            "Thordia",
            "Tilda",
            "Tildi",
            "Tildie",
            "Tildy",
            "Tita",
            "Tito",
            "Tjader",
            "Toddie",
            "Toddy",
            "Torto",
            "Tuddor",
            "Tudor",
            "Turtle",
            "Tuttle",
            "Tutto",
        ];

        for name in names {
            assert_eq!(caverphone.encode(name), "TTA1111111", "{} cause the error", name);
        }
    }

    #[test]
    fn test_caverphone_revisited_random_words() {
        let caverphone = Caverphone2::new();

        assert_eq!(caverphone.encode("rather"), "RTA1111111");
        assert_eq!(caverphone.encode("ready"), "RTA1111111");
        assert_eq!(caverphone.encode("writer"), "RTA1111111");

        assert_eq!(caverphone.encode("social"), "SSA1111111");

        assert_eq!(caverphone.encode("able"), "APA1111111");
        assert_eq!(caverphone.encode("appear"), "APA1111111");
    }

    #[test]
    fn test_end_mb_caverphone2() {
        let caverphone = Caverphone2::new();

        assert_eq!(caverphone.encode("mb"), "M111111111");
        assert_eq!(caverphone.encode("mbmb"), "MPM1111111");
    }

    #[test]
    fn test_is_caverphone2_equals(){
        let caverphone = Caverphone2::new();

        assert_eq!(caverphone.is_encoded_equals("Peter", "Stevenson"), false);
        assert!(caverphone.is_encoded_equals("Peter", "Peady"));
    }

    #[test]
    fn test_specification_examples(){
        let caverphone = Caverphone2::new();

        assert_eq!(caverphone.encode("Peter"), "PTA1111111");
        assert_eq!(caverphone.encode("ready"), "RTA1111111");
        assert_eq!(caverphone.encode("social"), "SSA1111111");
        assert_eq!(caverphone.encode("able"), "APA1111111");
        assert_eq!(caverphone.encode("Tedder"), "TTA1111111");
        assert_eq!(caverphone.encode("Karleen"), "KLN1111111");
        assert_eq!(caverphone.encode("Dyun"), "TN11111111");
    }
}