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
    asset::Amount,
    wallet::Wallet,
    WalletBuilder
};
use simula_viz::{
    follow_ui::{FollowUI, FollowUIVisibility},
};
use crate::{MissionToken, FollowPanel};


pub struct WalletUIPlugin;

impl Plugin for WalletUIPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(SelectedWallet(0))
            .insert_resource(ImageTextureIds {
                time_icon: None,
                energy_icon: None,
                trust_icon: None,
                labor_icon: None,
            })    
            .add_startup_system(initialize_images)
            .add_system(wallet_creation_window)
            .add_system(wallet_ui_draw::<DefaultWalletUI>)
            .add_system(wallet_ui_draw::<MyCoolInGameWalletUI>);
            // .add_system(wallet_ui_system);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectedWallet(usize);

fn _wallet_ui_system(
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

                egui::Grid::new("accounts_grid").striped(false).show(ui, |ui| {
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
            });
    }
}

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
            ui.small_button("normal Follow Window").on_hover_text("display follow window").clicked().then(|| {
                create_wallet_ui(&mut commands, WalletUIType::Follow, DefaultWalletUI);
            });
            ui.small_button("normal Tool Window").on_hover_text("display tool window").clicked().then(|| {
                create_wallet_ui(&mut commands, WalletUIType::Tool, DefaultWalletUI);
            });
            ui.small_button("cool Follow Window").on_hover_text("display follow window").clicked().then(|| {
                create_wallet_ui(&mut commands, WalletUIType::Follow, MyCoolInGameWalletUI);
            });
            ui.small_button("cool Tool Window").on_hover_text("display tool window").clicked().then(|| {
                create_wallet_ui(&mut commands, WalletUIType::Tool, MyCoolInGameWalletUI);
            });
        });
}


impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        if let Some(asset_server) = world.get_resource_mut::<AssetServer>() {
            Self {
                time_icon: asset_server.load("../assets/mission/Balance.png"),
                trust_icon: asset_server.load("../assets/mission/Money - Cash.png"),
                energy_icon: asset_server.load("../assets/mission/Money - Coins.png"),
                labor_icon: asset_server.load("../assets/mission/labor-icon.png")
            }
        } else {
            Self {
                time_icon: Handle::default(),
                trust_icon: Handle::default(),
                energy_icon: Handle::default(),
                labor_icon: Handle::default()
            }
        }
    }
}

pub struct Images {
    time_icon: Handle<Image>,
    trust_icon: Handle<Image>,
    energy_icon: Handle<Image>,
    labor_icon: Handle<Image>
}

pub struct ImageTextureIds {
    time_icon: Option<egui::TextureId>,
    trust_icon: Option<egui::TextureId>,
    energy_icon: Option<egui::TextureId>,
    labor_icon: Option<egui::TextureId>,
}

