use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render::view::NoFrustumCulling,
};
use bevy_inspector_egui::WorldInspectorPlugin;
use enum_iterator::IntoEnumIterator;
use monkey::MonkeyPlugin;
use rand::distributions::{Distribution, Uniform};
use simula_action::ActionPlugin;
#[cfg(not(target_arch = "wasm32"))]
use simula_cad::shapes::{self, ShapeMesh};
use simula_camera::orbitcam::*;
use simula_core::{
    ease::EaseFunction,
    force_graph::{NodeData, NodeIndex, SimulationParameters},
    signal::{SignalController, SignalFunction, SignalGenerator},
};
use simula_net::NetPlugin;
#[cfg(feature = "gst")]
use simula_video::create_gst;
#[cfg(feature = "gif")]
use simula_video::GifAsset;
#[cfg(feature = "webp")]
use simula_video::WebPAsset;
use simula_video::{VideoPlayer, VideoPlugin};
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    ease::{ease_lines, EaseLine},
    force_graph::{ForceGraph, ForceGraphBundle},
    grid::{Grid, GridBundle, GridPlugin},
    lines::{LineMesh, Lines, LinesBundle, LinesMaterial, LinesPlugin},
    pointcloud::{PointData, Pointcloud, PointcloudPlugin},
    signal::{
        signal_control_lines, signal_generator_lines, SignalControlLine, SignalGeneratorLine,
    },
    voxel::{Voxel, VoxelMesh, Voxels, VoxelsBundle, VoxelsMaterial, VoxelsPlugin},
};

mod monkey;

