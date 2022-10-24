use behaviors::AgentBehaviorPlugin;
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{egui,egui::{vec2,Id,Label,Sense}, EguiContext};
use egui_extras::{TableBuilder, Size};
use simula_action::ActionPlugin;
use simula_behavior::{editor::BehaviorEditorState, editor::BehaviorGraphState, BehaviorPlugin};
use simula_camera::orbitcam::*;
use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};
use simula_mission::{asset::{Asset,Amount}, account::Account, MissionPlugin, WalletBuilder, wallet::Wallet};
use simula_net::NetPlugin;
#[cfg(feature = "gif")]
use simula_video::GifAsset;
use simula_video::{VideoPlayer, VideoPlugin};
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::{LineMesh, LinesMaterial, LinesPlugin},
    follow_ui::{FollowUI, FollowUICamera, FollowUIPlugin, FollowUIVisibility}
};
mod behaviors;
mod drag_and_drop;
use drag_and_drop::*;

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
    .insert_resource(ImageTextureIds{time_icon: None, energy_icon: None, trust_icon: None})
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
    .add_plugin(FollowUIPlugin)
    .register_type::<MissionToken>()
    .add_startup_system(setup)
    .add_startup_system(initialize_images)
    .add_startup_system(setup_behaviors)
    .add_system(debug_info)
    .add_system(increase_mission_time)
    //.add_system(check_increase)
    .add_system(wallet_ui_system)
    .add_system(debug_info)
    .add_system(drag_and_drop);

    app.register_inspectable::<MissionToken>();

    app.run();
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectedWallet(usize);

#[derive(Debug, Clone, PartialEq)]
pub struct SelectedWallet2(usize);

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

