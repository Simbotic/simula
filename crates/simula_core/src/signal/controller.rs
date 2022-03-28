use bevy::prelude::*;
use std::time::Duration;

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct SignalController<Sample: Default + Reflect + PartialEq> {
    /// proportional gain
    pub kp: Sample,
    /// integral gain
    pub ki: Sample,
    /// derivative gain
    pub kd: Sample,
    /// setpoint
    pub setpoint: Sample,
    /// error to determine pi gain
    pub last_error: Sample,
    /// accumulated error
    pub integral: Sample,
    /// manipulated variable
    pub output: Sample,
    /// process variable
    pub pv: Sample,
}

impl<Sample> SignalController<Sample>
where
    Sample: Copy
        + Default
        + Reflect
        + PartialEq
        + std::ops::Add<Output = Sample>
        + std::ops::Sub<Output = Sample>
        + std::ops::Mul<Output = Sample>
        + std::ops::Div<Output = Sample>
        + AsSample<Sample>,
{
    pub fn control(&mut self, setpoint: Sample, pv: Sample, dt: Duration) -> Sample {
        let dt = dt.as_secs_f32();
        if dt > 0.0 {
            let dt: Sample = Sample::from(dt);
            self.setpoint = setpoint;
            let error = setpoint - pv;
            let p = error;
            self.integral = self.integral + error * dt;
            let i = self.integral;
            let d = (error - self.last_error) / dt;
            self.last_error = error;
            self.output = self.kp * p + self.ki * i + self.kd * d;
            self.pv = pv + self.output * dt;
            self.output
        } else {
            Sample::from(0.0)
        }
    }
}

pub trait AsSample<Sample> {
    fn from(src: f32) -> Sample;
}

impl AsSample<f32> for f32 {
    fn from(src: f32) -> f32 {
        src
    }
}

impl AsSample<Vec3> for Vec3 {
    fn from(src: f32) -> Vec3 {
        Vec3::new(src, src, src)
    }
}

#[test]
fn test_types_compile() {
    let dt: Duration = Duration::from_millis(16000);
    let mut s_f32 = SignalController::<f32>::default();
    let mut s_vec3 = SignalController::<Vec3>::default();
    let _c = s_f32.control(0.0, 0.0, dt);
    let _c = s_vec3.control(Vec3::ZERO, Vec3::ZERO, dt);
}
