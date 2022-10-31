use bevy::{
    prelude::*
};
use bevy_egui::{
    egui,
    EguiContext,
};
use egui_extras::{Size, TableBuilder};
use simula_mission::{
    account::Account,
    wallet::Wallet,
    WalletBuilder
};
use simula_viz::{
    follow_ui::{FollowUI, FollowUIVisibility},
};
use crate::{MissionToken, FollowPanel, asset_ui::{ImageTextureIds, AssetInfo, TokenUiPlugin}};

pub struct WalletUIPlugin;

impl Plugin for WalletUIPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(SelectedWallet(0))
            .add_plugin(TokenUiPlugin)
            .add_system(wallet_creation_window)
            .add_system(wallet_ui_draw::<DefaultWalletUI>)
            .add_system(wallet_ui_draw::<MyCoolInGameWalletUI>);
            // .add_system(wallet_ui_system);
    }
}

#[derive(Debug, Clone, PartialEq, Component)]
pub struct SelectedWallet(usize);

fn wallet_creation_window(
    mut commands: Commands,
    mut egui_ctx: ResMut<EguiContext>,
) {
    egui::Window::new("Creation Panel")
        .default_width(200.0)
        .resizable(true)
        .collapsible(false)
        .title_bar(true)
        .vscroll(false)
        .drag_bounds(egui::Rect::EVERYTHING)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.small_button("Create wallet").on_hover_text("generate wallet").clicked().then(|| {
                add_wallet(&mut commands);
            });
            ui.small_button("normal Window").on_hover_text("display window").clicked().then(|| {
                create_wallet_ui(&mut commands, DefaultWalletUI{selected_wallet: 0});
            });
            ui.small_button("cool Window").on_hover_text("display window").clicked().then(|| {
                create_wallet_ui(&mut commands, MyCoolInGameWalletUI{selected_wallet: 0});
            });
        });
}

#[derive(Component)]
struct WalletUI;

// Mark wallet to be used with FollowUI
#[derive(Component)]
struct WalletUIFollow;

// Mark wallet to be used with as tool
#[derive(Component)]
struct WalletUITool;

enum WalletUIResponse {
    CloseTitlebar,
    //ChooseWallet(Entity),
    //StartDrag(Entity),
}

trait WalletUIOptions { 
    fn insert(entity: Entity, commands: &mut Commands) {
        commands.entity(entity).insert(WalletUITool);
    }
    fn titlebar(ui: &mut egui::Ui) -> Option<WalletUIResponse> {
        let mut response: Option<WalletUIResponse> = None;
        ui.horizontal(|ui| {
            ui.label("Wallet");
            response = ui.button("x").clicked().then(|| WalletUIResponse::CloseTitlebar);
        });
        response
    }
    fn wallet_selector(&mut self, ui: &mut egui::Ui, wallets: &Query<(&Wallet, &Children)>);
    fn window_frame() -> Option<egui::containers::Frame> {
        None
    }
    fn fixed_size(window: egui::Window, _x: f32, _y: f32) -> egui::Window {
        window
    }
    fn fixed_pos(window: egui::Window, _x: f32, _y: f32) -> egui::Window {
        window
    }
    fn collapsible() -> bool {
        false
    }
    fn vscroll() -> bool {
        false
    }
    fn resizable() -> bool {
        false
    }
    fn drag_bounds() -> Option<egui::Rect> {
        None
    }
    fn show_title_bar() -> bool {
        true
    }
    fn wallet_title() -> &'static str {
        "Wallets"
    }
    fn wallets(&mut self, ui: &mut egui::Ui, wallets: &Query<(&Wallet, &Children)>, accounts: &Query<(&Account, &Children)>, assets: &Query<&MissionToken>, image_texture_ids: &Res<ImageTextureIds>);
}

#[derive(Component)]
struct DefaultWalletUI {
    selected_wallet: usize,
}

