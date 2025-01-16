use crate::file_reader::Settings;

use super::renderer;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Write,
    sync::{mpsc, Arc},
    thread,
    time::Duration,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct Frame {
    frame_num: usize,
    pointcloud: Option<Vec<PointCloudPoint>>,
    range_profile: Option<Vec<f64>>,
}

impl Frame {
    pub fn empty(frame_num: usize) -> Frame {
        Frame {
            frame_num,
            pointcloud: None,
            range_profile: None,
        }
    }

    pub fn set_pointcloud(&mut self, pc: Vec<PointCloudPoint>) {
        self.pointcloud = Some(pc);
    }

    pub fn set_range_profile(&mut self, rp: Vec<f64>) {
        self.range_profile = Some(rp);
    }

    pub fn render_range_profile(&self) {
        if let Some(rp) = &self.range_profile {
            let _ = std::fs::create_dir_all("./plots/range_profile/");
            let name = format!("./plots/range_profile/{}.png", self.frame_num);
            renderer::render_range_profile(&rp, name.as_str());
        }
    }
}

/// This struct is supposed to represent pointclouds, via the
/// uart specification of TI.
/// In this specification each point in a pointcloud is
/// represented by an x, y and z value, as well as its
/// doppler velocity.
/// Each of these variables takes up exactly 4 bytes.
#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PointCloudPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub d: f32,
}

#[repr(C)]
union PcHelper {
    data: [u8; 16],
    pc: PointCloudPoint,
}

impl PointCloudPoint {
    pub fn from_bytes(data: [u8; 16]) -> PointCloudPoint {
        let pc_helper = PcHelper { data };
        // this is actually safe since the data size of Pointcloudpoint
        // (4 x 32 = 128) and pcHelper.data (8 x 16 = 128) are the same
        // AND because both are layed out in c representation
        return unsafe { pc_helper.pc };
    }

    pub fn empty() -> PointCloudPoint {
        PointCloudPoint {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            d: 0.0,
        }
    }
}

/// The header part has 40 Bytes (320 bits) of data seperated into:
/// 8   -   Magic Word
/// 4   -   Version
/// 4   -   Total Package Length
/// 4   -   Platform
/// 4   -   Frame Number
/// 4   -   Time [in CPU Cycles]
/// 4   -   Num Detected Obj
/// 4   -   Num TLV's
/// 4   -   Subframe Number
#[repr(C)]
#[derive(Clone, Copy)]
struct FrameHeaderBackend {
    magic_word: u64,
    version: u32,
    package_length: u32,
    platform: u32,
    frame_number: u32,
    time: u32,
    num_detected_obj: u32,
    num_tlv: u32,
    subframe_number: u32,
}

/// `FrameHeader`  is a `union` type, which holds 40 bytes of raw
/// data, and links those to the different headers defined in its
/// `headers` field.
#[repr(C)]
union FrameHeader {
    data: [u8; 40],
    headers: FrameHeaderBackend,
}

impl FrameHeader {
    fn new(data: [u8; 40]) -> FrameHeader {
        FrameHeader { data }
    }

    fn frame_len(&self) -> usize {
        // Union field access is ALWAYS specified as unsafe
        // This access is safe, since the types are stored as
        // C represented arrays, meaning the data is just thrown
        // on a large heap.
        unsafe { self.headers.package_length }.try_into().unwrap()
    }

    fn frame_num(&self) -> usize {
        // Union field access is ALWAYS specified as unsafe
        // This access is safe, since the types are stored as
        // C represented arrays, meaning the data is just thrown
        // on a large heap.
        unsafe { self.headers.frame_number }.try_into().unwrap()
    }

    fn subframe_num(&self) -> usize {
        // Union field access is ALWAYS specified as unsafe
        // This access is safe, since the types are stored as
        // C represented arrays, meaning the data is just thrown
        // on a large heap.
        unsafe { self.headers.subframe_number }.try_into().unwrap()
    }

