use serial2::SerialPort;
use std::io::Error;
use std::{thread, time::Duration};

use super::file_reader::{Config, Settings};

#[allow(dead_code)]
pub struct Fmcw {
    cfg: SerialPort,
    data: SerialPort,
    config: Config,
}

impl Fmcw {
    /// create a new FMCW object, based off of a settings struct
    pub fn new(settings: Settings, config: Config) -> Result<Fmcw, Error> {
        let cfg = match SerialPort::open(settings.cfg_port, settings.cfg_baud) {
            Ok(v) => v,
            Err(v) => return Err(v),
        };
        let data = match SerialPort::open(settings.data_port, settings.data_baud) {
            Ok(v) => v,
            Err(v) => return Err(v),
        };
        return Ok(Fmcw { cfg, data, config });
    }

    pub fn send_config(&self) {
        for line in self.config.raw_input.lines() {
            let char_buf = line.as_bytes();
            let buf_size = char_buf.len();
            match self.cfg.write(char_buf) {
                Ok(n) => {
                    if buf_size != n {
                        println!("Buffersize and written characters not equal, buffer size is {buf_size} but only {n} characters where written");
                    }
                }
                Err(_) => {
                    println!("Line write was unsuccesful")
                }
            };
            // Let some time elapse before continuing
            thread::sleep(Duration::from_millis(10));
        }
        println!("Finished sending Config to the FMCW");
    }
}