impl WalletUIOptions for DefaultWalletUI{
    fn wallet_selector(&mut self, ui: &mut egui::Ui, wallets: &Query<(&Wallet, &Children)>) {
        let mut wallet_list: Vec<(String, &Children)> = vec![];
        for (wallet, wallet_accounts) in wallets.iter() {
            wallet_list.push((trim_wallet(wallet), wallet_accounts));
        }
        egui::ComboBox::from_label("Select a wallet").show_index(
            ui,
            &mut self.selected_wallet,
            wallet_list.len(),
            |i| wallet_list[i].0.to_owned(),
        );
    }
    fn wallets(&mut self, ui: &mut egui::Ui, wallets: &Query<(&Wallet, &Children)>, accounts: &Query<(&Account, &Children)>, assets: &Query<&MissionToken>, image_texture_ids: &Res<ImageTextureIds>) {
        let mut wallet_list: Vec<(String, &Children)> = vec![];
        for (wallet, wallet_accounts) in wallets.iter() {
            wallet_list.push((trim_wallet(wallet), wallet_accounts));
        }

        egui::Grid::new("accounts_grid").striped(false).show(ui, |ui| {
            if !wallet_list[0].1.is_empty() {
                ui.heading("Accounts");
                ui.end_row();
            } else {
                ui.heading("No accounts in selected wallet");
                ui.end_row();
            }
            for &wallet_account in wallet_list[self.selected_wallet].1.iter() {
                if let Ok((account, account_assets)) = accounts.get(wallet_account) {
                    ui.collapsing(trim_account(account), |ui| {
                        let mut asset_list: Vec<(String, i128, Option<egui::TextureId>)> = vec![];
                        for &account_asset in account_assets.iter() {
                            if let Ok(asset) = assets.get(account_asset) {
                                let asset_name = asset.name();
                                let asset_value = asset.amount();
                                let asset_icon = asset.icon(&image_texture_ids);
                                asset_list.push((asset_name.to_string(), asset_value.0, asset_icon));
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
                                for &account_asset in account_assets.iter() {
                                    if let Ok(asset) = assets.get(account_asset) {
                                        let asset_name = asset.name();
                                        let asset_value = asset.amount();
                                        let asset_icon = asset.icon(&image_texture_ids);
                                        asset_list.push((asset_name.to_string(), asset_value.0, asset_icon));
                                    }
                                }
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
    }
}

#[derive(Component)]
struct MyCoolInGameWalletUI {
    selected_wallet: usize,
}

impl WalletUIOptions for MyCoolInGameWalletUI {
    fn insert(entity: Entity, commands: &mut Commands) {
        commands.entity(entity).insert(WalletUIFollow);
    }
    fn titlebar(ui: &mut egui::Ui) -> Option<WalletUIResponse> {
        let mut response: Option<WalletUIResponse> = None;
        ui.horizontal(|ui| {
            ui.label("My Cool In Game Wallet");
            response = ui.button("x").clicked().then(|| WalletUIResponse::CloseTitlebar);
        });
        response
    }
    fn wallet_selector(&mut self, ui: &mut egui::Ui, wallets: &Query<(&Wallet, &Children)>) {
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
        egui::ComboBox::from_label("Select a cool wallet").show_index(
            ui,
            &mut self.selected_wallet,
            wallet_list.len(),
            |i| wallet_list[i].0.to_owned(),
        );
    }
    fn window_frame() -> Option<egui::containers::Frame> {
        Some(egui::containers::Frame {
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
        })
    }
    fn fixed_size(window: egui::Window, x: f32, y: f32) -> egui::Window {
        window.fixed_size(egui::vec2(x, y))
    }
    fn fixed_pos(window: egui::Window, x: f32, y: f32) -> egui::Window {
        window.fixed_pos(egui::Pos2::new(x, y))
    }
    fn show_title_bar() -> bool {
        false
    }
    fn wallets(&mut self, ui: &mut egui::Ui, wallets: &Query<(&Wallet, &Children)>, accounts: &Query<(&Account, &Children)>, assets: &Query<&MissionToken>, image_texture_ids: &Res<ImageTextureIds>) {
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

        egui::Grid::new("accounts_grid").striped(false).show(ui, |ui| {
            if !wallet_list[0].1.is_empty() {
                ui.heading("Accounts");
                ui.end_row();
            } else {
                ui.heading("No accounts in selected wallet");
                ui.end_row();
            }
            for &wallet_account in wallet_list[self.selected_wallet].1.iter() {
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
                                let asset_name = asset.name();
                                let asset_value = asset.amount();
                                let asset_icon = asset.icon(&image_texture_ids);
                                asset_list.push((asset_name.to_string(), asset_value.0, asset_icon));
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
    }
}

fn wallet_ui_draw<T: WalletUIOptions + Component>(
    mut commands: Commands,
    wallets: Query<(&Wallet, &Children)>,
    accounts: Query<(&Account, &Children)>,
    assets: Query<&MissionToken>,
    mut egui_context: ResMut<EguiContext>,
    mut wallet_ui: Query<(Entity, &mut T), With<WalletUI>>,
    follow_uis: Query<(&FollowUI, &FollowUIVisibility), With<FollowPanel>>,
    image_texture_ids: Res<ImageTextureIds>,
) {
    
    let mut ui_pos = None;
    let mut ui_size = None;
    for (follow_ui, visibility) in follow_uis.iter() {
        ui_size = Some(follow_ui.size);
        ui_pos = Some(visibility.screen_pos);
    }

    for (entity, mut options) in wallet_ui.iter_mut() {

        let mut window = egui::Window::new(T::wallet_title())
            .id(egui::Id::new(entity));
            
        window = window.title_bar(T::show_title_bar());
        window = window.collapsible(T::collapsible());
        window = window.vscroll(T::vscroll());
        window = window.resizable(T::resizable());

        if let Some(frame) = T::window_frame() {
            window = window.frame(frame);
        };

        if let Some(drag_bounds) = T::drag_bounds() {
            window = window.drag_bounds(drag_bounds);
        };

        if let Some(size) = ui_size {
            window = T::fixed_size(window, size.x, size.y);
        }

        if let Some(pos) = ui_pos {
            window = T::fixed_pos(window, pos.x, pos.y);
        }

        window
            .collapsible(false)
            .show(egui_context.ctx_mut(), |ui| {
                if let Some(response) = T::titlebar(ui) {
                    match response {
                        WalletUIResponse::CloseTitlebar => {
                            commands.entity(entity).despawn();
                        }
                    }
                }
                options.wallet_selector(ui, &wallets);
                options.wallets(ui, &wallets, &accounts, &assets, &image_texture_ids);  
            });
    }
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

fn create_wallet_ui<T: WalletUIOptions + Component>(
    commands: &mut Commands,
    configuration: T,
) {
    let entity = commands
        .spawn()
        .insert(WalletUI)
        .insert(configuration)
        .id();
    
    T::insert(entity, commands)
}

fn trim_id(id: String)-> String{
    id
        .get(0..8)
        .unwrap_or_default()
        .to_string()
}

pub fn trim_wallet(wallet: &Wallet)-> String{
    trim_id(wallet.wallet_id.to_string())
}

pub fn trim_account(account: &Account)->String{
    trim_id(account.account_id.to_string())
}