fn initialize_images(
    mut egui_ctx: ResMut<EguiContext>,
    images: Local<Images>,
    mut image_texture_ids: ResMut<ImageTextureIds>,
) {
    image_texture_ids.trust_icon = Some(egui_ctx.add_image(images.trust_icon.clone()));
    image_texture_ids.time_icon = Some(egui_ctx.add_image(images.time_icon.clone()));
    image_texture_ids.energy_icon = Some(egui_ctx.add_image(images.energy_icon.clone()));
    image_texture_ids.labor_icon = Some(egui_ctx.add_image(images.labor_icon.clone()));
}
pub trait AssetInfo {
    fn name(&self) -> &'static str;
    fn icon(&self, texture_ids: &Res<ImageTextureIds>) -> Option<egui::TextureId>;
    fn amount(&self) -> Amount;
    fn is_draggable(&self) -> bool;
    fn render(&self,ui: &mut egui::Ui, texture_ids: &Res<ImageTextureIds>){
        ui.horizontal(|ui| {
            if let Some(icon) = self.icon(&texture_ids){
                ui.add(egui::widgets::Image::new(
                    icon,
                    [20.0, 20.0],
                ));
            }
            let label = egui::Label::new(format!(
                "{}: {}",
                self.name(), self.amount().0
            ));
            if self.is_draggable(){
                ui.add(label.sense(egui::Sense::click()));
                
            }else{
                ui.add(label);
            }
        });
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

    fn icon(&self, image_texture_ids: &Res<ImageTextureIds>) -> Option<egui::TextureId> {
        match self {
            MissionToken::Time(_) => image_texture_ids.time_icon,
            MissionToken::Trust(_) => image_texture_ids.trust_icon,
            MissionToken::Energy(_) => image_texture_ids.energy_icon,
            MissionToken::Labor(_) => image_texture_ids.labor_icon,
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

    fn is_draggable(&self) -> bool {
        match self {
            MissionToken::None => false,
            MissionToken::Time(_) => false,
            MissionToken::Trust(_) => true,
            MissionToken::Energy(_) => true,
            MissionToken::Labor(_) => true,
        }
    }

    fn render(&self,ui: &mut egui::Ui, texture_ids: &Res<ImageTextureIds>) {
        match self {
            MissionToken::None => {},
            MissionToken::Time(_) => {
                ui.horizontal(|ui| {
                    if let Some(icon) = self.icon(&texture_ids){
                        ui.add(egui::widgets::Image::new(
                            icon,
                            [20.0, 20.0],
                        ));
                    }

                    let label = egui::Label::new(format!(
                        ": {}",self.amount().0
                    ));

                    if self.is_draggable(){
                        ui.add(label.sense(egui::Sense::click()));
                        
                    }else{
                        ui.add(label);
                    }
                });
            },
            MissionToken::Trust(_) => {
                if let Some(icon) = self.icon(&texture_ids){
                    ui.add(egui::widgets::Image::new(
                        icon,
                        [20.0, 20.0],
                    ));
                }
            },
            MissionToken::Energy(_) => {
                ui.add(egui::widgets::SelectableLabel::new(true,format!(
                    "{}: {}",
                    self.name(), self.amount().0
                )));
            },
            MissionToken::Labor(_) => {
                ui.vertical(|ui| {
                    if let Some(icon) = self.icon(&texture_ids){
                        ui.add(egui::widgets::Image::new(
                            icon,
                            [20.0, 20.0],
                        ));
                        let label = egui::widgets::Label::new(format!(
                            "{}: {}",
                            self.name(), self.amount().0
                        ));
                        if self.is_draggable(){
                            ui.add(label.sense(egui::Sense::click()));
                        }else{
                            ui.add(label);
                        }
                    }
                });

            },
        }
    }
}


#[derive(Component)]
struct WalletUI;

// Mark wallet to be used with FollowUI
#[derive(Component)]
struct WalletUIFollow;

// Mark wallet to be used with as tool
#[derive(Component)]
struct WalletUITool;

enum WalletUIType {
    Follow,
    Tool,
}

enum WalletUIResponse {
    CloseTitlebar,
    _ChooseWallet(Entity),
    _StartDrag(Entity),
}

trait WalletUIOptions {
    fn titlebar(ui: &mut egui::Ui) -> Option<WalletUIResponse> {
        let mut response: Option<WalletUIResponse> = None;
        ui.horizontal(|ui| {
            ui.label("Wallet");
            response = ui.button("x").clicked().then(|| WalletUIResponse::CloseTitlebar);
        });
        response
    }
    fn wallet_selector(ui: &mut egui::Ui, selected_wallet: &mut usize, len: usize, get: impl Fn(usize) -> String ) {
        egui::ComboBox::from_label("Select a wallet").show_index(
            ui,
            selected_wallet,
            len,
            get,
        );
    }
    fn window_frame() -> Option<egui::containers::Frame> {
        None
    }
}

#[derive(Component)]
struct DefaultWalletUI;

impl WalletUIOptions for DefaultWalletUI{}

#[derive(Component)]
struct MyCoolInGameWalletUI;

impl WalletUIOptions for MyCoolInGameWalletUI {
    fn titlebar(ui: &mut egui::Ui) -> Option<WalletUIResponse> {
        let mut response: Option<WalletUIResponse> = None;
        ui.horizontal(|ui| {
            ui.label("My Cool In Game Wallet");
            response = ui.button("x").clicked().then(|| WalletUIResponse::CloseTitlebar);
        });
        response
    }
    fn wallet_selector(ui: &mut egui::Ui, selected_wallet: &mut usize, len: usize, get: impl Fn(usize) -> String ) {
        egui::ComboBox::from_label("Select a cool wallet").show_index(
            ui,
            selected_wallet,
            len,
            get,
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
}

fn wallet_ui_draw<T: WalletUIOptions + Component>(
    mut commands: Commands,
    wallets: Query<(&Wallet, &Children)>,
    mut egui_context: ResMut<EguiContext>,
    mut wallet_ui: Query<(Entity, &mut WalletUI, &mut T)>,
    mut selected_wallet: ResMut<SelectedWallet>,
) {
    for (entity, _, _) in wallet_ui.iter_mut() {

        let mut window = egui::Window::new("Wallet window")
            .id(egui::Id::new(entity));

        if let Some(frame) = T::window_frame() {
            window = window.frame(frame);
        };

        window
            .collapsible(false)
            .title_bar(true)
            .show(egui_context.ctx_mut(), |ui| {
                if let Some(response) = T::titlebar(ui) {
                    match response {
                        WalletUIResponse::CloseTitlebar => {
                            commands.entity(entity).despawn();
                        }
                        _ => {}
                    }
                }

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

                T::wallet_selector(ui, &mut selected_wallet.0, wallet_list.len(), |i| wallet_list[i].0.to_owned())

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

fn create_wallet_ui<T: WalletUIOptions + Component>(commands: &mut Commands, wallet_type: WalletUIType, configuration: T) {
    let entity = commands
        .spawn()
        .insert(WalletUI)
        .insert(configuration)
        .id();
    
    match wallet_type {
        WalletUIType::Follow => {
            commands.entity(entity)
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
                .insert(WalletUIFollow)
                .insert(FollowPanel);
        }
        WalletUIType::Tool => {
            commands.entity(entity).insert(WalletUITool);
        }
    }
}

fn _add_follow_ui_panel(commands: &mut Commands) {
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
}