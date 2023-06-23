use crate::ease::*;
use num_traits::{clamp, Float};

/// Linearly interpolates between two values.
pub fn lerp<T: Float>(value: T, range: (T, T)) -> T {
    let (start, end) = range;
    start + value * (end - start)
}

/// Gets the percentage of a value within a range.
pub fn get_range_pct<T: Float>(value: T, range: (T, T)) -> T {
    let (start, end) = range;
    let epsilon = T::epsilon();
    let one = T::one();
    let zero = T::zero();

    // Avoid divide by Zero.
    // But also if our range is a point, output whether Value is before or after.
    let divisor = end - start;
    if divisor.abs() < epsilon {
        return if value >= end { one } else { zero };
    }

    (value - start) / divisor
}

/// Maps a value from one range to another.
pub fn map_range<T: Float>(value: T, input: (T, T), output: (T, T)) -> T {
    let pct = get_range_pct(value, input);
    let (output_start, output_end) = output;
    lerp(pct, (output_start, output_end))
}

/// Maps a value from one range to another, clamping the output to the output range.
pub fn map_range_clamped<T: Float>(value: T, input: (T, T), output: (T, T)) -> T {
    let pct = clamp(get_range_pct(value, input), T::zero(), T::one());
    let (output_start, output_end) = output;
    lerp(pct, (output_start, output_end))
}

/// Maps a value from one range to another, easing the output.
pub fn map_range_eased<T: Float + Into<f32> + From<f32>>(
    value: T,
    input: (T, T),
    output: (T, T),
    ease: EaseFunction,
) -> T {
    let pct = clamp(get_range_pct(value, input), T::zero(), T::one());
    let (output_start, output_end) = output;
    let eased_pct = pct.into().calc(ease);
    lerp(eased_pct.into(), (output_start, output_end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_range() {
        assert_eq!(map_range(2.0, (1.0, 0.0), (1.0, 2.0)), 0.0);
    }
}
