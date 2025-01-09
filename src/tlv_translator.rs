use plotters::prelude::*;
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

pub fn is_magic(input: &Vec<u8>, index: usize) -> bool {
    const MAGIC_WORD: [u8; 8] = [0x02, 0x01, 0x04, 0x03, 0x06, 0x05, 0x08, 0x07];

    let input_size = input.len();
    if input_size < (index + 8) {
        return false;
    }
    &input[index..(index + 8)] == MAGIC_WORD
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
    let input_size = input.len();
    if input_size < 8 {
        println!("`translate_tlv` was called with an input of size {}, try calling it with an input of *at least* size 8", input_size);
        return (vec![], 0);
    }
    let mut magic_indexes: Vec<usize> = Vec::new();
    for i in 0..(input_size - 8) {
        if is_magic(input, i) {
            magic_indexes.push(i);
        }
    }

    println!(
        "First and second magic index are at locations {}, {}",
        magic_indexes[0], magic_indexes[1]
    );

    let mut consumed_bytes = 0;
    let mut i = 0;
    let mut prev_size = input.len();
    let mut cur_size = 0;
    for mi in magic_indexes {
        // if mi != consumed_bytes {
        //     println!("The current magic index does not seem to be at the current start of the data, magic index at {mi} encountered, while only {consumed_bytes} bytes where consumed");
        // }
        println!("Start is a magic word");
        if let Some((pc, n)) = read_frame(input) {
            println!("Succesfully parsed a frame of size {}", n);
            cur_size = input.len();
            println!(
                "Amount of bytes actually consumed: {}",
                prev_size - cur_size
            );
            prev_size = cur_size;
            if is_magic(input, 0) {
            } else {
                println!("Start is NOT a magic word");
            }
        }
        println!("");
        i += 1;
        if i > 5 {
            break;
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
fn read_frame(data: &mut Vec<u8>) -> Option<(PointCloud, usize)> {
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
    parse_frame(raw_frame);

    return Some((PointCloud {}, frame_len));
    // Removing a "frame" can be done effectively with `std::vec::Vec::drain()`.
}

fn parse_frame(mut data: Vec<u8>) {
    // Remove the frame header
    data.drain(0..40);
    loop {
        if let Some(tlv_header) = TlvHeader::extract_tlv_header(&mut data) {
            println!(
                "Tlv data, type: {} - len: {}, type debug: {:#034b}",
                tlv_header.tlv_type(),
                tlv_header.tlv_len(),
                tlv_header.tlv_type(),
            );
            if tlv_header.tlv_len() > data.len() {
                break;
            }

            let raw_tlv_data: Vec<u8> = data.drain(0..tlv_header.tlv_len()).collect();
            match TlvType::from_num(tlv_header.tlv_type()) {
                Some(TlvType::DetectedPoints) => {}
                Some(TlvType::RangeProfile) => {
                    println!("Range profile");
                    let data = parse_raw_range_profile(raw_tlv_data);
                    render_kde(&data);
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
            println!("- Current data remaining: {}", data.len());
        } else {
            break;
        }
    }
    println!("leftover data in frame: {}", data.len())
}

fn render_kde(data: &Vec<f64>) {
    // We need to convert our series to a Kernel Density Estimate
    // Then we want to render the kernel density estimate as an
    // Area series with the Plotter crate.

    // Set variables for KDE
    const SHARPNESS: isize = 3; // Number of points per 1 distance
    const MARGINS: isize = 10; // Margins to both ends of the min and max val
    const KERNEL_SIZE: f64 = 17.0f64; // Size of the kernel

    let min = data
        .iter()
        .min_by(|a, b| a.total_cmp(b))
        .expect("The data passed to kde should not contain NaN numbers");
    let max = data
        .iter()
        .max_by(|a, b| a.total_cmp(b))
        .expect("The data passed to kde should not contain NaN numbers");

    // Get the datarange on which we will calculate height
    let datarange: Vec<f64> = (((min * SHARPNESS as f64) as isize - MARGINS * SHARPNESS)
        ..=((max * SHARPNESS as f64) as isize - MARGINS * SHARPNESS))
        .map(|v| v as f64 / SHARPNESS as f64)
        .collect();

    // Apply KDE
    let kde: Vec<(f64, f64)> = datarange
        .iter()
        .map(|x| {
            let mut y: f64 = 0.0f64;
            for p in data {
                if (p - *x).abs() < KERNEL_SIZE as f64 {
                    y += (p - *x).abs() / KERNEL_SIZE as f64;
                }
            }
            (*x, y as f64)
        })
        .collect();
    let max_y = kde
        .iter()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(x, y)| y)
        .expect("Y values should always be comparable");

    // Render result with plotters
    let root = BitMapBackend::new("./plotters-doc-data/5.png", (640, 480)).into_drawing_area();
    let _ = root.fill(&WHITE);
    let root = root.margin(10, 10, 10, 10);

    // After this point, we should be able to construct a chart context
    let mut chart = match ChartBuilder::on(&root)
        // Set the caption of the chart
        .caption("KDE Range Profile", ("sans-serif", 40).into_font())
        // Set the size of the label region
        .x_label_area_size(20)
        .y_label_area_size(40)
        // Finally attach a coordinate on the drawing area and make a chart context
        .build_cartesian_2d(*min..*max, 0f64..*max_y)
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    // Then we can draw a mesh
    let _ = chart
        .configure_mesh()
        // We can customize the maximum number of labels allowed for each axis
        .x_labels(5)
        .y_labels(5)
        // We can also change the format of the label text
        .y_label_formatter(&|x| format!("{:.3}", x))
        .draw();

    // And we can draw something in the drawing area
    let _ = chart.draw_series(AreaSeries::new(kde, 0., &RED));
    // Similarly, we can draw point series
    let _ = root.present();
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
