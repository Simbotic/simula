use bevy::prelude::Reflect;
use bevy_inspector_egui::Inspectable;
use enum_iterator::Sequence;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[allow(missing_docs)]
#[derive(
    Reflect, Debug, Copy, Clone, PartialEq, Serialize, Deserialize, Sequence, Display, Inspectable,
)]
pub enum EaseFunction {
    Linear,

    QuadraticIn,
    QuadraticOut,
    QuadraticInOut,

    CubicIn,
    CubicOut,
    CubicInOut,

    QuarticIn,
    QuarticOut,
    QuarticInOut,

    QuinticIn,
    QuinticOut,
    QuinticInOut,

    SineIn,
    SineOut,
    SineInOut,

    CircularIn,
    CircularOut,
    CircularInOut,

    ExponentialIn,
    ExponentialOut,
    ExponentialInOut,

    ElasticIn,
    ElasticOut,
    ElasticInOut,

    BackIn,
    BackOut,
    BackInOut,

    BounceIn,
    BounceOut,
    BounceInOut,
}

impl Default for EaseFunction {
    fn default() -> Self {
        EaseFunction::Linear
    }
}

#[allow(missing_docs)]
pub trait Ease {
    /// Calculate the eased value, normalized
    fn calc(self, f: EaseFunction) -> Self;

    fn linear(self) -> Self;

    fn quadratic_in(self) -> Self;
    fn quadratic_out(self) -> Self;
    fn quadratic_in_out(self) -> Self;

    fn cubic_in(self) -> Self;
    fn cubic_out(self) -> Self;
    fn cubic_in_out(self) -> Self;

    fn quartic_in(self) -> Self;
    fn quartic_out(self) -> Self;
    fn quartic_in_out(self) -> Self;

    fn quintic_in(self) -> Self;
    fn quintic_out(self) -> Self;
    fn quintic_in_out(self) -> Self;

    fn sine_in(self) -> Self;
    fn sine_out(self) -> Self;
    fn sine_in_out(self) -> Self;

    fn circular_in(self) -> Self;
    fn circular_out(self) -> Self;
    fn circular_in_out(self) -> Self;

    fn exponential_in(self) -> Self;
    fn exponential_out(self) -> Self;
    fn exponential_in_out(self) -> Self;

    fn elastic_in(self) -> Self;
    fn elastic_out(self) -> Self;
    fn elastic_in_out(self) -> Self;

    fn back_in(self) -> Self;
    fn back_out(self) -> Self;
    fn back_in_out(self) -> Self;

    fn bounce_in(self) -> Self;
    fn bounce_out(self) -> Self;
    fn bounce_in_out(self) -> Self;
}