    fn tlv_count(&self) -> usize {
        // Union field access is ALWAYS specified as unsafe
        // This access is safe, since the types are stored as
        // C represented arrays, meaning the data is just thrown
        // on a large heap.
        unsafe { self.headers.num_tlv }.try_into().unwrap()
    }

    fn obj_count(&self) -> usize {
        // Union field access is ALWAYS specified as unsafe
        // This access is safe, since the types are stored as
        // C represented arrays, meaning the data is just thrown
        // on a large heap.
        unsafe { self.headers.num_detected_obj }.try_into().unwrap()
    }
}

/// Enum describing the different TLV frame types
/// as defined in the [specifications of TI](https://dev.ti.com/tirex/explore/content/radar_toolbox_2_30_00_12/software_docs/Understanding_UART_Data_Output_Format.html#statistics)
#[derive(Clone, Copy)]
enum TlvType {
    DetectedPoints = 1,
    RangeProfile = 2,
    NoiseFloorProfile = 3,
    AzimuthStaticHeatmap = 4,
    RangeDopplerHeatmap = 5,
    PerformanceStatistics = 6,
    SideInforForDetectedPoints = 7,
    AzimuthElevationStaticHeatmap = 8,
    TemperatureStatistics = 9,
}

impl TlvType {
    /// Parses an index number (as specified by TI) into
    /// the related TlvType, returns None on an invalid
    /// number
    ///
    /// # Arguments
    /// * `n`: Must be in range (1-9)
    fn from_num(n: usize) -> Option<TlvType> {
        if !(1..=9).contains(&n) {
            return None;
        }
        let result = match n {
            1 => TlvType::DetectedPoints,
            2 => TlvType::RangeProfile,
            3 => TlvType::NoiseFloorProfile,
            4 => TlvType::AzimuthStaticHeatmap,
            5 => TlvType::RangeDopplerHeatmap,
            6 => TlvType::PerformanceStatistics,
            7 => TlvType::SideInforForDetectedPoints,
            8 => TlvType::AzimuthElevationStaticHeatmap,
            9 => TlvType::TemperatureStatistics,
            _ => panic!("This code should be unreachable."),
        };
        return Some(result);
    }
}

/// `TlvHeader` is a `union` type, which holds 8 bytes of raw
/// data and links those to the `tlv_type` and `tlv_len` field
/// in the `TlvheaderBackend` object, stored in te `headers` field
#[repr(C)]
union TlvHeader {
    data: [u8; 8],
    headers: [u32; 2],
}

impl TlvHeader {
    /// Extracts the TLV header from a u8 vector.
    /// Tis operation is **destructive** and removes
    /// the 8 bytes parsed if succeful.
    ///
    /// # Arguments
    /// * `input`:  a reference to a byte vector contaiing
    ///             the data from which a TLV header should
    ///             be extracted
    ///
    /// # Returns
    /// * `None`:   if it was not possible to extract 8 bytes
    ///          from the input, the input is unchanged
    /// * If parsing is succesfull it returns `Some(TlvHeader)`
    ///         and removed the 8 bytes from which it was extracted
    ///         from the input.
    fn extract_tlv_header(input: &mut Vec<u8>) -> Option<TlvHeader> {
        if input.len() < 8 {
            return None;
        }
        let data = input.drain(0..8).collect::<Vec<u8>>().try_into().unwrap();
        Some(TlvHeader::new(data))
    }

    fn new(data: [u8; 8]) -> TlvHeader {
        TlvHeader { data }
    }

    fn tlv_type(&self) -> usize {
        // Union field access is ALWAYS specified as unsafe
        // This access is safe, since the types are stored as
        // C represented arrays, meaning the data is just thrown
        // on a large heap.
        unsafe { self.headers[0] }.try_into().unwrap()
    }

    fn tlv_len(&self) -> usize {
        // Union field access is ALWAYS specified as unsafe
        // This access is safe, since the types are stored as
        // C represented arrays, meaning the data is just thrown
        // on a large heap.
        unsafe { self.headers[1] }.try_into().unwrap()
    }
}

