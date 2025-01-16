use super::tlv_translator::Frame;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::mpsc;

/// Sends all data which is provided through the
/// `data_input_stream` channel to a different process via
/// a _Unix Socket_, located at `/tmp/fmcw_ipc_socket`
pub fn ipc_sender(data_input_stream: mpsc::Receiver<Frame>) -> std::io::Result<()> {
    // Create a Unix socket connection to the Python server.
    let socket_path = "/tmp/fmcw_ipc_socket";
    let mut stream = UnixStream::connect(socket_path)?;

    while let Ok(data) = data_input_stream.recv() {
        let serialized_frame = serde_json::to_string(&data).unwrap();

        // Send the data to the python program
        stream.write_all(serialized_frame.as_bytes())?;
        stream.write_all(b"\n")?; // Add a newline delimiter for easier message framing.
        stream.flush()?;

        // // Read the response from the server.
        // let mut buffer = String::new();
        // stream.read_to_string(&mut buffer)?;
        //
        // // Deserialize and print the response.
        // if !buffer.is_empty() {
        //     let response: String = serde_json::from_str(&buffer).unwrap();
        //     println!("Response from Python: {}", response);
        // }
    }

    Ok(())
}

/// Sends all data which is provided through the
/// `data_input_stream` channel to a different process via
/// a _Unix Socket_, located at `/tmp/fmcw_ipc_socket`
pub fn ipc_test_sender(frame: Frame) -> std::io::Result<()> {
    // Create a Unix socket connection to the Python server.
    let socket_path = "/tmp/fmcw_ipc_socket";
    let mut stream = UnixStream::connect(socket_path)?;

    let serialized_frame = serde_json::to_string(&frame).unwrap();

    // Send the data to the python program
    stream.write_all(serialized_frame.as_bytes())?;
    stream.write_all(b"\n")?; // Add a newline delimiter for easier message framing.
    stream.flush()?;

    let serialized_frame = serde_json::to_string(&frame).unwrap();

    // Send the data to the python program
    stream.write_all(serialized_frame.as_bytes())?;
    stream.write_all(b"\n")?; // Add a newline delimiter for easier message framing.
    stream.flush()?;

    let serialized_frame = serde_json::to_string(&frame).unwrap();

    // Send the data to the python program
    stream.write_all(serialized_frame.as_bytes())?;
    stream.write_all(b"\n")?; // Add a newline delimiter for easier message framing.
    stream.flush()?;

    loop {}
    // // Read the response from the server.
    // let mut buffer = String::new();
    // stream.read_to_string(&mut buffer)?;
    //
    // // Deserialize and print the response.
    // if !buffer.is_empty() {
    //     let response: String = serde_json::from_str(&buffer).unwrap();
    //     println!("Response from Python: {}", response);
    // }

    Ok(())
}
