use crate::{
    color_hex_utils::color_from_hex, BehaviorChildren, BehaviorCursor, BehaviorFailure,
    BehaviorNode, BehaviorRunning, BehaviorSuccess, BehaviorType,
};
use bevy::prelude::*;
use bevy_inspector_egui::{
    egui::{self, Rounding},
    options::EntityAttributes,
    Context, Inspectable,
};

#[derive(Default, Clone)]
pub struct BehaviorInspectorNodeAttributes;

#[derive(Default, Clone)]
pub struct BehaviorInspectorNode {
    pub entity: Option<Entity>,
}

fn titlebar_color(behavior: &BehaviorNode) -> egui::Color32 {
    if behavior.typ == BehaviorType::Action {
        color_from_hex("#4284F3").unwrap()
    } else if behavior.typ == BehaviorType::Composite {
        if behavior.name == "Selector" {
            color_from_hex("#CC0100").unwrap()
        } else if behavior.name == "Sequencer" {
            color_from_hex("#36980D").unwrap()
        } else {
            color_from_hex("#F3F342").unwrap()
        }
    } else if behavior.typ == BehaviorType::Decorator {
        color_from_hex("#ACA000").unwrap()
    } else {
        color_from_hex("#3f3f3f").unwrap()
    }
}

macro_rules! some_or_return {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => return false,
        }
    };
}

impl Inspectable for BehaviorInspectorNode {
    type Attributes = BehaviorInspectorNodeAttributes;

    fn ui(&mut self, ui: &mut egui::Ui, _options: Self::Attributes, context: &mut Context) -> bool {
        let mut changed = false;

        let entity = some_or_return!(self.entity);
        let world = some_or_return!(unsafe { context.world_mut() });
        let behavior_name = some_or_return!(world.get::<Name>(entity));
        let behavior_node = some_or_return!(world.get::<BehaviorNode>(entity));
        let behavior_children = some_or_return!(world.get::<BehaviorChildren>(entity));

        let behavior_running = world.get::<BehaviorRunning>(entity);
        let behavior_failure = world.get::<BehaviorFailure>(entity);
        let behavior_success = world.get::<BehaviorSuccess>(entity);
        let behavior_cursor = world.get::<BehaviorCursor>(entity);

        let cursor_stroke = if behavior_cursor.is_some() {
            egui::Stroke::new(1.0, egui::Color32::GREEN)
        } else {
            egui::Stroke::none()
        };

        ui.set_min_width(100.0);
        ui.push_id(format!("bhtins-{}", entity.id()), |ui| {
            ui.vertical(|ui| {
                // Node frame
                egui::Frame::none()
                    .stroke(cursor_stroke)
                    .fill(color_from_hex("#303030").unwrap())
                    .rounding(Rounding::same(3.0))
                    .show(ui, |ui| {
                        egui::Frame::none()
                            .inner_margin(egui::Vec2::new(4.0, 1.0))
                            .rounding(Rounding::same(3.0))
                            .fill(titlebar_color(&behavior_node))
                            .show(ui, |ui| {
                                ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
                                ui.horizontal(|ui| {
                                    ui.set_min_width(100.0);
                                    ui.label(
                                        egui::RichText::new(behavior_name.to_string()).strong(),
                                    );
                                });
                            });
                        ui.horizontal(|ui| {
                            let r = 3.0;
                            let size = egui::Vec2::splat(2.0 * r + 5.0);
                            let (rect, _response) =
                                ui.allocate_at_least(size, egui::Sense::hover());
                            if behavior_failure.is_some() {
                                ui.painter()
                                    .circle_filled(rect.center(), r, egui::Color32::RED);
                            } else if behavior_success.is_some() {
                                ui.painter().circle_filled(
                                    rect.center(),
                                    r,
                                    egui::Color32::DARK_GREEN,
                                );
                            } else if behavior_running.is_some() {
                                ui.painter()
                                    .circle_filled(rect.center(), r, egui::Color32::GREEN);
                            } else {
                                ui.painter()
                                    .circle_filled(rect.center(), r, egui::Color32::GRAY);
                            }
                            ui.label(egui::RichText::new(&behavior_node.name).small());
                        });

                        ui.collapsing("", |ui| {
                            ui.set_max_width(250.0);
                            if let Some(entity) = &mut self.entity {
                                let attributes = EntityAttributes { despawnable: false };
                                changed |= entity.ui(ui, attributes, context);
                            }
                        });
                    });

                ui.horizontal(|ui| {
                    for child in behavior_children.iter() {
                        let mut child_node = BehaviorInspectorNode {
                            entity: Some(*child),
                        };
                        let child_attributes = BehaviorInspectorNodeAttributes {};
                        changed |= child_node.ui(ui, child_attributes, context);
                    }
                });
            });
        });

        changed
    }
}
