use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    transform::TransformSystem,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use regex::Regex;
use simula_action::ActionPlugin;
use simula_camera::orbitcam::*;
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::{LineMesh, LinesMaterial, LinesPlugin},
};
use std::time::Duration;

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
        .add_plugin(RapierDebugRenderPlugin {
            mode: DebugRenderMode::COLLIDER_SHAPES,
            ..Default::default()
        })
        .add_plugin(WorldInspectorPlugin)
        .add_plugin(ActionPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(OrbitCameraPlugin)
        .add_plugin(LinesPlugin)
        .add_plugin(AxesPlugin)
        .add_plugin(GridPlugin)
        .add_startup_system(setup)
        .add_system(debug_info)
        .add_system(drop_box)
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
    // character
    commands
        .spawn(SceneBundle {
            scene: asset_server.load("models/character/X_Bot/Character.glb#Scene0"),
            ..default()
        })
        .insert(Name::new("Character"));
    commands.insert_resource(Animations(vec![
        asset_server.load("models/character/X_Bot/Martelo_Do_Chau.glb#Animation0")
    ]));

    // ground
    let ground_size = 200.1;
    let ground_height = 0.1;
    commands.spawn((
        TransformBundle::from(Transform::from_xyz(0.0, -ground_height, 0.0)),
        Collider::cuboid(ground_size, ground_height, ground_size),
        Name::new("Ground"),
    ));

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

// Drop box every few seconds
fn drop_box(mut commands: Commands, time: Res<Time>, mut timer: Local<Timer>) {
    timer.tick(time.delta());
    if timer.finished() {
        timer.reset();
        timer.set_duration(Duration::from_secs(1));
        let box_size = 0.2;
        let x = rand::random::<f32>() * 2.0 - 0.5;
        let z = rand::random::<f32>() + 0.5;
        commands.spawn((
            TransformBundle::from(Transform::from_xyz(x, 3.0, z)),
            RigidBody::Dynamic,
            Collider::cuboid(box_size, box_size, box_size),
            ColliderDebugColor(Color::SEA_GREEN),
            Name::new("Box"),
        ));
    }
}

#[derive(Resource)]
struct Animations(Vec<Handle<AnimationClip>>);

fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut player: Query<(Entity, &mut AnimationPlayer)>,
    names: Query<&Name>,
    children: Query<&Children>,
    global_transforms: Query<&GlobalTransform>,
    mut done: Local<bool>,
) {
    if !*done {
        if let Ok((anim_entity, mut player)) = player.get_single_mut() {
            print_hierarchy(anim_entity, &names, &children, 0);
            create_colliders(
                &mut commands,
                anim_entity,
                &global_transforms,
                &names,
                &children,
            );
            player.play(animations.0[0].clone_weak()).repeat();
            *done = true;
        }
    }
}

#[derive(Component)]
struct Bone;

fn create_colliders(
    commands: &mut Commands,
    bone: Entity,
    // transforms: &Query<&Transform>,
    global_transforms: &Query<&GlobalTransform>,
    names: &Query<&Name>,
    children_query: &Query<&Children>,
) {
    commands.entity(bone).insert(Bone);

    let bone_global_transform = global_transforms.get(bone);
    let bone_name = names.get(bone);
    let bone_children = children_query.get(bone);

    let (bone_global_transform, bone_name, bone_children) =
        match (bone_global_transform, bone_name, bone_children) {
            (Ok(bone_global_transform), Ok(bone_name), Ok(bone_children)) => {
                (bone_global_transform, bone_name, bone_children)
            }
            _ => return,
        };

    if bone_name.starts_with("mixamorig:") {
        // compute bone end
        let (sum, count) =
            bone_children
                .iter()
                .fold((Vec3::ZERO, 0), |(mut sum, mut count), child| {
                    if let Ok(child_global_transform) = global_transforms.get(*child) {
                        sum += child_global_transform.translation();
                        count += 1;
                    }
                    (sum, count)
                });
        let bone_end = sum / count as f32;
        let bone_end = bone_global_transform
            .compute_matrix()
            .inverse()
            .transform_point3(bone_end);

        // adjust bone radius based on bone name
        let bone_length = bone_end.length();
        let bone_radius = if bone_name.contains("Spine") {
            bone_length * 0.5
        } else if bone_name.contains("Head") {
            bone_length * 0.2
        } else if bone_name.contains("Leg") {
            bone_length * 0.08
        } else {
            bone_length * 0.1
        };

        if count > 0 {
            commands.entity(bone).with_children(|parent| {
                parent.spawn((
                    Transform::from_translation(bone_end / 2.0),
                    GlobalTransform::default(),
                    RigidBody::KinematicPositionBased,
                    Collider::round_cylinder(bone_length / 2.0, bone_radius, bone_radius * 1.0),
                    ColliderDebugColor(Color::rgb(0.0, 1.0, 0.0)),
                    Name::new(format!("collider:{}", bone_name)),
                ));
            });
        }
    }

    // ignore fingers and toes
    let re_fingers = Regex::new(r"mixamorig:.*Hand.").expect("failed to compile regex");
    let re_toes = Regex::new(r"mixamorig:.*Foot$").expect("failed to compile regex");

    if !re_fingers.is_match(bone_name) || !re_toes.is_match(bone_name) {
        if let Ok(children) = children_query.get(bone) {
            // recursively create colliders for all children
            for child in children.iter() {
                create_colliders(
                    commands,
                    *child,
                    // transforms,
                    global_transforms,
                    names,
                    children_query,
                    // meshes,
                    // materials,
                );
            }
        }
    }
}

fn print_hierarchy(
    entity: Entity,
    names: &Query<&Name>,
    children_query: &Query<&Children>,
    indent: usize,
) {
    let name = names.get(entity);
    let children = children_query.get(entity);

    let (name, children) = match (name, children) {
        (Ok(name), Ok(children)) => (name, children),
        _ => return,
    };

    println!("{}{}", " ".repeat(indent), name);

    for child in children.iter() {
        print_hierarchy(*child, names, children_query, indent + 2);
    }
}
