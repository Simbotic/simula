use behaviors::mission_behavior;
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{
    egui,
    EguiContext,
};
use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};
use mission_behavior::MissionBehaviorPlugin;
use simula_action::ActionPlugin;
use simula_behavior::prelude::*;
use simula_camera::orbitcam::*;
use simula_core::signal::{SignalFunction, SignalGenerator};
use simula_mission::{
    asset::{Amount, Asset, AssetBalance, self},
    MissionPlugin, WalletBuilder,
    wallet_ui::WalletUIPlugin,
    drag_and_drop::DragAndDropPlugin,
    asset_ui::{AssetInfo, ImageTextureIds}, account::Account,
};
use simula_net::NetPlugin;
#[cfg(feature = "gif")]
use simula_video::GifAsset;
use simula_video::{VideoPlayer, VideoPlugin};
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    follow_ui::{FollowUICamera, FollowUIPlugin, FollowUI},
    grid::{Grid, GridBundle, GridPlugin},
    lines::{LineMesh, LinesMaterial, LinesPlugin},
};
use std::time::Duration;
use ta::indicators::*;

mod behaviors;

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
    .insert_resource(TimeDuration{time: Duration::default()})
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
    .add_plugin(MissionBehaviorPlugin)
    .add_plugin(BehaviorPlugin)
    .add_plugin(FollowUIPlugin)
    .add_plugin(DragAndDropPlugin(MissionToken::None))
    .add_plugin(WalletUIPlugin(MissionToken::None))
    .register_type::<MissionToken>()
    .register_type::<SignalGenerator>()
    .add_startup_system(setup)
    .add_system(debug_info)
    .add_system(increase_mission_time)
    .add_system(increase_time_with_signal)
    .add_system(indicator_mission_time)
    .add_system(wallet_creation_window);

    // app.register_inspectable::<MissionToken>();
    app.register_inspectable::<SignalFunction>();

    app.run();
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectedWallet2(usize);

#[derive(Debug, Inspectable, Default, Reflect, Component, Clone, PartialEq)]
#[reflect(Component)]
pub enum MissionToken {
    #[default]
    None,
    Time(Asset<1000, 0>),
    Trust(Asset<1000, 1>),
    Energy(Asset<1000, 2>),
    Labor(Asset<1000, 3>),
}

impl Into<AssetBalance> for MissionToken{
    fn into(self) -> AssetBalance {
        match self {
            MissionToken::None => panic!("Cannot convert to AssetBalance"),
            MissionToken::Time(a) => a.into(),
            MissionToken::Trust(a) => a.into(),
            MissionToken::Energy(a) => a.into(),
            MissionToken::Labor(a) => a.into(),
        }
    }
}

