use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render::view::RenderLayers,
    utils::HashMap,
    window::PresentMode,
};
use bevy_egui::EguiPlugin;
use bevy_egui::{egui, EguiContexts};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use simula_action::ActionPlugin;
use simula_camera::{flycam::*, orbitcam::*};
#[cfg(feature = "gif")]
use simula_video::GifAsset;
#[cfg(feature = "video")]
use simula_video::VideoSrc;
#[cfg(feature = "webp")]
use simula_video::WebPAsset;
use simula_video::{rt, VideoMaterial, VideoPlayer, VideoPlugin};
#[cfg(feature = "gst")]
use simula_video::{GstSink, GstSrc};
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::LinesPlugin,
};

fn main() {
    let mut app = App::new();

    app.insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(Color::rgb(0.105, 0.10, 0.11)))
        .insert_resource(DeletedEntityVideoResource(HashMap::new()))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "[Simbotic] Simula - Video Player".to_string(),
                resolution: (940., 528.).into(),
                present_mode: PresentMode::AutoVsync,
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(EguiPlugin)
        .add_plugin(WorldInspectorPlugin::default())
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
    mut video_materials: ResMut<Assets<VideoMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // grid
    commands
        .spawn(GridBundle {
            grid: Grid {
                size: 10,
                divisions: 10,
                start_color: Color::BLUE,
                end_color: Color::RED,
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
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
            ..default()
        })
        .insert(Name::new("Axes: World"));

    // x - axis
    commands
        .spawn(AxesBundle {
            axes: Axes {
                size: 3.,
                inner_offset: 0.,
            },
            transform: Transform::from_xyz(7.0, 0.0, 0.0),
            ..default()
        })
        .insert(Name::new("Axis: X"));

    // y - axis
    commands
        .spawn(AxesBundle {
            axes: Axes {
                size: 3.,
                inner_offset: 0.,
            },
            transform: Transform::from_xyz(0.0, 7.0, 0.0),
            ..default()
        })
        .insert(Name::new("Axis: Y"));

    // z - axis
    commands
        .spawn(AxesBundle {
            axes: Axes {
                size: 3.,
                inner_offset: 0.,
            },
            transform: Transform::from_xyz(0.0, 0.0, -7.0),
            ..default()
        })
        .insert(Name::new("Axis: Z"));

    let theta = std::f32::consts::FRAC_PI_4;
    let light_transform = Mat4::from_euler(EulerRot::ZYX, 0.0, std::f32::consts::FRAC_PI_2, -theta);
    commands.spawn(DirectionalLightBundle {
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
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 2.0, -10.0)
                .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
            ..default()
        })
        .insert(RenderLayers::all())
        .with_children(|parent| {
            let mut _child = parent.spawn(Camera3dBundle {
                camera_3d: Camera3d {
                    clear_color: ClearColorConfig::Custom(Color::BLACK),
                    ..default()
                },
                camera: Camera {
                    order: -1,
                    target: bevy::render::camera::RenderTarget::Image(rt_image.clone()),
                    ..default()
                },
                ..default()
            });

            #[cfg(feature = "gst")]
            _child.insert(GstSrc {
                size: UVec2 { x: 256, y: 256 },
                ..default()
            });
        })
        .insert(FlyCamera::default());

    // FPS on screen
    commands.spawn(TextBundle {
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
        let video_material = VideoMaterial {
            color: Color::rgb(1.0, 1.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        };
        let video_asset: Handle<GifAsset> = asset_server.load("videos/robot.gif");
        let video_rotation =
            Quat::from_euler(EulerRot::YXZ, -std::f32::consts::FRAC_PI_3 * 0.0, 0.0, 0.0);
        let video_position = Vec3::new(0.0, 0.5, -2.0);

        commands
            .spawn(SpatialBundle { ..default() })
            .with_children(|parent| {
                parent
                    .spawn(SpatialBundle {
                        transform: Transform::from_translation(video_position)
                            .with_rotation(video_rotation),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn(MaterialMeshBundle {
                                mesh: meshes.add(Mesh::from(shape::Plane {
                                    size: 1.0,
                                    ..default()
                                })),
                                material: video_materials.add(video_material),
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
                            .spawn(AxesBundle {
                                axes: Axes {
                                    size: 1.,
                                    ..default()
                                },
                                ..default()
                            })
                            .insert(Name::new("LookAt Coords"));
                    });

                parent
                    .spawn(AxesBundle {
                        transform: Transform::from_translation(video_position)
                            .with_rotation(video_rotation),
                        axes: Axes {
                            size: 3.,
                            ..default()
                        },
                        visibility: Visibility::Hidden,
                        ..default()
                    })
                    .insert(Name::new("LookAt Coords"));
            });
    }

    #[cfg(feature = "webp")]
    {
        // video robot
        let video_material = VideoMaterial {
            color: Color::rgb(1.0, 1.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        };
        let video_asset: Handle<WebPAsset> = asset_server.load("videos/robot.webp");
        commands
            .spawn(MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
                material: video_materials.add(video_material),
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
        let video_material = VideoMaterial {
            color: Color::rgb(1.0, 1.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        };
        // let video_asset: Handle<WebPAsset> = asset_server.load("videos/robot.webp");
        commands
            .spawn(SpatialBundle {
                transform: Transform::from_xyz(2.0, 0.5, -2.0),
                ..default()
            })
            .insert(Name::new("Video: Robot"))
            .with_children(|parent| {
                parent
                    .spawn(MaterialMeshBundle {
                        mesh: meshes.add(Mesh::from(shape::Plane {
                            size: 1.0,
                            ..default()
                        })),
                        material: video_materials.add(video_material),
                        transform: Transform::from_rotation(Quat::from_rotation_x(
                            -std::f32::consts::FRAC_PI_2,
                        ))
                        .with_scale(Vec3::new(1.8, 1.0, 1.0)),
                        ..default()
                    })
                    .insert(VideoSrc {
                        size: UVec2 { x: 320, y: 176 },
                        src: "assets/videos/mov_bbb.mp4".into(),
                        playing: true,
                        _loop: true,
                    })
                    .insert(Visibility::Visible)
                    .insert(Name::new("Robot: Body"));
                parent
                    .spawn(AxesBundle {
                        axes: Axes {
                            size: 1.,
                            ..default()
                        },
                        ..default()
                    })
                    .insert(Name::new("LookAt Coords"));
            });
    }

    #[cfg(feature = "gst")]
    {
        // video stream
        let video_material = VideoMaterial {
            color: Color::rgb(1.0, 1.0, 1.0),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        };
        commands
            .spawn(MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
                material: video_materials.add(video_material),
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
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: 1.0,
                ..default()
            })),
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
    mut commands: Commands,
    mut egui_ctx: EguiContexts,
    mut video_sources: Query<(Entity, &mut VideoSrc, Option<&mut Visibility>)>,
    mut deleted_video_sources: ResMut<DeletedEntityVideoResource>,
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
                    for (_, mut video, _) in video_sources.iter_mut() {
                        video.playing = true;
                    }
                });
            ui.small_button("pause")
                .on_hover_text("pause video")
                .clicked()
                .then(|| {
                    for (_, mut video, _) in video_sources.iter_mut() {
                        video.playing = false;
                    }
                });
            ui.small_button("remove")
                .on_hover_text("remove video")
                .clicked()
                .then(|| {
                    for (entity, src, _) in video_sources.iter_mut() {
                        deleted_video_sources.0.insert(entity, src.to_owned());
                        commands.entity(entity).remove::<VideoSrc>();
                    }
                });
            ui.small_button("spawn")
                .on_hover_text("spawn video")
                .clicked()
                .then(|| {
                    for video in deleted_video_sources.0.iter() {
                        let mut src = video.1.to_owned();
                        src.playing = true;
                        commands.entity(video.0.to_owned()).insert(src);
                        commands
                            .entity(video.0.to_owned())
                            .insert(Visibility::Visible);
                    }
                    deleted_video_sources.0 = HashMap::new();
                });
            ui.small_button("visibility")
                .on_hover_text("show or hide video")
                .clicked()
                .then(|| {
                    for (_, _, visible) in video_sources.iter_mut() {
                        if let Some(mut visible) = visible {
                            let visible_match = visible.clone();
                            match visible_match {
                                Visibility::Visible => {
                                    *visible = Visibility::Hidden;
                                }
                                Visibility::Hidden => {
                                    *visible = Visibility::Visible;
                                }
                                Visibility::Inherited => {}
                            }
                        }
                    }
                });
        });
}

#[derive(Debug, Clone, Component, Resource)]
pub struct DeletedEntityVideoResource(HashMap<Entity, VideoSrc>);
