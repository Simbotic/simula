use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render::primitives::Aabb,
    transform::TransformSystem,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use simula_action::ActionPlugin;
use simula_camera::orbitcam::*;
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::{LineMesh, LinesMaterial, LinesPlugin},
};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.105, 0.10, 0.11)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "[Simbotic] Simula - Character Physics".to_string(),
                width: 1920.,
                height: 1080.,
                ..default()
            },
            ..default()
        }))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(WorldInspectorPlugin)
        .add_plugin(ActionPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(OrbitCameraPlugin)
        .add_plugin(LinesPlugin)
        .add_plugin(AxesPlugin)
        .add_plugin(GridPlugin)
        .add_startup_system(setup_physics)
        .add_startup_system(setup)
        // .add_system(setup_scene_once_loaded)
        .add_system(debug_info)
        .add_system_to_stage(
            CoreStage::PostUpdate,
            setup_scene_once_loaded.after(TransformSystem::TransformPropagate),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut lines_materials: ResMut<Assets<LinesMaterial>>,
    line_mesh: Res<LineMesh>,
    asset_server: Res<AssetServer>,
) {
    // Character
    commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/character/CharacterPhysics.glb#Scene0"),
            ..default()
        })
        // .insert(KinematicCharacterController::new(0.5, 0))
        .insert(Name::new("Character"));
    commands.insert_resource(Animations(vec![
        asset_server.load("models/character/CharacterPhysics.glb#Animation0")
    ]));

    // grid
    let grid_color = Color::rgb(0.08, 0.06, 0.08);
    commands
        .spawn(GridBundle {
            grid: Grid {
                size: 10,
                divisions: 10,
                start_color: grid_color,
                end_color: grid_color,
                ..Default::default()
            },
            mesh: meshes.add(line_mesh.clone()),
            material: lines_materials.add(LinesMaterial {}),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        .insert(Name::new("Grid"));

    // axes
    commands
        .spawn(AxesBundle {
            axes: Axes {
                size: 1.,
                inner_offset: 5.,
            },
            mesh: meshes.add(line_mesh.clone()),
            material: lines_materials.add(LinesMaterial {}),
            transform: Transform::from_xyz(0.0, 0.01, 0.0),
            ..Default::default()
        })
        .insert(Name::new("Axes: World"));

    let theta = std::f32::consts::FRAC_PI_4;
    let light_transform = Mat4::from_euler(EulerRot::ZYX, 0.0, std::f32::consts::FRAC_PI_2, -theta);
    commands
        .spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::rgb(1.0, 1.0, 1.0),
                illuminance: 5000.,
                ..Default::default()
            },
            transform: Transform::from_matrix(light_transform),
            ..Default::default()
        })
        .insert(Name::new("Light: Directional"));

    // orbit camera
    commands
        .spawn(Camera3dBundle {
            ..Default::default()
        })
        .insert(OrbitCamera {
            x: std::f32::consts::FRAC_PI_4 * 5.0,
            y: 80_f32.to_radians(),
            center: Vec3::new(0.0, 1.0, 1.0),
            distance: 4.0,
            ..Default::default()
        })
        .insert(Name::new("Camera: Orbit"));

    // FPS on screen
    commands
        .spawn(TextBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "\nFPS: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 12.0,
                        color: Color::rgb(0.0, 1.0, 0.0),
                    },
                }],
                ..Default::default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Name::new("FPS"));
}

fn debug_info(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text>) {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            for mut text in query.iter_mut() {
                text.sections[0].value = format!("{:.2}", average);
            }
        }
    };
}

pub fn setup_physics(mut commands: Commands) {
    let ground_size = 200.1;
    let ground_height = 0.1;
    commands.spawn((
        TransformBundle::from(Transform::from_xyz(0.0, -ground_height, 0.0)),
        Collider::cuboid(ground_size, ground_height, ground_size),
        Name::new("Ground"),
    ));

    commands.spawn((
        TransformBundle::from(Transform::from_xyz(0.0, 3.0, 0.0)),
        RigidBody::Dynamic,
        Collider::cuboid(0.1, 0.1, 0.1),
        ColliderDebugColor(Color::SEA_GREEN),
        Name::new("Box"),
    ));
}

#[derive(Resource)]
struct Animations(Vec<Handle<AnimationClip>>);

fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut player: Query<(Entity, &mut AnimationPlayer)>,
    names: Query<&Name>,
    children: Query<&Children>,
    transforms: Query<&GlobalTransform>,
    mut done: Local<bool>,
) {
    if !*done {
        if let Ok((anim_entity, mut player)) = player.get_single_mut() {
            player.play(animations.0[0].clone_weak()).repeat();
            create_colliders(&mut commands, anim_entity, &transforms, &names, &children);
            *done = true;
        }
    }
}

// Iterate recursively over all bones and create a collider for each bone, starting from a parent entity.
// Colliders are created as children of the bone entity. All colliders are rounded cylinders.
// The length of the cylinder is the distance between the bone and its parent.
fn create_colliders(
    commands: &mut Commands,
    parent: Entity,
    transforms: &Query<&GlobalTransform>,
    names: &Query<&Name>,
    children_query: &Query<&Children>,
) {
    if let Ok(children) = children_query.get(parent) {
        for child in children.iter() {
            if let Ok(name) = names.get(*child) {
                // name starts with mixamorig:
                if name.starts_with("mixamorig:") {
                    if let Ok(transform) = transforms.get(parent) {
                        if let Ok(child_transform) = transforms.get(*child) {
                            //
                            let length =
                                (child_transform.translation() - transform.translation()).length();
                            let radius = 0.05;
                            commands.entity(*child).with_children(|parent| {
                                parent.spawn((
                                    Transform::from_translation(Vec3::new(0.0, length / 2.0, 0.0)),
                                    RigidBody::Dynamic,
                                    Collider::round_cylinder(length, radius, radius),
                                    ColliderDebugColor(Color::rgb(0.0, 1.0, 0.0)),
                                    Name::new(format!("Collider: {}", name)),
                                ));
                            });
                        }
                    }
                    create_colliders(commands, *child, transforms, &names, &children_query);
                }
            } else {
                continue;
            }
        }
    }
}
