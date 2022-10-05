use bevy::prelude::*;
use simula_core::ease::*;

pub struct LookAtPlugin;

impl Plugin for LookAtPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(smooth_look_at);
    }
}

#[derive(Debug, Component)]
pub struct SmoothLookAt {
    pub target: Option<Entity>,
    pub initial_pose: Quat,
    pub max_yaw: f32,
    pub max_pitch: f32,
    pub yaw_ease: EaseFunction,
    pub pitch_ease: EaseFunction,
}

impl Default for SmoothLookAt {
    fn default() -> Self {
        Self {
            target: None,
            initial_pose: Quat::IDENTITY,
            max_yaw: 45.0,
            max_pitch: 10.0,
            yaw_ease: EaseFunction::Linear,
            pitch_ease: EaseFunction::Linear,
        }
    }
}

pub fn smooth_look_at(
    mut query: Query<(&SmoothLookAt, &mut Transform)>,
    transforms: Query<&GlobalTransform>,
) {
    for (look_at, mut transform) in query.iter_mut() {
        if let Some(target) = look_at.target {
            if let Ok(target_transform) = transforms.get(target) {
                let initial_pose = look_at.initial_pose.normalize();
                let target_position = target_transform.translation();
                let position = transform.translation;

                // Compute yaw that can be eased
                let yaw = {
                    let look_transform = Transform::identity()
                        .with_translation(position)
                        .looking_at(target_position, Vec3::Y);
                    let rotation = (look_transform.rotation * initial_pose.inverse()).normalize();
                    let (yaw, _pitch, _roll) = rotation.to_euler(EulerRot::YXZ);
                    yaw
                };

                // Compute pitch that can be eased
                let pitch = {
                    let local_target_position = initial_pose.inverse() * target_position;
                    let local_target_position = Vec3::new(
                        local_target_position.x,
                        local_target_position.y,
                        -local_target_position.z.abs(),
                    );
                    Vec3::Y.dot(local_target_position.normalize()) * std::f32::consts::PI / 2.0
                };

                // Clamp and ease yaw
                let max_yaw = look_at.max_yaw.to_radians();
                let yaw = yaw.clamp(-max_yaw, max_yaw);
                let yaw_alpha = yaw.abs() / max_yaw;
                let yaw = yaw.signum() * max_yaw * yaw_alpha.calc(look_at.yaw_ease);

                // Clamp and ease pitch
                let max_pitch = look_at.max_pitch.to_radians();
                let pitch = pitch.clamp(-max_pitch, max_pitch);
                let pitch_alpha = pitch.abs() / max_pitch;
                let pitch = pitch.signum() * max_pitch * pitch_alpha.calc(look_at.pitch_ease);

                // Apply yaw and pitch
                transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw)
                    * Quat::from_axis_angle(initial_pose * Vec3::X, pitch)
                    * initial_pose;
            }
        }
    }
}