impl AssetInfo for MissionToken {
    fn name(&self) -> &'static str {
        match self {
            MissionToken::None => "None",
            MissionToken::Time(_) => "Time",
            MissionToken::Trust(_) => "Trust",
            MissionToken::Energy(_) => "Energy",
            MissionToken::Labor(_) => "Labor",
        }
    }

    fn icon_dir(&self) -> &'static str {
        match self {
            MissionToken::Time(_) => "../assets/mission/Balance.png",
            MissionToken::Trust(_) => "../assets/mission/Money - Cash.png",
            MissionToken::Energy(_) => "../assets/mission/Money - Coins.png",
            MissionToken::Labor(_) => "../assets/mission/labor-icon.png",
            _ => "",
        }
    }

    fn amount(&self) -> Amount {
        match self {
            MissionToken::None => 0.into(),
            MissionToken::Time(asset) => asset.0,
            MissionToken::Trust(asset) => asset.0,
            MissionToken::Energy(asset) => asset.0,
            MissionToken::Labor(asset) => asset.0,
        }
    }

    fn is_draggable(&self) -> bool {
        match self {
            MissionToken::None => false,
            MissionToken::Time(_) => false,
            MissionToken::Trust(_) => true,
            MissionToken::Energy(_) => true,
            MissionToken::Labor(_) => true,
        }
    }

    fn render(&self, ui: &mut egui::Ui, texture_ids: &ImageTextureIds) {
        match self {
            MissionToken::None => {}
            MissionToken::Time(_) => {
                ui.horizontal(|ui| {
                    if let Some(icon) = texture_ids.get_texture(self.icon_dir()) {
                        ui.add(egui::widgets::Image::new(icon, [20.0, 20.0]));
                    }

                    let label = egui::Label::new(format!(": {}", self.amount().0));

                    if self.is_draggable() {
                        ui.add(label.sense(egui::Sense::click()));
                    } else {
                        ui.add(label);
                    }
                });
            }
            MissionToken::Trust(_) => {
                if let Some(icon) = texture_ids.get_texture(self.icon_dir()) {
                    ui.add(egui::widgets::Image::new(icon, [20.0, 20.0]));
                }
            }
            MissionToken::Energy(_) => {
                ui.add(egui::widgets::SelectableLabel::new(
                    true,
                    format!("{}: {}", self.name(), self.amount().0),
                ));
            }
            MissionToken::Labor(_) => {
                ui.vertical(|ui| {
                    if let Some(icon) = texture_ids.get_texture(self.icon_dir()) {
                        ui.add(egui::widgets::Image::new(icon, [20.0, 20.0]));
                        let label = egui::widgets::Label::new(format!(
                            "{}: {}",
                            self.name(),
                            self.amount().0
                        ));
                        if self.is_draggable() {
                            ui.add(label.sense(egui::Sense::click()));
                        } else {
                            ui.add(label);
                        }
                    }
                });
            }
        }
    }
    fn asset_id(&self)->u64 {
        1000
    }
    fn class_id(&self)->u64 {
        match self{
            MissionToken::Time(_) => 0,
            MissionToken::Trust(_) => 1,
            MissionToken::Energy(_) => 2,
            MissionToken::Labor(_) => 3,
            MissionToken::None => panic!("Can't return an id for none!")
        }
    }
   

    fn drag(&mut self)-> bool {
        match self {
            MissionToken::Energy(_) => {
                *self = MissionToken::Energy(Asset(Amount(0.into())))
            }
            MissionToken::Labor(_) => {
                *self = MissionToken::Labor(Asset(Amount(0.into())))
            }
            MissionToken::Trust(_) => {
                *self = MissionToken::Trust(Asset(Amount(0.into())))
            }
            MissionToken::Time(_) => {
                *self = MissionToken::Time(Asset(Amount(0.into())))
            }
            MissionToken::None=>{return false}
        }
        return true;
    }

    fn drop(&mut self, src_class_id: u64, src_asset_id: u64, src_amount: Amount)-> bool {
        
        if self.class_id() == src_class_id {
            if self.asset_id() == src_asset_id{
                match self {
                    MissionToken::Energy(value) => {
                        *self = MissionToken::Energy(Asset(Amount(
                            value.0 .0 + src_amount.0,
                        )));
                        return true;
                    }
                    MissionToken::Labor(value) => {
                        *self = MissionToken::Labor(Asset(Amount(
                            value.0 .0 + src_amount.0,
                        )));
                        return true;
                    }
                    MissionToken::Trust(value) => {
                        *self = MissionToken::Trust(Asset(Amount(
                            value.0 .0 + src_amount.0,
                        )));
                        return true;
                    }
                    MissionToken::Time(value) => {
                        *self = MissionToken::Time(Asset(Amount(
                            value.0 .0 + src_amount.0,
                        )));
                        return true;
                    }
                    MissionToken::None=>{return false}
                }
            }
        }
        return false;
    }

    fn push_as_children(&self,commands: &mut Commands, parent: Entity) {
        let new_asset = commands.spawn().insert(self.clone()).id();
        commands.entity(parent).push_children(&[new_asset]);
    }
    
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut lines_materials: ResMut<Assets<LinesMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    line_mesh: Res<LineMesh>,
    mut behavior_inspector: ResMut<BehaviorInspector>,
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
                })
                .with_asset(|asset| {
                    asset.amount(MissionToken::Labor(630.into()));
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
        // .insert(BehaviorEditorState {
        //     show: true,
        //     ..default()
        // })
        // .insert(BehaviorGraphState::default())
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
            let mut child = parent.spawn_bundle(SpatialBundle {
                transform: Transform::from_translation(Vec3::new(0.0, 2.0, 0.0)),
                ..default()
            });
            child.insert(
                FollowUI {
                    min_distance: 0.1,
                    max_distance: 20.0,
                    min_height: -5.0,
                    max_height: 5.0,
                    max_view_angle: 45.0,
                    ..default()
                }
            )
            .insert(Name::new("Follow UI Robot"));
            // .insert(FollowPanel)
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

    // Build Agent 001
    let behavior = mission_behavior::create_from_data(None, &mut commands);
    if let Some(root) = behavior.root {
        commands.entity(root).insert(BehaviorCursor);
    }
    commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_xyz(-2.0, 0.0, 0.0),
            ..default()
        })
        .push_children(&[
            agent_wallet,
            agent_behavior_graph,
            agent_body,
            behavior.root.unwrap(),
        ])
        .insert(behavior)
        .insert(Name::new("Agent: 001"));

    // Build Agent 002
    let document: Handle<BehaviorAsset> = asset_server.load("behaviors/debug_sequence.bht.ron");
    let behavior = BehaviorTree::from_asset::<mission_behavior::MissionBehavior>(
        None,
        &mut commands,
        document,
    );
    if let Some(root) = behavior.root {
        commands.entity(root).insert(BehaviorCursor);
    }
    let agent_id = commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_xyz(-2.0, 0.0, 0.0),
            ..default()
        })
        .insert(behavior)
        .insert(Name::new("Agent: 002"))
        .id();

    behavior_inspector.select(agent_id, "Agent: 002".into());
    behavior_inspector.unselect();

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
        })
        .insert(FollowUICamera);

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
        commands.spawn()
            .insert( SignalGenerator {
                func: SignalFunction::Pulse,
                amplitude: 1.0,
                frequency: 1.0,
                phase: 1.0,
                offset: 1.0,
                invert: false,
                ..default()
            })
            .insert(Name::new("Signal Generator"));
}

