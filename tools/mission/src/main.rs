use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{egui,egui::{vec2,Id,Label,Sense}, EguiContext};
use egui_extras::{TableBuilder, Size};
use simula_action::ActionPlugin;
use simula_camera::orbitcam::*;
use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};
use simula_decision::{
    BehaviorBundle, BehaviorState, DecisionEditorState, DecisionGraphState, DecisionPlugin,
};
use simula_mission::{asset::{Asset,Amount}, wallet::Wallet, account::Account, MissionPlugin, WalletBuilder};
use simula_net::NetPlugin;
use simula_viz::{
    axes::{Axes, AxesBundle, AxesPlugin},
    grid::{Grid, GridBundle, GridPlugin},
    lines::{LineMesh, LinesMaterial, LinesPlugin},
};
// mod graph;
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
    .insert_resource(DragAndDropDemo{wallet_list: vec![]})
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
    .add_plugin(DecisionPlugin)
    .register_type::<MissionToken>()
    .add_startup_system(setup)
    .add_system(debug_info)
    .add_system(increase_mission_time)
    .add_system(wallet_ui_system)
    .add_system_set(
        SystemSet::new()
            .with_system(behavior_agent_rest)
            .with_system(behavior_agent_work),
    )
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

pub fn wallet_ui_system (
    mut egui_ctx: ResMut<EguiContext>,
    wallets: Query<(&Wallet, &Children)>,
    accounts: Query<(&Account, &Children)>,
    assets: Query<&MissionToken>,
    mut selected_wallet: ResMut<SelectedWallet>,
) {
    egui::Window::new("Wallets")
        .default_width(200.0)
        .resizable(true)
        .vscroll(true)
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
            for &wallet_account in wallet_list[selected_wallet.0].1.iter() {
                if let Ok((account, account_assets)) = accounts.get(wallet_account) {
                    let account_id_trimmed = account.account_id
                            .to_string()
                            .get(0..8)
                            .unwrap_or_default()
                            .to_string();
                    ui.separator();
                    ui.collapsing(format!("account {:?}", account_id_trimmed), |ui| {
                        let mut asset_list: Vec<(String, i128)> = vec![];
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
                                asset_list.push((asset_name.to_string(), asset_value));
                            }
                        }
                        
                        TableBuilder::new(ui)
                            .column(Size::remainder())
                            .column(Size::remainder())
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
                                            ui.label(asset.0.clone());
                                        });
                                        row.col(|ui| {
                                            ui.label(asset.1.to_string());
                                        });
                                    });
                                }
                            });
                    });
                }
            }
        });
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut lines_materials: ResMut<Assets<LinesMaterial>>,
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
        .insert(DecisionEditorState {
            show: true,
            ..default()
        })
        .insert(DecisionGraphState::default())
        .with_children(|parent| {
            parent.spawn_bundle(BehaviorBundle::<AgentRest>::default());
            parent.spawn_bundle(BehaviorBundle::<AgentWork>::default());
        })
        .insert(Name::new("Decision Graph"))
        .id();

    let agent_decision_graph_2 = commands
        .spawn()
        .insert(DecisionEditorState {
            show: true,
            ..default()
        })
        .insert(DecisionGraphState::default())
        .with_children(|parent| {
            parent.spawn_bundle(BehaviorBundle::<AgentRest>::default());
            parent.spawn_bundle(BehaviorBundle::<AgentWork>::default());
        })
        .insert(Name::new("Decision Graph"))
        .id();

    commands
        .spawn()
        .push_children(&[agent_wallet, agent_decision_graph])
        .push_children(&[agent_wallet_2, agent_decision_graph_2])
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
}

#[derive(Default, Component, Reflect, Clone)]
struct AgentRest;

fn behavior_agent_rest(
    agents: Query<(
        &AgentRest,
        &mut BehaviorState,
        &Wallet,
        &mut DecisionGraphState,
    )>,
) {
    for _agent in agents.iter() {}
}

#[derive(Default, Component, Reflect, Clone)]
struct AgentWork;

fn behavior_agent_work(
    agents: Query<(
        &AgentWork,
        &mut BehaviorState,
        &Wallet,
        &mut DecisionGraphState,
    )>,
) {
    for _agent in agents.iter() {}
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
    mut drag_drop: ResMut<DragAndDropDemo>
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
