use crate::{
    inspector::{BehaviorInspectorNode, BehaviorInspectorNodeAttributes},
    BehaviorTree,
};
use bevy::prelude::*;
use bevy_inspector_egui::{egui, Context, Inspectable};

#[derive(Default, Clone)]
pub struct BehaviorInspectorAttributes;

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
                .scroll2([true, true])
                .show(ui.ctx(), |ui| {
                    changed |= node.ui(ui, BehaviorInspectorNodeAttributes::default(), context);
                });
        }

        changed
    }
}
