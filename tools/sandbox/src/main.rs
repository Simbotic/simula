use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    // log::LogPlugin,
    prelude::*,
    render::view::{NoFrustumCulling, RenderLayers},
};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_inspector_egui::WorldInspectorPlugin;
use enum_iterator::IntoEnumIterator;
use monkey::MonkeyPlugin;
use rand::distributions::{Distribution, Uniform};
use simula_action::ActionPlugin;
use simula_authority::{Minion, NetAuthorityPlugin, Worker};
#[cfg(not(target_arch = "wasm32"))]
use simula_cad::shapes::{self, ShapeMesh};
use simula_camera::{flycam::*, orbitcam::*};
use simula_core::{
    ease::EaseFunction,
    force_graph::{NodeData, NodeIndex, SimulationParameters},
    signal::{SignalController, SignalFunction, SignalGenerator},
};
use simula_net::{NetId, NetPlugin, Replicate};
#[cfg(feature = "gif")]
use simula_video::GifAsset;
#[cfg(feature = "webp")]
use simula_video::WebPAsset;
use simula_video::{rt, RawSrc, VideoPlayer, VideoPlugin};
#[cfg(feature = "gst")]
use simula_video::{GstSink, GstSrc};
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    ease::{ease_lines, EaseLine},
    follow_ui::{FollowUI, FollowUICamera, FollowUIPlugin},
    force_graph::{ForceGraph, ForceGraphBundle},
    grid::{Grid, GridBundle, GridPlugin},
    lines::{LineMesh, Lines, LinesBundle, LinesMaterial, LinesPlugin},
    lookat::{LookAtPlugin, SmoothLookAt},
    pointcloud::{PointData, Pointcloud, PointcloudPlugin},
    signal::{
        signal_control_lines, signal_generator_lines, SignalControlLine, SignalGeneratorLine,
    },
    voxel::{Voxel, VoxelMesh, Voxels, VoxelsBundle, VoxelsMaterial, VoxelsPlugin},
};

mod monkey;

