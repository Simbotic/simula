use behaviors::AgentBehaviorPlugin;
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};
use simula_action::ActionPlugin;
use simula_behavior::{editor::BehaviorEditorState, editor::BehaviorGraphState, BehaviorPlugin};
use simula_camera::orbitcam::*;
use simula_mission::{asset::Asset, MissionPlugin, WalletBuilder};
use simula_net::NetPlugin;
#[cfg(feature = "gif")]
use simula_video::GifAsset;
use simula_video::{VideoPlayer, VideoPlugin};
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::{LineMesh, LinesMaterial, LinesPlugin},
};

mod behaviors;

fn main() {
    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "[Simbotic] Simula - Mission".to_string(),
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
    .add_plugin(VideoPlugin)
    .add_plugin(MissionPlugin)
    .add_plugin(BehaviorPlugin)
    .add_plugin(AgentBehaviorPlugin)
    .register_type::<MissionToken>()
    .add_startup_system(setup)
    .add_startup_system(setup_behaviors)
    .add_system(debug_info);

    app.register_inspectable::<MissionToken>();

    app.run();
}

#[derive(Debug, Inspectable, Default, Reflect, Component, Clone)]
#[reflect(Component)]
pub enum MissionToken {
    #[default]
    None,
    Time(Asset<1000, 0>),
    Trust(Asset<1000, 1>),
    Energy(Asset<1000, 2>),
    Labor(Asset<1000, 3>),
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut lines_materials: ResMut<Assets<LinesMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    line_mesh: Res<LineMesh>,
    asset_server: Res<AssetServer>,
) {
    let agent_wallet = WalletBuilder::<MissionToken>::default()
        .id("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")
        .with_account(|account| {
            account
                .id("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60")
                .with_asset(|asset| {
                    asset.amount(MissionToken::Energy(10000.into()));
                })
                .with_asset(|asset| {
                    asset.amount(MissionToken::Trust(200.into()));
                })
                .with_asset(|asset| {
                    asset.amount(MissionToken::Time(1000.into()));
                });
        })
        .with_account(|account| {
            account
                .id("ede3354e133f9c8e337ddd6ee5415ed4b4ffe5fc7d21e933f4930a3730e5b21c")
                .with_asset(|asset| {
                    asset.amount(MissionToken::Energy(99999.into()));
                })
                .with_asset(|asset| {
                    asset.amount(MissionToken::Trust(99999.into()));
                })
                .with_asset(|asset| {
                    asset.amount(MissionToken::Time(99999.into()));
                });
        })
        .build(&mut commands);

    let agent_behavior_graph = commands
        .spawn()
        .insert(BehaviorEditorState {
            show: true,
            ..default()
        })
        .insert(BehaviorGraphState::default())
        .with_children(|_parent| {
            // parent.spawn_bundle(BehaviorBundle::<AgentRest>::default());
            // parent.spawn_bundle(BehaviorBundle::<AgentWork>::default());
        })
        .insert(Name::new("Behavior Graph"))
        .id();

    let video_material = StandardMaterial {
        base_color: Color::rgb(1.0, 1.0, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    };
    let video_rotation =
        Quat::from_euler(EulerRot::YXZ, -std::f32::consts::FRAC_PI_3 * 0.0, 0.0, 0.0);
    let video_position = Vec3::new(0.0, 0.5, 0.0);

    let agent_body = commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_translation(video_position).with_rotation(video_rotation),
            ..default()
        })
        .with_children(|parent| {
            let mut child = parent.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
                material: materials.add(video_material),
                transform: Transform::from_rotation(Quat::from_euler(
                    EulerRot::YXZ,
                    0.0,
                    -std::f32::consts::FRAC_PI_2,
                    0.0,
                )),
                ..default()
            });
            child
                .insert(VideoPlayer {
                    start_frame: 0,
                    end_frame: 80,
                    framerate: 20.0,
                    playing: true,
                    ..default()
                })
                .insert(Name::new("Video: RenderTarget"));

            #[cfg(feature = "gif")]
            {
                let video_asset: Handle<GifAsset> = asset_server.load("videos/robot.gif");
                child.insert(video_asset);
            }
        })
        .insert(Name::new("Agent: Body"))
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
        })
        .id();

    commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_xyz(-2.0, 0.0, 0.0),
            ..default()
        })
        .push_children(&[agent_wallet, agent_behavior_graph, agent_body])
        .insert(Name::new("Agent: 001"));

    // grid
    let grid_color = Color::rgb(0.08, 0.06, 0.08);
    commands
        .spawn_bundle(GridBundle {
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
        .spawn_bundle(AxesBundle {
            axes: Axes {
                size: 6.,
                inner_offset: 0.,
            },
            mesh: meshes.add(line_mesh.clone()),
            material: lines_materials.add(LinesMaterial {}),
            transform: Transform::from_xyz(0.0, 0.01, 0.0),
            ..Default::default()
        })
        .insert(Name::new("Axes: World"));

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

fn setup_behaviors(mut commands: Commands) {
    behaviors::create(&mut commands);
}