fn is_magic(input: &Vec<u8>, index: usize) -> bool {
    const MAGIC_WORD: [u8; 8] = [0x02, 0x01, 0x04, 0x03, 0x06, 0x05, 0x08, 0x07];

    let input_size = input.len();
    if input_size < (index + 8) {
        return false;
    }
    &input[index..(index + 8)] == MAGIC_WORD
}

/// Parses data which is provided, in packets, along the
/// channel receiver `rx`.
///
/// **NOTE** This function never returns. It should be
/// called as a new thread.
pub fn parse_stream(
    rx: mpsc::Receiver<Vec<u8>>,
    ipc_tx: mpsc::Sender<Frame>,
    settings: Arc<Settings>,
) -> ! {
    let mut byte_stream: Vec<u8> = vec![];

    // If there is an error in reading the file we WILL crash, this
    // is not ideal behaviour
    let raw_data_file: Option<File> = if settings.raw_data_save {
        Some(File::create("./output_tls.dat").unwrap())
    } else {
        None
    };
    let frame_file: Option<File> = if settings.raw_data_save {
        Some(File::create("./frame_output.json").unwrap())
    } else {
        None
    };
    loop {
        // Add all new received packages to the byte stream
        let mut received: bool = false;
        if let Ok(new_bytes) = rx.recv() {
            byte_stream.append(&mut new_bytes.clone());
            received = true;

            if let Some(ref f) = raw_data_file {
                let mut file: &File = f;
                _ = file.write_all(&new_bytes);
            }
        }

        // Process received bytes
        if received {
            println!(
                "Received packages, bytestream length = {}",
                byte_stream.len()
            );
            // Process the byte stream
            for frame in translate_tlv(&mut byte_stream) {
                if let Some(ref f) = frame_file {
                    let mut file: &File = f;
                    let data = format!("{},", serde_json::to_string(&frame.clone()).unwrap());
                    _ = file.write_all(&data.as_bytes());
                }
                _ = ipc_tx.send(frame);
            }
        } else {
            println!("Did not receive packages");
            // If there where no packets to be received, sleep for ⅒   second
            thread::sleep(Duration::from_millis(100));
        }
    }
}

/// The function takes a `TLV byte array` as input and
/// parses as much of it as it can.
///
/// # Arguments
///
/// * `tlv_bytes` - A mutable reference to an array of bytes
///                 representing the tlv bytes which should
///                 be parsed. All data which is consumed
///                 (and thus translated into a point cloud
///                 ) will be removed from the `tlv_bytes`
///                 variable in this process will
///
/// # Returns
/// A tuple containing:
/// * `Vec<PointCloud>` - a vector of frames in the pointcloud
/// * `usize` - The length of the pointcloud consumed.
pub fn translate_tlv(input: &mut Vec<u8>) -> Vec<Frame> {
    let input_size = input.len();
    if input_size < 8 {
        println!("`translate_tlv` was called with an input of size {}, try calling it with an input of *at least* size 8", input_size);
        return vec![];
    }
    let mut magic_indexes: Vec<usize> = Vec::new();
    for i in 0..(input_size - 8) {
        if is_magic(input, i) {
            magic_indexes.push(i);
        }
    }

    let mut result: Vec<Frame> = vec![];

    for _mi in magic_indexes {
        if let Some(frame) = read_frame(input) {
            result.push(frame);
        }
    }

    result
}

/// A function for reading and  processing a single frame of our data.
///
/// # Arguments:
/// * `input`: The full byte set, from which it should extract 1 frame
///
/// # Returns:
/// An option of a tuple containing:
/// * A `PointCloud` object for this single frame
/// * A `usize`, representing the number of bytes from the input consumed
/// Or `None` if the frame is not complete.
fn read_frame(data: &mut Vec<u8>) -> Option<Frame> {
    let header: FrameHeader = read_header(data)?;

    // Check that the frame is complete
    let frame_len = header.frame_len();
    if frame_len > data.len() {
        return None;
    } else {
        // We can read the full frame so can drop the header
        data.drain(0..40);
    }
    println!(
        "Currently handeling frame nuber {}, with {} objects and {} tlv frames",
        header.frame_num(),
        header.obj_count(),
        header.tlv_count()
    );
    // remove 40 from the drainage size as we already removed the header
    let raw_frame: Vec<u8> = data.drain(0..(frame_len - 40)).collect();
    Some(parse_frame(header, raw_frame))
}

