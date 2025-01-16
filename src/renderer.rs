// use super::tlv_translator::PointCloudPoint;
use plotters::prelude::*;
use std::path::Path;

// pub fn render_pointcloud(pointcloud: Vec<PointCloudPoint>, filename: &Path) {
//     let x_range = -5f32..5f32;
//     let y_range = -5f32..15f32;
//
//     let root = BitMapBackend::new(filename, (640, 480)).into_drawing_area();
//     let _ = root.fill(&WHITE);
//     let root = root.margin(10, 10, 10, 10);
//
//     // After this point, we should be able to construct a chart context
//     let mut chart = match ChartBuilder::on(&root)
//         // Set the caption of the chart
//         .caption("FMCW data points", ("sans-serif", 40).into_font())
//         // Set the size of the label region
//         .x_label_area_size(20)
//         .y_label_area_size(40)
//         // Finally attach a coordinate on the drawing area and make a chart context
//         .build_cartesian_2d(x_range, y_range)
//     {
//         Ok(v) => v,
//         Err(e) => {
//             eprintln!("{}", e);
//             return;
//         }
//     };
//
//     // Then we can draw a mesh
//     let _ = chart
//         .configure_mesh()
//         // We can customize the maximum number of labels allowed for each axis
//         .x_labels(5)
//         .y_labels(5)
//         // We can also change the format of the label text
//         .y_label_formatter(&|x| format!("{:.3}", x))
//         .draw();
//
//     for point in pointcloud {
//         let it: std::slice::Iter<'_, (f32, f32)> = [(point.x, point.y)].iter();
//         let _: PointElement<&(f64, f64), f64> =
//             chart.draw_series(PointSeries::new([&(point.x, point.y)], 2., &RED));
//     }
//     // And we can draw something in the drawing area
//     // let _ = chart.draw_series(AreaSeries::new(kde, 0., &RED));
//     // Similarly, we can draw point series
//     let _ = root.present();
// }
//
pub fn render_range_profile(data: &Vec<f64>, filename: &str) {
    // We need to convert our series to a Kernel Density Estimate
    // Then we want to render the kernel density estimate as an
    // Area series with the Plotter crate.

    let min = data
        .iter()
        .min_by(|a, b| a.total_cmp(b))
        .expect("The data passed to kde should not contain NaN numbers");
    let max = data
        .iter()
        .max_by(|a, b| a.total_cmp(b))
        .expect("The data passed to kde should not contain NaN numbers");

    let kde = kde_transform(data, min, max);
    let max_y = kde
        .iter()
        .max_by(|(_ax, ay), (_bx, by)| ay.total_cmp(by))
        .map(|(_x, y)| y)
        .expect("Y values should always be comparable");

    // Render result with plotters
    let root = BitMapBackend::new(filename, (640, 480)).into_drawing_area();
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

fn kde_transform(data: &Vec<f64>, min: &f64, max: &f64) -> Vec<(f64, f64)> {
    // Set variables for KDE
    const SHARPNESS: isize = 3; // Number of points per 1 distance
    const MARGINS: isize = 10; // Margins to both ends of the min and max val
    const KERNEL_SIZE: f64 = 17.0f64; // Size of the kernel

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
                    y += ((p - *x) / KERNEL_SIZE as f64 * 2.0).cos();
                }
            }
            (*x, y as f64)
        })
        .collect();
    kde
}
