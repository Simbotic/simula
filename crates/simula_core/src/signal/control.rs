use bevy::prelude::*;
use std::time::Duration;

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Control<Sample: Default + Reflect + PartialEq> {
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

// impl From<f32> for Vec3 {
//     fn from(self) -> Vec3 {
//         Vec3::new(self, self, self)
//     }
// }

// impl Into<Vec3> for f32 {

// }

pub fn control<Sample>(
    control: &mut Control<Sample>,
    setpoint: Sample,
    pv: Sample,
    dt: Duration,
) -> Sample
where
    Sample: Copy
        + Default
        + Reflect
        + PartialEq
        + std::ops::Add<Output = Sample>
        + std::ops::Sub<Output = Sample>
        + std::ops::Mul<Output = Sample>
        + std::ops::Div<Output = Sample>
        + From<f32>,
{
    let dt: Sample = dt.as_secs_f32().into();

    control.setpoint = setpoint;
    control.pv = pv;

    let error = setpoint - pv;

    let p = error;
    control.integral = control.integral + error * dt;
    let d = (error - control.last_error)  / dt;

    control.output
}

fn test(dt: Duration) {
    let mut s_f32 = Control::<f32>::default();
    let mut s_vec3 = Control::<Vec3>::default();

    let c = control(&mut s_f32, 0.0, 0.0, dt);
    // let c = control(&mut s_vec3, Vec3::ZERO, Vec3::ZERO, dt);
}