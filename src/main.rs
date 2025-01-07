use std::path::Path;
use std::{thread, time::Duration};

mod file_reader;
mod fmcw_manager;
mod tlv_translator;

use file_reader::{read_byte_file, Config, Settings};
use fmcw_manager::Fmcw;
use tlv_translator::translate_tlv;

fn main() {
    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        std::process::exit(-1);
    })
    .expect("Error setting Ctrl-C handler");

    // Following code does the actual work, and can be interrupted by pressing
    // Ctrl-C. As an example: Let's wait a few seconds.
    thread::sleep(Duration::from_secs(2));
    let settings_path = Path::new("./settings.toml");
    let settings: Settings = Settings::from_file(&settings_path);

    let config_path = Path::new("./iwr68xx_config.cfg");
    let config: Config = get_result(Config::from_file(&config_path));

    let fmcw = get_result(Fmcw::new(settings, config));
    fmcw.send_config();

    let tlv_path = Path::new("./tlv_example_file");
    let tlv_bytes: Vec<u8> = get_result(read_byte_file(tlv_path));
    loop {}
}

fn get_result<T>(maybe_result: Result<T, std::io::Error>) -> T {
    match maybe_result {
        Ok(res) => res,
        Err(e) => {
            eprintln!("?{}", e.to_string());
            std::process::exit(-1);
        }
    }
}
