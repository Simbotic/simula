//! A simple plugin and components for 2d/3d flying cameras in Bevy.
//!
//! # 3D
//!
//! Movement system is based on Minecraft, flying along the horizontal plane no matter the mouse's vertical angle, with two extra buttons for moving vertically.
//!
//! Default keybinds are:
//! - <kbd>W</kbd> / <kbd>A</kbd> / <kbd>S</kbd> / <kbd>D</kbd> - Move along the horizontal plane
//! - Shift - Move downward
//! - Space - Move upward
//!
//! ## Example
//! ```no_compile
//! use bevy::prelude::*;
//! use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
//!
//! fn setup(commands: &mut Commands) {
//!	  commands
//!     .spawn(Camera3dBundle::default())
//!     .with(FlyCamera::default());
//! }
//!
//! fn main() {
//!	  App::build()
//!     .add_plugins(DefaultPlugins)
//!     .add_startup_system(setup)
//!     .add_plugin(FlyCameraPlugin)
//!     .run();
//! }
//! ```
//!

use bevy::{
    prelude::*,
};
use simula_action::{
    action_axis_map, action_map, Action, ActionAxis, ActionAxisMap, ActionMap, ActionMapInput,
    AxisMapInput, AxisMapSource, MouseAxis,
};
use bevy::reflect::FromReflect;

/// A set of options for initializing a FlyCamera.
/// Attach this component to a [`Camera3dBundle`](https://docs.rs/bevy/0.4.0/bevy/prelude/struct.Camera3dBundle.html) bundle to control it with your mouse and keyboard.
/// # Example
/// ```no_compile
/// fn setup(mut commands: Commands) {
///	  commands
///     .spawn(Camera3dBundle::default())
///     .with(FlyCamera::default());
/// }
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FlyCamera {
    /// The speed the FlyCamera accelerates at. Defaults to `1.0`
    pub accel: f32,
    /// The maximum speed the FlyCamera can move at. Defaults to `0.5`
    pub max_speed: f32,
    /// The sensitivity of the FlyCamera's motion based on mouse movement. Defaults to `3.0`
    pub sensitivity: f32,
    /// The amount of deceleration to apply to the camera's motion. Defaults to `1.0`
    pub friction: f32,
    /// The current pitch of the FlyCamera in degrees. This value is always up-to-date, enforced by [FlyCameraPlugin](struct.FlyCameraPlugin.html)
    pub pitch: f32,
    /// The current pitch of the FlyCamera in degrees. This value is always up-to-date, enforced by [FlyCameraPlugin](struct.FlyCameraPlugin.html)
    pub yaw: f32,
    /// The current velocity of the FlyCamera. This value is always up-to-date, enforced by [FlyCameraPlugin](struct.FlyCameraPlugin.html)
    pub velocity: Vec3,
    /// Key used to move forward. Defaults to <kbd>W</kbd>
    #[reflect(ignore)]
    pub key_forward: KeyCode,
    /// Key used to move backward. Defaults to <kbd>S</kbd>
    #[reflect(ignore)]
    pub key_backward: KeyCode,
    /// Key used to move left. Defaults to <kbd>A</kbd>
    #[reflect(ignore)]
    pub key_left: KeyCode,
    /// Key used to move right. Defaults to <kbd>D</kbd>
    #[reflect(ignore)]
    pub key_right: KeyCode,
    /// Key used to move up. Defaults to <kbd>Space</kbd>
    #[reflect(ignore)]
    pub key_up: KeyCode,
    /// Key used to move forward. Defaults to <kbd>LShift</kbd>
    #[reflect(ignore)]
    pub key_down: KeyCode,
    /// If `false`, disable keyboard control of the camera. Defaults to `true`
    pub enabled: bool,
    /// If `false`, disable mouse look control of the camera. Defaults to `false`
    pub look_enabled: bool,
}
impl Default for FlyCamera {
    fn default() -> Self {
        Self {
            accel: 1.2,
            max_speed: 0.5,
            sensitivity: 90.0,
            friction: 1.0,
            pitch: 0.0,
            yaw: 0.0,
            velocity: Vec3::ZERO,
            key_forward: KeyCode::W,
            key_backward: KeyCode::S,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            key_up: KeyCode::E,
            key_down: KeyCode::Q,
            enabled: false,
            look_enabled: false,
        }
    }
}

