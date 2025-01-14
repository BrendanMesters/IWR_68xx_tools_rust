use std::path::Path;
use std::sync::mpsc;
use std::{thread, time::Duration};

mod file_reader;
mod fmcw_manager;
mod tlv_translator;

use file_reader::{read_byte_file, Config, Settings};
use fmcw_manager::Fmcw;
use tlv_translator::translate_tlv;

fn main() {
    let settings_path = Path::new("./settings.toml");
    let settings: Settings = Settings::from_file(&settings_path);
    println!("Settings read succesfully");

    // let config_path = Path::new("./iwr68xx_config.cfg");
    let config_path = Path::new("./current_try.cfg");
    let config: Config = get_result(Config::from_file(&config_path));
    println!("Config read succesfully");

    let (tx, rx): (mpsc::Sender<Vec<u8>>, mpsc::Receiver<Vec<u8>>) = mpsc::channel();

    // Byte capture
    let fmcw_thread: thread::JoinHandle<()> = match Fmcw::new(settings, config) {
        Ok(fmcw) => {
            println!("FMCW module loaded succesfully");
            thread::spawn(move || fmcw.run(tx))
        }
        Err(e) => {
            eprintln!("FMCW module could not connect, with error: {}\n    This error is most likely caused due to the FMCW not being connected.", e);
            thread::spawn(|| {})
        }
    };

    // let tlv_path = Path::new("./tlv_example_file.dat");
    // read_tlv_file(tlv_path);

    // Byte processing
    let tlv_reader_thread = thread::spawn(move || tlv_translator::parse_stream(rx));

    fmcw_thread.join().unwrap();
    tlv_reader_thread.join().unwrap()
}

fn read_tlv_file(filepath: &Path) {
    let mut tlv_bytes: Vec<u8> = get_result(read_byte_file(filepath));
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
