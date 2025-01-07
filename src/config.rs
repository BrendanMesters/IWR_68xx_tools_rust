use std::fs;
use std::io::Error;
use std::path::Path;

pub struct Config {
    pub raw_input: String,
}

impl Config {
    pub fn init_conf(config_path: &Path) -> Result<Config, Error> {
        let possible_conf = fs::read_to_string(config_path);
        if possible_conf.is_err() {
            let err: std::io::Error = possible_conf
                .err()
                .expect("Error checking has already been done");
            return Err(err);
        }
        let conf_str: String = possible_conf.expect("checked to be non-eroneous");
        return Ok(Config::parse_conf(conf_str));
    }

    fn parse_conf(config: String) -> Config {
        let raw_input = config.clone();
        let conf = Config { raw_input };
        return conf;
    }
}
