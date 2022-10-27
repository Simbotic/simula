use behaviors::mission_behavior;
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{egui, EguiContext};
use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};
use egui_extras::{Size, TableBuilder};
use mission_behavior::MissionBehaviorPlugin;
use simula_action::ActionPlugin;
use simula_behavior::{
    editor::BehaviorEditorState, editor::BehaviorGraphState, BehaviorAsset, BehaviorCursor,
    BehaviorPlugin, BehaviorTree,
};
use simula_camera::orbitcam::*;
use simula_mission::{
    account::Account,
    asset::{Amount, Asset},
    wallet::Wallet,
    MissionPlugin, WalletBuilder,
};
use simula_net::NetPlugin;
#[cfg(feature = "gif")]
use simula_video::GifAsset;
use simula_video::{VideoPlayer, VideoPlugin};
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    follow_ui::{FollowUI, FollowUICamera, FollowUIPlugin, FollowUIVisibility},
    grid::{Grid, GridBundle, GridPlugin},
    lines::{LineMesh, LinesMaterial, LinesPlugin},
};

mod behaviors;
mod drag_and_drop;
use drag_and_drop::DragAndDropPlugin;

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
    .insert_resource(SelectedWallet(0))
    .insert_resource(ImageTextureIds {
        time_icon: None,
        energy_icon: None,
        trust_icon: None,
    })
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
    .add_plugin(DragAndDropPlugin)
    .register_type::<MissionToken>()
    .add_startup_system(setup)
    .add_startup_system(initialize_images)
    .add_system(debug_info)
    .add_system(increase_mission_time)
    .add_system(wallet_ui_system);

    app.register_inspectable::<MissionToken>();

    app.run();
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectedWallet(usize);

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

