use bevy::prelude::*;
use bevy::reflect::FromReflect;
use bevy::render::camera::Camera;
use simula_action::{
    action_axis_map, action_map,
    touch_sides::{TouchAxis, TouchSide, TouchSideAxis},
    Action, ActionAxis, ActionAxisMap, ActionMap, ActionMapInput, AxisMapInput, AxisMapSource,
    MouseAxis,
};
use std::ops::RangeInclusive;

pub enum CameraEvents {
    Orbit(Vec2),
    Pan(Vec2),
    Zoom(f32),
}

pub struct OrbitCameraAction;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct OrbitCamera {
    pub x: f32,
    pub y: f32,
    #[reflect(ignore)]
    pub pitch_range: RangeInclusive<f32>,
    pub distance: f32,
    pub center: Vec3,
    pub rotate_sensitivity: f32,
    pub pan_sensitivity: f32,
    pub zoom_sensitivity: f32,
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
            rotate_sensitivity: 10.0,
            pan_sensitivity: 10.0,
            zoom_sensitivity: 0.8,
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
    fn camera_update(
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

    fn camera_motion(
        time: Res<Time>,
        mut query: Query<(
            &mut OrbitCamera,
            &mut Transform,
            &mut Camera,
            &mut Action<OrbitCameraMode>,
            &mut ActionAxis<OrbitCameraMotion>,
        )>,
    ) {
        for (mut camera, transform, _camera, mut mode, mut motion) in query.iter_mut() {
            if camera.enabled {
                if mode.on(OrbitCameraMode::Orbit) {
                    let x = motion.get(OrbitCameraMotion::Right).unwrap_or_default();
                    let y = motion.get(OrbitCameraMotion::Up).unwrap_or_default();
                    let delta = Vec2::new(x, y);
                    camera.x -= delta.x * camera.rotate_sensitivity * time.delta_seconds();
                    camera.y -= delta.y * camera.rotate_sensitivity * time.delta_seconds();
                    camera.y = camera
                        .y
                        .max(*camera.pitch_range.start())
                        .min(*camera.pitch_range.end());
                }
                if mode.on(OrbitCameraMode::Pan) {
                    let x = motion.get(OrbitCameraMotion::Right).unwrap_or_default();
                    let y = motion.get(OrbitCameraMotion::Up).unwrap_or_default();
                    let delta = Vec2::new(x, y);
                    let right_dir = transform.rotation * -Vec3::X;
                    let up_dir = transform.rotation * Vec3::Y;
                    let pan_vector = (delta.x * right_dir + delta.y * up_dir)
                        * camera.pan_sensitivity
                        * time.delta_seconds();
                    camera.center += pan_vector;
                }
                // Zoom
                let delta = motion.get(OrbitCameraMotion::Zoom).unwrap_or_default();
                camera.distance *= camera.zoom_sensitivity.powf(delta);
            }
            mode.clear();
            motion.clear();
        }
    }
}

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<OrbitCamera>()
            .add_system(setup)
            .add_system(action_map::<OrbitCameraMode, OrbitCamera>)
            .add_system(action_axis_map::<OrbitCameraMotion, OrbitCamera>)
            .add_system(Self::camera_motion)
            .add_system(Self::camera_update)
            .add_event::<CameraEvents>();
    }
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy, Reflect, FromReflect)]
pub enum OrbitCameraMode {
    #[default]
    Idle,
    Orbit,
    Pan,
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy, Reflect, FromReflect)]
pub enum OrbitCameraMotion {
    #[default]
    Idle,
    Zoom,
    Right,
    Up,
}

