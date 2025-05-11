//! Implementation of the Douglas-Peucker algorithm for trajectory simplification.
//! This module provides functions to reduce the number of points in a trajectory
//! while maintaining its essential shape.

/// Calculate the squared perpendicular distance from a point to a line segment.
/// This is an optimized version that avoids unnecessary calculations.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn perpendicular_distance_squared(
    x: i64,
    y: i64,
    x1: i64,
    y1: i64,
    x2: i64,
    y2: i64,
    dx: i128,
    dy: i128,
    line_length_squared: i128,
) -> i64 {
    if dx == 0 && dy == 0 {
        let dx = (x - x1) as i128;
        let dy = (y - y1) as i128;
        return (dx * dx + dy * dy) as i64;
    }

    let area = ((x2 - x1) as i128) * ((y1 - y) as i128) - ((x1 - x) as i128) * ((y2 - y1) as i128);
    if line_length_squared == 0 {
        0
    } else {
        ((area * area) / line_length_squared) as i64
    }
}

/// Iterative implementation of the Douglas-Peucker algorithm using a stack.
/// This version is optimized for performance and avoids recursion.
#[inline(always)]
fn douglas_peucker_iterative(
    positions_x: &[i64],
    positions_y: &[i64],
    epsilon: i64,
    result: &mut [bool],
) {
    assert_eq!(positions_x.len(), positions_y.len());
    assert_eq!(positions_x.len(), result.len());

    let mut stack = Vec::with_capacity(64);
    let len = positions_x.len();
    stack.push((0, len - 1));
    let epsilon_squared = epsilon * epsilon;

    while let Some((start, end)) = stack.pop() {
        if end - start <= 1 {
            continue;
        }
        // Inline find_max_distance
        let mut max_distance = 0;
        let mut max_index = start;
        let sx = positions_x[start];
        let sy = positions_y[start];
        let ex = positions_x[end];
        let ey = positions_y[end];
        let dx = (ex as i128) - (sx as i128);
        let dy = (ey as i128) - (sy as i128);
        let llsq = dx * dx + dy * dy;
        let mut i = start + 1;
        while i + 7 < end {
            let xs = &positions_x[i..i+8];
            let ys = &positions_y[i..i+8];
            let ds: Vec<i64> = xs.iter().zip(ys.iter())
                .map(|(&x, &y)| perpendicular_distance_squared(x, y, sx, sy, ex, ey, dx, dy, llsq))
                .collect();
            for (k, &d) in ds.iter().enumerate() {
                if d > max_distance { max_distance = d; max_index = i + k; }
            }
            i += 8;
        }
        let rem = end - i;
        if rem >= 4 {
            for (k, (&x, &y)) in positions_x[i..i+4].iter().zip(&positions_y[i..i+4]).enumerate() {
                let d = perpendicular_distance_squared(x, y, sx, sy, ex, ey, dx, dy, llsq);
                if d > max_distance { max_distance = d; max_index = i + k; }
            }
            i += 4;
        }
        positions_x[i..end]
            .iter()
            .zip(&positions_y[i..end])
            .enumerate()
            .for_each(|(offset, (&x, &y))| {
                let d = perpendicular_distance_squared(x, y, sx, sy, ex, ey, dx, dy, llsq);
                if d > max_distance {
                    max_distance = d;
                    max_index = i + offset;
                }
            });
        // End inlined find_max_distance
        if max_distance > epsilon_squared {
            result[max_index] = true;
            stack.push((start, max_index));
            stack.push((max_index, end));
        }
    }
}

/// Simplify a sequence of points using the Douglas-Peucker algorithm.
///
/// # Arguments
///
/// * `positions_x` - A slice of x coordinates
/// * `positions_y` - A slice of y coordinates
/// * `epsilon` - The maximum allowed distance between the original line and the simplified line
///
/// # Returns
///
/// A vector of booleans indicating which points to keep in the simplified path
///
/// # Panics
///
/// This function will panic if:
/// * `positions_x` and `positions_y` have different lengths
/// * `epsilon` is negative
#[inline(always)]
pub fn simplify(positions_x: &[i64], positions_y: &[i64], epsilon: i64) -> Vec<bool> {
    assert_eq!(
        positions_x.len(),
        positions_y.len(),
        "positions_x.len() == positions_y.len()"
    );
    assert!(epsilon >= 0, "epsilon must be non-negative");

    if positions_x.len() <= 2 {
        return vec![true; positions_x.len()];
    }

    let mut result = vec![false; positions_x.len()];
    result[0] = true;
    result[positions_x.len() - 1] = true;

    douglas_peucker_iterative(positions_x, positions_y, epsilon, &mut result);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplify_empty() {
        let result = simplify(&[], &[], 1);
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_simplify_single_point() {
        let result = simplify(&[1], &[1], 1);
        assert_eq!(result, vec![true]);
    }

    #[test]
    fn test_simplify_two_points() {
        let result = simplify(&[1, 2], &[1, 2], 1);
        assert_eq!(result, vec![true, true]);
    }

    #[test]
    fn test_simplify_straight_line() {
        // A straight line of 5 points
        let x = vec![0, 1, 2, 3, 4];
        let y = vec![0, 1, 2, 3, 4];
        let result = simplify(&x, &y, 1);
        // Should only keep first and last points
        assert_eq!(result, vec![true, false, false, false, true]);
    }

    #[test]
    fn test_simplify_zigzag() {
        // A zigzag pattern with more pronounced changes
        let x = vec![0, 1, 2, 3, 4];
        let y = vec![0, 5, 0, 5, 0]; // Increased amplitude for more significant changes
        let result = simplify(&x, &y, 1);
        // With a small epsilon, we should keep all points due to the significant changes
        assert_eq!(result, vec![true, true, true, true, true]);
    }

    #[test]
    #[should_panic(expected = "epsilon must be non-negative")]
    fn test_simplify_negative_epsilon() {
        simplify(&[1, 2], &[1, 2], -1);
    }

    #[test]
    #[should_panic(expected = "positions_x.len() == positions_y.len()")]
    fn test_simplify_mismatched_lengths() {
        simplify(&[1, 2], &[1], 1);
    }
}