fn wallet_ui_system (
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
        egui::Window::new("Wallets")
            .id(egui::Id::new(entity))
            .default_width(200.0)
            .resizable(true)
            .vscroll(true)
            .fixed_size(egui::Vec2::new(follow_ui.size.x, follow_ui.size.y))
            .fixed_pos(egui::Pos2::new(ui_pos.x, ui_pos.y))
            .drag_bounds(egui::Rect::EVERYTHING)
            .show(egui_ctx.ctx_mut(), |ui| {
                let mut wallet_list: Vec<(String, &Children)> = vec![];
                for (wallet, wallet_accounts) in wallets.iter() {
                    let wallet_id_trimmed = wallet.wallet_id
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
                    |i| wallet_list[i].0.to_owned()
                );

                egui::Grid::new("accounts_grid").striped(true).show(ui, |ui| {
                    if !wallet_list[selected_wallet.0].1.is_empty() {
                        ui.heading("Accounts");
                        ui.end_row();
                    } else {
                        ui.heading("No accounts in selected wallet");
                        ui.end_row();
                    }
                    for &wallet_account in wallet_list[selected_wallet.0].1.iter() {
                        if let Ok((account, account_assets)) = accounts.get(wallet_account) {
                            let account_id_trimmed = account.account_id
                                    .to_string()
                                    .get(0..8)
                                    .unwrap_or_default()
                                    .to_string();
                            ui.collapsing(account_id_trimmed.clone(), |ui| {
                                let mut asset_list: Vec<(String, i128, Option<egui::TextureId>)> = vec![];
                                for &account_asset in account_assets.iter() {
                                    if let Ok(asset) = assets.get(account_asset) {
                                        let asset_name = match asset {
                                            MissionToken::Time(_) => "Time",
                                            MissionToken::Trust(_) => "Trust",
                                            MissionToken::Energy(_) => "Energy",
                                            MissionToken::Labor(_) => "Labor",
                                            MissionToken::None => "None",
                                        };
                                        let asset_value = match asset {
                                            MissionToken::Time(asset) => asset.0.0,
                                            MissionToken::Trust(asset) => asset.0.0,
                                            MissionToken::Energy(asset) => asset.0.0,
                                            MissionToken::Labor(asset) => asset.0.0,
                                            MissionToken::None => 0,
                                        };
                                        let asset_icon = match asset {
                                            MissionToken::Time(_) => image_texture_ids.time_icon,
                                            MissionToken::Trust(_) => image_texture_ids.trust_icon,
                                            MissionToken::Energy(_) => image_texture_ids.energy_icon,
                                            MissionToken::Labor(_) => None,
                                            MissionToken::None => None,
                                        };
                                        asset_list.push((asset_name.to_string(), asset_value, asset_icon));
                                    }
                                }
                                TableBuilder::new(ui)
                                    .column(Size::remainder().at_least(100.0))
                                    .column(Size::remainder().at_least(100.0))
                                    .striped(true)
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

    let agent_wallet_2 = WalletBuilder::<MissionToken>::default()
        .id("f75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")
        .with_account(|account| {
            account
                .id("2d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60")
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
                .id("ude3354e133f9c8e337ddd6ee5415ed4b4ffe5fc7d21e933f4930a3730e5b21c")
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
    
    let agent_decision_graph = commands
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
        .push_children(&[agent_wallet, agent_decision_graph, agent_body])
        .insert(Name::new("Agent: 001"));

    //commands
    //    .spawn()
    //    //.insert(Check{timer: Timer::from_seconds(1.0, true)})
    //    .insert(graph::MyEditorState(GraphEditorState::new(1.0)))
    //    .insert(graph::MyGraphState::default());

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
        }).insert(FollowUICamera);

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
        },
        )
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


fn increase_mission_time(_time: Res<Time>,mut query: Query<&mut MissionToken>){
    for mut token in query.iter_mut(){
        match *token{
            MissionToken::Time(asset)=>{
                //asset.0.0 += 1
                *token = MissionToken::Time(Asset(Amount(asset.0.0 + 1)))
            }
            _=>{}
        }
    }
}

#[derive(Clone, PartialEq,Component,Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DragAndDropDemo {
    wallet_list: Vec<(String, Vec<(String, Vec<(String, i128)>)>)> 
}

pub fn drag_and_drop (
    mut egui_ctx: ResMut<EguiContext>,
    mut wallets: Query<(&mut Wallet, &Children)>,
    mut accounts: Query<(&mut Account, &Children)>,
    mut assets: Query<&mut MissionToken>,
    //mut drag_drop: ResMut<DragAndDropDemo>
) {
    //drag_drop.wallet_list = vec![];
    //for (wallet, wallet_accounts) in wallets.iter() {
    //    let wallet_id_trimmed = wallet.wallet_id
    //        .to_string()
    //        .get(0..8)
    //        .unwrap_or_default()
    //        .to_string();
    //    let mut account_list: Vec<(String, Vec<(String, i128)>)> = vec![];
    //    for &wallet_account in wallet_accounts {
    //        if let Ok((account, account_assets)) = accounts.get(wallet_account) {
    //            let account_id_trimmed = account.account_id
    //                .to_string()
    //                .get(0..8)
    //                .unwrap_or_default()
    //                .to_string();
    //            let mut asset_list: Vec<(String, i128)> = vec![];
    //            for &account_asset in account_assets.iter() {
    //                if let Ok(asset) = assets.get(account_asset) {
    //                    let asset_name = match asset {
    //                        MissionToken::Time(_) => "Time",
    //                        MissionToken::Trust(_) => "Trust",
    //                        MissionToken::Energy(_) => "Energy",
    //                        MissionToken::Labor(_) => "Labor",
    //                        MissionToken::None => "None",
    //                    };
    //                    let asset_value = match asset {
    //                        MissionToken::Time(asset) => asset.0.0,
    //                        MissionToken::Trust(asset) => asset.0.0,
    //                        MissionToken::Energy(asset) => asset.0.0,
    //                        MissionToken::Labor(asset) => asset.0.0,
    //                        MissionToken::None => 0,
    //                    };
    //                    asset_list.push((asset_name.to_string(), asset_value));
    //                }
    //            }
    //            account_list.push((account_id_trimmed, asset_list));
    //        }
    //    }
    //    drag_drop.wallet_list.push((wallet_id_trimmed, account_list));
    //}
    //println!("{:?}",drag_drop.wallet_list );
    //egui::Window::new("Transfer assets")
    //    .open(&mut true)
    //    .default_size(vec2(256.0, 256.0))
    //    .vscroll(false)
    //    .resizable(true)
    //    .show(egui_ctx.ctx_mut(), |ui| {
    //        
    //        //ui.label(".");
    //        let id_source = "my_drag_and_drop_demo";
    //        let mut source_col_row_aisle = None;  //this will hold the dragged asset position: (column, row, aisle) (it's a pseudo 3D array)
    //        let mut drop_col = None;
    //        ui.columns(drag_drop.wallet_list.len(), |uis| {
    //            for (wallet_idx, wallet) in drag_drop.wallet_list.clone().into_iter().enumerate() { // iterate wallets
    //                let ui = &mut uis[wallet_idx];  // our current column, index comes from the iteration of wallets
    //                let can_accept_what_is_being_dragged = true; // We accept anything being dragged (for now) ¯\_(ツ)_/¯
    //                let response = drop_target(ui, can_accept_what_is_being_dragged, |ui| {  // Call the drag and drop function
    //                    ui.set_min_size(vec2(64.0, 100.0)); // set window size (To be Modified)
    //                    for (account_idx, account) in wallet.1.iter().enumerate() { // iterate accounts
    //                        ui.add(Label::new(account.0.clone()));
    //                        for (asset_idx, asset) in account .1.iter().enumerate(){  // iterate assets
    //                        
    //                            let item_id = Id::new(id_source).with(wallet_idx).with(account_idx).with(asset_idx); // we create an id with all index
    //                            drag_source(ui, item_id, |ui| {
    //                                let response = ui.add(Label::new(asset.0.clone()).sense(Sense::click())); //añadir vainas locas
    //                                response.context_menu(|ui| {
    //                                    if ui.button("Remove").clicked() {
    //                                        drag_drop.wallet_list[wallet_idx].1[account_idx].1.remove(asset_idx); // we remove the selected asset
    //                                        ui.close_menu();
    //                                    }
    //                                });
    //                            });
    //                            if ui.memory().is_being_dragged(item_id) {
    //                                source_col_row_aisle = Some((wallet_idx, account_idx, asset_idx));
    //                            }
    //                        }
    //                    }
    //                })
    //                .response;
    //                let response = response.context_menu(|ui| {
    //                    if ui.button("New Item").clicked() {
    //                        drag_drop.wallet_list[wallet_idx].1[source_col_row_aisle.unwrap().1].1.push(("New Item".to_owned(),1));
    //                        ui.close_menu();
    //                    }
    //                });
    //                let is_being_dragged = ui.memory().is_anything_being_dragged();
    //                if is_being_dragged && can_accept_what_is_being_dragged && response.hovered() {
    //                    drop_col = Some(wallet_idx);
    //                }
    //            }
    //        });
    //        if let Some((source_col, source_row, source_aisle)) = source_col_row_aisle {
    //            if let Some(drop_col) = drop_col {
    //                if ui.input().pointer.any_released() {
    //                    // do the drop:
    //                    let item = drag_drop.wallet_list[source_col].1[source_row].1.remove(source_aisle);
    //                    drag_drop.wallet_list[drop_col].1[source_row].1.push(item);
    //                }
    //            }
    //        }
    //    });
    
egui::Window::new("Transfer assets")
.open(&mut true)
.default_size(vec2(256.0, 256.0))
.vscroll(false)
.resizable(true)
.show(egui_ctx.ctx_mut(), |ui| {
    
    //ui.label(".");
    let id_source = "my_drag_and_drop_demo";
    let mut source_col_row_aisle = None;  //this will hold the dragged asset position: (column, row, aisle) (it's a pseudo 3D array)
    let mut drop_col = None;
    //let mut drop_row = None;
    ui.columns(wallets.into_iter().len(), |uis| {

        for (wallet_idx, wallet) in wallets.into_iter().enumerate() { // iterate wallets

            let ui = &mut uis[wallet_idx];  // our current column, index comes from the iteration of wallets

            let can_accept_what_is_being_dragged = true; // We accept anything being dragged (for now) ¯\_(ツ)_/¯

            let response = drop_target(ui, can_accept_what_is_being_dragged, |ui| {  // Call the drag and drop function

                ui.set_min_size(vec2(64.0, 100.0)); // set window size (To be Modified)

                for (account_idx, account) in wallet.1.into_iter().enumerate() { // iterate accounts

                    if let Ok((account, account_assets)) = accounts.get(*account) { // obtain al the assets from the current account

                        let account_id_trimmed = account.account_id
                            .to_string()
                            .get(0..8)
                            .unwrap_or_default()
                            .to_string();

                        ui.add(Label::new(account_id_trimmed));

                        for (asset_idx, asset) in account_assets.iter().enumerate(){  // iterate assets

                            if let Ok(asset) = assets.get(*asset){

                                let asset_name = match asset {
                                    MissionToken::Time(_) => "Time",
                                    MissionToken::Trust(_) => "Trust",
                                    MissionToken::Energy(_) => "Energy",
                                    MissionToken::Labor(_) => "Labor",
                                    MissionToken::None => "None",
                                };
    
                                let asset_value = match asset{
                                    MissionToken::Time(asset) => asset.0.0,
                                    MissionToken::Trust(asset) => asset.0.0,
                                    MissionToken::Energy(asset) => asset.0.0,
                                    MissionToken::Labor(asset) => asset.0.0,
                                    MissionToken::None => 0,
                                };

                                let item_id = Id::new(id_source).with(wallet_idx).with(account_idx).with(asset_idx); // we create an id with all index
                                drag_source(ui, item_id, |ui| {
                                   ui.add(Label::new(format!("{}: {}",asset_name,asset_value)).sense(Sense::click())); //añadir vainas locas
                                    //let response = 
                                    //response.context_menu(|ui| {
                                    //    if ui.button("Remove").clicked() {
                                    //        //drag_drop.wallet_list[wallet_idx].1[account_idx].1.remove(asset_idx); // we remove the selected asset
                                    //        println!("restamos");
                                    //        ui.close_menu();
                                    //    }
                                    //});
                                });
                                if ui.memory().is_being_dragged(item_id) {
                                    source_col_row_aisle = Some((wallet_idx, account_idx, asset_idx));
                                }

                            }
                        
                        }
                        
                    }
                }
            })
            .response;
            //let response = response.context_menu(|ui| {
            //    if ui.button("New Item").clicked() {
            //        drag_drop.wallet_list[wallet_idx].1[source_col_row_aisle.unwrap().1].1.push(("New Item".to_owned(),1));
            //        ui.close_menu();
            //    }
            //});
            let is_being_dragged = ui.memory().is_anything_being_dragged();
            if is_being_dragged && can_accept_what_is_being_dragged && response.hovered() {
                drop_col = Some(wallet_idx);
                //drop_row = Some(account_idx);
            }
        }
    });
    if let Some((source_col, source_row, source_aisle)) = source_col_row_aisle {
        if let Some(drop_col) = drop_col {
            if ui.input().pointer.any_released() {
                // do the drop:
                for (wallet_idx, wallet) in wallets.into_iter().enumerate() {
                    for (account_idx, account) in wallet.1.into_iter().enumerate() {
                        if let Ok((account, account_assets)) = accounts.get(*account) { 
                            for (asset_idx, asset) in account_assets.into_iter().enumerate(){
                                if wallet_idx == source_col
                                    && account_idx == source_row
                                    && asset_idx == source_aisle {
                                        println!("match");
                                        //if let Ok(mut asset) = assets.get(*asset){
                                        //    match asset{
                                        //        MissionToken::Energy(value)=>{
                                        //            *asset = MissionToken::Energy(Asset(Amount(0)))
                                        //        }
                                        //        _=>{}
                                        //    }
                                        //}
                                }
                            }
                        }
                    }
                }
                //let item = drag_drop.wallet_list[source_col].1[source_row].1.remove(source_aisle);
                //drag_drop.wallet_list[drop_col].1[source_row].1.push(item);
            }
        }
    }
});
}

//#[derive(Component)]
//pub struct Check{
// timer: Timer
//}

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

fn initialize_images (
    mut egui_ctx: ResMut<EguiContext>,
    images: Local<Images>,
    mut image_texture_ids: ResMut<ImageTextureIds>,
) {
    image_texture_ids.trust_icon = Some(egui_ctx.add_image(images.trust_icon.clone()));
    image_texture_ids.time_icon = Some(egui_ctx.add_image(images.time_icon.clone()));
    image_texture_ids.energy_icon = Some(egui_ctx.add_image(images.energy_icon.clone()));
}

fn setup_behaviors(mut commands: Commands) {
    behaviors::create(&mut commands);
}
