use serial2::SerialPort;
use std::io::Error;
use std::sync::mpsc;
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

    /// Collects data from the FMCW hardware and continuously
    /// publishes this to the provided channel `tx`.
    ///
    /// **NOTE** This function never returns. It should be
    /// called as a new thread.
    pub fn run(&self, tx: mpsc::Sender<Vec<u8>>) -> ! {
        self.send_config();

        // Continuously receive data
        loop {
            let bytes = match self.receive_bytes() {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("{}", e);
                    continue;
                }
            };
            if bytes.len() == 0 {
                println!("received no length");
                // If there was no data to be read, wait ⅒   of a second
                thread::sleep(Duration::from_millis(100));
                continue;
            }

            // Try to transmit and print the error if there is any.
            if let Err(e) = tx.send(bytes) {
                println!("Error sending");
                eprint!("!!Channel sending caused an error: {}!!", e);
            } else {
                println!("succesfully send bytes");
            };
        }
    }

    pub fn send_config(&self) {
        for line in self.config.raw_input.lines() {
            let line = format!("{}\n", line);
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

    // Tries to read bytes from the FMCW, passing through any IO
    // errors encountered from reading the serial port.
    //
    // The data can be an empty vector if no data was received
    // during the call
    pub fn receive_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut buf: [u8; 1024] = [0; 1024];
        let mut result: Vec<u8> = vec![];
        loop {
            let read_bytes: usize = self.data.read(&mut buf)?;
            if read_bytes == 0 {
                println!("received bytes where of size 0");
                break;
            }
            // println!("received {} bytes", read_bytes);
            // Copy all read bytes into the result vec
            result.append(&mut buf[0..read_bytes].to_vec());
            if result.len() > 2048 {
                break;
            }
        }
        println!("returning {} bytes", result.len());
        Ok(result)
    }
}
