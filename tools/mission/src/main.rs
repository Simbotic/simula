use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use egui_node_graph::*;
use simula_action::ActionPlugin;
use simula_camera::orbitcam::*;
use simula_mission::{asset::{Asset, Amount}, MissionPlugin, WalletBuilder};
use simula_net::NetPlugin;
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::{LineMesh, LinesMaterial, LinesPlugin},
};

mod graph;

// A unit struct to help identify the FPS UI component, since there may be many Text components
#[derive(Component)]
struct FpsText;

// A unit struct to help identify the color-changing Text component
#[derive(Component)]
struct ColorText;

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
    .add_plugin(MissionPlugin)
    .register_type::<MissionToken>()
    .add_startup_system(setup)
    .add_system(debug_info)
    .add_system(increase_mission_time)
    //.add_system(check_increase)
    .add_system(graph::egui_update);

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
    MissionTime(Asset<1000,3>)
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut lines_materials: ResMut<Assets<LinesMaterial>>,
    line_mesh: Res<LineMesh>,
    asset_server: Res<AssetServer>,
) {
    WalletBuilder::<MissionToken>::default()
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
                })
                .with_asset(|asset| {
                    asset.amount(MissionToken::MissionTime(0.into()));
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
    
    //FPS ON SCREEN
    commands
        .spawn_bundle(
            TextBundle::from_sections([
                TextSection::new(
                    "FPS: ",
                    TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 30.0,
                        color: Color::WHITE,
                    },
                ),
                TextSection::from_style(TextStyle {
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                    font_size: 30.0,
                    color: Color::GOLD,
                }),
            ])
            .with_style(Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(24.0),
                    left: Val::Px(15.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(FpsText);

    commands
        .spawn()
        //.insert(Check{timer: Timer::from_seconds(1.0, true)})
        .insert(graph::MyEditorState(GraphEditorState::new(1.0)))
        .insert(graph::MyGraphState::default());
}

fn debug_info(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            // Update the value of the second section
            for mut text in &mut query {
                text.sections[1].value = format!("{average:.2}");
            }
        }
    }
    
}

fn increase_mission_time(_time: Res<Time>,mut query: Query<&mut MissionToken>){
    for mut token in query.iter_mut(){
        match *token{
            MissionToken::MissionTime(asset)=>{
                //asset.0.0 += 1
                *token = MissionToken::MissionTime(Asset(Amount(asset.0.0 + 1)))
            }
            _=>{}
        }
    }
}

//fn check_increase(time: Res<Time>,mut q: Query<&mut Check>,mut query: Query<&mut MissionToken>){
//    for mut check in q.iter_mut(){
//        check.timer.tick(time.delta());
//        if check.timer.just_finished(){
//            for token in query.iter_mut(){
//                match *token{
//                    MissionToken::MissionTime(asset)=>{
//                        println!("{:?}",asset.0.0)
//                    }
//                    _=>{}
//                }
//            }
//        }
//    }
//}

//#[derive(Component)]
//pub struct Check{
// timer: Timer
//}