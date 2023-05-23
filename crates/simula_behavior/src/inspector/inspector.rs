use crate::{inspector::BehaviorInspectorNode, BehaviorCursor, BehaviorNode, BehaviorTree};
use bevy::{ecs::system::SystemState, prelude::*};
use bevy_inspector_egui::{
    bevy_egui::EguiContexts, egui, restricted_world_view::RestrictedWorldView,
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

fn find_cursor(world: &mut World, tree_entity: Entity) -> Option<Entity> {
    let mut query = world.query_filtered::<(Entity, &BehaviorNode), With<BehaviorCursor>>();
    for (entity, node) in query.iter(world) {
        if node.tree == Some(tree_entity) {
            return Some(entity);
        }
    }
    None
}

pub fn behavior_inspector_ui(world: &mut World) {
    let mut behavior_trees = world.query::<(Entity, Option<&Name>, &BehaviorTree)>();
    let behavior_inspector = world.resource_mut::<BehaviorInspector>().clone();

    let mut system_state: SystemState<EguiContexts> = SystemState::new(world);

    let mut context1 = system_state.get_mut(world).ctx_mut().clone();
    let mut context2 = system_state.get_mut(world).ctx_mut().clone();

    egui::Window::new("Behavior Inspector").show(&mut context1, |ui| {
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
            let (tree_entity, tree_root) = {
                let Ok((tree_entity, _, behavior_tree)) = behavior_trees.get(world, *entity) else {
                return;};
                let Some(tree_root) = behavior_tree.root else {return;};
                (tree_entity, tree_root)
            };

            let mut node = BehaviorInspectorNode {
                entity: Some(tree_root),
            };

            match find_cursor(world, tree_entity) {
                Some(_cursor) => {
                    ui.label("Running");
                }
                None => {
                    if ui.button("Run").clicked() {
                        world.entity_mut(tree_root).insert(BehaviorCursor::Delegate);
                    }
                }
            }

            let type_registry = world.resource::<AppTypeRegistry>().0.clone();

            let mut world = RestrictedWorldView::new(world);

            egui::Window::new(format!("Behavior: {}", name))
                .title_bar(true)
                .resizable(true)
                .collapsible(true)
                .scroll2([true, true])
                .show(ui.ctx(), |ui| {
                    behavior_inspector_node_ui(
                        0,
                        0,
                        &mut context2,
                        &mut world,
                        &mut node,
                        ui,
                        &type_registry,
                    );
                });
            // }
        }
    });
}
