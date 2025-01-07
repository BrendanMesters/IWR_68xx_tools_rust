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
    println!("Settings success");

    let config_path = Path::new("./iwr68xx_config.cfg");
    let config: Config = get_result(Config::from_file(&config_path));
    println!("Config success");

    match Fmcw::new(settings, config) {
        Ok(fmcw) => {
            println!("FMCW module loaded succesfully");
            fmcw.send_config();
        }
        Err(e) => {
            eprintln!("FMCW module could not connect, with error: {}\n    This error is most likely caused due to the FMCW not being connected.", e);
        }
    }

    let tlv_path = Path::new("./tlv_example_file.dat");
    let mut tlv_bytes: Vec<u8> = get_result(read_byte_file(tlv_path));
    println!("Size of buf: {}", tlv_bytes.len());
    translate_tlv(&mut tlv_bytes);
}

fn get_result<T>(maybe_result: Result<T, std::io::Error>) -> T {
    match maybe_result {
        Ok(res) => res,
        Err(e) => {
            eprintln!("{}", e.to_string());
            std::process::exit(-1);
        }
    }
}