fn forward_vector(rotation: &Quat) -> Vec3 {
    rotation.mul_vec3(Vec3::Z).normalize()
}

fn forward_walk_vector(rotation: &Quat) -> Vec3 {
    let f = forward_vector(rotation);
    let f_flattened = Vec3::new(f.x, 0.0, f.z).normalize();
    f_flattened
}

fn strafe_vector(rotation: &Quat) -> Vec3 {
    // Rotate it 90 degrees to get the strafe direction
    Quat::from_rotation_y(90.0f32.to_radians())
        .mul_vec3(forward_walk_vector(rotation))
        .normalize()
}

pub struct FlyCameraPlugin;
impl FlyCameraPlugin {

    fn camera_motion(
        time: Res<Time>,
        mut query: Query<(
            &mut FlyCamera,
            &mut Transform,
            &mut Camera,
            &mut Action<FlyCameraMode>,
            &mut ActionAxis<FlyCameraMotion>,
        )>,
    ) {
        for (mut camera, mut transform, _camera, mut mode, mut motion) in query.iter_mut() {
            if camera.enabled {
                if mode.on(FlyCameraMode::Rotate) {
                    //Get mouse/arrow motion
                    let x = motion.get(FlyCameraMotion::Right).unwrap_or_default();
                    let y = motion.get(FlyCameraMotion::Up).unwrap_or_default();
                    let delta = Vec2::new(x, y);

                    //Convert to camera angle (degree)
                    camera.yaw -= delta.x * camera.sensitivity * time.delta_seconds();
                    camera.pitch -= delta.y * camera.sensitivity * time.delta_seconds();
                    camera.pitch = camera.pitch.clamp(-89.0, 89.9);

                    //Convert degree to radian
                    let yaw_radians = camera.yaw.to_radians();
                    let pitch_radians = camera.pitch.to_radians();

                    //Rotate camera
                    transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw_radians)
                                * Quat::from_axis_angle(Vec3::X, pitch_radians);
                }

                if mode.on(FlyCameraMode::Pan) {
                    //Get WASD button pressed
                    let (axis_h, axis_v, axis_float) = if camera.enabled {
                        (
                        motion.get(FlyCameraMotion::Right).unwrap_or_default(),
                        motion.get(FlyCameraMotion::Forward).unwrap_or_default(),
                        motion.get(FlyCameraMotion::Up).unwrap_or_default()
                        )
                    } else {
                        (0.0, 0.0, 0.0)
                    };

                    //Calculate corresponding vector
                    let accel_vector: Vec3 = (strafe_vector(&transform.rotation) * -axis_h)
                    + (forward_walk_vector(&transform.rotation) * -axis_v)
                    + (Vec3::Y * axis_float);

                    //Apply acceleration
                    let accel: Vec3 = if accel_vector.length() != 0.0 {
                        accel_vector.normalize() * camera.accel
                    } else {
                        Vec3::ZERO
                    };

                    //Calculate camera velocity
                    camera.velocity += accel * time.delta_seconds();
                }

                //Calculate movement friction
                let friction: Vec3 = if camera.velocity.length() != 0.0 {
                    camera.velocity.normalize() * -1.0 * camera.friction
                } else {
                    Vec3::ZERO
                };

                //Clamp within max speed
                if camera.velocity.length() > camera.max_speed {
                    camera.velocity = camera.velocity.normalize() * camera.max_speed;
                }

                //Apply friction
                let delta_friction = friction * time.delta_seconds();
                camera.velocity =
                    if (camera.velocity + delta_friction).signum() != camera.velocity.signum() {
                        Vec3::ZERO
                    } else {
                        camera.velocity + delta_friction
                    };
                
                //Move camera    
                transform.translation += camera.velocity;
            }
            mode.clear();
            motion.clear();
        }
    }
}

