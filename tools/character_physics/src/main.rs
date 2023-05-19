use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    transform::TransformSystem,
    window::PresentMode,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier3d::prelude::*;
use regex::Regex;
use simula_action::ActionPlugin;
use simula_camera::orbitcam::*;
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::LinesPlugin,
    rod::Rod,
};
use std::time::Duration;

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(Color::rgb(0.105, 0.10, 0.11)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "[Simbotic] Simula - Character Physics".to_string(),
                resolution: (1920., 1080.).into(),
                present_mode: PresentMode::AutoVsync,
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin {
            // mode: DebugRenderMode::COLLIDER_SHAPES,
            ..Default::default()
        })
        .add_plugin(WorldInspectorPlugin::default())
        .add_plugin(ActionPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(OrbitCameraPlugin)
        .add_plugin(LinesPlugin)
        .add_plugin(AxesPlugin)
        .add_plugin(GridPlugin)
        .add_startup_system(setup)
        .add_system(debug_info)
        .add_system(drop_box)
        .add_system(
            setup_scene_once_loaded
                .after(TransformSystem::TransformPropagate)
                .in_base_set(CoreSet::PostUpdate),
        )
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, asset_server: Res<AssetServer>) {
    // rod as bone example
    let rod = Rod {
        depth: 0.3,
        north_radius: 0.1,
        south_radius: 0.2,
        ..Default::default()
    };
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(rod)),
            material: asset_server.load("materials/rod.mat"),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::cylinder(0.1, 0.5))
        .insert(Name::new("Rod"));

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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    animations: Res<Animations>,
    mut player: Query<(Entity, &mut AnimationPlayer)>,
    names: Query<&Name>,
    _parent_query: Query<&Parent>,
    children_query: Query<&Children>,
    global_transforms: Query<&GlobalTransform>,
    mut done: Local<bool>,
) {
    if !*done {
        if let Ok((anim_entity, mut player)) = player.get_single_mut() {
            print_hierarchy(anim_entity, &names, &children_query, 0);

            let ragdoll = commands
                .spawn(SpatialBundle {
                    ..Default::default()
                })
                .id();

            create_ragdoll(
                &mut commands,
                ragdoll,
                anim_entity,
                anim_entity,
                &global_transforms,
                &names,
                &children_query,
            );

            create_skeleton(
                &mut commands,
                &mut meshes,
                &mut materials,
                anim_entity,
                anim_entity,
                &global_transforms,
                &names,
                &children_query,
            );

            player.play(animations.0[0].clone_weak()).repeat();
            *done = true;
        }
    }
}

#[derive(Component)]
struct Bone;

fn create_ragdoll(
    commands: &mut Commands,
    ragdoll: Entity,
    root: Entity,
    bone: Entity,
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

    // ignore fingers and toes
    let re_fingers = Regex::new(r"mixamorig:.*Hand.+$").expect("failed to compile regex");
    let re_toes = Regex::new(r"mixamorig:.*Toe.+$").expect("failed to compile regex");
    if re_fingers.is_match(bone_name) || re_toes.is_match(bone_name) {
        return;
    }

    if bone_name.starts_with("mixamorig:")
        && !re_fingers.is_match(bone_name)
        && !re_toes.is_match(bone_name)
    {
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
        } else if bone_name.contains("Hips") {
            0.05
        } else if bone_name.contains("Neck") {
            bone_length * 0.2
        } else if bone_name.contains("Head") {
            bone_length * 0.2
        } else if bone_name.contains("Hand") {
            bone_length * 0.3
        } else if bone_name.contains("Foot") {
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

    // recursively create colliders for all children
    if let Ok(children) = children_query.get(bone) {
        for child in children.iter() {
            create_ragdoll(
                commands,
                ragdoll,
                root,
                *child,
                global_transforms,
                names,
                children_query,
            );
        }
    }
}

fn create_skeleton(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    root: Entity,
    bone: Entity,
    global_transforms: &Query<&GlobalTransform>,
    names: &Query<&Name>,
    children_query: &Query<&Children>,
) {
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

    // ignore fingers and toes
    let re_fingers = Regex::new(r"mixamorig:.*Hand.+$").expect("failed to compile regex");
    let re_toes = Regex::new(r"mixamorig:.*Toe.+$").expect("failed to compile regex");
    if re_fingers.is_match(bone_name) || re_toes.is_match(bone_name) {
        return;
    }

    if bone_name.starts_with("mixamorig:")
        && !re_fingers.is_match(bone_name)
        && !re_toes.is_match(bone_name)
    {
        commands.entity(bone).insert(Bone);

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
        } else if bone_name.contains("Hips") {
            0.05
        } else if bone_name.contains("Neck") {
            bone_length * 0.2
        } else if bone_name.contains("Head") {
            bone_length * 0.2
        } else if bone_name.contains("Hand") {
            bone_length * 0.3
        } else if bone_name.contains("Foot") {
            bone_length * 0.2
        } else if bone_name.contains("Leg") {
            bone_length * 0.08
        } else {
            bone_length * 0.1
        };

        if count > 0 {
            println!("{}: {} {}", bone_name, bone_length, bone_radius);
            commands.entity(bone).with_children(|parent| {
                // create bone using rod mesh
                let rod = Rod {
                    depth: bone_length,
                    north_radius: bone_radius * 0.1,
                    south_radius: bone_radius * 1.0,
                    ..Default::default()
                };
                parent
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(rod)),
                        transform: Transform::from_xyz(0.0, bone_length * 0.5, 0.0),
                        material: materials.add(Color::rgb(1.0, 0.5, 0.3).into()),
                        ..Default::default()
                    })
                    .insert(Name::new("Rod"));

                // parent.spawn((
                //     Transform::from_translation(bone_end / 2.0),
                //     GlobalTransform::default(),
                //     RigidBody::KinematicPositionBased,
                //     Collider::round_cylinder(bone_length / 2.0, bone_radius, bone_radius * 1.0),
                //     ColliderDebugColor(Color::rgb(0.0, 1.0, 0.0)),
                //     Name::new(format!("skeleton_bone:{}", bone_name)),
                // ));
            });
        }
    }

    // recursively create colliders for all children
    if let Ok(children) = children_query.get(bone) {
        for child in children.iter() {
            create_skeleton(
                commands,
                meshes,
                materials,
                root,
                *child,
                global_transforms,
                names,
                children_query,
            );
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
