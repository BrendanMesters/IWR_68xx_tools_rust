pub struct PointCloud {}

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
pub fn translate_tlv(input: &mut [u8]) -> (Vec<PointCloud>, usize) {
    const MAGIC_WORD: [u8; 8] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    let input_size = input.len();

    let mut magic_index = 0;
    loop {
        if &input[magic_index..(magic_index + 8)] == MAGIC_WORD {
            break;
        }
        magic_index += 1;
        // We did not find a magic word
        if magic_index + 8 == input_size {
            return (vec![], 0);
        }
    }
    println!("Magic word found at index {}", magic_index);
    (vec![], 0)
}
