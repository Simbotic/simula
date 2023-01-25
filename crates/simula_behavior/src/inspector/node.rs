use crate::{
    color_hex_utils::color_from_hex, BehaviorChildren, BehaviorCursor, BehaviorFailure,
    BehaviorNode, BehaviorRunning, BehaviorSuccess, BehaviorType,
};
use bevy::{ecs::query::WorldQuery, prelude::*, reflect::TypeRegistryInternal};
use bevy_inspector_egui::{
    bevy_inspector,
    egui::{self, Rounding},
};

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

#[derive(WorldQuery)]
pub struct BehaviorQuery {
    pub entity: Entity,
    pub name: &'static Name,
    pub node: &'static BehaviorNode,
    pub children: &'static BehaviorChildren,
    pub running: Option<&'static BehaviorRunning>,
    pub failure: Option<&'static BehaviorFailure>,
    pub success: Option<&'static BehaviorSuccess>,
    pub cursor: Option<&'static BehaviorCursor>,
}

pub struct Behavior {
    pub entity: Entity,
    pub name: Name,
    pub node: BehaviorNode,
    pub children: BehaviorChildren,
    pub running: Option<BehaviorRunning>,
    pub failure: Option<BehaviorFailure>,
    pub success: Option<BehaviorSuccess>,
    pub cursor: Option<BehaviorCursor>,
}

pub fn behavior_inspector_node_ui(
    world: &mut World,
    node: &mut BehaviorInspectorNode,
    ui: &mut egui::Ui,
    type_registry: &TypeRegistryInternal,
) {
    let mut behaviors = world.query::<BehaviorQuery>();
    let behaviors: Vec<Behavior> = behaviors
        .iter(world)
        .filter(|item| item.entity == node.entity.unwrap())
        .map(|item| Behavior {
            entity: item.entity.clone(),
            name: item.name.clone(),
            node: item.node.clone(),
            children: item.children.clone(),
            running: item.running.clone().copied(),
            failure: item.failure.clone().copied(),
            success: item.success.clone().copied(),
            cursor: item.cursor.clone().copied(),
        })
        .collect::<Vec<_>>();

    let node_entity = node.entity.unwrap();
    for behavior in behaviors {
        // Cursor stroke (if cursor is on this node
        let cursor_stroke = if behavior.cursor.is_some() {
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
                            .fill(titlebar_color(&behavior.node))
                            .show(ui, |ui| {
                                ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
                                ui.horizontal(|ui| {
                                    ui.set_min_width(100.0);
                                    ui.label(
                                        egui::RichText::new(behavior.name.to_string()).strong(),
                                    );
                                });
                            });
                        ui.horizontal(|ui| {
                            let r = 3.0;
                            let size = egui::Vec2::splat(2.0 * r + 5.0);
                            let (rect, _response) =
                                ui.allocate_at_least(size, egui::Sense::hover());
                            if behavior.failure.is_some() {
                                ui.painter()
                                    .circle_filled(rect.center(), r, egui::Color32::RED);
                            } else if behavior.success.is_some() {
                                ui.painter().circle_filled(
                                    rect.center(),
                                    r,
                                    egui::Color32::DARK_GREEN,
                                );
                            } else if behavior.running.is_some() {
                                ui.painter()
                                    .circle_filled(rect.center(), r, egui::Color32::GREEN);
                            } else {
                                ui.painter()
                                    .circle_filled(rect.center(), r, egui::Color32::GRAY);
                            }
                            ui.label(egui::RichText::new(&*behavior.name.as_str()).small());
                        });

                        ui.collapsing("", |ui| {
                            ui.set_max_width(250.0);
                            let mut behavior_node_clone = behavior.node.clone();
                            bevy_inspector::ui_for_value(&mut behavior_node_clone, ui, world);
                        });
                    });

                ui.horizontal(|ui| {
                    for child in behavior.children.iter() {
                        let mut child_node = BehaviorInspectorNode {
                            entity: Some(*child),
                        };
                        behavior_inspector_node_ui(world, &mut child_node, ui, type_registry);
                    }
                });
            });
        });
    }
}
