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
            max_pitch: 20.0,
            yaw_ease: EaseFunction::Linear,
            pitch_ease: EaseFunction::Linear,
        }
    }
}

pub fn smooth_look_at(
    mut query: Query<(&SmoothLookAt, &mut Transform)>,
    transforms: Query<&Transform, Without<SmoothLookAt>>,
) {
    for (look_at, mut transform) in query.iter_mut() {
        if let Some(target) = look_at.target {
            if let Ok(target_transform) = transforms.get(target) {
                let target_position = target_transform.translation;
                let position = transform.translation;

                let look_transform = Transform::identity()
                    .with_translation(position)
                    .looking_at(target_position, Vec3::Y);

                // let max_yaw = look_at.max_yaw.to_radians();
                // let max_pitch = look_at.max_pitch.to_radians();

                let (yaw, pitch, _) = look_transform.rotation.to_euler(EulerRot::YXZ);
                // let yaw = yaw.clamp(-max_yaw, max_yaw);
                // let pitch = pitch.clamp(-max_pitch, max_pitch);

                // let yaw_alpha = yaw.abs() / max_yaw;
                // let pitch_alpha = pitch.abs() / max_pitch;

                // let yaw = yaw * yaw_alpha.calc(look_at.yaw_ease);
                // let pitch = pitch * pitch_alpha.calc(look_at.pitch_ease);

                transform.rotation = Quat::from_rotation_y(yaw)
                    * Quat::from_rotation_x(pitch)
                    * look_at.initial_pose;
            }
        }
    }
}
