use bevy::prelude::*;
use bevy_inspector_egui::{
    egui, options::EntityAttributes, Context, Inspectable, RegisterInspectable,
};
use simula_core::ease::*;

pub struct LookAtPlugin;

impl Plugin for LookAtPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(smooth_look_at)
            .register_inspectable::<SmoothLookAt>();
    }
}

#[derive(Debug, Component, Inspectable)]
pub struct SmoothLookAt {
    pub target: Option<Entity>,
    pub max_yaw: f32,
    pub max_pitch: f32,
    pub yaw_ease: EaseFunction,
    pub pitch_ease: EaseFunction,
    pub initial_pose: Option<Quat>,
}

impl Default for SmoothLookAt {
    fn default() -> Self {
        Self {
            target: None,
            initial_pose: None,
            max_yaw: 45.0,
            max_pitch: 10.0,
            yaw_ease: EaseFunction::Linear,
            pitch_ease: EaseFunction::Linear,
        }
    }
}

pub fn smooth_look_at(
    mut look_ats: Query<(Entity, &mut SmoothLookAt, &mut Transform, Option<&Parent>)>,
    transforms: Query<&GlobalTransform>,
) {
    for (entity, mut look_at, mut transform, parent) in &mut look_ats {
        if let Some(target) = look_at.target {
            if let Ok(global_target_transform) = transforms.get(target) {
                // Get the initial pose of the look_at entity
                if look_at.initial_pose.is_none() {
                    if let Ok(global_transform) = transforms.get(entity) {
                        look_at.initial_pose = Some(global_transform.compute_transform().rotation);
                    }
                }
                let initial_pose = look_at.initial_pose.unwrap_or_default();

                // Compute everything in the local space of the parent
                let mut global_transform = Transform::identity();
                if let Some(parent) = parent {
                    if let Ok(parent_transform) = transforms.get(parent.get()) {
                        global_transform = parent_transform.compute_transform();
                    }
                }

                let global_matrix = global_transform.compute_matrix();
                let inv_global_matrix = global_matrix.inverse();

                let target_position =
                    inv_global_matrix.transform_point3(global_target_transform.translation());

                let initial_pose = Mat4::from_quat(initial_pose) * inv_global_matrix;
                let (_, inv_initial_pose, _) =
                    initial_pose.inverse().to_scale_rotation_translation();
                let (_, initial_pose, _) = initial_pose.to_scale_rotation_translation();

                // Compute yaw that can be eased
                let yaw = {
                    let look_transform = Transform::identity()
                        .with_translation(transform.translation)
                        .looking_at(target_position, Vec3::Y);
                    let rotation = (look_transform.rotation * inv_initial_pose).normalize();
                    let (yaw, _pitch, _roll) = rotation.to_euler(EulerRot::YXZ);
                    yaw
                };

                // Compute pitch that can be eased
                let pitch = {
                    let local_target_position = inv_initial_pose * target_position;
                    let local_target_position = Vec3::new(
                        local_target_position.x,
                        local_target_position.y,
                        -local_target_position.z.abs(),
                    );
                    let target_position = initial_pose * local_target_position;
                    let look_transform = Transform::identity()
                        .with_translation(transform.translation)
                        .looking_at(target_position, Vec3::Y);
                    let (_yaw, pitch, _roll) = look_transform.rotation.to_euler(EulerRot::YXZ);
                    pitch
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
                let local_rotation = Quat::from_axis_angle(Vec3::Y, yaw)
                    * Quat::from_axis_angle(initial_pose * Vec3::X, pitch)
                    * initial_pose;
                transform.rotation = local_rotation;
            }
        }
    }
}
