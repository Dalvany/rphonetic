use std::path::PathBuf;

use rphonetic::ConfigFiles;

fn main() {
    let config_file = ConfigFiles::new(&PathBuf::from("./test_assets/cc-rules")).unwrap();
    println!("{:?}", config_file);
    let ten_seconds = std::time::Duration::from_secs(10);
    std::thread::sleep(ten_seconds);
}