impl Plugin for FlyCameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FlyCamera>()
            .add_system(setup)
            .add_system(action_map::<FlyCameraMode, FlyCamera>)
            .add_system(action_axis_map::<FlyCameraMotion, FlyCamera>)
            .add_system(Self::camera_motion);
    }
}

/**
Include this plugin to add the systems for the FlyCamera bundle.

```no_compile
fn main() {
    App::build().add_plugin(FlyCameraPlugin);
}
```

**/

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy, Reflect, FromReflect)]
pub enum FlyCameraMode {
    #[default]
    Idle,
    Rotate,
    Pan,
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy, Reflect, FromReflect)]
pub enum FlyCameraMotion {
    #[default]
    Idle,
    Right,
    Up,
    Forward
}

fn setup(
    mut commands: Commands,
    cameras: Query<
        Entity,
        (
            With<FlyCamera>,
            Without<ActionMap<FlyCameraMode>>,
            Without<ActionAxisMap<FlyCameraMotion>>,
        ),
    >,
) {
    for entity in cameras.iter() {
        let mut action_map = ActionMap::<FlyCameraMode>::default();
        *action_map = vec![
            // Rotate Mode
            ActionMapInput {
                action: FlyCameraMode::Rotate,
                button: KeyCode::Right.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            ActionMapInput {
                action: FlyCameraMode::Rotate,
                button: KeyCode::Left.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            ActionMapInput {
                action: FlyCameraMode::Rotate,
                button: KeyCode::Up.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            ActionMapInput {
                action: FlyCameraMode::Rotate,
                button: KeyCode::Down.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            ActionMapInput {
                action: FlyCameraMode::Rotate,
                button: MouseButton::Right.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            // Pan Mode
            ActionMapInput {
                action: FlyCameraMode::Pan,
                button: KeyCode::A.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            ActionMapInput {
                action: FlyCameraMode::Pan,
                button: KeyCode::D.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            ActionMapInput {
                action: FlyCameraMode::Pan,
                button: KeyCode::W.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            ActionMapInput {
                action: FlyCameraMode::Pan,
                button: KeyCode::S.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            ActionMapInput {
                action: FlyCameraMode::Pan,
                button: KeyCode::Q.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            ActionMapInput {
                action: FlyCameraMode::Pan,
                button: KeyCode::E.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
        ];

        let mut axis_map: ActionAxisMap<FlyCameraMotion> = Default::default();
        *axis_map = vec![
            // WASD
            AxisMapInput {
                axis: FlyCameraMotion::Up,
                source: AxisMapSource::Keyboard {
                    positive: KeyCode::Q.into(),
                    negative: KeyCode::E.into(),
                },
            },
            AxisMapInput {
                axis: FlyCameraMotion::Right,
                source: AxisMapSource::Keyboard {
                    positive: KeyCode::A.into(),
                    negative: KeyCode::D.into(),
                },
            },
            AxisMapInput {
                axis: FlyCameraMotion::Forward,
                source: AxisMapSource::Keyboard {
                    positive: KeyCode::W.into(),
                    negative: KeyCode::S.into(),
                },
            },
            // Arrows
            AxisMapInput {
                axis: FlyCameraMotion::Right,
                source: AxisMapSource::Keyboard {
                    positive: KeyCode::Right.into(),
                    negative: KeyCode::Left.into(),
                },
            },
            AxisMapInput {
                axis: FlyCameraMotion::Up,
                source: AxisMapSource::Keyboard {
                    positive: KeyCode::Down.into(),
                    negative: KeyCode::Up.into(),
                },
            },
            // Mouse X Y
            AxisMapInput {
                axis: FlyCameraMotion::Right,
                source: AxisMapSource::MouseAxis(MouseAxis::X),
            },
            AxisMapInput {
                axis: FlyCameraMotion::Up,
                source: AxisMapSource::MouseAxis(MouseAxis::Y),
            },
        ];

        commands
            .entity(entity)
            .insert(Action::<FlyCameraMode>::default())
            .insert(ActionAxis::<FlyCameraMotion>::default())
            .insert(action_map)
            .insert(axis_map);
    }
}
