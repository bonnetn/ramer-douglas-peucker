use crate::point::Point;
use crate::proto;

/// A trajectory represents a sequence of GPS points with their timestamps.
/// The coordinates are stored as scaled integers for efficient storage and processing.
pub struct Trajectory {
    /// Latitude values scaled by 10^6
    pub latitudes: Vec<i64>,
    /// Longitude values scaled by 10^6
    pub longitudes: Vec<i64>,
    /// Unix timestamps in seconds
    pub timestamps: Vec<u64>,
}

/// Scale factor for coordinate precision (10^6 = 1 microdegree ≈ 11cm at equator)
const SCALE: u32 = 6;

impl Trajectory {
    /// Creates a new trajectory from a sequence of GPS points.
    ///
    /// # Arguments
    ///
    /// * `points` - A vector of GPS points to convert into a trajectory
    ///
    /// # Returns
    ///
    /// A new `Trajectory` instance with coordinates scaled to integers
    pub fn new(points: Vec<Point>) -> Self {
        let capacity = points.len();
        let mut trajectory = Trajectory {
            latitudes: Vec::with_capacity(capacity),
            longitudes: Vec::with_capacity(capacity),
            timestamps: Vec::with_capacity(capacity),
        };

        for point in points {
            let ts: u64 = point.datetime.timestamp().try_into().unwrap();
            let mut latitude = point.latitude;
            let mut longitude = point.longitude;

            latitude.rescale(SCALE);
            longitude.rescale(SCALE);

            let latitude_i64: i64 = latitude.mantissa().try_into().unwrap();
            let longitude_i64: i64 = longitude.mantissa().try_into().unwrap();

            trajectory.latitudes.push(latitude_i64);
            trajectory.longitudes.push(longitude_i64);
            trajectory.timestamps.push(ts);
        }

        trajectory
    }

    /// Converts the trajectory to a protobuf message using delta encoding.
    /// Delta encoding stores the difference between consecutive values,
    /// which can lead to better compression for smooth trajectories.
    pub fn to_delta_proto(&self) -> proto::Trajectory {
        let latitudes: Vec<i64> = self.latitudes.iter().copied()
            .scan(0_i64, |last, lat| {
                let delta = lat - *last;
                *last = lat;
                Some(delta)
            })
            .collect();

        let longitudes: Vec<i64> = self.longitudes.iter().copied()
            .scan(0_i64, |last, lon| {
                let delta = lon - *last;
                *last = lon;
                Some(delta)
            })
            .collect();

        let timestamps: Vec<u64> = self.timestamps.iter().copied()
            .scan(0_u64, |last, ts| {
                let delta = ts - *last;
                *last = ts;
                Some(delta)
            })
            .collect();

        proto::Trajectory {
            latitudes,
            longitudes,
            timestamps,
        }
    }

    /// Converts the trajectory to a protobuf message using absolute values.
    /// This is useful when delta encoding doesn't provide good compression
    /// or when random access to coordinates is needed.
    pub fn to_proto(&self) -> proto::Trajectory {
        proto::Trajectory {
            latitudes: self.latitudes.clone(),
            longitudes: self.longitudes.clone(),
            timestamps: self.timestamps.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use chrono::DateTime;
    use std::str::FromStr;

    fn create_test_point(lat: f64, lon: f64, timestamp: i64) -> Point {
        Point {
            latitude: Decimal::from_str(&lat.to_string()).unwrap(),
            longitude: Decimal::from_str(&lon.to_string()).unwrap(),
            datetime: DateTime::from_timestamp(timestamp, 0).unwrap(),
        }
    }

    #[test]
    fn test_trajectory_new() {
        let points = vec![
            create_test_point(1.0, 2.0, 1000),
            create_test_point(2.0, 3.0, 2000),
        ];
        let trajectory = Trajectory::new(points);

        assert_eq!(trajectory.latitudes.len(), 2);
        assert_eq!(trajectory.longitudes.len(), 2);
        assert_eq!(trajectory.timestamps.len(), 2);

        // Check scaling
        assert_eq!(trajectory.latitudes[0], 1_000_000);
        assert_eq!(trajectory.longitudes[0], 2_000_000);
        assert_eq!(trajectory.timestamps[0], 1000);
    }

    #[test]
    fn test_trajectory_to_proto() {
        let points = vec![
            create_test_point(1.0, 2.0, 1000),
            create_test_point(2.0, 3.0, 2000),
        ];
        let trajectory = Trajectory::new(points);
        let proto = trajectory.to_proto();

        assert_eq!(proto.latitudes, vec![1_000_000, 2_000_000]);
        assert_eq!(proto.longitudes, vec![2_000_000, 3_000_000]);
        assert_eq!(proto.timestamps, vec![1000, 2000]);
    }

    #[test]
    fn test_trajectory_to_delta_proto() {
        let points = vec![
            create_test_point(1.0, 2.0, 1000),
            create_test_point(2.0, 3.0, 2000),
        ];
        let trajectory = Trajectory::new(points);
        let proto = trajectory.to_delta_proto();

        assert_eq!(proto.latitudes, vec![1_000_000, 1_000_000]);
        assert_eq!(proto.longitudes, vec![2_000_000, 1_000_000]);
        assert_eq!(proto.timestamps, vec![1000, 1000]);
    }
}
