use crate::{inspector::BehaviorInspectorNode, BehaviorTree};
use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::{self},
    egui,
};

use super::node::behavior_inspector_node_ui;

#[derive(Default, Clone, Resource)]
pub struct BehaviorInspectorAttributes;

#[derive(Default, Clone, Resource, Reflect)]
pub struct BehaviorInspector {
    pub selected: BehaviorInspectorItem,
}

impl BehaviorInspector {
    pub fn select(&mut self, entity: Entity, name: String) {
        self.selected = BehaviorInspectorItem {
            entity: Some(entity),
            name,
        };
    }

    pub fn unselect(&mut self) {
        self.selected = BehaviorInspectorItem::default();
    }
}

#[derive(Clone, PartialEq, Reflect)]
pub struct BehaviorInspectorItem {
    pub entity: Option<Entity>,
    pub name: String,
}

impl Default for BehaviorInspectorItem {
    fn default() -> Self {
        Self {
            entity: None,
            name: "Select Behavior".to_string(),
        }
    }
}

fn item_label(item: &BehaviorInspectorItem) -> String {
    let entity = if let Some(entity) = item.entity {
        format!("[{}]: ", entity.index())
    } else {
        "".to_string()
    };
    format!("{}{}", entity, item.name)
}

pub fn behavior_inspector_ui(world: &mut World) {
    let mut egui_context = world.resource_mut::<bevy_egui::EguiContext>().clone();
    let mut behavior_trees = world.query::<(Entity, Option<&Name>, &BehaviorTree)>();
    let behavior_inspector = world.resource_mut::<BehaviorInspector>().clone();

    egui::Window::new("Behavior Inspector").show(egui_context.ctx_mut(), |ui| {
        egui::ComboBox::from_id_source("Behavior Inspector Selector")
            .selected_text(item_label(&behavior_inspector.selected))
            .show_ui(ui, |ui| {
                let mut selectable_behaviors: Vec<BehaviorInspectorItem> = {
                    behavior_trees
                        .iter(world)
                        .map(|(entity, name, _)| BehaviorInspectorItem {
                            entity: Some(entity),
                            name: name.unwrap_or(&Name::new("")).to_string(),
                        })
                        .collect::<Vec<_>>()
                };
                selectable_behaviors.insert(0, BehaviorInspectorItem::default());
                for selectable_behavior in selectable_behaviors {
                    if ui
                        .selectable_label(
                            behavior_inspector.selected == selectable_behavior,
                            item_label(&selectable_behavior),
                        )
                        .clicked()
                    {
                        let mut behavior_inspector = world.resource_mut::<BehaviorInspector>();
                        behavior_inspector.selected = selectable_behavior;
                    }
                }
            });

        if let BehaviorInspectorItem {
            entity: Some(entity),
            name,
        } = &behavior_inspector.selected
        {
            if let Ok((_, _, behavior_tree)) = behavior_trees.get(world, *entity) {
                let mut node = BehaviorInspectorNode {
                    entity: behavior_tree.root,
                };
                egui::Window::new(format!("Behavior: {}", name))
                    .title_bar(true)
                    .resizable(true)
                    .collapsible(true)
                    .scroll2([true, true])
                    .show(ui.ctx(), |ui| {
                        behavior_inspector_node_ui(world, &mut node, ui);
                    });
            }
        }
    });
}
