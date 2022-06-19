mod caverphone;

pub use crate::caverphone::Caverphone1;
pub use crate::caverphone::Caverphone2;

pub trait Encoder {
    fn encode(&self, s: &str) -> String;

    fn is_encoded_equals(&self, first:&str, second:&str) -> bool {
        let f = self.encode(first);
        let s = self.encode(second);

        return f == s;
    }
}


