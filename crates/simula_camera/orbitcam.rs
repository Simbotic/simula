//! An orbit controls plugin for bevy.
//!
//! To control the camera, drag the mouse. The left button rotates. The wheel
//! zooms.
//!
//! ## Usage
//!
//! Register the [`OrbitCameraPlugin`], and insert the [`OrbitCamera`] struct
//! into the entity containing the camera.
//!
//! For example, within the startup system:
//!
//! ```no_compile
//! commands
//!     .spawn_bundle(PerspectiveCameraBundle {
//!         transform: Transform::from_translation(Vec3::new(-3.0, 3.0, 5.0))
//!             .looking_at(Vec3::default(), Vec3::Y),
//!         ..Default::default()
//!     })
//!     .insert(OrbitCamera::default());
//! ```
//!
//! ## Compatibility
//!
//! - `v2.x` – Bevy `0.5`.
//! - `v1.x` – Bevy `0.4`.

use bevy::input::mouse::MouseMotion;
use bevy::input::mouse::MouseScrollUnit::{Line, Pixel};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use std::ops::RangeInclusive;

const LINE_TO_PIXEL_RATIO: f32 = 0.1;

pub enum CameraEvents {
    Orbit(Vec2),
    Pan(Vec2),
    Zoom(f32),
}

pub struct OrbitCamera {
    pub x: f32,
    pub y: f32,
    pub pitch_range: RangeInclusive<f32>,
    pub distance: f32,
    pub center: Vec3,
    pub rotate_sensitivity: f32,
    pub pan_sensitivity: f32,
    pub zoom_sensitivity: f32,
    pub rotate_button: MouseButton,
    pub pan_button: MouseButton,
    pub enabled: bool,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        OrbitCamera {
            x: 0.0,
            y: std::f32::consts::FRAC_PI_2,
            pitch_range: 0.01..=3.13,
            distance: 5.0,
            center: Vec3::ZERO,
            rotate_sensitivity: 1.0,
            pan_sensitivity: 1.0,
            zoom_sensitivity: 0.8,
            rotate_button: MouseButton::Left,
            pan_button: MouseButton::Right,
            enabled: true,
        }
    }
}

impl OrbitCamera {
    pub fn new(dist: f32, center: Vec3) -> OrbitCamera {
        OrbitCamera {
            distance: dist,
            center,
            ..Self::default()
        }
    }
}

pub struct OrbitCameraPlugin;
impl OrbitCameraPlugin {
    pub fn update_transform_system(
        mut query: Query<(&OrbitCamera, &mut Transform), (Changed<OrbitCamera>, With<Camera>)>,
    ) {
        for (camera, mut transform) in query.iter_mut() {
            if camera.enabled {
                let rot = Quat::from_axis_angle(Vec3::Y, camera.x)
                    * Quat::from_axis_angle(-Vec3::X, camera.y);
                transform.translation = (rot * Vec3::Y) * camera.distance + camera.center;
                transform.look_at(camera.center, Vec3::Y);
            }
        }
    }

    pub fn emit_motion_events(
        mut events: EventWriter<CameraEvents>,
        mut mouse_motion_events: EventReader<MouseMotion>,
        mouse_button_input: Res<Input<MouseButton>>,
        mut query: Query<&OrbitCamera>,
    ) {
        let mut delta = Vec2::ZERO;
        for event in mouse_motion_events.iter() {
            delta += event.delta;
        }
        for camera in query.iter_mut() {
            if camera.enabled {
                if mouse_button_input.pressed(camera.rotate_button) {
                    events.send(CameraEvents::Orbit(delta))
                }

                if mouse_button_input.pressed(camera.pan_button) {
                    events.send(CameraEvents::Pan(delta))
                }
            }
        }
    }

    pub fn mouse_motion_system(
        time: Res<Time>,
        mut events: EventReader<CameraEvents>,
        mut query: Query<(&mut OrbitCamera, &mut Transform, &mut Camera)>,
    ) {
        for (mut camera, transform, _) in query.iter_mut() {
            if !camera.enabled {
                continue;
            }

            for event in events.iter() {
                match event {
                    CameraEvents::Orbit(delta) => {
                        camera.x -= delta.x * camera.rotate_sensitivity * time.delta_seconds();
                        camera.y -= delta.y * camera.rotate_sensitivity * time.delta_seconds();
                        camera.y = camera
                            .y
                            .max(*camera.pitch_range.start())
                            .min(*camera.pitch_range.end());
                    }
                    CameraEvents::Pan(delta) => {
                        let right_dir = transform.rotation * -Vec3::X;
                        let up_dir = transform.rotation * Vec3::Y;
                        let pan_vector = (delta.x * right_dir + delta.y * up_dir)
                            * camera.pan_sensitivity
                            * time.delta_seconds();
                        camera.center += pan_vector;
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn emit_zoom_events(
        mut events: EventWriter<CameraEvents>,
        mut mouse_wheel_events: EventReader<MouseWheel>,
        mut query: Query<&OrbitCamera>,
    ) {
        let mut total = 0.0;
        for event in mouse_wheel_events.iter() {
            total += event.y
                * match event.unit {
                    Line => 1.0,
                    Pixel => LINE_TO_PIXEL_RATIO,
                };
        }

        if total != 0.0 {
            for camera in query.iter_mut() {
                if camera.enabled {
                    events.send(CameraEvents::Zoom(total));
                }
            }
        }
    }

    pub fn zoom_system(
        mut query: Query<&mut OrbitCamera, With<Camera>>,
        mut events: EventReader<CameraEvents>,
    ) {
        for mut camera in query.iter_mut() {
            for event in events.iter() {
                if camera.enabled {
                    if let CameraEvents::Zoom(distance) = event {
                        camera.distance *= camera.zoom_sensitivity.powf(*distance);
                    }
                }
            }
        }
    }
}
impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(Self::emit_motion_events.system())
            .add_system(Self::mouse_motion_system.system())
            .add_system(Self::emit_zoom_events.system())
            .add_system(Self::zoom_system.system())
            .add_system(Self::update_transform_system.system())
            .add_event::<CameraEvents>();
    }
}
