use std::path::PathBuf;

use rphonetic::{BeiderMorseBuilder, ConfigFiles, Encoder};

#[allow(clippy::disallowed_macros)]
fn main() {
    let config_file = ConfigFiles::new(&PathBuf::from("./test_assets/cc-rules")).unwrap();
    let beider_morse = BeiderMorseBuilder::new(&config_file).build();
    let mut count = 100;
    while count > 0 {
        println!("{}", beider_morse.encode("test"));
        let ten_seconds = std::time::Duration::from_millis(50);
        std::thread::sleep(ten_seconds);
        count -= 1;
    }
}
