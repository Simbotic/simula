use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    // log::LogPlugin,
    prelude::*,
    render::view::RenderLayers,
};
use bevy_egui::EguiPlugin;
use bevy_egui::{egui, EguiContext};
use bevy_inspector_egui::WorldInspectorPlugin;
use simula_action::ActionPlugin;
use simula_camera::{flycam::*, orbitcam::*};
#[cfg(feature = "gif")]
use simula_video::GifAsset;
#[cfg(feature = "video")]
use simula_video::VideoSrc;
#[cfg(feature = "webp")]
use simula_video::WebPAsset;
use simula_video::{rt, VideoPlayer, VideoPlugin};
#[cfg(feature = "gst")]
use simula_video::{GstSink, GstSrc};
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::{LineMesh, LinesMaterial, LinesPlugin},
};

fn main() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "[Simbotic] Simula - Sandbox".to_string(),
        width: 940.,
        height: 528.,
        ..default()
    })
    .insert_resource(Msaa { samples: 4 })
    .insert_resource(ClearColor(Color::rgb(0.105, 0.10, 0.11)))
    .add_plugins(DefaultPlugins)
    .add_plugin(EguiPlugin)
    .add_plugin(WorldInspectorPlugin::new())
    .add_plugin(ActionPlugin)
    .add_plugin(FrameTimeDiagnosticsPlugin::default())
    .add_plugin(OrbitCameraPlugin)
    .add_plugin(FlyCameraPlugin)
    .add_plugin(LinesPlugin)
    .add_plugin(AxesPlugin)
    .add_plugin(GridPlugin)
    .add_plugin(VideoPlugin)
    .add_startup_system(setup)
    .add_system(video_control_window)
    .add_system(debug_info);

    app.run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut lines_materials: ResMut<Assets<LinesMaterial>>,
    line_mesh: Res<LineMesh>,
    asset_server: Res<AssetServer>,
) {
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
        .insert(Name::new("Axis: X"));

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
        .insert(Name::new("Axis: Y"));

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
        .insert(Name::new("Axis: Z"));

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

    let rt_image = images.add(rt::common_render_target_image(UVec2 { x: 256, y: 256 }));

    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 2.0, -10.0)
                .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
            ..default()
        })
        .insert(RenderLayers::all())
        .with_children(|parent| {
            let mut _child = parent.spawn_bundle(Camera3dBundle {
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

            #[cfg(feature = "gst")]
            child.insert(GstSrc {
                size: UVec2 { x: 256, y: 256 },
                ..default()
            });
        })
        .insert(FlyCamera::default());

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
            .spawn_bundle(SpatialBundle { ..default() })
            .with_children(|parent| {
                parent
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

                parent
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
            });
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

    #[cfg(feature = "video")]
    {
        // video robot
        let video_material = StandardMaterial {
            base_color: Color::rgb(1.0, 1.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: Some(wgpu_types::Face::Front),
            ..default()
        };
        // let video_asset: Handle<WebPAsset> = asset_server.load("videos/robot.webp");
        commands
            .spawn_bundle(SpatialBundle {
                transform: Transform::from_xyz(2.0, 0.5, -2.0),
                ..default()
            })
            .insert(Name::new("Video: Robot"))
            .with_children(|parent| {
                parent
                    .spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
                        material: materials.add(video_material),
                        transform: Transform::from_rotation(Quat::from_euler(
                            EulerRot::ZXY,
                            -std::f32::consts::PI,
                            std::f32::consts::FRAC_PI_2,
                            0.0,
                        ))
                        .with_scale(Vec3::new(1.6, 1.0, 1.0)),
                        ..default()
                    })
                    .insert(VideoSrc {
                        size: UVec2 { x: 320, y: 240 },
                        src: "assets/videos/mov_bbb.mp4".into(),
                        playing: true,
                    })
                    .insert(Name::new("Robot: Body"));
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
            .insert(GstSink {
                size: UVec2::new(256, 256),
                ..default()
            })
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

fn video_control_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut video_sources: Query<&mut VideoSrc>,
) {
    egui::Window::new("Panel")
        .default_width(200.0)
        .resizable(true)
        .collapsible(false)
        .title_bar(true)
        .vscroll(false)
        .drag_bounds(egui::Rect::EVERYTHING)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.small_button("play")
                .on_hover_text("play video")
                .clicked()
                .then(|| {
                    for mut video in video_sources.iter_mut() {
                        video.playing = true;
                    }
                });
            ui.small_button("pause")
                .on_hover_text("pause video")
                .clicked()
                .then(|| {
                    for mut video in video_sources.iter_mut() {
                        video.playing = false;
                    }
                });
        });
}
