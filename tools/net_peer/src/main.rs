use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    log::LogPlugin,
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use simula_action::ActionPlugin;
use simula_authority::{Minion, NetAuthorityPlugin, Worker};
use simula_camera::flycam::*;
use simula_net::{NetId, NetPlugin, Replicate};
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::LinesPlugin,
};

fn main() {
    let mut app = App::new();

    app.insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.105, 0.10, 0.11)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: "[Simbotic] Simula - NetPeer".to_string(),
                        width: 940.,
                        height: 528.,
                        ..default()
                    },
                    ..default()
                })
                .set(LogPlugin {
                    filter:
                        "info,wgpu_core=warn,wgpu_hal=warn,simula_socket=debug,simula_net=debug"
                            .into(),
                    level: bevy::log::Level::DEBUG,
                }),
        )
        .add_plugin(NetPlugin)
        .add_plugin(NetAuthorityPlugin)
        .add_plugin(WorldInspectorPlugin)
        .add_plugin(ActionPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(FlyCameraPlugin)
        .add_plugin(LinesPlugin)
        .add_plugin(AxesPlugin)
        .add_plugin(GridPlugin)
        .add_startup_system(setup)
        .add_system(debug_info);

    app.run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, asset_server: Res<AssetServer>) {
    // network minion that will only communicate with authority
    commands
        .spawn_empty()
        .insert(Minion::default())
        .insert(NetId::default())
        .insert(Name::new("Minion"));

    // network worker that will sync with everyone
    commands
        .spawn_empty()
        .insert(Worker::default())
        .insert(Replicate::<Worker>::default())
        .insert(NetId::default())
        .insert(Name::new("Worker"));

    // network minion worker that will sync worker state with others
    // but only minion state with authority
    commands
        .spawn_empty()
        .insert(Minion::default())
        .insert(Worker::default())
        .insert(Replicate::<Worker>::default())
        .insert(NetId::default())
        .insert(Name::new("MinionWorker"));

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

    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 2.0, -10.0)
                .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
            ..default()
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
