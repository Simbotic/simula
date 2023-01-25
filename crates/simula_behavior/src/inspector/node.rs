use std::any::TypeId;

use crate::{
    color_hex_utils::color_from_hex, BehaviorChildren, BehaviorCursor, BehaviorFailure,
    BehaviorNode, BehaviorRunning, BehaviorSuccess, BehaviorType,
};
use bevy::{prelude::*, reflect::TypeRegistryInternal};
use bevy_inspector_egui::{
    egui::{self, Rounding},
    reflect_inspector::{Context, InspectorUi},
    restricted_world_view::RestrictedWorldView,
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

pub fn behavior_inspector_node_ui(
    world: &mut RestrictedWorldView,
    node: &mut BehaviorInspectorNode,
    ui: &mut egui::Ui,
    type_registry: &TypeRegistryInternal,
) {
    let node_entity = node.entity.unwrap();

    let (mut component_view, mut world) =
        world.split_off_component((node_entity, TypeId::of::<Name>()));
    let behavior_name_result = component_view.get_entity_component_reflect(
        node_entity,
        TypeId::of::<Name>(),
        &type_registry,
    );
    let behavior_name = if let Ok(behavior_name_result) = behavior_name_result {
        let behavior_name = behavior_name_result.0.downcast_ref::<Name>().unwrap();
        Some(behavior_name)
    } else {
        None
    };

    let (mut component_view, mut world) =
        world.split_off_component((node_entity, TypeId::of::<BehaviorNode>()));
    let behavior_node_result = component_view.get_entity_component_reflect(
        node_entity,
        TypeId::of::<BehaviorNode>(),
        &type_registry,
    );
    let behavior_node = if let Ok(behavior_node_result) = behavior_node_result {
        let behavior_node = behavior_node_result
            .0
            .downcast_mut::<BehaviorNode>()
            .unwrap();
        Some(behavior_node)
    } else {
        None
    };

    let (mut component_view, mut world) =
        world.split_off_component((node_entity, TypeId::of::<BehaviorChildren>()));
    let behavior_children_result = component_view.get_entity_component_reflect(
        node_entity,
        TypeId::of::<BehaviorChildren>(),
        &type_registry,
    );
    let behavior_children = if let Ok(behavior_children_result) = behavior_children_result {
        let behavior_children = behavior_children_result
            .0
            .downcast_ref::<BehaviorChildren>()
            .unwrap();
        Some(behavior_children)
    } else {
        None
    };

    let (mut component_view, mut world) =
        world.split_off_component((node_entity, TypeId::of::<BehaviorRunning>()));
    let behavior_running_result = component_view.get_entity_component_reflect(
        node_entity,
        TypeId::of::<BehaviorRunning>(),
        &type_registry,
    );

    let (mut component_view, mut world) =
        world.split_off_component((node_entity, TypeId::of::<BehaviorFailure>()));
    let behavior_failure_result = component_view.get_entity_component_reflect(
        node_entity,
        TypeId::of::<BehaviorFailure>(),
        &type_registry,
    );

    let (mut component_view, mut world) =
        world.split_off_component((node_entity, TypeId::of::<BehaviorSuccess>()));
    let behavior_success_result = component_view.get_entity_component_reflect(
        node_entity,
        TypeId::of::<BehaviorSuccess>(),
        &type_registry,
    );

    let (mut component_view, world) =
        world.split_off_component((node_entity, TypeId::of::<BehaviorCursor>()));
    let behavior_cursor_result = component_view.get_entity_component_reflect(
        node_entity,
        TypeId::of::<BehaviorCursor>(),
        &type_registry,
    );

    let mut cx = Context { world: Some(world) };

    // Cursor stroke (if cursor is on this node
    let cursor_stroke = if behavior_cursor_result.is_ok() {
        egui::Stroke::new(3.0, color_from_hex("#FF00FF").unwrap())
    } else {
        egui::Stroke::NONE
    };

    if let (Some(behavior_name), Some(behavior_node), Some(behavior_children)) =
        (behavior_name, behavior_node, behavior_children)
    {
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
                            if behavior_failure_result.is_ok() {
                                ui.painter()
                                    .circle_filled(rect.center(), r, egui::Color32::RED);
                            } else if behavior_success_result.is_ok() {
                                ui.painter().circle_filled(
                                    rect.center(),
                                    r,
                                    egui::Color32::DARK_GREEN,
                                );
                            } else if behavior_running_result.is_ok() {
                                ui.painter()
                                    .circle_filled(rect.center(), r, egui::Color32::GREEN);
                            } else {
                                ui.painter()
                                    .circle_filled(rect.center(), r, egui::Color32::GRAY);
                            }
                            ui.label(egui::RichText::new(&*behavior_name.as_str()).small());
                        });

                        ui.collapsing("", |ui| {
                            ui.set_max_width(250.0);
                            let mut env = InspectorUi::for_bevy(type_registry, &mut cx);
                            let _changed = env.ui_for_reflect(behavior_node, ui);
                        });
                    });

                ui.horizontal(|ui| {
                    for child in behavior_children.iter() {
                        let mut child_node = BehaviorInspectorNode {
                            entity: Some(*child),
                        };
                        behavior_inspector_node_ui(
                            cx.world.as_mut().unwrap(),
                            &mut child_node,
                            ui,
                            type_registry,
                        );
                    }
                });
            });
        });
    }
}