#[derive(Component)]
struct FollowPanel;

#[derive(Default, Component, Reflect, Clone)]
struct AgentRest;

fn debug_info(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text>) {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            // Update the value of the second section
            for mut text in &mut query {
                text.sections[1].value = format!("{average:.2}");
            }
        }
    }
}

fn increase_mission_time(_time: Res<Time>, mut query: Query<&mut MissionToken>) {
    for mut token in query.iter_mut() {
        match *token {
            MissionToken::Time(asset) => *token = MissionToken::Time(Asset(Amount(asset.0 .0 + 1))),
            _ => {}
        }
    }
}

#[derive(Default)]
pub struct TimeDuration {
    time: Duration,
}

fn increase_time_with_signal(
    mut generator_mission: Query<&mut SignalGenerator>,
    time_duration: Res<TimeDuration>,
    mut query: Query<&mut MissionToken>,
) {
    for mut token in query.iter_mut() {
        for mut signal_generator in generator_mission.iter_mut() {
            let generate = SignalGenerator::sample(
                &mut signal_generator, 
                time_duration.time,
            );
            let generate = generate.round() as i128;
            match *token {
                MissionToken::Time(asset) => 
                    *token = MissionToken::Time(Asset(Amount(asset.0 .0 + generate))),
                    _ => {}
            }
        }
    }
}

fn indicator_mission_time(_time: Res<Time>, mut assets: Query<&mut MissionToken>) {
    for asset in assets.iter_mut() {
        let asset_value = match *asset {
            MissionToken::Time(asset) => asset.0 .0,
            _ => default(),
        };
        let _time_indicator = ExponentialMovingAverage::new(asset_value as usize);
    }
}

fn wallet_creation_window(
    mut commands: Commands,
    mut egui_ctx: ResMut<EguiContext>,
) {
    egui::Window::new("Wallet Creation")
        .default_width(200.0)
        .resizable(true)
        .collapsible(false)
        .title_bar(true)
        .vscroll(false)
        .drag_bounds(egui::Rect::EVERYTHING)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.small_button("Create Wallet").on_hover_text("generate wallet").clicked().then(|| {
                add_wallet(&mut commands);
            });
        });
}

fn gen_id() -> String {
    format!("{:0<64x}", rand::random::<u128>())
}

fn add_wallet(commands: &mut Commands) {
    WalletBuilder::<MissionToken>::default()
        .id(gen_id().as_str())
        .with_account(|account| {
            account
                .id(gen_id().as_str())
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
                .id(gen_id().as_str())
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
        .build(commands);
}