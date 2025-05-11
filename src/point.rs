use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::io::{self, BufRead};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Error while reading line from file: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid number of fields in line")]
    InvalidFieldCount,
    #[error("Failed to parse date: {0}")]
    DateParse(String),
    #[error("Failed to parse latitude: {0}")]
    LatitudeParse(String),
    #[error("Failed to parse longitude: {0}")]
    LongitudeParse(String),
    #[error("Invalid timestamp")]
    InvalidTimestamp,
}

#[derive(Debug)]
pub struct Point {
    pub latitude: Decimal,
    pub longitude: Decimal,
    pub datetime: DateTime<Utc>,
}

pub fn parse_plt_file(reader: impl BufRead) -> Result<Vec<Point>, ParseError> {
    let lines = reader.lines();
    let mut points = Vec::new();

    let line_iter = lines.skip(6);

    for line in line_iter {
        let line = line?;
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() != 7 {
            return Err(ParseError::InvalidFieldCount);
        }

        // Convert Excel date number to Unix timestamp
        // Excel date starts from 1899-12-30, Unix from 1970-01-01
        // Excel date is in days, Unix timestamp is in seconds
        let excel_date: f64 = parts[4]
            .parse()
            .map_err(|e: std::num::ParseFloatError| ParseError::DateParse(e.to_string()))?;
        let unix_timestamp = ((excel_date - 25569.0) * 86400.0) as i64;

        let datetime =
            DateTime::from_timestamp(unix_timestamp, 0).ok_or(ParseError::InvalidTimestamp)?;

        let point = Point {
            latitude: parts[0]
                .parse()
                .map_err(|e: rust_decimal::Error| ParseError::LatitudeParse(e.to_string()))?,
            longitude: parts[1]
                .parse()
                .map_err(|e: rust_decimal::Error| ParseError::LongitudeParse(e.to_string()))?,
            datetime,
        };

        points.push(point);
    }

    Ok(points)
}
