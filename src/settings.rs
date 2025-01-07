use std::fs;
use std::path::Path;

pub struct Settings {
    pub cfg_port: String,
    pub cfg_baud: u32,
    pub data_port: String,
    pub data_baud: u32,
}

impl Settings {
    /// This function reads the file at the provided path
    /// and tries to generate settings for the IWR64xx fmcw module.
    pub fn new(settings_file_path: &Path) -> Settings {
        let possible_contents = fs::read_to_string(settings_file_path);
        if possible_contents.is_err() {
            println!("Settings file path was incorrect, returning the default path instead.");
            return Settings::default();
        }
        let mut settings = Settings::default();
        let contents = possible_contents.expect("checked to be non-eroneous");
        for line in contents.lines() {
            let kv: Vec<&str> = line.splitn(2, '=').collect();
            if kv.len() < 2 {
                print!("During settings reading a line did not have the right format");
                continue;
            }
            match kv[0] {
                "cfg_port" => settings.cfg_port = kv[1].to_string(),
                "cfg_baud" => {
                    let baud = match kv[1].parse::<u32>() {
                        Ok(val) => val,
                        Err(_) => continue,
                    };
                    settings.cfg_baud = baud
                }
                "data_port" => settings.data_port = kv[1].to_string(),
                "data_baud" => {
                    let baud = match kv[1].parse::<u32>() {
                        Ok(val) => val,
                        Err(_) => continue,
                    };
                    settings.data_baud = baud
                }
                _ => {}
            };
        }
        return settings;
    }

    // Some sane default values when using the code on linux
    fn default() -> Settings {
        Settings {
            cfg_port: "/dev/ttyUSB0".to_string(),
            cfg_baud: 115200,
            data_port: "/dev/ttyUSB1".to_string(),
            data_baud: 921600,
        }
    }
}
