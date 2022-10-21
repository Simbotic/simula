use bevy::prelude::*;
use simula_core::{ease::EaseFunction, map_range::map_range};

pub struct FollowUIPlugin;

impl Plugin for FollowUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(follow_ui);
    }
}

#[derive(Component)]
pub struct FollowUICamera;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct FollowUIVisibility {
    pub screen_pos: Vec3,
    pub alpha: f32,
}

#[derive(Component)]
pub struct FollowUI {
    pub min_distance: f32,
    pub max_distance: f32,
    pub min_height: f32,
    pub max_height: f32,
    pub max_view_angle: f32,
    pub size: Vec2,
}

impl Default for FollowUI {
    fn default() -> Self {
        Self {
            min_distance: 0.1,
            max_distance: 10.0,
            min_height: -2.0,
            max_height: 2.0,
            max_view_angle: 30.0,
            size: Vec2::new(100.0, 100.0),
        }
    }
}

pub fn follow_ui(
    mut commands: Commands,
    windows: Res<Windows>,
    mut query: Query<(
        Entity,
        &FollowUI,
        Option<&mut FollowUIVisibility>,
        &GlobalTransform,
    )>,
    camera_query: Query<(&Camera, &GlobalTransform), With<FollowUICamera>>,
) {
    if camera_query.iter().count() > 1 {
        warn!("Only one FollowUICamera is allowed");
    }

    if let Some((camera, camera_global_transform)) = camera_query.iter().next() {
        for (entity, follow_ui, visibility, ui_global_transform) in query.iter_mut() {
            let camera_height =
                camera_global_transform.translation().y - ui_global_transform.translation().y;
            let camera_distance = Vec3::distance(
                camera_global_transform.translation(),
                ui_global_transform.translation(),
            );
            let camera_front = camera_global_transform.forward();
            let ui_front = ui_global_transform.forward();
            let view_angle = Vec3::angle_between(-camera_front, ui_front).to_degrees();

            let screen_pos;
            if true
                && camera_height > follow_ui.min_height
                && camera_height < follow_ui.max_height
                && camera_distance > follow_ui.min_distance
                && camera_distance < follow_ui.max_distance
                && view_angle < follow_ui.max_view_angle
            {
                screen_pos = if let Some(mut screen_pos) =
                    camera.world_to_ndc(camera_global_transform, ui_global_transform.translation())
                {
                    if screen_pos.z > 0.0 {
                        if let Some(window) = windows.get_primary() {
                            screen_pos.x = map_range(
                                screen_pos.x,
                                (-1.0, 1.0),
                                (0.0, window.width()),
                                EaseFunction::Linear,
                            );
                            screen_pos.y = map_range(
                                screen_pos.y,
                                (-1.0, 1.0),
                                (window.height(), 0.0),
                                EaseFunction::Linear,
                            );
                            screen_pos.x = screen_pos.x - follow_ui.size.x / 2.0;
                            screen_pos.y = screen_pos.y - follow_ui.size.y / 2.0;
                            Some(screen_pos)
                        } else {
                            // No primary window
                            None
                        }
                    } else {
                        // behind camera
                        None
                    }
                } else {
                    // off screen
                    None
                }
            } else {
                // out of range
                screen_pos = None;
            };

            if let Some(screen_pos) = screen_pos {
                let camera_pos_height_alpha = map_range(
                    camera_height,
                    (0.0, follow_ui.max_height),
                    (1.0, 0.0),
                    EaseFunction::SineInOut,
                )
                .clamp(0.0, 1.0);
                let camera_neg_height_alpha = map_range(
                    camera_height,
                    (0.0, follow_ui.min_height),
                    (1.0, 0.0),
                    EaseFunction::SineInOut,
                )
                .clamp(0.0, 1.0);
                let camera_distance_alpha = map_range(
                    camera_distance,
                    (follow_ui.min_distance, follow_ui.max_distance),
                    (1.0, 0.0),
                    EaseFunction::SineInOut,
                )
                .clamp(0.0, 1.0);
                let view_angle_alpha = map_range(
                    view_angle,
                    (0.0, follow_ui.max_view_angle),
                    (1.0, 0.0),
                    EaseFunction::SineInOut,
                )
                .clamp(0.0, 1.0);
                let alpha = camera_pos_height_alpha
                    .min(camera_neg_height_alpha)
                    .min(camera_distance_alpha)
                    .min(view_angle_alpha);

                if let Some(mut visibility) = visibility {
                    visibility.screen_pos = screen_pos;
                    visibility.alpha = alpha;
                } else {
                    commands
                        .entity(entity)
                        .insert(FollowUIVisibility { screen_pos, alpha });
                }
            } else {
                commands.entity(entity).remove::<FollowUIVisibility>();
            }
        }
    }
}