macro_rules! impl_ease_trait_for {
    ($T: ident) => {
        mod $T {
            pub const PI_2: $T = 6.28318530717958647692528676655900576;

            pub fn clamp(p: $T) -> $T {
                match () {
                    _ if p > 1.0 => 1.0,
                    _ if p < 0.0 => 0.0,
                    _ => p,
                }
            }
        }
        impl Ease for $T {
            fn calc(self, f: EaseFunction) -> Self {
                match f {
                    EaseFunction::Linear => self.linear(),

                    EaseFunction::QuadraticIn => self.quadratic_in(),
                    EaseFunction::QuadraticOut => self.quadratic_out(),
                    EaseFunction::QuadraticInOut => self.quadratic_in_out(),

                    EaseFunction::CubicIn => self.cubic_in(),
                    EaseFunction::CubicOut => self.cubic_out(),
                    EaseFunction::CubicInOut => self.cubic_in_out(),

                    EaseFunction::QuarticIn => self.quartic_in(),
                    EaseFunction::QuarticOut => self.quartic_out(),
                    EaseFunction::QuarticInOut => self.quartic_in_out(),

                    EaseFunction::QuinticIn => self.quintic_in(),
                    EaseFunction::QuinticOut => self.quintic_out(),
                    EaseFunction::QuinticInOut => self.quintic_in_out(),

                    EaseFunction::SineIn => self.sine_in(),
                    EaseFunction::SineOut => self.sine_out(),
                    EaseFunction::SineInOut => self.sine_in_out(),

                    EaseFunction::CircularIn => self.circular_in(),
                    EaseFunction::CircularOut => self.circular_out(),
                    EaseFunction::CircularInOut => self.circular_in_out(),

                    EaseFunction::ExponentialIn => self.exponential_in(),
                    EaseFunction::ExponentialOut => self.exponential_out(),
                    EaseFunction::ExponentialInOut => self.exponential_in_out(),

                    EaseFunction::ElasticIn => self.elastic_in(),
                    EaseFunction::ElasticOut => self.elastic_out(),
                    EaseFunction::ElasticInOut => self.elastic_in_out(),

                    EaseFunction::BackIn => self.back_in(),
                    EaseFunction::BackOut => self.back_out(),
                    EaseFunction::BackInOut => self.back_in_out(),

                    EaseFunction::BounceIn => self.bounce_in(),
                    EaseFunction::BounceOut => self.bounce_out(),
                    EaseFunction::BounceInOut => self.bounce_in_out(),
                }
            }

            fn linear(self) -> Self {
                self
            }

            fn quadratic_in(self) -> Self {
                let p = $T::clamp(self);
                p * p
            }

            fn quadratic_out(self) -> Self {
                let p = $T::clamp(self);
                -(p * (p - 2.0))
            }

            fn quadratic_in_out(self) -> Self {
                let p = $T::clamp(self);
                if p < 0.5 {
                    2.0 * p * p
                } else {
                    (-2.0 * p * p) + (4.0 * p) - 1.0
                }
            }

            fn cubic_in(self) -> Self {
                let p = $T::clamp(self);
                p * p * p
            }

            fn cubic_out(self) -> Self {
                let p = $T::clamp(self);
                let f = p - 1.0;
                f * f * f + 1.0
            }

            fn cubic_in_out(self) -> Self {
                let p = $T::clamp(self);
                if p < 0.5 {
                    4.0 * p * p * p
                } else {
                    let f = (2.0 * p) - 2.0;
                    0.5 * f * f * f + 1.0
                }
            }

            fn quartic_in(self) -> Self {
                let p = $T::clamp(self);
                p * p * p * p
            }

            fn quartic_out(self) -> Self {
                let p = $T::clamp(self);
                let f = p - 1.0;
                f * f * f * (1.0 - p) + 1.0
            }

            fn quartic_in_out(self) -> Self {
                let p = $T::clamp(self);
                if p < 0.5 {
                    8.0 * p * p * p * p
                } else {
                    let f = p - 1.0;
                    -8.0 * f * f * f * f + 1.0
                }
            }

            fn quintic_in(self) -> Self {
                let p = $T::clamp(self);
                p * p * p * p * p
            }

            fn quintic_out(self) -> Self {
                let p = $T::clamp(self);
                let f = p - 1.0;
                f * f * f * f * f + 1.0
            }

            fn quintic_in_out(self) -> Self {
                let p = $T::clamp(self);
                if p < 0.5 {
                    16.0 * p * p * p * p * p
                } else {
                    let f = (2.0 * p) - 2.0;
                    0.5 * f * f * f * f * f + 1.0
                }
            }

            fn sine_in(self) -> Self {
                use std::$T::consts::PI;
                let p = $T::clamp(self);
                1.0 - (PI / 2.0 * p).cos()
            }

            fn sine_out(self) -> Self {
                use self::$T::PI_2;
                let p = $T::clamp(self);
                (p * PI_2 / 4.0).sin()
            }

            fn sine_in_out(self) -> Self {
                use std::$T::consts::PI;
                let p = $T::clamp(self);
                0.5 * (1.0 - (p * PI).cos())
            }

            fn circular_in(self) -> Self {
                let p = $T::clamp(self);
                1.0 - (1.0 - (p * p)).sqrt()
            }

            fn circular_out(self) -> Self {
                let p = $T::clamp(self);
                ((2.0 - p) * p).sqrt()
            }

            fn circular_in_out(self) -> Self {
                let p = $T::clamp(self);
                if p < 0.5 {
                    0.5 * (1.0 - (1.0 - 4.0 * (p * p)).sqrt())
                } else {
                    0.5 * ((-((2.0 * p) - 3.0) * ((2.0 * p) - 1.0)).sqrt() + 1.0)
                }
            }

            fn exponential_in(self) -> Self {
                if self <= 0.0 {
                    0.0
                } else {
                    (2.0 as $T).powf(10.0 * self - 10.0)
                }
            }

            fn exponential_out(self) -> Self {
                if self >= 1.0 {
                    1.0
                } else {
                    1.0 - (2.0 as $T).powf(-10.0 * self)
                }
            }

            fn exponential_in_out(self) -> Self {
                if self <= 0.0 {
                    return 0.0;
                }
                if self >= 1.0 {
                    return 1.0;
                }

                if self < 0.5 {
                    (2.0 as $T).powf((20.0 * self) - 10.0) / 2.0
                } else {
                    (2.0 - (2.0 as $T).powf((-20.0 * self) + 10.0)) / 2.0
                }
            }

            fn elastic_in(self) -> Self {
                use self::$T::PI_2;
                let p = $T::clamp(self);
                if p == 0.0 {
                    return 0.0;
                }
                if p == 1.0 {
                    return 1.0;
                }
                ((p * 10.0 - 10.75) * (PI_2 / 3.0)).sin() * -(2.0 as $T).powf(10.0 * p - 10.0)
            }

            fn elastic_out(self) -> Self {
                use self::$T::PI_2;
                let p = $T::clamp(self);
                if p == 0.0 {
                    return 0.0;
                }
                if p == 1.0 {
                    return 1.0;
                }
                ((p * 10.0 - 0.75) * (PI_2 / 3.0)).sin() * (2.0 as $T).powf(-10.0 * p) + 1.0
            }

            fn elastic_in_out(self) -> Self {
                use self::$T::PI_2;
                let p = $T::clamp(self);
                if p == 0.0 {
                    return 0.0;
                }
                if p == 1.0 {
                    return 1.0;
                }
                if p < 0.5 {
                    ((20.0 * p - 11.125) * PI_2 / 4.5).sin() * -(2.0 as $T).powf(20.0 * p - 10.0)
                        / 2.0
                } else {
                    (((20.0 * p - 11.125) * PI_2 / 4.5).sin() * (2.0 as $T).powf(-20.0 * p + 10.0))
                        / 2.0
                        + 1.0
                }
            }

            fn back_in(self) -> Self {
                let p = $T::clamp(self);
                (1.70158 + 1.0) * p * p * p - 1.70158 * p * p
            }

            fn back_out(self) -> Self {
                let p = $T::clamp(self);
                1.0 + (1.70158 + 1.0) * ((p - 1.0) as $T).powf(3.0)
                    + 1.70158 * ((p - 1.0) as $T).powf(2.0)
            }

            fn back_in_out(self) -> Self {
                let p = $T::clamp(self);
                if p < 0.5 {
                    (((2.0 * p) as $T).powf(2.0)
                        * (((1.70158 * 1.525) + 1.0) * 2.0 * p - (1.70158 * 1.525)))
                        / 2.0
                } else {
                    (((2.0 * p - 2.0) as $T).powf(2.0)
                        * (((1.70158 * 1.525) + 1.0) * (p * 2.0 - 2.0) + (1.70158 * 1.525))
                        + 2.0)
                        / 2.0
                }
            }

            fn bounce_in(self) -> Self {
                let p = $T::clamp(self);
                1.0 - Ease::bounce_out(1.0 - p)
            }

            fn bounce_out(self) -> Self {
                let p = $T::clamp(self);
                if p < 4.0 / 11.0 {
                    (121.0 * p * p) / 16.0
                } else if p < 8.0 / 11.0 {
                    (363.0 / 40.0 * p * p) - (99.0 / 10.0 * p) + 17.0 / 5.0
                } else if p < 9.0 / 10.0 {
                    (4356.0 / 361.0 * p * p) - (35442.0 / 1805.0 * p) + 16061.0 / 1805.0
                } else {
                    (54.0 / 5.0 * p * p) - (513.0 / 25.0 * p) + 268.0 / 25.0
                }
            }

            fn bounce_in_out(self) -> Self {
                let p = $T::clamp(self);
                if p < 0.5 {
                    0.5 * Ease::bounce_in(p * 2.0)
                } else {
                    0.5 * Ease::bounce_out(p * 2.0 - 1.0) + 0.5
                }
            }
        }
    };
}