fn wallet_ui_system(
    mut egui_ctx: ResMut<EguiContext>,
    wallets: Query<(&Wallet, &Children)>,
    accounts: Query<(&Account, &Children)>,
    assets: Query<&MissionToken>,
    mut selected_wallet: ResMut<SelectedWallet>,
    image_texture_ids: Res<ImageTextureIds>,
    follow_uis: Query<(Entity, &FollowUI, &FollowUIVisibility), With<FollowPanel>>,
) {
    for (entity, follow_ui, visibility) in follow_uis.iter() {
        let ui_pos = visibility.screen_pos;

        let window_frame = egui::containers::Frame {
            rounding: egui::Rounding {
                nw: 6.0,
                ne: 6.0,
                sw: 6.0,
                se: 6.0,
            },
            fill: egui::Color32::from_rgba_premultiplied(50, 0, 50, 50),
            inner_margin: egui::style::Margin {
                top: 10.0,
                bottom: 10.0,
                left: 10.0,
                right: 10.0,
            },
            ..default()
        };

        egui::Window::new("Wallets")
            .id(egui::Id::new(entity))
            .default_width(200.0)
            .resizable(true)
            .frame(window_frame)
            .collapsible(false)
            .title_bar(false)
            .vscroll(false)
            .fixed_size(egui::Vec2::new(follow_ui.size.x, follow_ui.size.y))
            .fixed_pos(egui::Pos2::new(ui_pos.x, ui_pos.y))
            .drag_bounds(egui::Rect::EVERYTHING)
            .show(egui_ctx.ctx_mut(), |ui| {
                ui.style_mut().spacing = egui::style::Spacing {
                    item_spacing: egui::vec2(5.0, 5.0),
                    ..default()
                };
                let mut wallet_list: Vec<(String, &Children)> = vec![];
                for (wallet, wallet_accounts) in wallets.iter() {
                    let wallet_id_trimmed = wallet
                        .wallet_id
                        .to_string()
                        .get(0..8)
                        .unwrap_or_default()
                        .to_string();
                    wallet_list.push((wallet_id_trimmed, wallet_accounts));
                }
                egui::ComboBox::from_label("Select a wallet").show_index(
                    ui,
                    &mut selected_wallet.0,
                    wallet_list.len(),
                    |i| wallet_list[i].0.to_owned(),
                );

                egui::Grid::new("accounts_grid")
                    .striped(false)
                    .show(ui, |ui| {
                        if !wallet_list[selected_wallet.0].1.is_empty() {
                            ui.heading("Accounts");
                            ui.end_row();
                        } else {
                            ui.heading("No accounts in selected wallet");
                            ui.end_row();
                        }
                        for &wallet_account in wallet_list[selected_wallet.0].1.iter() {
                            if let Ok((account, account_assets)) = accounts.get(wallet_account) {
                                let account_id_trimmed = account
                                    .account_id
                                    .to_string()
                                    .get(0..8)
                                    .unwrap_or_default()
                                    .to_string();
                                ui.collapsing(account_id_trimmed.clone(), |ui| {
                                    let mut asset_list: Vec<(
                                        String,
                                        i128,
                                        Option<egui::TextureId>,
                                    )> = vec![];
                                    for &account_asset in account_assets.iter() {
                                        if let Ok(asset) = assets.get(account_asset) {
                                            let asset_name = asset.name();
                                            let asset_value = asset.amount();
                                            let asset_icon = asset.icon(&image_texture_ids);
                                            asset_list.push((
                                                asset_name.to_string(),
                                                asset_value.0,
                                                asset_icon,
                                            ));
                                        }
                                    }
                                    TableBuilder::new(ui)
                                        .column(Size::remainder().at_least(100.0))
                                        .column(Size::remainder().at_least(100.0))
                                        .striped(false)
                                        .header(20.0, |mut header| {
                                            header.col(|ui| {
                                                ui.heading(format!("Asset"));
                                            });
                                            header.col(|ui| {
                                                ui.heading("Amount");
                                            });
                                        })
                                        .body(|mut body| {
                                            for asset in asset_list.iter() {
                                                body.row(20.0, |mut row| {
                                                    row.col(|ui| {
                                                        ui.horizontal(|ui| {
                                                            if let Some(icon) = asset.2 {
                                                                ui.add(egui::widgets::Image::new(
                                                                    icon,
                                                                    [20.0, 20.0],
                                                                ));
                                                            }
                                                            ui.label(asset.0.clone());
                                                        });
                                                    });
                                                    row.col(|ui| {
                                                        ui.label(asset.1.to_string());
                                                    });
                                                });
                                            }
                                        });
                                });
                            }
                            ui.end_row();
                        }
                    });
            });
    }
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

    let _agent_wallet_2 = WalletBuilder::<MissionToken>::default()
        .id("e75b980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")
        .with_account(|account| {
            account
                .id("8d61c19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60")
                .with_asset(|asset| {
                    asset.amount(MissionToken::Energy(2000.into()));
                })
                .with_asset(|asset| {
                    asset.amount(MissionToken::Trust(500.into()));
                })
                .with_asset(|asset| {
                    asset.amount(MissionToken::Time(6900.into()));
                });
        })
        .with_account(|account| {
            account
                .id("fde4354e133f9c8e337ddd6ee5415ed4b4ffe5fc7d21e933f4930a3730e5b21c")
                .with_asset(|asset| {
                    asset.amount(MissionToken::Energy(50.into()));
                })
                .with_asset(|asset| {
                    asset.amount(MissionToken::Trust(750.into()));
                })
                .with_asset(|asset| {
                    asset.amount(MissionToken::Time(0.into()));
                });
        })
        .build(&mut commands);

    let _agent_wallet_3 = WalletBuilder::<MissionToken>::default()
        .id("e75d880182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")
        .with_account(|account| {
            account
                .id("7d61d19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60")
                .with_asset(|asset| {
                    asset.amount(MissionToken::Energy(3000.into()));
                })
                .with_asset(|asset| {
                    asset.amount(MissionToken::Trust(800000.into()));
                })
                .with_asset(|asset| {
                    asset.amount(MissionToken::Time(10500.into()));
                });
        })
        .with_account(|account| {
            account
                .id("ffe4454e133f9c8e337ddd6ee5415ed4b4ffe5fc7d21e933f4930a3730e5b21c")
                .with_asset(|asset| {
                    asset.amount(MissionToken::Energy(650.into()));
                })
                .with_asset(|asset| {
                    asset.amount(MissionToken::Trust(15000.into()));
                })
                .with_asset(|asset| {
                    asset.amount(MissionToken::Time(10.into()));
                })
                .with_asset(|asset| {
                    asset.amount(MissionToken::Labor(100.into()));
                });
        })
        .build(&mut commands);

    let _agent_wallet_4 = WalletBuilder::<MissionToken>::default()
        .id("d76a990182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")
        .with_account(|account| {
            account.id("fde3454e133f9c8e337ddd6ee5415ed4b4ffe5fc7d21e933f4930a3730e5b21c");
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
    let document: Handle<BehaviorAsset<mission_behavior::MissionBehavior>> =
        asset_server.load("behaviors/debug_sequence.bht.ron");
    println!("Document: {:?}", document);
    let behavior = BehaviorTree::from_asset(None, &mut commands, document);
    if let Some(root) = behavior.root {
        commands.entity(root).insert(BehaviorCursor);
    }
    commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_xyz(-2.0, 0.0, 0.0),
            ..default()
        })
        .push_children(&[_agent_wallet_4, behavior.root.unwrap()])
        .insert(behavior)
        .insert(Name::new("Agent: 002"));

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

    commands
        .spawn()
        .insert_bundle(TransformBundle {
            local: Transform::from_xyz(2.0, 0.0, 2.0),
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
        .insert(FollowPanel)
        .insert(Name::new("Follow UI: Shape"));

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

pub struct Images {
    time_icon: Handle<Image>,
    trust_icon: Handle<Image>,
    energy_icon: Handle<Image>,
}

pub struct ImageTextureIds {
    time_icon: Option<egui::TextureId>,
    trust_icon: Option<egui::TextureId>,
    energy_icon: Option<egui::TextureId>,
}

fn increase_mission_time(_time: Res<Time>, mut query: Query<&mut MissionToken>) {
    for mut token in query.iter_mut() {
        match *token {
            MissionToken::Time(asset) => *token = MissionToken::Time(Asset(Amount(asset.0 .0 + 1))),
            _ => {}
        }
    }
}

impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        if let Some(asset_server) = world.get_resource_mut::<AssetServer>() {
            Self {
                time_icon: asset_server.load("../assets/mission/Balance.png"),
                trust_icon: asset_server.load("../assets/mission/Money - Cash.png"),
                energy_icon: asset_server.load("../assets/mission/Money - Coins.png"),
            }
        } else {
            Self {
                time_icon: Handle::default(),
                trust_icon: Handle::default(),
                energy_icon: Handle::default(),
            }
        }
    }
}

fn initialize_images(
    mut egui_ctx: ResMut<EguiContext>,
    images: Local<Images>,
    mut image_texture_ids: ResMut<ImageTextureIds>,
) {
    image_texture_ids.trust_icon = Some(egui_ctx.add_image(images.trust_icon.clone()));
    image_texture_ids.time_icon = Some(egui_ctx.add_image(images.time_icon.clone()));
    image_texture_ids.energy_icon = Some(egui_ctx.add_image(images.energy_icon.clone()));
}

trait AssetInfo {
    fn name(&self) -> &'static str;
    fn icon(&self, texture_ids: &Res<ImageTextureIds>) -> Option<egui::TextureId>;
    fn amount(&self) -> Amount;
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

    fn icon(&self, image_texture_ids: &Res<ImageTextureIds>) -> Option<egui::TextureId> {
        match self {
            MissionToken::Time(_) => image_texture_ids.time_icon,
            MissionToken::Trust(_) => image_texture_ids.trust_icon,
            MissionToken::Energy(_) => image_texture_ids.energy_icon,
            MissionToken::Labor(_) => None,
            MissionToken::None => None,
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
}
