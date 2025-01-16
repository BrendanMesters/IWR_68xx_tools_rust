use std::sync::mpsc;
use std::thread;
use std::{path::Path, sync::Arc};

mod file_reader;
mod fmcw_manager;
mod ipc;
mod renderer;
mod tlv_translator;

use file_reader::{read_byte_file, Config, Settings};
use fmcw_manager::Fmcw;
use tlv_translator::{translate_tlv, Frame};

fn main() {
    // Test the IPC code with random data.
    if false {
        test_ipc();
    }

    let settings_path = Path::new("./settings.toml");
    let settings: Arc<Settings> = Arc::new(Settings::from_file(&settings_path));
    println!("Settings read succesfully");

    // let config_path = Path::new("./iwr68xx_config.cfg");
    let config_path = Path::new("./iwr6843_config.cfg");
    let config: Config = get_result(Config::from_file(&config_path));
    println!("Config read succesfully");

    // Test the tlv parser on a local raw data file.
    if settings.read_from_file {
        let tlv_path = Path::new("./tlv_example_file.dat");
        read_tlv_file(tlv_path);
    }

    println!("\n    Data transfer starting: ");
    // Byte capture
    let (fmcw_tx, fmcw_rx) = mpsc::channel::<Vec<u8>>();
    let fmcw_thread: thread::JoinHandle<()> = match Fmcw::new(settings.clone(), config) {
        Ok(fmcw) => {
            println!("FMCW module loaded succesfully\n");
            thread::spawn(move || fmcw.run(fmcw_tx))
        }
        Err(e) => {
            eprintln!("FMCW module could not connect, with error: {}\n    This error is most likely caused due to the FMCW not being connected.", e);
            thread::spawn(|| {})
        }
    };

    let (ipc_tx, ipc_rx) = mpsc::channel::<Frame>();

    // Byte processing
    let tlv_set = settings.clone();
    let tlv_reader_thread =
        thread::spawn(move || tlv_translator::parse_stream(fmcw_rx, ipc_tx, tlv_set));
    let ipc_thread = if settings.ipc_send {
        thread::spawn(move || ipc::ipc_sender(ipc_rx))
    } else {
        thread::spawn(|| Ok(()))
    };

    // ipc thread is joined firstly, as this is the only one who can potentially return a result
    // (error)
    if let Err(e) = ipc_thread.join().unwrap() {
        eprintln!("Error received in the IPC thread: {}\n    This is most likely occuring due to the python script not yet running", e);
    }
    fmcw_thread.join().unwrap();
    tlv_reader_thread.join().unwrap();
}

fn test_ipc() {
    let pc1 = tlv_translator::PointCloudPoint::empty();
    let pc2 = tlv_translator::PointCloudPoint {
        x: 1.0,
        y: 2.0,
        z: 3.0,
        d: 4.0,
    };
    let mut frame = tlv_translator::Frame::empty(63);
    frame.set_pointcloud(vec![pc1, pc2]);
    frame.set_range_profile(vec![6.3f64, 3.6f64, 63.0f64]);

    ipc::ipc_test_sender(frame);
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