fn main() {
    App::new()
        .register_type::<SignalGenerator>()
        .register_type::<SignalFunction>()
        .register_type::<SignalController<f32>>()
        .register_type::<ForceGraph<SandboxNodeData, SandboxEdgeData>>()
        .register_type::<SimulationParameters>()
        .register_type::<SandboxNode>()
        .insert_resource(WindowDescriptor {
            title: "[Simbotic] Simula - Sandbox".to_string(),
            width: 940.,
            height: 528.,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.105, 0.10, 0.11)))
        .add_plugins(DefaultPlugins)
        .add_plugin(NetPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(ActionPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(OrbitCameraPlugin)
        .add_plugin(LinesPlugin)
        .add_plugin(AxesPlugin)
        .add_plugin(GridPlugin)
        .add_plugin(VoxelsPlugin)
        .add_plugin(PointcloudPlugin)
        .add_plugin(MonkeyPlugin)
        .add_plugin(VideoPlugin)
        .add_startup_system(setup)
        .add_system(debug_info)
        .add_system(line_test)
        .add_system(ease_lines)
        .add_system(signal_generator_lines)
        .add_system(signal_control_lines)
        .add_system(rotate_system)
        .add_system(force_graph_test)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut voxels_materials: ResMut<Assets<VoxelsMaterial>>,
    mut lines_materials: ResMut<Assets<LinesMaterial>>,
    line_mesh: Res<LineMesh>,
    voxel_mesh: Res<VoxelMesh>,
    asset_server: Res<AssetServer>,
) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // CAD shape
        let shape = shapes::star(5, Color::BLUE);
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(shape.to_mesh()),
                material: materials.add(Color::rgb(0.0, 0.0, 1.0).into()),
                transform: Transform::from_xyz(0.0, -10.0, 0.0),
                ..Default::default()
            })
            .insert(Name::new("Shape: Star"));
    }

    // plane
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform::from_xyz(2.0, 0.01, 2.0),
            ..Default::default()
        })
        .insert(Name::new("Shape: Plane"));

    // cube
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(-2.0, 0.0, -2.0),
            ..Default::default()
        })
        .insert(Name::new("Shape: Cube"));

    // grid
    commands
        .spawn_bundle(GridBundle {
            grid: Grid {
                size: 10,
                divisions: 10,
                start_color: Color::BLUE,
                end_color: Color::RED,
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
        .spawn_bundle(AxesBundle {
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

    // x - axis
    commands
        .spawn_bundle(AxesBundle {
            axes: Axes {
                size: 3.,
                inner_offset: 0.,
            },
            mesh: meshes.add(line_mesh.clone()),
            material: lines_materials.add(LinesMaterial {}),
            transform: Transform::from_xyz(7.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(Name::new("Axis: X"))
        .insert(Rotate {
            axis: Vec3::new(1.0, 0.0, 0.0),
            angle: 1.0,
        })
        .insert(RandomLines);

    // y - axis
    commands
        .spawn_bundle(AxesBundle {
            axes: Axes {
                size: 3.,
                inner_offset: 0.,
            },
            mesh: meshes.add(line_mesh.clone()),
            material: lines_materials.add(LinesMaterial {}),
            transform: Transform::from_xyz(0.0, 7.0, 0.0),
            ..Default::default()
        })
        .insert(Name::new("Axis: Y"))
        .insert(Rotate {
            axis: Vec3::new(0.0, 1.0, 0.0),
            angle: 1.0,
        })
        .insert(RandomLines);

    // z - axis
    commands
        .spawn_bundle(AxesBundle {
            axes: Axes {
                size: 3.,
                inner_offset: 0.,
            },
            mesh: meshes.add(line_mesh.clone()),
            material: lines_materials.add(LinesMaterial {}),
            transform: Transform::from_xyz(0.0, 0.0, -7.0),
            ..Default::default()
        })
        .insert(Name::new("Axis: Z"))
        .insert(Rotate {
            axis: Vec3::new(0.0, 0.0, 1.0),
            angle: 1.0,
        })
        .insert(RandomLines);

    let theta = std::f32::consts::FRAC_PI_4;
    let light_transform = Mat4::from_euler(EulerRot::ZYX, 0.0, std::f32::consts::FRAC_PI_2, -theta);
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(1.0, 1.0, 1.0),
            illuminance: 5000.,
            ..Default::default()
        },
        transform: Transform::from_matrix(light_transform),
        ..Default::default()
    });

    // orbit camera
    commands
        .spawn_bundle(Camera3dBundle {
            ..Default::default()
        })
        .insert(OrbitCamera {
            center: Vec3::new(0.0, 1.0, 0.0),
            distance: 10.0,
            ..Default::default()
        });

    // FPS on screen
    commands.spawn_bundle(TextBundle {
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
    });

    // voxels
    let voxels: Vec<Voxel> = vec![
        Voxel {
            position: Vec3::new(6., 0., 0.),
            size: 0.5,
            color: *Color::RED.clone().set_a(0.1),
        },
        Voxel {
            position: Vec3::new(0., 6., 0.),
            size: 0.5,
            color: *Color::GREEN.clone().set_a(0.1),
        },
        Voxel {
            position: Vec3::new(0., 0., -6.),
            size: 0.5,
            color: *Color::BLUE.clone().set_a(0.1),
        },
    ];
    commands
        .spawn_bundle(VoxelsBundle {
            voxels: Voxels { voxels },
            mesh: meshes.add(voxel_mesh.clone()),
            material: voxels_materials.add(VoxelsMaterial {}),
            ..Default::default()
        })
        .insert(Name::new("Voxels"))
        .insert(Rotate {
            axis: Vec3::new(0.0, 1.0, 0.0),
            angle: 1.0,
        });

    // rod mesh
    let rod_mesh = simula_viz::rod::Rod {
        ..Default::default()
    };
    let rod_mesh = simula_viz::rod::RodMesh::from(rod_mesh);
    commands
        .spawn()
        .insert_bundle(PbrBundle {
            mesh: meshes.add(rod_mesh.mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::PINK,
                ..Default::default()
            }),
            transform: Transform::from_xyz(5.0, 0.0, -5.0),
            ..Default::default()
        })
        .insert(Name::new("Shape: Rod"));

    // metric plane mesh
    commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.load("models/metric_plane/metric_plane_8x8.gltf#Scene0"),
            ..default()
        })
        .insert(Name::new("Metric: Plane"));

    // metric box mesh
    commands
        .spawn_bundle(SceneBundle {
            scene: asset_server.load("models/metric_box/metric_box_1x1.gltf#Scene0"),
            transform: Transform::from_xyz(-2.5, 0.0, 2.5),
            ..default()
        })
        .insert(Name::new("Metric: Box"));

    // ease functions
    let points: Vec<Vec3> = (0i32..=100)
        .map(|i| Vec3::new((i as f32) * 0.01, 0.0, 0.0))
        .collect();
    commands
        .spawn()
        .insert_bundle(SpatialBundle {
            transform: Transform::from_xyz(7.5, -8.0, 0.0)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            ..Default::default()
        })
        .with_children(|parent| {
            for (i, ease_func) in EaseFunction::into_enum_iter().enumerate().skip(1) {
                let i = i - 1;
                // println!("{:2}: {:?}", i, ease_func);
                let name = ease_func.to_string();
                parent
                    .spawn_bundle(LinesBundle {
                        mesh: meshes.add(line_mesh.clone()),
                        material: lines_materials.add(LinesMaterial {}),
                        transform: Transform::from_xyz(
                            ((i / 3) as f32) * 1.5,
                            3.0 - ((i % 3) as f32),
                            0.0,
                        ),
                        ..Default::default()
                    })
                    .insert(EaseLine {
                        points: points.clone(),
                        ease_func,
                    })
                    .insert(Name::new(name));
            }
        })
        .insert(Name::new("Easings"));

    // generator signals
    let points: Vec<Vec3> = (-100i32..=100)
        .map(|i| Vec3::new((i as f32) * 0.01, 0.0, 0.0))
        .collect();
    for (i, signal_func) in SignalFunction::into_enum_iter().enumerate().skip(1) {
        let name = signal_func.to_string();
        commands
            .spawn_bundle(LinesBundle {
                mesh: meshes.add(line_mesh.clone()),
                material: lines_materials.add(LinesMaterial {}),
                transform: Transform::from_xyz(0.0, 3.0 - (i as f32 * 0.2), 0.0),
                ..Default::default()
            })
            .insert(SignalGenerator {
                func: signal_func,
                amplitude: 0.1,
                frequency: 3.0,
                ..Default::default()
            })
            .insert(SignalGeneratorLine {
                points: points.clone(),
            })
            .insert(Name::new(name));
    }

    // control signals
    commands
        .spawn_bundle(LinesBundle {
            mesh: meshes.add(line_mesh.clone()),
            material: lines_materials.add(LinesMaterial {}),
            transform: Transform::from_xyz(0.0, 4.0, 0.0),
            ..Default::default()
        })
        .insert(SignalGenerator {
            func: SignalFunction::Pulse,
            amplitude: 1.0,
            frequency: 1.0,
            ..Default::default()
        })
        .insert(SignalGeneratorLine {
            points: points.clone(),
        })
        .insert(SignalController::<f32> {
            kp: 0.1,
            ki: 0.0,
            kd: 0.0,
            ..Default::default()
        })
        .insert(SignalControlLine {
            points: points.clone(),
        })
        .insert(Name::new("Signal: Controller"));

    // force graph
    let mut graph_bundle = ForceGraphBundle::<SandboxNodeData, SandboxEdgeData> {
        mesh: meshes.add(line_mesh.clone()),
        material: lines_materials.add(LinesMaterial {}),
        transform: Transform::from_xyz(0.0, 3.5, 0.0),
        ..Default::default()
    };
    let graph = &mut graph_bundle.graph.graph;

    commands
        .spawn()
        .insert(Name::new("Force-directed Graph"))
        .insert(SandboxGraph)
        .with_children(|parent| {
            let root_index = graph.add_node(NodeData::<SandboxNodeData> {
                is_anchor: true,
                ..Default::default()
            });

            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::UVSphere {
                        radius: 0.1,
                        ..Default::default()
                    })),
                    material: materials.add(Color::GOLD.into()),
                    transform: Transform::from_xyz(0.0, 0.5, 0.0),
                    ..Default::default()
                })
                .insert(SandboxNode {
                    node_index: root_index,
                });

            for _ in 0..10 {
                let node_index = graph.add_node(NodeData::<SandboxNodeData> {
                    position: Vec3::new(rand::random(), rand::random(), rand::random()) * 0.01,
                    mass: 1.0,
                    ..Default::default()
                });

                graph.add_edge(root_index, node_index, Default::default());

                parent
                    .spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::UVSphere {
                            radius: 0.1,
                            ..Default::default()
                        })),
                        material: materials.add(Color::ALICE_BLUE.into()),
                        transform: Transform::from_xyz(0.0, 0.5, 0.0),
                        ..Default::default()
                    })
                    .insert(SandboxNode { node_index });

                let parent_index = node_index;
                for _ in 0..3 {
                    let node_index = graph.add_node(NodeData::<SandboxNodeData> {
                        position: Vec3::new(rand::random(), rand::random(), rand::random()) * 0.01,
                        mass: 1.0,
                        ..Default::default()
                    });

                    graph.add_edge(parent_index, node_index, Default::default());

                    parent
                        .spawn_bundle(PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::UVSphere {
                                radius: 0.1,
                                ..Default::default()
                            })),
                            material: materials.add(Color::ALICE_BLUE.into()),
                            transform: Transform::from_xyz(0.0, 0.5, 0.0),
                            ..Default::default()
                        })
                        .insert(SandboxNode { node_index });
                }
            }
        })
        .insert_bundle(graph_bundle);

    // pointcloud
    commands.spawn().insert_bundle((
        meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
        Transform::from_xyz(0.0, 10.0, 10.0),
        GlobalTransform::default(),
        Pointcloud(
            (1..=10)
                .flat_map(|x| (1..=10).map(move |y| (x as f32 / 10.0, y as f32 / 10.0)))
                .map(|(x, y)| PointData {
                    position: Vec3::new(x * 10.0 - 5.0, y * 10.0 - 5.0, 0.0),
                    scale: 1.0,
                    color: Color::hsla(x * 360., y, 0.5, 1.0).as_rgba_f32(),
                })
                .collect(),
        ),
        Visibility::default(),
        ComputedVisibility::default(),
        NoFrustumCulling,
    ));

    #[cfg(feature = "gif")]
    {
        // video robot
        let video_material = StandardMaterial {
            base_color: Color::rgb(1.0, 1.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..Default::default()
        };
        let video_asset: Handle<GifAsset> = asset_server.load("videos/robot.gif");
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
                material: materials.add(video_material),
                transform: Transform::from_xyz(0.0, 0.5, -2.0)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                ..Default::default()
            })
            .insert(VideoPlayer {
                start_frame: 0,
                end_frame: 80,
                framerate: 20.0,
                playing: true,
                ..Default::default()
            })
            .insert(video_asset)
            .insert(Name::new("Video: Robot"));
    }

    #[cfg(feature = "webp")]
    {
        // video robot
        let video_material = StandardMaterial {
            base_color: Color::rgb(1.0, 1.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..Default::default()
        };
        let video_asset: Handle<WebPAsset> = asset_server.load("videos/robot.webp");
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
                material: materials.add(video_material),
                transform: Transform::from_xyz(0.0, 0.5, -2.0)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                ..Default::default()
            })
            .insert(VideoPlayer {
                start_frame: 0,
                end_frame: 80,
                framerate: 20.0,
                playing: true,
                ..Default::default()
            })
            .insert(video_asset)
            .insert(Name::new("Video: Robot"));
    }

    #[cfg(feature = "gst")]
    {
        // video stream
        let video_material = StandardMaterial {
            base_color: Color::rgb(1.0, 1.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..Default::default()
        };
        let gst_asset = create_gst();
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
                material: materials.add(video_material),
                transform: Transform::from_xyz(2.0, 0.5, -3.0)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::new(1.6, 1.0, 1.0)),
                ..Default::default()
            })
            .insert(VideoPlayer {
                start_frame: 0,
                end_frame: 80,
                framerate: 20.0,
                playing: true,
                ..Default::default()
            })
            .insert(gst_asset)
            .insert(Name::new("Video: Gst"));
    }
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

