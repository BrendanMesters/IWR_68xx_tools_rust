use std::path::Path;
use std::{thread, time::Duration};

mod config;
mod fmcw_manager;
mod settings;
mod tlv_translator;

use config::Config;
use fmcw_manager::Fmcw;
use settings::Settings;

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
    let settings: Settings = Settings::new(&settings_path);

    let config_path = Path::new("./iwr68xx_config.cfg");
    let config: Config = match Config::init_conf(config_path) {
        Ok(conf) => conf,
        Err(e) => {
            eprintln!("?{}", e.to_string());
            std::process::exit(-1);
        }
    };

    let fmcw = match Fmcw::new(settings, config) {
        Ok(fmcw) => fmcw,
        Err(e) => {
            eprintln!("?{}", e.to_string());
            std::process::exit(-1);
        }
    };
    fmcw.send_config();
    loop {}
}