fn main() {
    let mut app = App::new();

    app.register_type::<SignalGenerator>()
        .register_type::<SignalFunction>()
        .register_type::<SignalController<f32>>()
        .register_type::<ForceGraph<SandboxNodeData, SandboxEdgeData>>()
        .register_type::<SimulationParameters>()
        .register_type::<SandboxNode>()
        .insert_resource(WindowDescriptor {
            title: "[Simbotic] Simula - Sandbox".to_string(),
            width: 940.,
            height: 528.,
            ..default()
        })
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.105, 0.10, 0.11)))
        .add_plugins(DefaultPlugins)
        // .add_plugins_with(DefaultPlugins, |plugins| plugins.disable::<LogPlugin>())
        .add_plugin(NetPlugin)
        .add_plugin(NetAuthorityPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(ActionPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(OrbitCameraPlugin)
        .add_plugin(FlyCameraPlugin)
        .add_plugin(LinesPlugin)
        .add_plugin(AxesPlugin)
        .add_plugin(GridPlugin)
        .add_plugin(VoxelsPlugin)
        .add_plugin(PointcloudPlugin)
        .add_plugin(MonkeyPlugin)
        .add_plugin(VideoPlugin)
        .add_plugin(LookAtPlugin)
        .add_plugin(FollowUIPlugin)
        .add_startup_system(setup)
        .add_system(debug_info)
        .add_system(line_test)
        .add_system(follow_ui)
        .add_system(ease_lines)
        .add_system(signal_generator_lines)
        .add_system(signal_control_lines)
        .add_system(rotate_system)
        .add_system(force_graph_test);

    // bevy_mod_debugdump::print_schedule(&mut app);
    // bevy_mod_debugdump::print_render_schedule(&mut app);
    // bevy_mod_debugdump::print_render_graph(&mut app);

    app.run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut voxels_materials: ResMut<Assets<VoxelsMaterial>>,
    mut lines_materials: ResMut<Assets<LinesMaterial>>,
    line_mesh: Res<LineMesh>,
    voxel_mesh: Res<VoxelMesh>,
    asset_server: Res<AssetServer>,
) {
    // network minion that will only communicate with authority
    commands
        .spawn()
        .insert(Minion::default())
        .insert(NetId::default())
        .insert(Name::new("Minion"));

    // network worker that will sync with everyone
    commands
        .spawn()
        .insert(Worker::default())
        .insert(Replicate::<Worker>::default())
        .insert(NetId::default())
        .insert(Name::new("Worker"));

    // network minion worker that will sync worker state with others
    // but only minion state with authority
    commands
        .spawn()
        .insert(Minion::default())
        .insert(Worker::default())
        .insert(Replicate::<Worker>::default())
        .insert(NetId::default())
        .insert(Name::new("MinionWorker"));

    #[cfg(not(target_arch = "wasm32"))]
    {
        // CAD shape
        let shape = shapes::star(5, Color::BLUE);
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(shape.to_mesh()),
                material: materials.add(Color::rgb(0.0, 0.0, 1.0).into()),
                transform: Transform::from_xyz(0.0, -10.0, 0.0),
                ..default()
            })
            .insert(Name::new("Shape: Star"));
    }

    // plane
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform::from_xyz(2.0, 0.01, 2.0),
            ..default()
        })
        .insert(Name::new("Shape: Plane"));

    // cube
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(-2.5, 0.0, -1.5),
            ..default()
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
                ..default()
            },
            mesh: meshes.add(line_mesh.clone()),
            material: lines_materials.add(LinesMaterial {}),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
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
            ..default()
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
            ..default()
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
            ..default()
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
            ..default()
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
            ..default()
        },
        transform: Transform::from_matrix(light_transform),
        ..default()
    });

    let rt_image = images.add(rt::default_render_target_image());

    let camera_entity = commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(-10.0, 2.0, 0.0)
                .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
            ..default()
        })
        .insert(RenderLayers::all())
        .with_children(|parent| {
            let mut child = parent.spawn_bundle(Camera3dBundle {
                camera_3d: Camera3d {
                    clear_color: ClearColorConfig::Custom(Color::BLACK),
                    ..default()
                },
                camera: Camera {
                    priority: -1,
                    target: bevy::render::camera::RenderTarget::Image(rt_image.clone()),
                    ..default()
                },
                ..default()
            });

            // RawSrc is optional when using GstSrc, as it will be set automatically
            child.insert(RawSrc::default());

            #[cfg(feature = "gst")]
            child.insert(GstSrc::default());
        })
        .insert(FlyCamera { ..default() })
        .insert(FollowUICamera)
        .id();

    for i in 0..5 {
        // Follow UI over torus
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Torus {
                    radius: 0.5,
                    ring_radius: 0.1,
                    ..default()
                })),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(3.0 + i as f32 * 5.0, 0.0, 0.0),
                ..default()
            })
            .with_children(|parent| {
                parent
                    .spawn_bundle(AxesBundle {
                        axes: Axes {
                            size: 1.,
                            inner_offset: 1.,
                            ..default()
                        },
                        mesh: meshes.add(line_mesh.clone()),
                        material: lines_materials.add(LinesMaterial {}),
                        transform: Transform::from_xyz(0.0, 1.0, 0.0),
                        ..default()
                    })
                    .insert(FollowUI {
                        min_distance: 0.1,
                        max_distance: 10.0,
                        min_height: -5.0,
                        max_height: 5.0,
                        max_view_angle: 45.0,
                        ..default()
                    })
                    .insert(FollowPanel)
                    .insert(Name::new("FollowUI: Axes"));
            })
            .insert(Name::new("Follow UI: Shape"));
    }

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
            ..default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
            ..default()
        },
        ..default()
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
            ..default()
        })
        .insert(Name::new("Voxels"))
        .insert(Rotate {
            axis: Vec3::new(0.0, 1.0, 0.0),
            angle: 1.0,
        });

    // rod mesh
    let rod_mesh = simula_viz::rod::Rod { ..default() };
    let rod_mesh = simula_viz::rod::RodMesh::from(rod_mesh);
    commands
        .spawn()
        .insert_bundle(PbrBundle {
            mesh: meshes.add(rod_mesh.mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::PINK,
                ..default()
            }),
            transform: Transform::from_xyz(5.0, 0.0, -5.0),
            ..default()
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
            ..default()
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
                        ..default()
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
                ..default()
            })
            .insert(SignalGenerator {
                func: signal_func,
                amplitude: 0.1,
                frequency: 3.0,
                ..default()
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
            ..default()
        })
        .insert(SignalGenerator {
            func: SignalFunction::Pulse,
            amplitude: 1.0,
            frequency: 1.0,
            ..default()
        })
        .insert(SignalGeneratorLine {
            points: points.clone(),
        })
        .insert(SignalController::<f32> {
            kp: 0.1,
            ki: 0.0,
            kd: 0.0,
            ..default()
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
        ..default()
    };
    let graph = &mut graph_bundle.graph.graph;

    commands
        .spawn()
        .insert(Name::new("Force-directed Graph"))
        .insert(SandboxGraph)
        .with_children(|parent| {
            let root_index = graph.add_node(NodeData::<SandboxNodeData> {
                is_anchor: true,
                ..default()
            });

            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::UVSphere {
                        radius: 0.1,
                        ..default()
                    })),
                    material: materials.add(Color::GOLD.into()),
                    transform: Transform::from_xyz(0.0, 0.5, 0.0),
                    ..default()
                })
                .insert(SandboxNode {
                    node_index: root_index,
                });

            for _ in 0..10 {
                let node_index = graph.add_node(NodeData::<SandboxNodeData> {
                    position: Vec3::new(rand::random(), rand::random(), rand::random()) * 0.01,
                    mass: 1.0,
                    ..default()
                });

                graph.add_edge(root_index, node_index, Default::default());

                parent
                    .spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::UVSphere {
                            radius: 0.1,
                            ..default()
                        })),
                        material: materials.add(Color::ALICE_BLUE.into()),
                        transform: Transform::from_xyz(0.0, 0.5, 0.0),
                        ..default()
                    })
                    .insert(SandboxNode { node_index });

                let parent_index = node_index;
                for _ in 0..3 {
                    let node_index = graph.add_node(NodeData::<SandboxNodeData> {
                        position: Vec3::new(rand::random(), rand::random(), rand::random()) * 0.01,
                        mass: 1.0,
                        ..default()
                    });

                    graph.add_edge(parent_index, node_index, Default::default());

                    parent
                        .spawn_bundle(PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::UVSphere {
                                radius: 0.1,
                                ..default()
                            })),
                            material: materials.add(Color::ALICE_BLUE.into()),
                            transform: Transform::from_xyz(0.0, 0.5, 0.0),
                            ..default()
                        })
                        .insert(SandboxNode { node_index });
                }
            }
        })
        .insert_bundle(graph_bundle);

    // pointcloud
    commands
        .spawn()
        .insert_bundle((
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
        ))
        .insert(Name::new("Pointcloud"));

    #[cfg(feature = "gif")]
    {
        // video robot
        let video_material = StandardMaterial {
            base_color: Color::rgb(1.0, 1.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        };
        let video_asset: Handle<GifAsset> = asset_server.load("videos/robot.gif");
        let video_rotation =
            Quat::from_euler(EulerRot::YXZ, -std::f32::consts::FRAC_PI_3 * 0.0, 0.0, 0.0);
        let video_position = Vec3::new(0.0, 0.5, -2.0);

        commands
            .spawn_bundle(SpatialBundle {
                transform: Transform::from_translation(video_position)
                    .with_rotation(video_rotation),
                ..default()
            })
            .with_children(|parent| {
                parent
                    .spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
                        material: materials.add(video_material),
                        transform: Transform::from_rotation(Quat::from_euler(
                            EulerRot::YXZ,
                            0.0,
                            -std::f32::consts::FRAC_PI_2,
                            0.0,
                        )),
                        ..default()
                    })
                    .insert(VideoPlayer {
                        start_frame: 0,
                        end_frame: 80,
                        framerate: 20.0,
                        playing: true,
                        ..default()
                    })
                    .insert(video_asset)
                    .insert(Name::new("Video: RenderTarget"));
            })
            .insert(Name::new("Video: Robot"))
            .insert(SmoothLookAt {
                target: Some(camera_entity),
                initial_pose: video_rotation,
                yaw_ease: EaseFunction::SineInOut,
                pitch_ease: EaseFunction::SineInOut,
                ..default()
            })
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
            });

        commands
            .spawn_bundle(AxesBundle {
                transform: Transform::from_translation(video_position)
                    .with_rotation(video_rotation),
                axes: Axes {
                    size: 3.,
                    ..default()
                },
                visibility: Visibility { is_visible: false },
                mesh: meshes.add(line_mesh.clone()),
                material: lines_materials.add(LinesMaterial {}),
                ..default()
            })
            .insert(Name::new("LookAt Coords"));
    }

    #[cfg(feature = "webp")]
    {
        // video robot
        let video_material = StandardMaterial {
            base_color: Color::rgb(1.0, 1.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        };
        let video_asset: Handle<WebPAsset> = asset_server.load("videos/robot.webp");
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
                material: materials.add(video_material),
                transform: Transform::from_xyz(0.0, 0.5, -2.0)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                ..default()
            })
            .insert(VideoPlayer {
                start_frame: 0,
                end_frame: 80,
                framerate: 20.0,
                playing: true,
                ..default()
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
            ..default()
        };
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
                material: materials.add(video_material),
                transform: Transform::from_xyz(2.5, 0.5, -3.0)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                    .with_scale(Vec3::new(1.0, 1.0, 1.0)),
                ..default()
            })
            .insert(VideoPlayer {
                start_frame: 0,
                end_frame: 80,
                framerate: 20.0,
                playing: true,
                ..default()
            })
            .insert(GstSink::default())
            .insert(Name::new("Video: Gst"));
    }

    // render target
    let rt_material = StandardMaterial {
        base_color: Color::rgb(1.0, 1.0, 1.0),
        base_color_texture: Some(rt_image),
        unlit: true,
        ..default()
    };
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
            material: materials.add(rt_material),
            transform: Transform::from_xyz(-2.5, 0.5, -3.0)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::new(1.0, 1.0, 1.0)),
            ..default()
        })
        .insert(RenderLayers::layer(1))
        .insert(Name::new("Video: RenderTarget"));
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

