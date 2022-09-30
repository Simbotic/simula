use std::ops::*;
use crate::ease::*;
use bevy::prelude::*;

// Map a value from one range to another
pub fn map_range<
    T: Copy + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> + EasingCalc<T>,
>(
    value: T,
    from_range: (T, T),
    to_range: (T, T),
    ease: EaseFunction
) -> T {
    let (from_min, from_max) = from_range;
    let (to_min, to_max) = to_range;
    let from_range = from_max - from_min;
    let to_range = to_max - to_min;
    let value = value - from_min;
    let value = value / from_range;
    let value = value * to_range;
    let value = value + to_min;
    let value = value.ease_calc(ease);
    value
}

pub trait EasingCalc<T> where 
    T: Copy + Add<Output = T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> {
    fn ease_calc(&self, f: EaseFunction) -> T;
}

impl EasingCalc<f32> for f32 {
    fn ease_calc(&self, f: EaseFunction) -> f32 {
        self.calc(f)
    }
}

impl EasingCalc<Vec3> for Vec3 {
    fn ease_calc(&self, f: EaseFunction) -> Vec3 {
        Vec3::new(self.x.ease_calc(f), self.y.calc(f), self.z.calc(f))
    }
}
 
// Test the map_range function
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_range_float() {
        assert_eq!(map_range(0.0, (0.0, 1.0), (0.0, 1.0), EaseFunction::Linear), 0.0);
        assert_eq!(map_range(0.5, (0.0, 1.0), (0.0, 1.0), EaseFunction::Linear), 0.5);
        assert_eq!(map_range(1.0, (0.0, 1.0), (0.0, 1.0), EaseFunction::Linear), 1.0);
        assert_eq!(map_range(0.0, (0.0, 1.0), (1.0, 2.0), EaseFunction::Linear), 1.0);
        assert_eq!(map_range(0.5, (0.0, 1.0), (1.0, 2.0), EaseFunction::Linear), 1.5);
        assert_eq!(map_range(1.0, (0.0, 1.0), (1.0, 2.0), EaseFunction::Linear), 2.0);
        assert_eq!(map_range(0.0, (1.0, 0.0), (0.0, 1.0), EaseFunction::Linear), 1.0);
        assert_eq!(map_range(0.5, (1.0, 0.0), (0.0, 1.0), EaseFunction::Linear), 0.5);
        assert_eq!(map_range(1.0, (1.0, 0.0), (0.0, 1.0), EaseFunction::Linear), 0.0);
        assert_eq!(map_range(0.0, (1.0, 0.0), (1.0, 2.0), EaseFunction::Linear), 2.0);
        assert_eq!(map_range(0.5, (1.0, 0.0), (1.0, 2.0), EaseFunction::Linear), 1.5);
        assert_eq!(map_range(1.0, (1.0, 0.0), (1.0, 2.0), EaseFunction::Linear), 1.0);
    }

    #[test]
    fn test_map_range_vec3() {
        assert_eq!(
            map_range(
                Vec3::new(0.0, 0.0, 0.0),
                (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
                (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
                EaseFunction::Linear
            ),
            Vec3::new(0.0, 0.0, 0.0)
        );
        assert_eq!(
            map_range(
                Vec3::new(0.5, 0.5, 0.5),
                (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
                (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
                EaseFunction::Linear
            ),
            Vec3::new(0.5, 0.5, 0.5)
        );
        assert_eq!(
            map_range(
                Vec3::new(1.0, 1.0, 1.0),
                (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
                (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
                EaseFunction::Linear
            ),
            Vec3::new(1.0, 1.0, 1.0)
        );
        assert_eq!(
            map_range(
                Vec3::new(0.0, 0.0, 0.0),
                (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 2.0)),
                EaseFunction::Linear
            ),
            Vec3::new(1.0, 1.0, 1.0)
        );
        assert_eq!(
            map_range(
                Vec3::new(0.5, 0.5, 0.5),
                (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 2.0)),
                EaseFunction::Linear
            ),
            Vec3::new(1.5, 1.5, 1.5)
        );
        assert_eq!(
            map_range(
                Vec3::new(1.0, 1.0, 1.0),
                (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 2.0)),
                EaseFunction::Linear
            ),
            Vec3::new(2.0, 2.0, 2.0)
        );
        assert_eq!(
            map_range(
                Vec3::new(0.0, 0.0, 0.0),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.0, 0.0, 0.0)),
                (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
                EaseFunction::Linear
            ),
            Vec3::new(1.0, 1.0, 1.0)
        );
        assert_eq!(
            map_range(
                Vec3::new(0.5, 0.5, 0.5),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.0, 0.0, 0.0)),
                (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
                EaseFunction::Linear
            ),
            Vec3::new(0.5, 0.5, 0.5)
        );
        assert_eq!(
            map_range(
                Vec3::new(1.0, 1.0, 1.0),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.0, 0.0, 0.0)),
                (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
                EaseFunction::Linear
            ),
            Vec3::new(0.0, 0.0, 0.0)
        );
        assert_eq!(
            map_range(
                Vec3::new(0.0, 0.0, 0.0),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.0, 0.0, 0.0)),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 2.0)),
                EaseFunction::Linear
            ),
            Vec3::new(2.0, 2.0, 2.0)
        );
        assert_eq!(
            map_range(
                Vec3::new(0.5, 0.5, 0.5),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.0, 0.0, 0.0)),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 2.0)),
                EaseFunction::Linear
            ),
            Vec3::new(1.5, 1.5, 1.5)
        );
        assert_eq!(
            map_range(
                Vec3::new(1.0, 1.0, 1.0),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.0, 0.0, 0.0)),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 2.0)),
                EaseFunction::Linear
            ),
            Vec3::new(1.0, 1.0, 1.0)
        );
    }

    #[test]
    fn test_map_range_vec3_negative() {
        assert_eq!(
            map_range(
                Vec3::new(-0.5, -0.5, -0.5),
                (Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0)),
                (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
                EaseFunction::Linear
            ),
            Vec3::new(0.25, 0.25, 0.25)
        );
        assert_eq!(
            map_range(
                Vec3::new(0.0, 0.0, 0.0),
                (Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0)),
                (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
                EaseFunction::Linear
            ),
            Vec3::new(0.5, 0.5, 0.5)
        );
        assert_eq!(
            map_range(
                Vec3::new(0.5, 0.5, 0.5),
                (Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0)),
                (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
                EaseFunction::Linear
            ),
            Vec3::new(0.75, 0.75, 0.75)
        );
        assert_eq!(
            map_range(
                Vec3::new(-0.5, -0.5, -0.5),
                (Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0)),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 2.0)),
                EaseFunction::Linear
            ),
            Vec3::new(1.25, 1.25, 1.25)
        );
        assert_eq!(
            map_range(
                Vec3::new(0.0, 0.0, 0.0),
                (Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0)),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 2.0)),
                EaseFunction::Linear
            ),
            Vec3::new(1.5, 1.5, 1.5)
        );
        assert_eq!(
            map_range(
                Vec3::new(0.5, 0.5, 0.5),
                (Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0)),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 2.0)),
                EaseFunction::Linear
            ),
            Vec3::new(1.75, 1.75, 1.75)
        );
        assert_eq!(
            map_range(
                Vec3::new(-0.5, -0.5, -0.5),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(-1.0, -1.0, -1.0)),
                (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
                EaseFunction::Linear
            ),
            Vec3::new(0.75, 0.75, 0.75)
        );
        assert_eq!(
            map_range(
                Vec3::new(0.0, 0.0, 0.0),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(-1.0, -1.0, -1.0)),
                (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
                EaseFunction::Linear
            ),
            Vec3::new(0.5, 0.5, 0.5)
        );
        assert_eq!(
            map_range(
                Vec3::new(0.5, 0.5, 0.5),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(-1.0, -1.0, -1.0)),
                (Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)),
                EaseFunction::Linear
            ),
            Vec3::new(0.25, 0.25, 0.25)
        );
        assert_eq!(
            map_range(
                Vec3::new(-0.5, -0.5, -0.5),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(-1.0, -1.0, -1.0)),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 2.0)),
                EaseFunction::Linear
            ),
            Vec3::new(1.75, 1.75, 1.75)
        );
        assert_eq!(
            map_range(
                Vec3::new(0.0, 0.0, 0.0),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(-1.0, -1.0, -1.0)),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 2.0)),
                EaseFunction::Linear
            ),
            Vec3::new(1.5, 1.5, 1.5)
        );
        assert_eq!(
            map_range(
                Vec3::new(0.5, 0.5, 0.5),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(-1.0, -1.0, -1.0)),
                (Vec3::new(1.0, 1.0, 1.0), Vec3::new(2.0, 2.0, 2.0)),
                EaseFunction::Linear
            ),
            Vec3::new(1.25, 1.25, 1.25)
        );
    }
}
