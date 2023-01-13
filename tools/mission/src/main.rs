use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};
use simula_action::ActionPlugin;
use simula_camera::orbitcam::*;
use simula_core::ease::EaseFunction;
use simula_mission::{
    prelude::{Asset, AssetBalance, WalletBuilder},
    MissionPlugin,
};
use simula_video::{GifAsset, VideoMaterial, VideoPlayer, VideoPlugin};
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    follow_ui::{FollowUI, FollowUICamera, FollowUIPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::{LineMesh, LinesMaterial, LinesPlugin},
    lookat::{LookAtPlugin, SmoothLookAt},
};

pub mod ui;

pub const CLASS_ID: u64 = 1000;
pub const TIME_ASSET_ID: u64 = 0;
pub const ENERGY_ASSET_ID: u64 = 1;

#[derive(Debug, Inspectable, Reflect, Component, Clone, PartialEq)]
#[reflect(Component)]
pub enum MissionToken {
    Time(Asset<CLASS_ID, TIME_ASSET_ID>),
    Energy(Asset<CLASS_ID, ENERGY_ASSET_ID>),
}

impl Default for MissionToken {
    fn default() -> Self {
        Self::Time(0.into())
    }
}

impl From<AssetBalance> for MissionToken {
    fn from(asset: AssetBalance) -> Self {
        match (asset.class_id, asset.asset_id) {
            (CLASS_ID, TIME_ASSET_ID) => MissionToken::Time(asset.balance.into()),
            (CLASS_ID, ENERGY_ASSET_ID) => MissionToken::Energy(asset.balance.into()),
            _ => panic!("Unknown asset"),
        }
    }
}

impl From<MissionToken> for AssetBalance {
    fn from(token: MissionToken) -> Self {
        match token {
            MissionToken::Time(asset) => asset.into(),
            MissionToken::Energy(asset) => asset.into(),
        }
    }
}

#[derive(Component)]
struct Rotate {
    axis: Vec3,
    angle: f32,
}

fn main() {
    App::new()
        .register_type::<MissionToken>()
        .register_inspectable::<MissionToken>()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::rgb(0.105, 0.10, 0.11)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "[Simbotic] Simula - Mission".to_string(),
                width: 940.,
                height: 528.,
                ..default()
            },
            ..default()
        }))
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(ActionPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(OrbitCameraPlugin)
        .add_plugin(LinesPlugin)
        .add_plugin(AxesPlugin)
        .add_plugin(GridPlugin)
        .add_plugin(VideoPlugin)
        .add_plugin(LookAtPlugin)
        .add_plugin(FollowUIPlugin)
        .add_plugin(MissionPlugin)
        .add_startup_system(setup)
        .add_system(debug_info)
        .add_system(rotate_system)
        .add_system(ui::follow_ui)
        .run();
}

fn build_wallet(commands: &mut Commands) -> Entity {
    WalletBuilder::<MissionToken>::default()
        .id("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")
        .with_account(|account| {
            account
                .id("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60")
                .with_asset(|asset| {
                    asset.amount(MissionToken::Energy(10000.into()));
                })
                .with_asset(|asset| {
                    asset.amount(MissionToken::Time(1000.into()));
                });
        })
        .with_account(|account| {
            account
                .id("ede3354e133f9c8e337ddd6ee5415ed4b4ffe5fc7d21e933f4930a3730e5b21c")
                .with_asset(|asset| {
                    asset.amount(MissionToken::Energy(5000.into()));
                });
        })
        .build(commands)
}

fn spawn_robot_gif(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    video_materials: &mut ResMut<Assets<VideoMaterial>>,
    asset_server: Res<AssetServer>,
    camera_entity: Entity,
) -> Entity {
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
        .insert(Rotate {
            axis: Vec3::Y,
            angle: 0.6,
        })
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
                            mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
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
                .insert(SmoothLookAt {
                    target: Some(camera_entity),
                    yaw_ease: EaseFunction::SineInOut,
                    pitch_ease: EaseFunction::SineInOut,
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(SpatialBundle {
                            transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
                            ..Default::default()
                        })
                        .insert(FollowUI {
                            min_distance: 0.1,
                            max_distance: 20.0,
                            min_height: -5.0,
                            max_height: 5.0,
                            max_view_angle: 45.0,
                            ..default()
                        })
                        .insert(SmoothLookAt {
                            target: Some(camera_entity),
                            yaw_ease: EaseFunction::SineInOut,
                            pitch_ease: EaseFunction::SineInOut,
                            ..default()
                        })
                        .insert(ui::RobotPanel)
                        .insert(Name::new("FollowUI: Axes"));
                });
        })
        .insert(Name::new("Robot"))
        .id()
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut lines_materials: ResMut<Assets<LinesMaterial>>,
    mut video_materials: ResMut<Assets<VideoMaterial>>,
    line_mesh: Res<LineMesh>,
    asset_server: Res<AssetServer>,
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
    let camera_entity = commands
        .spawn(Camera3dBundle {
            ..Default::default()
        })
        .insert(OrbitCamera {
            center: Vec3::new(0.0, 1.0, 0.0),
            distance: 10.0,
            ..Default::default()
        })
        .insert(FollowUICamera)
        .id();

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

    // robot gif
    let robot_entity = spawn_robot_gif(
        &mut commands,
        &mut meshes,
        &mut video_materials,
        asset_server,
        camera_entity,
    );

    // wallet with multiple accounts using MissionTokens
    let wallet_entity = build_wallet(&mut commands);

    // attach wallet to robot
    commands
        .entity(robot_entity)
        .push_children(&[wallet_entity]);
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

fn rotate_system(time: Res<Time>, mut query: Query<(&Rotate, &mut Transform)>) {
    for (rotate, mut transform) in query.iter_mut() {
        transform.rotate(Quat::from_axis_angle(
            rotate.axis,
            rotate.angle * time.delta_seconds(),
        ));
    }
}
