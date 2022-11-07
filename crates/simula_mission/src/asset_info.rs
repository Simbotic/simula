use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use crate::asset::Amount;

pub trait AssetInfo: Component + Default {
    type AssetAttributes: Component;

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
}