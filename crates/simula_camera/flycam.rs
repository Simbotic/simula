use bevy::prelude::*;
use bevy::reflect::FromReflect;
use simula_action::{
    action_axis_map, action_map,
    touch_sides::{TouchAxis, TouchSide, TouchSideAxis},
    Action, ActionAxis, ActionAxisMap, ActionMap, ActionMapInput, ActionStage, AxisMapInput,
    AxisMapSource, MouseAxis,
};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct FlyCamera {
    pub accel: f32,
    pub velocity: Vec3,
    pub max_speed: f32,
    pub sensitivity: f32,
    pub friction: f32,
    pub invert_pitch: bool,
}

impl Default for FlyCamera {
    fn default() -> Self {
        Self {
            accel: 5.0,
            velocity: Vec3::ZERO,
            max_speed: 0.2,
            sensitivity: 20.0,
            friction: 1.5,
            invert_pitch: false,
        }
    }
}

fn forward_vector(rotation: &Quat) -> Vec3 {
    rotation.mul_vec3(Vec3::Z).normalize()
}

fn forward_xy_vector(rotation: &Quat) -> Vec3 {
    let f = forward_vector(rotation);
    Vec3::new(f.x, 0.0, f.z).normalize()
}

fn strafe_vector(rotation: &Quat) -> Vec3 {
    // Rotate it 90 degrees to get the strafe direction
    Quat::from_rotation_y(90.0f32.to_radians())
        .mul_vec3(forward_xy_vector(rotation))
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
            // info!("mode: {:?}", mode);
            if mode.on(FlyCameraMode::Look) {
                // Recover yaw and pitch state from rotation
                let mut front = forward_vector(&transform.rotation);
                front.y = front.y.clamp(-0.8, 0.8);
                let front = front.normalize();
                let right = strafe_vector(&transform.rotation);
                let up = front.cross(right).normalize();
                let rotation = Mat3::from_cols(right, up, front);
                let rotation = Quat::from_mat3(&rotation).normalize();

                // Get look motion
                let x = motion.get(FlyCameraMotion::LookRight).unwrap_or_default();
                let y = motion.get(FlyCameraMotion::LookUp).unwrap_or_default();
                let delta_look = Vec2::new(x, y);

                // Look delta
                let yaw = delta_look.x * camera.sensitivity * time.delta_seconds();
                let pitch = delta_look.y * camera.sensitivity * time.delta_seconds();
                let pitch = if camera.invert_pitch {
                    pitch * -1.0
                } else {
                    pitch
                };

                // Rotate camera
                let rotation = Quat::from_axis_angle(Vec3::Y, -yaw)
                    * Quat::from_axis_angle(right, -pitch)
                    * rotation;
                transform.rotation = rotation;
            }

            // Get motion
            let (delta_strafe, delta_forward, delta_up) = (
                motion.get(FlyCameraMotion::Strafe).unwrap_or_default(),
                motion.get(FlyCameraMotion::Forward).unwrap_or_default(),
                motion.get(FlyCameraMotion::Up).unwrap_or_default(),
            );

            // Compute motion vector
            let accel_vector: Vec3 = (strafe_vector(&transform.rotation) * -delta_strafe)
                + (forward_xy_vector(&transform.rotation) * -delta_forward)
                + (Vec3::Y * delta_up);

            // Apply acceleration
            let accel: Vec3 = if accel_vector.length() != 0.0 {
                accel_vector.normalize() * camera.accel
            } else {
                Vec3::ZERO
            };

            // Calculate velocity
            camera.velocity += accel * time.delta_seconds();

            // Calculate movement friction
            let friction: Vec3 = if camera.velocity.length() != 0.0 {
                camera.velocity.normalize() * -1.0 * camera.friction
            } else {
                Vec3::ZERO
            };

            // Clamp within max speed
            if camera.velocity.length() > camera.max_speed {
                camera.velocity = camera.velocity.normalize() * camera.max_speed;
            }

            // Apply friction
            let delta_friction = friction * time.delta_seconds();
            camera.velocity =
                if (camera.velocity + delta_friction).signum() != camera.velocity.signum() {
                    Vec3::ZERO
                } else {
                    camera.velocity + delta_friction
                };

            // Move camera
            transform.translation += camera.velocity;

            // Reset actions
            mode.clear();
            motion.clear();
        }
    }
}

impl Plugin for FlyCameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FlyCamera>()
            .add_system(setup)
            .add_systems(
                (
                    action_map::<FlyCameraMode, FlyCamera>,
                    action_axis_map::<FlyCameraMotion, FlyCamera>,
                    Self::camera_motion,
                )
                    .chain()
                    .after(ActionStage::Update),
            );
    }
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy, Reflect, FromReflect)]
pub enum FlyCameraMode {
    #[default]
    Look,
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, Copy, Reflect, FromReflect)]
pub enum FlyCameraMotion {
    #[default]
    Idle,
    Forward,
    Strafe,
    Up,
    LookUp,
    LookRight,
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
            ActionMapInput {
                action: FlyCameraMode::Look,
                button: MouseButton::Left.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
            ActionMapInput {
                action: FlyCameraMode::Look,
                button: TouchSide::Right.into(),
                ctrl: false,
                shift: false,
                alt: false,
            },
        ];
        let mut axis_map: ActionAxisMap<FlyCameraMotion> = Default::default();
        *axis_map = vec![
            // Up/Down
            AxisMapInput {
                axis: FlyCameraMotion::Up,
                source: AxisMapSource::Keyboard {
                    positive: KeyCode::E,
                    negative: KeyCode::Q,
                },
            },
            // WASD
            AxisMapInput {
                axis: FlyCameraMotion::Forward,
                source: AxisMapSource::Keyboard {
                    positive: KeyCode::W,
                    negative: KeyCode::S,
                },
            },
            AxisMapInput {
                axis: FlyCameraMotion::Strafe,
                source: AxisMapSource::Keyboard {
                    positive: KeyCode::A,
                    negative: KeyCode::D,
                },
            },
            // Touch
            AxisMapInput {
                axis: FlyCameraMotion::Forward,
                source: AxisMapSource::TouchSideAxis(TouchSideAxis::Left(TouchAxis::NegativeY)),
            },
            AxisMapInput {
                axis: FlyCameraMotion::Strafe,
                source: AxisMapSource::TouchSideAxis(TouchSideAxis::Left(TouchAxis::NegativeX)),
            },
            // Touch
            AxisMapInput {
                axis: FlyCameraMotion::LookUp,
                source: AxisMapSource::TouchSideAxis(TouchSideAxis::Right(TouchAxis::PositiveY)),
            },
            AxisMapInput {
                axis: FlyCameraMotion::LookRight,
                source: AxisMapSource::TouchSideAxis(TouchSideAxis::Right(TouchAxis::PositiveX)),
            },
            // Mouse X Y
            AxisMapInput {
                axis: FlyCameraMotion::LookUp,
                source: AxisMapSource::MouseAxis(MouseAxis::Y),
            },
            AxisMapInput {
                axis: FlyCameraMotion::LookRight,
                source: AxisMapSource::MouseAxis(MouseAxis::X),
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
