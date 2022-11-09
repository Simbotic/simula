use crate::behaviors::robber::*;
use bevy::prelude::*;
use simula_behavior::prelude::*;
use simula_video::GifAsset;
use simula_video::VideoPlayer;
use simula_viz::{
    axes::{Axes, AxesBundle},
    follow_ui::FollowUI,
    lines::{LineMesh, LinesMaterial},
    lookat::SmoothLookAt,
};

pub struct RobberAgentPlugin;

impl Plugin for RobberAgentPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawner);
    }
}

#[derive(Component)]
pub struct RobberSpawner {
    pub transform: Transform,
}

fn spawner(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut lines_materials: ResMut<Assets<LinesMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    line_mesh: Res<LineMesh>,
    asset_server: Res<AssetServer>,
    spawners: Query<(Entity, &RobberSpawner)>,
    cameras: Query<Entity, With<Camera>>,
) {
    for (entity, spawner) in &spawners {
        let agent_body = commands
            .spawn_bundle(SpatialBundle {
                transform: Transform::from_xyz(0.0, 0.5, 0.0).with_rotation(Quat::from_euler(
                    EulerRot::YXZ,
                    -std::f32::consts::FRAC_PI_3 * 0.0,
                    0.0,
                    0.0,
                )),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
                    material: materials.add(StandardMaterial {
                        base_color_texture: Some(asset_server.load("textures/games/robber.png")),
                        base_color: Color::rgb(1.0, 1.0, 1.0),
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        ..default()
                    }),
                    transform: Transform::from_rotation(Quat::from_euler(
                        EulerRot::YXZ,
                        0.0,
                        -std::f32::consts::FRAC_PI_2,
                        0.0,
                    )),
                    ..default()
                });
            })
            .insert(Name::new("Robber: Body"))
            .with_children(|parent| {
                parent
                    .spawn_bundle(AxesBundle {
                        axes: Axes {
                            size: 1.,
                            ..default()
                        },
                        mesh: meshes.add(line_mesh.clone()),
                        material: lines_materials.add(LinesMaterial {}),
                        ..default()
                    })
                    .insert(Name::new("LookAt Coords"));
            })
            .id();

        if let Ok(camera_entity) = cameras.get_single() {
            commands.entity(entity).insert(SmoothLookAt {
                target: Some(camera_entity),
                max_pitch: 0.1,
                ..default()
            });
        }

        let document: Handle<BehaviorAsset> = asset_server.load("behaviors/debug_any.bht.ron");
        let behavior = BehaviorTree::from_asset::<RobberBehavior>(None, &mut commands, document);
        if let Some(root) = behavior.root {
            commands.entity(root).insert(BehaviorCursor);
        }

        commands
            .entity(entity)
            .remove::<RobberSpawner>()
            .insert_bundle(SpatialBundle {
                transform: spawner.transform,
                ..default()
            })
            .push_children(&[agent_body, behavior.root.unwrap()])
            .insert(behavior)
            .insert(Name::new("Robber"));
    }
}
