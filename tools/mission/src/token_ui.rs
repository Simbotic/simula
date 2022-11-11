use crate::MissionToken;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use simula_mission::prelude::*;

pub struct TokenUiPlugin;

impl Plugin for TokenUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(initialize_images)
            .insert_resource(ImageTextureIds {
                time_icon: None,
                energy_icon: None,
                trust_icon: None,
                labor_icon: None,
            });
    }
}

pub struct Images {
    time_icon: Handle<Image>,
    trust_icon: Handle<Image>,
    energy_icon: Handle<Image>,
    labor_icon: Handle<Image>,
}

impl FromWorld for Images {
    fn from_world(world: &mut World) -> Self {
        if let Some(asset_server) = world.get_resource_mut::<AssetServer>() {
            Self {
                time_icon: asset_server.load("../assets/mission/Balance.png"),
                trust_icon: asset_server.load("../assets/mission/Money - Cash.png"),
                energy_icon: asset_server.load("../assets/mission/Money - Coins.png"),
                labor_icon: asset_server.load("../assets/mission/labor-icon.png"),
            }
        } else {
            Self {
                time_icon: Handle::default(),
                trust_icon: Handle::default(),
                energy_icon: Handle::default(),
                labor_icon: Handle::default(),
            }
        }
    }
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

#[derive(Component, Default, Clone)]
pub struct MissionTokenAttributes {
    icon: Option<egui::TextureId>,
}

impl AssetInfo for MissionToken {
    type AssetAttributes = MissionTokenAttributes;

    fn name(&self) -> &'static str {
        match self {
            MissionToken::Time(_) => "Time",
            MissionToken::Trust(_) => "Trust",
            MissionToken::Energy(_) => "Energy",
            MissionToken::Labor(_) => "Labor",
        }
    }

    // fn icon(&self, image_texture_ids: &Res<ImageTextureIds>) -> Option<egui::TextureId> {
    //     match self {
    //         MissionToken::Time(_) => image_texture_ids.time_icon,
    //         MissionToken::Trust(_) => image_texture_ids.trust_icon,
    //         MissionToken::Energy(_) => image_texture_ids.energy_icon,
    //         MissionToken::Labor(_) => image_texture_ids.labor_icon,
    //     }
    // }

    fn amount(&self) -> Amount {
        match self {
            MissionToken::Time(asset) => asset.0,
            MissionToken::Trust(asset) => asset.0,
            MissionToken::Energy(asset) => asset.0,
            MissionToken::Labor(asset) => asset.0,
        }
    }

    fn is_draggable(&self) -> bool {
        match self {
            MissionToken::Time(_) => false,
            MissionToken::Trust(_) => true,
            MissionToken::Energy(_) => true,
            MissionToken::Labor(_) => true,
        }
    }

    fn render(&self, ui: &mut egui::Ui, attributes: &Self::AssetAttributes) {
        match self {
            MissionToken::Time(_) => {
                ui.horizontal(|ui| {
                    if let Some(icon) = attributes.icon {
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
                if let Some(icon) = attributes.icon {
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
                    if let Some(icon) = attributes.icon {
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
                    ui.add(egui::widgets::SelectableLabel::new(
                        true,
                        format!("{}: {}", self.name(), self.amount().0),
                    ));
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