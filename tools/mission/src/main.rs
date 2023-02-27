use behavior_trees::MissionBehaviorPlugin;
use behaviors::movement::Movement;
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use common::Robot;
use simula_action::ActionPlugin;
use simula_camera::orbitcam::*;
use simula_core::ease::EaseFunction;
use simula_mission::{
    prelude::{Account, Amount, Asset, AssetBalance, WalletBuilder},
    wallet::Wallet,
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

pub mod behavior_trees;
pub mod behaviors;
pub mod common;
pub mod components;
pub mod ui;

pub const CLASS_ID: u64 = 1000;
pub const TIME_ASSET_ID: u64 = 0;
pub const MONEY_ASSET_ID: u64 = 1;
pub const MOVEMENT_RADIUS: f32 = 2.0;

#[derive(Debug, Reflect, Component, Clone, PartialEq)]
#[reflect(Component)]
pub enum MissionToken {
    Time(Asset<CLASS_ID, TIME_ASSET_ID>),
    Money(Asset<CLASS_ID, MONEY_ASSET_ID>),
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
            (CLASS_ID, MONEY_ASSET_ID) => MissionToken::Money(asset.balance.into()),
            _ => panic!("Unknown asset"),
        }
    }
}

impl From<MissionToken> for AssetBalance {
    fn from(token: MissionToken) -> Self {
        match token {
            MissionToken::Time(asset) => asset.into(),
            MissionToken::Money(asset) => asset.into(),
        }
    }
}

fn main() {
    App::new()
        .register_type::<Movement>()
        .register_type::<MissionToken>()
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
        .add_plugin(WorldInspectorPlugin)
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
        .add_plugin(MissionBehaviorPlugin)
        .add_startup_system(setup)
        .add_system(debug_info)
        .add_system(components::cop::cop_spawner)
        .add_system(components::robber::robber_spawner)
        .add_system(components::bank::bank_spawner)
        .add_system(components::citizen::citizen_spawner)
        .add_system(ui::follow_ui::<components::cop::Cop>)
        .add_system(ui::follow_ui::<components::robber::Robber>)
        .add_system(ui::follow_ui::<components::bank::Bank>)
        .add_system(ui::follow_ui::<components::citizen::Citizen>)
        .add_system(behaviors::movement::calculate_movement)
        .run();
}

fn gen_id() -> String {
    format!("{:0<64x}", rand::random::<u128>())
}

fn build_wallet(commands: &mut Commands, money: u64) -> Entity {
    WalletBuilder::<MissionToken>::default()
        .id(&gen_id())
        .with_account(|account| {
            account
                .id(&gen_id())
                .with_asset(|asset| {
                    asset.amount(MissionToken::Money(Asset(Amount(money.into()))));
                })
                .with_asset(|asset| {
                    asset.amount(MissionToken::Time(0.into()));
                });
        })
        .build(commands)
}

fn _simulate_transfer<T>(
    robot_query: Query<&Children, With<T>>,
    wallet_query: Query<&Children, With<Wallet>>,
    account_query: Query<&Children, With<Account>>,
    mut tokens_query: Query<&mut MissionToken>,
) where
    T: Component + Robot + Clone + Copy,
{
    for robot_childrens in robot_query.iter() {
        for child in robot_childrens.iter() {
            if let Ok(wallet_childrens) = wallet_query.get(*child) {
                for wallet_child in wallet_childrens.iter() {
                    if let Ok(account_children) = account_query.get(*wallet_child) {
                        for account_child in account_children.iter() {
                            if let Ok(mut token) = tokens_query.get_mut(*account_child) {
                                match *token {
                                    MissionToken::Money(asset) => {
                                        *token = MissionToken::Money(Asset(Amount(*asset.0 + 1)));
                                    }
                                    MissionToken::Time(asset) => {
                                        *token = MissionToken::Money(Asset(Amount(*asset.0 + 1)));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn spawn_robot_gif<T>(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    video_materials: &mut ResMut<Assets<VideoMaterial>>,
    asset_server: &Res<AssetServer>,
    camera_entity: &Entity,
    asset_name: &str,
    duration: f32,
    robot: &mut T,
) -> Entity
where
    T: Component + Robot + Clone + Copy,
{
    let video_material = VideoMaterial {
        color: Color::rgb(1.0, 1.0, 1.0),
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    };
    let video_asset: Handle<GifAsset> =
        asset_server.load(format!("videos/mission/{}.gif", asset_name));
    let video_rotation =
        Quat::from_euler(EulerRot::YXZ, -std::f32::consts::FRAC_PI_3 * 0.0, 0.0, 0.0);
    let video_position = Vec3::new(0.0, 0.5, -2.0);

    let follow_ui_entity = commands
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
            target: Some(*camera_entity),
            yaw_ease: EaseFunction::SineInOut,
            pitch_ease: EaseFunction::SineInOut,
            ..default()
        })
        .insert(ui::RobotPanel)
        .insert(Name::new("FollowUI"))
        .id();

    // attach follow_ui to robot
    robot.set_follow_ui(follow_ui_entity);

    let video_entity = commands
        .spawn(SpatialBundle {
            transform: Transform::from_translation(video_position).with_rotation(video_rotation),
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
        .insert(Name::new(format!("Video: {}", asset_name)))
        .insert(SmoothLookAt {
            target: Some(*camera_entity),
            yaw_ease: EaseFunction::SineInOut,
            pitch_ease: EaseFunction::SineInOut,
            ..default()
        })
        .push_children(&[follow_ui_entity])
        .id();

    let points = vec![
        Vec3::new(-MOVEMENT_RADIUS, 0.0, MOVEMENT_RADIUS),
        Vec3::new(MOVEMENT_RADIUS, 0.0, MOVEMENT_RADIUS),
        Vec3::new(MOVEMENT_RADIUS, 0.0, -MOVEMENT_RADIUS),
        Vec3::new(-MOVEMENT_RADIUS, 0.0, -MOVEMENT_RADIUS),
    ];

    let direction = (points[1] - points[0]).normalize();

    commands
        .spawn(SpatialBundle { ..default() })
        .insert(Movement {
            points,
            duration,
            elapsed: 0.0,
            direction,
            index: 0,
        })
        .push_children(&[video_entity])
        .insert(*robot)
        .insert(Name::new(asset_name.to_string()))
        .id()
}

fn spawn_robot_with_wallet<T>(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    video_materials: &mut ResMut<Assets<VideoMaterial>>,
    asset_server: &Res<AssetServer>,
    camera_entity: &Entity,
    asset_name: &str,
    duration: f32,
    robot: &mut T,
) -> Entity
where
    T: Component + Robot + Copy + Clone,
{
    // robot gif
    let robot_entity = spawn_robot_gif(
        commands,
        meshes,
        video_materials,
        asset_server,
        camera_entity,
        asset_name,
        duration,
        robot,
    );

    // wallet with multiple accounts using MissionTokens
    let wallet_entity = build_wallet(commands, robot.get_money());

    // attach wallet to robot
    commands
        .entity(robot_entity)
        .push_children(&[wallet_entity])
        .id()
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut lines_materials: ResMut<Assets<LinesMaterial>>,
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
    commands
        .spawn(Camera3dBundle {
            ..Default::default()
        })
        .insert(OrbitCamera {
            center: Vec3::new(0.0, 1.0, 0.0),
            distance: 10.0,
            ..Default::default()
        })
        .insert(FollowUICamera);

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
