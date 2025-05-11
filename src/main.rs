//! A proof of concept for trajectory data processing and compression.
//! This program processes GPS trajectory data, simplifies it using the Douglas-Peucker algorithm,
//! and demonstrates different serialization approaches.

mod point;
mod simplify;
mod trajectory;

use num_format::{Locale, ToFormattedString};
use point::{parse_plt_file, ParseError};
use prost::Message;
use std::fs;
use std::time::Instant;
use thiserror::Error;

// Include the generated protobuf code
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/trajectory.rs"));
}

use crate::trajectory::Trajectory;

/// Locale for number formatting
const LOCALE: Locale = Locale::en;

/// Epsilon for simplification (before 1e-6 multiplier), 100 meters precision:
const EPSILON: i64 = 1000;

/// Custom error type for the application
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}


/// Main entry point for the trajectory processing application.
///
/// # Returns
///
/// Returns `Result<(), AppError>` where:
/// - `Ok(())` indicates successful processing
/// - `Err(AppError)` contains details about any errors encountered
fn main() -> Result<(), AppError> {
    let dir_path = "geolife/";
    let mut total_size = 0;

    let start = Instant::now();
    let all_points = {
        let mut all_points = Vec::new();

        // Read all files in the directory
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("plt") {
                let file_size = fs::metadata(&path)?.len();
                total_size += file_size;

                let file = fs::File::open(&path)?;
                let reader = std::io::BufReader::new(file);
                let points = parse_plt_file(reader)?;
                all_points.extend(points);
            }
        }

        // Sort all points by timestamp
        all_points.sort_by_key(|p| p.datetime);
        all_points
    };
    let total_points = all_points.len();
    let duration = start.elapsed();

    println!(
        "Read {} points in {duration:?}",
        all_points.len().to_formatted_string(&LOCALE),
        duration = duration
    );

    let trajectory = Trajectory::new(all_points);

    // Simplify the points using Douglas-Peucker algorithm
    let start = Instant::now();
    let keep_points = simplify::simplify(&trajectory.latitudes, &trajectory.longitudes, EPSILON);
    let duration = start.elapsed();

    println!("Ran simplification in {duration:?}");

    let start = Instant::now();
    let simplified_trajectory = {
        let mut latitudes = trajectory.latitudes;
        let mut longitudes = trajectory.longitudes;
        let mut timestamps = trajectory.timestamps;

        let mut index = 0;
        latitudes.retain(|_| {
            let v = keep_points[index];
            index += 1;
            v
        });

        let mut index = 0;
        longitudes.retain(|_| {
            let v = keep_points[index];
            index += 1;
            v
        });

        let mut index = 0;
        timestamps.retain(|_| {
            let v = keep_points[index];
            index += 1;
            v
        });
        Trajectory {
            latitudes,
            longitudes,
            timestamps,
        }
    };
    let duration = start.elapsed();

    println!(
        "Filtered {} points in {duration:?}",
        simplified_trajectory
            .latitudes
            .len()
            .to_formatted_string(&LOCALE),
        duration = duration
    );

    let protobuf_value = simplified_trajectory.to_delta_proto();
    let serialized_delta = protobuf_value.encode_to_vec();
    let protobuf_value = simplified_trajectory.to_proto();
    let serialized = protobuf_value.encode_to_vec();

    println!();

    println!(
        "Original size:        {:>21} bytes",
        total_size.to_formatted_string(&LOCALE)
    );

    println!(
        "Size after simplification: {:>16} bytes",
        serialized.len().to_formatted_string(&LOCALE)
    );

    println!(
        "Serialized DELTA size: {:>20} bytes",
        serialized_delta.len().to_formatted_string(&LOCALE)
    );
    println!(
        "Total points: {:>29} points",
        total_points.to_formatted_string(&LOCALE)
    );
    println!(
        "simplified points: {:>24} points",
        simplified_trajectory
            .latitudes
            .len()
            .to_formatted_string(&LOCALE)
    );

    println!(
        "Ratio points: {:>29.2} %",
        (simplified_trajectory.latitudes.len() as f64 / total_points as f64) * 100.0
    );

    println!(
        "Ratio bytes delta vs non-delta: {:>11.2} %",
        (serialized_delta.len() as f64 / serialized.len() as f64) * 100.0
    );

    println!(
        "Ratio bytes delta vs original: {:>12.2} %",
        (serialized_delta.len() as f64 / total_size as f64) * 100.0
    );

    Ok(())
}
