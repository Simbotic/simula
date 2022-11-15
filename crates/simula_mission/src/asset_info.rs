use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use crate::asset::Amount;
use std::collections::HashMap;

pub trait AssetInfo: Component + Default {
    type AssetAttributes: Component + Clone + Default;

    fn name(&self) -> &'static str;
    fn icon_dir(&self) -> &'static str;
    fn amount(&self) -> Amount;
    fn is_draggable(&self) -> bool;
    fn render(&self, ui: &mut egui::Ui, texture_ids: &mut ResMut<ImageTextureIds>, asset_server: &mut Res<AssetServer>, egui_ctx: &mut ResMut<EguiContext>, _attributes: &Self::AssetAttributes) {
        ui.horizontal(|ui| {
            if let Some(icon) = texture_ids.get_or_create_texture(self.icon_dir(), asset_server, egui_ctx) {
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
    fn class_id(&self)->u64;
    fn asset_id(&self)->u64;
    fn drag(&mut self)-> bool;
    fn drop(&mut self, src_class_id: u64, src_asset_id: u64, source_amount: Amount)-> bool;
    fn push_as_children(&self,commands: &mut Commands, parent: Entity);
}


#[derive(Deref, DerefMut, Debug, Default, Clone, Component)]
pub struct ImageTextureIds(pub HashMap<&'static str, (Handle<Image>, Option<egui::TextureId>)>);

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