fn parse_frame(frame_header: FrameHeader, mut data: Vec<u8>) -> Frame {
    // Remove the frame header
    println!(
        "Subframe num: {}, tlv count: {}",
        frame_header.subframe_num(),
        frame_header.tlv_count()
    );

    let mut frame = Frame::empty(frame_header.frame_num());

    while let Some(tlv_header) = TlvHeader::extract_tlv_header(&mut data) {
        if tlv_header.tlv_len() > data.len() {
            break;
        }

        let raw_tlv_data: Vec<u8> = data.drain(0..tlv_header.tlv_len()).collect();
        match TlvType::from_num(tlv_header.tlv_type()) {
            Some(TlvType::DetectedPoints) => {
                let point_cloud = parse_detected_points(raw_tlv_data);
                frame.set_pointcloud(point_cloud);
            }
            Some(TlvType::RangeProfile) => {
                let range_profile: Vec<f64> = parse_raw_range_profile(raw_tlv_data);
                frame.set_range_profile(range_profile)
            }
            Some(TlvType::NoiseFloorProfile) => {}
            Some(TlvType::AzimuthStaticHeatmap) => {}
            Some(TlvType::RangeDopplerHeatmap) => {}
            Some(TlvType::PerformanceStatistics) => {}
            Some(TlvType::SideInforForDetectedPoints) => {}
            Some(TlvType::AzimuthElevationStaticHeatmap) => {}
            Some(TlvType::TemperatureStatistics) => {}
            None => break,
        }
    }
    frame.render_range_profile();
    return frame;
}

fn parse_detected_points(data: Vec<u8>) -> Vec<PointCloudPoint> {
    let mut result: Vec<PointCloudPoint> = vec![];
    //
    // Each point takes up 16 bytes, so we want to itterate over every point
    for _ in 0..(data.len() / 16) {
        let raw: [u8; 16] = match data[..16].try_into() {
            Ok(v) => v,
            Err(_) => {
                println!("Error when casting detected point data");
                break;
            }
        };
        result.push(PointCloudPoint::from_bytes(raw));
    }
    return result;
}

fn parse_raw_range_profile(data: Vec<u8>) -> Vec<f64> {
    // We need to parse the data (`rangebin` * 16_bit) into a vector
    // of `rangebin` values, where every value is interpreted according
    // to 'Q9' encoding (this is a fixed point encoding) and return
    // a vector (or array) containing the new values.
    // https://en.wikipedia.org/wiki/Q_(number_format)

    // Change the byte string into a list of u16s
    let even_indexed = data.iter().step_by(2);
    let uneven_indexed = data.iter().skip(1).step_by(2);
    even_indexed
        .zip(uneven_indexed)
        .map(|(lower, upper)| (((*lower as u16) << 8) | *upper as u16))
        // Now to do q9 encoding according to the following formula P[db] = 20 * log10( 2.^(logMagRange/2^9) )
        // Accoring to this forum post https://e2e.ti.com/support/sensors-group/sensors/f/sensors-forum/806905/linux-iwr1443boost-interpreting-data-log-magnitude-range-and-doppler-heatmap
        .map(|log_mag_range| 20.0 * f64::log10(2f64.powf(log_mag_range as f64 / 2.0f64.powi(9))))
        .collect()
}

/// Attempts to read a header located at the the top of the input,
/// This is a non-destructive operation, returning an `Option<Header>`
fn read_header(input: &Vec<u8>) -> Option<FrameHeader> {
    if input.len() < 40 {
        return None;
    }
    let data = match input[0..40].try_into() {
        Ok(v) => v,
        Err(_) => return None,
    };
    let headers: FrameHeader = FrameHeader::new(data);
    Some(headers)
}
