use bevy::prelude::*;
use bevy_egui::egui;
use crate::asset::Amount;

pub trait AssetInfo: Component + Default {
    type AssetAttributes: Component + Clone + Default;

    fn name(&self) -> &'static str;
    // fn icon(&self, texture_ids: &Res<ImageTextureIds>) -> Option<egui::TextureId>;
    fn amount(&self) -> Amount;
    fn is_draggable(&self) -> bool;
    fn render(&self, ui: &mut egui::Ui, _attributes: &Self::AssetAttributes) {
        ui.horizontal(|ui| {
            // if let Some(icon) = icon {
            //     ui.add(egui::widgets::Image::new(icon, [20.0, 20.0]));
            // }
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