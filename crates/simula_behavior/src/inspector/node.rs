use crate::{
    color_hex_utils::color_from_hex, BehaviorChildren, BehaviorCursor, BehaviorFailure,
    BehaviorNode, BehaviorRunning, BehaviorSuccess, BehaviorType,
};
use bevy::prelude::*;
use bevy_inspector_egui::egui::{self, Rounding};

#[derive(Default, Clone)]
pub struct BehaviorInspectorNodeAttributes;

#[derive(Default, Clone, Reflect)]
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
        } else if behavior.name == "All" {
            color_from_hex("#36980D").unwrap()
        } else if behavior.name == "Any" {
            color_from_hex("#CC0100").unwrap()
        } else {
            color_from_hex("#000000").unwrap()
        }
    } else if behavior.typ == BehaviorType::Decorator {
        color_from_hex("#ACA000").unwrap()
    } else {
        color_from_hex("#3f3f3f").unwrap()
    }
}

pub fn behavior_inspector_node_ui(
    behaviors_query: &Query<(
        &Name,
        &BehaviorNode,
        &BehaviorChildren,
        Option<&BehaviorRunning>,
        Option<&BehaviorFailure>,
        Option<&BehaviorSuccess>,
        Option<&BehaviorCursor>,
    )>,
    node: &mut BehaviorInspectorNode,
    ui: &mut egui::Ui,
) {
    let node_entity = node.entity.unwrap();
    if let Ok((
        behavior_name,
        behavior_node,
        behavior_children,
        behavior_running,
        behavior_failure,
        behavior_success,
        behavior_cursor,
    )) = behaviors_query.get(node_entity)
    {
        let cursor_stroke = if behavior_cursor.is_some() {
            egui::Stroke::new(3.0, color_from_hex("#FF00FF").unwrap())
        } else {
            egui::Stroke::NONE
        };

        ui.set_min_width(100.0);
        ui.push_id(format!("bhtins-{}", node_entity.index()), |ui| {
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
                            if let Some(_) = &mut node.entity {
                                // TODO: Add a way to display the value of the node using Bevy Inspector
                                // maybe? ui_for_value();
                            }
                        });
                    });

                ui.horizontal(|ui| {
                    for child in behavior_children.iter() {
                        let mut child_node = BehaviorInspectorNode {
                            entity: Some(*child),
                        };
                        behavior_inspector_node_ui(&behaviors_query, &mut child_node, ui);
                    }
                });
            });
        });
    }
}