#[derive(Component)]
struct FollowPanel;

fn follow_ui(
    time: Res<Time>,
    mut egui_context: ResMut<EguiContext>,
    follow_uis: Query<(Entity, &FollowUI), With<FollowPanel>>,
) {
    for (entity, follow_ui) in follow_uis.iter() {
        let ui_pos = if let Some(ui_pos) = follow_ui.screen_pos {
            ui_pos
        } else {
            break;
        };

        let my_frame = egui::containers::Frame {
            // margin: egui::style::Margin { left: 10., right: 10., top: 10., bottom: 10. },
            rounding: egui::Rounding {
                nw: 3.0,
                ne: 3.0,
                sw: 3.0,
                se: 3.0,
            },
            fill: egui::Color32::from_rgba_premultiplied(50, 0, 50, 50),
            ..default()
        };

        egui::Window::new("Follow UI")
            .id(egui::Id::new(entity))
            .frame(my_frame)
            .fixed_size(egui::Vec2::new(follow_ui.size.x, follow_ui.size.y))
            .fixed_pos(egui::Pos2::new(ui_pos.x, ui_pos.y))
            .collapsible(false)
            .title_bar(false)
            .show(egui_context.ctx_mut(), |ui| {
                let time = time.seconds_since_startup();

                let circle_line = {
                    let n = 512;
                    let circle_points: egui::plot::PlotPoints = (0..=n)
                        .map(|i| {
                            let t = egui::remap(
                                i as f64,
                                0.0..=(n as f64),
                                0.0..=std::f64::consts::TAU,
                            );
                            let r = 1.0;
                            [r * t.cos(), r * t.sin()]
                        })
                        .collect();
                    egui::plot::Line::new(circle_points)
                        .color(egui::Color32::from_rgb(100, 200, 100))
                        .style(egui::plot::LineStyle::Solid)
                };

                let sin_line = {
                    egui::plot::Line::new(egui::plot::PlotPoints::from_explicit_callback(
                        move |x| 0.5 * (2.0 * x).sin() * time.sin(),
                        ..,
                        512,
                    ))
                    .color(egui::Color32::from_rgb(200, 100, 100))
                    .style(egui::plot::LineStyle::Solid)
                };

                let thingy_line = {
                    egui::plot::Line::new(egui::plot::PlotPoints::from_parametric_callback(
                        move |t| ((2.0 * t + time).sin(), (3.0 * t).sin()),
                        0.0..=std::f64::consts::TAU,
                        256,
                    ))
                    .color(egui::Color32::from_rgb(100, 150, 250))
                    .style(egui::plot::LineStyle::Solid)
                    .name(format!("t = {:.2}", time))
                };

                let plot =
                    egui::plot::Plot::new("lines_demo").legend(egui::plot::Legend::default());
                plot.show_background(false).show(ui, |plot_ui| {
                    plot_ui.line(circle_line);
                    plot_ui.line(sin_line);
                    plot_ui.line(thingy_line);
                });
            });
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
