use crate::asset::Amount;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use std::collections::HashMap;
// use crate::MissionToken;

use crate::asset::Asset;

#[derive(Debug, Default, Reflect, Component, Clone, PartialEq)]
#[reflect(Component)]
pub enum MissionToken {
    #[default]
    None,
    Time(Asset<1000, 0>),
    Trust(Asset<1000, 1>),
    Energy(Asset<1000, 2>),
    Labor(Asset<1000, 3>),
}

pub struct TokenUiPlugin;

impl Plugin for TokenUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ImageTextureIds(HashMap::new()));
    }
}

#[derive(Deref, DerefMut, Debug, Default, Clone, Component)]
pub struct ImageTextureIds(HashMap<&'static str, (Handle<Image>, Option<egui::TextureId>)>);

impl ImageTextureIds {
    pub fn get_texture(&self, key: &'static str) -> Option<egui::TextureId> {
        if let Some(image_texture) = self.0.get(key) {
            if let Some(texture_id) = image_texture.1 {
                return Some(texture_id);
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn get_or_create_texture(
        &mut self,
        key: &'static str,
        asset_server: &mut Res<AssetServer>,
        egui_ctx: &mut EguiContext,
    ) -> Option<egui::TextureId> {
        if let Some(image_texture) = self.0.get(key) {
            if let Some(texture_id) = image_texture.1 {
                return Some(texture_id);
            } else {
                self.initialize_image_texture(key, asset_server, egui_ctx)
            }
        } else {
            self.initialize_image_texture(key, asset_server, egui_ctx)
        }
    }
    pub fn initialize_image_texture(
        &mut self,
        key: &'static str,
        asset_server: &mut Res<AssetServer>,
        egui_ctx: &mut EguiContext,
    ) -> Option<egui::TextureId> {
        let image = asset_server.load(key);
        let texture_id = Some(egui_ctx.add_image(image.clone()));
        self.0.insert(key, (image, texture_id));
        texture_id
    }
}

pub trait AssetInfo {
    fn name(&self) -> &'static str;
    fn icon_dir(&self) -> &'static str;
    fn amount(&self) -> Amount;
    fn is_draggable(&self) -> bool;
    fn render(&self, ui: &mut egui::Ui, texture_ids: &ImageTextureIds) {
        ui.horizontal(|ui| {
            if let Some(icon) = texture_ids.get_texture(self.icon_dir()) {
                ui.add(egui::widgets::Image::new(icon, [20.0, 20.0]));
            }
            let label = egui::Label::new(format!("{}: {}", self.name(), self.amount().0));
            if self.is_draggable() {
                ui.add(label.sense(egui::Sense::click()));
            } else {
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
}