impl_ease_trait_for!(f32);
impl_ease_trait_for!(f64);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_linear() {
        assert_eq!(0.0, Ease::linear(0.0));
        assert_eq!(0.5, Ease::linear(0.5));
        assert_eq!(1.0, Ease::linear(1.0));
    }

    #[test]
    fn test_quadratic_in() {
        assert_eq!(0.0, Ease::quadratic_in(0.0));
        assert_eq!(0.25, Ease::quadratic_in(0.5));
        assert_eq!(1.0, Ease::quadratic_in(1.0));
    }

    #[test]
    fn test_quadratic_out() {
        assert_eq!(0.0, Ease::quadratic_out(0.0));
        assert_eq!(0.75, Ease::quadratic_out(0.5));
        assert_eq!(1.0, Ease::quadratic_out(1.0));
    }

    #[test]
    fn test_quadratic_in_out() {
        assert_eq!(0.0, Ease::quadratic_in_out(0.0));
        assert_eq!(0.32000000000000006, Ease::quadratic_in_out(0.4));
        assert_eq!(1.0, Ease::quadratic_in_out(1.0));
    }

    #[test]
    fn test_cubic_in() {
        assert_eq!(0.0, Ease::cubic_in(0.0));
        assert_eq!(0.125, Ease::cubic_in(0.5));
        assert_eq!(1.0, Ease::cubic_in(1.0));
    }

    #[test]
    fn test_cubic_out() {
        assert_eq!(0.0, Ease::cubic_out(0.0));
        assert_eq!(0.875, Ease::cubic_out(0.5));
        assert_eq!(1.0, Ease::cubic_out(1.0));
    }

    #[test]
    fn test_cubic_in_out() {
        assert_eq!(0.0, Ease::cubic_in_out(0.0));
        assert_eq!(0.5, Ease::cubic_in_out(0.5));
        assert_eq!(1.0, Ease::cubic_in_out(1.0));
    }

    #[test]
    fn test_quartic_in() {
        assert_eq!(0.0, Ease::quartic_in(0.0));
        assert_eq!(0.0625, Ease::quartic_in(0.5));
        assert_eq!(1.0, Ease::quartic_in(1.0));
    }

    #[test]
    fn test_quartic_out() {
        assert_eq!(0.0, Ease::quartic_out(0.0));
        assert_eq!(0.9375, Ease::quartic_out(0.5));
        assert_eq!(1.0, Ease::quartic_out(1.0));
    }

    #[test]
    fn test_quartic_in_out() {
        assert_eq!(0.0, Ease::quartic_in_out(0.0));
        assert_eq!(0.5, Ease::quartic_in_out(0.5));
        assert_eq!(1.0, Ease::quartic_in_out(1.0));
    }

    #[test]
    fn test_quintic_in() {
        assert_eq!(0.0, Ease::quintic_in(0.0));
        assert_eq!(0.03125, Ease::quintic_in(0.5));
        assert_eq!(1.0, Ease::quintic_in(1.0));
    }

    #[test]
    fn test_quintic_out() {
        assert_eq!(0.0, Ease::quintic_out(0.0));
        assert_eq!(0.96875, Ease::quintic_out(0.5));
        assert_eq!(1.0, Ease::quintic_out(1.0));
    }

    #[test]
    fn test_quintic_in_out() {
        assert_eq!(0.0, Ease::quintic_in_out(0.0));
        assert_eq!(0.5, Ease::quintic_in_out(0.5));
        assert_eq!(1.0, Ease::quintic_in_out(1.0));
    }

    #[test]
    fn test_elastic_in() {
        assert_eq!(0.0, Ease::elastic_in(0.0));
        assert_eq!(1.0, Ease::elastic_in(1.0));
    }

    #[test]
    fn test_elastic_out() {
        assert_eq!(0.0, Ease::elastic_out(0.0));
        assert_eq!(1.015625, Ease::elastic_out(0.5));
        assert_eq!(1.0, Ease::elastic_out(1.0));
    }

    #[test]
    fn test_elastic_in_out() {
        assert_eq!(0.0, Ease::elastic_in_out(0.0));
        assert_eq!(-0.11746157759823855, Ease::elastic_in_out(0.4));
        assert_eq!(1.0, Ease::elastic_in_out(1.0));
    }

    #[test]
    fn test_expo_in() {
        assert_eq!(0.0, Ease::exponential_in(0.0));
        assert_eq!(0.03125, Ease::exponential_in(0.5));
        assert_eq!(1.0, Ease::exponential_in(1.0));
    }

    #[test]
    fn test_expo_out() {
        assert_eq!(0.0, Ease::exponential_out(0.0));
        assert_eq!(0.96875, Ease::exponential_out(0.5));
        assert_eq!(1.0, Ease::exponential_out(1.0));
    }

    #[test]
    fn test_expo_in_out() {
        assert_eq!(0.0, Ease::exponential_in_out(0.0));
        assert_eq!(0.875, Ease::exponential_in_out(0.6));
        assert_eq!(1.0, Ease::exponential_in_out(1.0));
    }

    #[test]
    fn test_back_in() {
        assert_eq!(0.0, Ease::back_in(0.0));
        assert_eq!(-0.08769750000000004, Ease::back_in(0.5));
        assert_eq!(0.9999999999999998, Ease::back_in(1.0));
    }

    #[test]
    fn test_back_out() {
        assert_eq!(2.220446049250313e-16, Ease::back_out(0.0));
        assert_eq!(1.0876975, Ease::back_out(0.5));
        assert_eq!(1.0, Ease::back_out(1.0));
    }

    #[test]
    fn test_back_in_out() {
        assert_eq!(0.0, Ease::back_in_out(0.0));
        assert_eq!(0.0899257920000001, Ease::back_in_out(0.4));
        assert_eq!(1.0, Ease::back_in_out(1.0));
    }

    #[test]
    fn test_sine_out() {
        assert_eq!(0.0, Ease::sine_out(0.0));
        assert_eq!(0.7071067811865475, Ease::sine_out(0.5));
        assert_eq!(1.0, Ease::sine_out(1.0));
    }
}