fn setup(
    mut commands: Commands,
    cameras: Query<
        Entity,
        (
            With<OrbitCamera>,
            Without<ActionMap<OrbitCameraMode>>,
            Without<ActionAxisMap<OrbitCameraMotion>>,
        ),
    >,
) {
    for entity in cameras.iter() {
        let mut action_map = ActionMap::<OrbitCameraMode>::default();
        *action_map = vec![
            // Orbit Mode
            ActionMapInput {
                action: OrbitCameraMode::Orbit,
                button: KeyCode::Right.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            ActionMapInput {
                action: OrbitCameraMode::Orbit,
                button: KeyCode::Left.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            ActionMapInput {
                action: OrbitCameraMode::Orbit,
                button: KeyCode::Up.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            ActionMapInput {
                action: OrbitCameraMode::Orbit,
                button: KeyCode::Down.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            ActionMapInput {
                action: OrbitCameraMode::Orbit,
                button: MouseButton::Left.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            ActionMapInput {
                action: OrbitCameraMode::Orbit,
                button: TouchSide::Left.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            // Pan Mode
            ActionMapInput {
                action: OrbitCameraMode::Pan,
                button: KeyCode::A.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            ActionMapInput {
                action: OrbitCameraMode::Pan,
                button: KeyCode::D.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            ActionMapInput {
                action: OrbitCameraMode::Pan,
                button: KeyCode::W.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            ActionMapInput {
                action: OrbitCameraMode::Pan,
                button: KeyCode::S.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            ActionMapInput {
                action: OrbitCameraMode::Pan,
                button: MouseButton::Right.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            ActionMapInput {
                action: OrbitCameraMode::Pan,
                button: TouchSide::Right.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
        ];

        let mut axis_map: ActionAxisMap<OrbitCameraMotion> = Default::default();
        *axis_map = vec![
            // WASD
            AxisMapInput {
                axis: OrbitCameraMotion::Up,
                source: AxisMapSource::Keyboard {
                    positive: KeyCode::W,
                    negative: KeyCode::S,
                },
            },
            AxisMapInput {
                axis: OrbitCameraMotion::Right,
                source: AxisMapSource::Keyboard {
                    positive: KeyCode::A,
                    negative: KeyCode::D,
                },
            },
            // Arrows
            AxisMapInput {
                axis: OrbitCameraMotion::Right,
                source: AxisMapSource::Keyboard {
                    positive: KeyCode::Left,
                    negative: KeyCode::Right,
                },
            },
            AxisMapInput {
                axis: OrbitCameraMotion::Up,
                source: AxisMapSource::Keyboard {
                    positive: KeyCode::Up,
                    negative: KeyCode::Down,
                },
            },
            // Mouse X Y
            AxisMapInput {
                axis: OrbitCameraMotion::Right,
                source: AxisMapSource::MouseAxis(MouseAxis::X),
            },
            AxisMapInput {
                axis: OrbitCameraMotion::Up,
                source: AxisMapSource::MouseAxis(MouseAxis::Y),
            },
            // Mouse wheel
            AxisMapInput {
                axis: OrbitCameraMotion::Zoom,
                source: AxisMapSource::MouseAxis(MouseAxis::Z),
            },
            // Touch
            AxisMapInput {
                axis: OrbitCameraMotion::Right,
                source: AxisMapSource::TouchSideAxis(TouchSideAxis::Left(TouchAxis::PositiveX)),
            },
            AxisMapInput {
                axis: OrbitCameraMotion::Up,
                source: AxisMapSource::TouchSideAxis(TouchSideAxis::Left(TouchAxis::PositiveY)),
            },
            AxisMapInput {
                axis: OrbitCameraMotion::Right,
                source: AxisMapSource::TouchSideAxis(TouchSideAxis::Right(TouchAxis::NegativeX)),
            },
            AxisMapInput {
                axis: OrbitCameraMotion::Up,
                source: AxisMapSource::TouchSideAxis(TouchSideAxis::Right(TouchAxis::NegativeY)),
            },
        ];

        commands
            .entity(entity)
            .insert(Action::<OrbitCameraMode>::default())
            .insert(ActionAxis::<OrbitCameraMotion>::default())
            .insert(action_map)
            .insert(axis_map);
    }
}