#[derive(Component)]
struct RandomLines;

#[derive(Component)]
struct Rotate {
    axis: Vec3,
    angle: f32,
}

fn rotate_system(time: Res<Time>, mut query: Query<(&Rotate, &mut Transform)>) {
    for (rotate, mut transform) in query.iter_mut() {
        transform.rotate(Quat::from_axis_angle(
            rotate.axis,
            rotate.angle * time.delta_seconds(),
        ));
    }
}

fn line_test(mut lines: Query<&mut Lines, With<RandomLines>>) {
    let mut rng = rand::thread_rng();
    let die = Uniform::from(0f32..1f32);

    for mut lines in lines.iter_mut() {
        for _ in 0..20 {
            let x = die.sample(&mut rng) * 0.2 - 0.1;
            let z = die.sample(&mut rng) * 0.2 - 0.1;
            let start = Vec3::new(x, -0.1, z);
            let end = Vec3::new(x, 0.1, z);

            let color = Color::Hsla {
                hue: die.sample(&mut rng) * 360.0,
                lightness: 0.5,
                saturation: 1.0,
                alpha: 1.0,
            };
            lines.line_colored(start, end, color);
        }
    }
}

#[derive(Reflect, Component, Default, Clone, PartialEq)]
#[reflect(Component)]
pub struct SandboxNode {
    #[reflect(ignore)]
    node_index: NodeIndex,
}

#[derive(Reflect, Component, Default, Clone, PartialEq)]
#[reflect(Component)]
pub struct SandboxNodeData;

#[derive(Reflect, Component, Default, Clone, PartialEq)]
#[reflect(Component)]
pub struct SandboxEdgeData;

#[derive(Reflect, Component, Default, Clone, PartialEq)]
#[reflect(Component)]
pub struct SandboxGraph;

fn force_graph_test(
    time: Res<Time>,
    mut graphs: Query<
        (
            &mut ForceGraph<SandboxNodeData, SandboxEdgeData>,
            &Children,
            &mut Lines,
        ),
        With<SandboxGraph>,
    >,
    mut nodes: Query<(&mut Transform, &SandboxNode)>,
) {
    for (mut graph, children, mut lines) in graphs.iter_mut() {
        graph.graph.parameters = graph.parameters.clone();
        graph.graph.update(time.delta());
        graph.graph.visit_edges(|a, b, _| {
            lines.line_gradient(a.position(), b.position(), Color::RED, Color::BLUE);
        });
        for child in children.iter() {
            if let Ok((mut transform, node)) = nodes.get_mut(*child) {
                let node = &graph.graph.get_graph()[*node.node_index];
                transform.translation = node.position();
            }
        }
    }
}
