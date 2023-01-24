use crate::prng::*;
use bevy::prelude::*;
use enum_iterator::Sequence;
use std::fmt::Display;
use std::{f32::consts::PI, time::Duration};

#[derive(Reflect, Clone, Sequence, Display)]
pub enum SignalFunction {
    Identity,
    Sine,
    Square,
    Triangle,
    Sawtooth,
    Pulse,
    WhiteNoise,
    GaussNoise,
    DigitalNoise,
}

impl Default for SignalFunction {
    fn default() -> Self {
        SignalFunction::Identity
    }
}

type Sample = f32;

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct SignalGenerator {
    pub func: SignalFunction,
    pub frequency: Sample,
    pub phase: Sample,
    pub amplitude: Sample,
    pub offset: Sample,
    pub invert: bool,
    pub seed: Sample,
    #[reflect(ignore)]
    pub rng: Prng,
}

impl Default for SignalGenerator {
    fn default() -> Self {
        let seed = rand::random();
        SignalGenerator {
            func: SignalFunction::default(),
            frequency: Sample::default(),
            phase: Sample::default(),
            amplitude: Sample::default(),
            offset: Sample::default(),
            invert: bool::default(),
            seed,
            rng: Prng::new(seed as u64),
        }
    }
}

impl SignalGenerator {
    pub fn sample(&mut self, time: Duration) -> Sample {
        let time = self.frequency * time.as_secs_f32() + self.phase;
        let sample = match self.func {
            SignalFunction::Identity => time,
            SignalFunction::Sine => (2.0 * PI * time).sin(),
            SignalFunction::Square => (2.0 * PI * time).sin().signum(),
            SignalFunction::Triangle => 1.0 - 4.0 * ((time - 0.25).round() - (time - 0.25)).abs(),
            SignalFunction::Sawtooth => 2.0 * (time - (time + 0.5).floor()),
            SignalFunction::Pulse => {
                if (2.0 * PI * time).sin().abs() < 1.0 - 10e-3 {
                    0.0
                } else {
                    1.0
                }
            }
            SignalFunction::WhiteNoise => self.rng.rand_float() * 2.0 - 1.0,
            SignalFunction::GaussNoise => norm_inv(self.rng.rand_float(), 0.0, 0.4),
            SignalFunction::DigitalNoise => self.rng.rand_float().round(),
        };
        let invert = if self.invert { -1.0 } else { 1.0 };
        invert * sample * self.amplitude + self.offset
    }
}

//
// Lower tail quantile for standard normal distribution function.
//
// This function returns an approximation of the inverse cumulative
// standard normal distribution function.  I.e., given P, it returns
// an approximation to the X satisfying P = Pr{Z <= X} where Z is a
// random variable from the standard normal distribution.
//
// The algorithm uses a minimax approximation by rational functions
// and the result has a relative error whose absolute value is less
// than 1.15e-9.
//
// Author:      Peter J. Acklam
// E-mail:      pjacklam@gmail.com
// WWW URL:     https://github.com/pjacklam

// An algorithm with a relative error less than 1.15*10-9 in the entire region.

fn norm_sinv(p: Sample) -> Sample {
    // Coefficients in rational approximations
    let a = [
        -39.696830f32,
        220.946098f32,
        -275.928510f32,
        138.357751f32,
        -30.664798f32,
        2.506628f32,
    ];

    let b = [
        -54.476098f32,
        161.585836f32,
        -155.698979f32,
        66.801311f32,
        -13.280681f32,
    ];

    let c = [
        -0.007784894002f32,
        -0.32239645f32,
        -2.400758f32,
        -2.549732f32,
        4.374664f32,
        2.938163f32,
    ];

    let d = [0.007784695709f32, 0.32246712f32, 2.445134f32, 3.754408f32];

    // Define break-points.
    let plow = 0.02425f32;
    let phigh = 1.0 - plow;

    // Rational approximation for lower region:
    if p < plow {
        let q = (-2.0 * p.ln()).sqrt();
        return (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0);
    }

    // Rational approximation for upper region:
    if phigh < p {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        return -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0);
    }

    // Rational approximation for central region:
    {
        let q = p - 0.5;
        let r = q * q;
        return (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q
            / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0);
    }
}

fn norm_inv(probability: Sample, mean: Sample, standard_deviation: Sample) -> Sample {
    norm_sinv(probability) * standard_deviation + mean
}
