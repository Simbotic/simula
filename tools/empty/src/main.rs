use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    window::PresentMode,
};
use simula_action::ActionPlugin;
use simula_camera::orbitcam::*;
use simula_inspector::{InspectorPlugin, WorldInspectorPlugin};
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::LinesPlugin,
};

use simula_video::{rt, VideoMaterial, VideoPlayer, VideoPlugin};
#[cfg(feature = "gst")]
use simula_video::{GstSink, GstSrc};


fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(Color::rgb(0.105, 0.10, 0.11)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "[Simbotic] Simula - Empty".to_string(),
                resolution: (940., 528.).into(),
                present_mode: PresentMode::AutoVsync,
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(InspectorPlugin)
        .add_plugin(WorldInspectorPlugin)
        .add_plugin(ActionPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(OrbitCameraPlugin)
        .add_plugin(LinesPlugin)
        .add_plugin(AxesPlugin)
        .add_plugin(GridPlugin)
        .add_plugin(VideoPlugin)
        .add_startup_system(setup)
        // .add_system(debug_info)
        .run();
}

fn setup(mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut video_materials: ResMut<Assets<VideoMaterial>>,
    asset_server: Res<AssetServer>
) {
    // grid
    let grid_color = Color::rgb(0.08, 0.06, 0.08);
    commands
        .spawn(GridBundle {
            grid: Grid {
                size: 10,
                divisions: 10,
                start_color: grid_color,
                end_color: grid_color,
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
    commands.spawn(DirectionalLightBundle {
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
        .spawn(Camera3dBundle {
            ..Default::default()
        })
        .insert(OrbitCamera {
            center: Vec3::new(0.0, 1.0, 0.0),
            distance: 10.0,
            ..Default::default()
        });

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
                mesh: meshes.add(Mesh::from(shape::Plane {
                    size: 1.4,
                    ..default()
                })),
                material: video_materials.add(video_material),
                transform: Transform::from_xyz(0.0, 1.0, -3.0)
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
                pipeline: String::from("rtspsrc location=rtsp://192.168.xx.xx:8554/stream ! appsink async=true name=simula")
            })
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
