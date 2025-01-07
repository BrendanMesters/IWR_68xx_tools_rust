use std::{collections::VecDeque, mem::ManuallyDrop};

pub struct PointCloud {}

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

/// `RawHeader`  is a `union` type, which holds 40 bytes of raw
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
        unsafe { self.headers.package_length }.try_into().unwrap()
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
pub fn translate_tlv(input: &mut Vec<u8>) -> (Vec<PointCloud>, usize) {
    const MAGIC_WORD: [u8; 8] = [0x02, 0x01, 0x04, 0x03, 0x06, 0x05, 0x08, 0x07];

    let input_size = input.len();
    if input_size < 8 {
        println!("`translate_tlv` was called with an input of size {}, try calling it with an input of *at least* size 8", input_size);
        return (vec![], 0);
    }
    let mut magic_indexes: Vec<usize> = Vec::new();
    for i in 0..(input_size - 8) {
        if &input[i..(i + 8)] == MAGIC_WORD {
            magic_indexes.push(i);
        }
    }

    println!(
        "First and second magic index are at locations {}, {}",
        magic_indexes[0], magic_indexes[1]
    );

    let mut consumed_bytes = 0;
    for mi in magic_indexes {
        if mi != consumed_bytes {
            println!("The current magic index does not seem to be at the current start of the data, magic index at {mi} encountered, while only {consumed_bytes} bytes where consumed");
        }
        if let Some((pc, n)) = read_frame(input) {
            println!("Succesfully parsed a frame of size {}", n);
        }
    }

    (vec![], 0)
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
fn read_frame(input: &mut Vec<u8>) -> Option<(PointCloud, usize)> {
    let header: FrameHeader = match read_header(input) {
        Some(h) => h,
        None => return None,
    };
    if header.frame_len() > input.len() {
        return None;
    }

    return Some((PointCloud {}, 0));
    // Removing a "frame" can be done effectively with `std::vec::Vec::drain()`.
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
