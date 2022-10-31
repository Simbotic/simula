use crate::{
    inspector::{BehaviorInspectorNode, BehaviorInspectorNodeAttributes},
    BehaviorChildQuery, BehaviorChildQueryFilter, BehaviorChildQueryItem, BehaviorChildren,
    BehaviorCursor, BehaviorFailure, BehaviorInfo, BehaviorParent, BehaviorPlugin,
    BehaviorRunQuery, BehaviorRunning, BehaviorSpawner, BehaviorSuccess, BehaviorTree,
    BehaviorType,
};
use bevy::{ecs::component::ComponentId, prelude::*};
use bevy_inspector_egui::{egui, Context, Inspectable};
use pretty_type_name::{pretty_type_name, pretty_type_name_str};

#[derive(Default, Clone)]
pub struct BehaviorInspectorAttributes {
    pub name: String,
    pub description: String,
}

#[derive(Default, Clone)]
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

#[derive(Clone, PartialEq)]
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
        format!("[{}]: ", entity.id())
    } else {
        "".to_string()
    };
    format!("{}{}", entity, item.name)
}

macro_rules! some_or_return {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => return false,
        }
    };
}

impl Inspectable for BehaviorInspector {
    type Attributes = BehaviorInspectorAttributes;

    fn ui(&mut self, ui: &mut egui::Ui, _options: Self::Attributes, context: &mut Context) -> bool {
        let mut changed = false;

        let world = some_or_return!(unsafe { context.world_mut() });

        egui::ComboBox::from_id_source("Behavior Inspector Selector")
            .selected_text(item_label(&self.selected))
            .show_ui(ui, |ui| {
                let mut selectable_behaviors: Vec<BehaviorInspectorItem> = {
                    let mut behavior_trees =
                        world.query_filtered::<(Entity, Option<&Name>), With<BehaviorTree>>();
                    behavior_trees
                        .iter(world)
                        .map(|(entity, name)| BehaviorInspectorItem {
                            entity: Some(entity),
                            name: name.unwrap_or(&Name::new("")).to_string(),
                        })
                        .collect::<Vec<_>>()
                };
                selectable_behaviors.insert(0, BehaviorInspectorItem::default());
                for selectable_behavior in selectable_behaviors {
                    if ui
                        .selectable_label(
                            self.selected == selectable_behavior,
                            item_label(&selectable_behavior),
                        )
                        .clicked()
                    {
                        self.selected = selectable_behavior;
                    }
                }
            });

        if let BehaviorInspectorItem {
            entity: Some(entity),
            name,
        } = &self.selected
        {
            let behavior_tree = some_or_return!(world.get::<BehaviorTree>(*entity));
            let mut node = BehaviorInspectorNode {
                entity: behavior_tree.root,
            };
            egui::Window::new(format!("Behavior: {}", name))
                .title_bar(true)
                .resizable(true)
                .collapsible(true)
                .show(ui.ctx(), |ui| {
                    changed |= node.ui(ui, BehaviorInspectorNodeAttributes::default(), context);
                });
        }

        // if let Some(root) = self.root {
        //     let world = unsafe { context.world_mut() };
        //     if let Some(world) = world {
        //         let child_nodes =
        //             world.query_filtered::<BehaviorChildQuery, BehaviorChildQueryFilter>();
        //     }
        // }

        // if let Some(root) = self.root {
        //     let world = context.world();
        //     if let Some(world) = world {

        //         let (_entity_location, components) = {
        //             let entity_ref = match world.get_entity(root) {
        //                 Some(entity_ref) => entity_ref,
        //                 None => {
        //                     ui.label("Entity does not exist");
        //                     return false;
        //                 }
        //             };
        //             let entity_location = entity_ref.location();
        //             let archetype = entity_ref.archetype();

        //             let table_components = archetype.table_components().into_iter();
        //             let sparse_components = archetype.sparse_set_components().into_iter();
        //             let components: Vec<ComponentId> = table_components.chain(sparse_components).cloned().collect();

        //             (entity_location, components)
        //         };

        //         let mut components: Vec<_> = components
        //         .iter()
        //         .map(|component_id| {
        //             let component_info = world.components().get_info(*component_id).unwrap();
        //             let name = pretty_type_name_str(component_info.name());
        //             (
        //                 name,
        //                 component_info.id(),
        //                 component_info.type_id(),
        //                 component_info.layout().size(),
        //             )
        //         })
        //         .collect();

        //         for (name, component_id, type_id, size) in components.iter() {
        //             ui.label(format!("{}: {:?} {:?} {}", name, component_id, type_id, size));
        //         }

        //         // let mut children = world.get::<Children>(root).unwrap().clone();
        //         // children.sort_by_key(|child| context.world.get::<Name>(child).unwrap().0.clone());

        //         // for child in children {
        //         //     let name = context.world.get::<Name>(child).unwrap().0.clone();
        //         //     let mut inspector = context.world.get_mut::<BehaviorInspector>(child).unwrap();
        //         //     changed |= inspector.ui(ui, options.clone(), context);
        //         // }

        //     }

        // }

        // changed |= self.component_kind_ui(
        //     ui,
        //     |archetype| archetype.table_components(),
        //     "Components",
        //     entity,
        //     params,
        //     id,
        // );

        // changed |= self.component_kind_ui(
        //     ui,
        //     |archetype| archetype.sparse_set_components(),
        //     "Components (Sparse)",
        //     entity,
        //     params,
        //     id,
        // );

        // ui.separator();

        changed
    }
}
