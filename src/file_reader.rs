use std::fs::{read_to_string, File};
use std::io::{Error, Read};
use std::path::Path;

pub struct Config {
    pub raw_input: String,
}

impl Config {
    pub fn from_file(config_path: &Path) -> Result<Config, Error> {
        let possible_conf = read_to_string(config_path);
        if possible_conf.is_err() {
            println!("config reading error");
            let err: std::io::Error = possible_conf
                .err()
                .expect("Error checking has already been done");
            return Err(err);
        }
        let conf_str: String = possible_conf.expect("checked to be non-eroneous");
        return Ok(Config::parse_conf_string(conf_str));
    }

    fn parse_conf_string(config: String) -> Config {
        let raw_input = config.clone();
        let conf = Config { raw_input };
        return conf;
    }
}

#[derive(Debug)]
pub struct Settings {
    pub cfg_port: String,
    pub cfg_baud: u32,
    pub data_port: String,
    pub data_baud: u32,
    pub read_from_file: bool,
    pub raw_data_save: bool,
    pub save_frames: bool,
    pub ipc_send: bool,
}

impl Settings {
    /// This function reads the file at the provided path
    /// and tries to generate settings for the IWR64xx fmcw module.
    pub fn from_file(settings_file_path: &Path) -> Settings {
        let possible_contents = read_to_string(settings_file_path);
        if possible_contents.is_err() {
            println!("Settings file path was incorrect, returning the default path instead.");
            return Settings::default();
        }
        let mut settings = Settings::default();
        let contents = possible_contents.expect("checked to be non-eroneous");
        for line in contents.lines() {
            let kv: Vec<&str> = line.splitn(2, '=').map(|s| s.trim()).collect();
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
                "read_from_file" => {
                    let read_file = match kv[1].parse::<bool>() {
                        Ok(val) => val,
                        Err(_) => continue,
                    };
                    settings.read_from_file = read_file
                }
                "save_raw_data" => {
                    let raw_save = match kv[1].parse::<bool>() {
                        Ok(val) => val,
                        Err(_) => continue,
                    };
                    settings.raw_data_save = raw_save
                }
                "save_frames" => {
                    let raw_save = match kv[1].parse::<bool>() {
                        Ok(val) => val,
                        Err(_) => continue,
                    };
                    settings.save_frames = raw_save
                }
                "ipc_send" => {
                    let ipc_send = match kv[1].parse::<bool>() {
                        Ok(val) => val,
                        Err(_) => continue,
                    };
                    settings.ipc_send = ipc_send
                }
                other => {
                    eprintln!(
                        "Found an unknown option in the settings: \"{}\" not recognized",
                        other
                    )
                }
            };
        }
        println!("Settings are: \n   {:?}", settings);
        return settings;
    }

    // Some sane default values when using the code on linux
    pub fn default() -> Settings {
        // Settings {
        //     cfg_port: "bullshit".to_string(),
        //     cfg_baud: 63,
        //     data_port: "bullshit".to_string(),
        //     data_baud: 63,
        //     raw_data_save: false,
        //     pointcloud_save: false,
        //     ipc_send: false,
        // }
        Settings {
            cfg_port: "/dev/ttyUSB0".to_string(),
            cfg_baud: 115200,
            data_port: "/dev/ttyUSB1".to_string(),
            data_baud: 921600,
            read_from_file: false,
            raw_data_save: true,
            save_frames: false,
            ipc_send: true,
        }
    }
}

/// `Read bytes file` reads in a raw bytes file
/// located at `path` and returns the contents
/// as a `Vec` if it exists and an `io::Error`
/// otherwise
///
/// # Arguments
///
/// * `path`: The path to the file
///
/// # Returns
///
/// * `Result<Vec<u8>, Error>`: Either the bytes
/// contained in the file or an error
pub fn read_byte_file(path: &Path) -> Result<Vec<u8>, Error> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    return Ok(buffer);
}
