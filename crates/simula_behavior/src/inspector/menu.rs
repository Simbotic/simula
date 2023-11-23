use crate::{
    inspector::{utils, BehaviorInspector, BehaviorInspectorItem, BehaviorInspectorState},
    protocol::{
        BehaviorClient, BehaviorFileId, BehaviorFileName, BehaviorProtocolClient, StartOption,
        StopOption,
    },
    BehaviorFactory,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_inspector::egui;

pub fn ui<T: BehaviorFactory + Serialize + for<'de> Deserialize<'de>>(
    ui: &mut egui::Ui,
    world: &mut World,
) {
    ui.push_id(T::TYPE_UUID, |ui| {
        let elapsed = world.get_resource::<Time>().unwrap().elapsed();

        let mut refresh_instances = false;
        let mut refresh_orphans = false;

        let behavior_type_name = pretty_type_name::pretty_type_name::<T>();
        egui::menu::menu_button(ui, format!("🏃 {}", behavior_type_name), |ui| {
            if ui.add(egui::Button::new("✚ New")).clicked() {
                let file_id = BehaviorFileId::new();
                let file_name = BehaviorFileName(format!("bht/u/bt_{}", *file_id).into());
                let mut behavior_inspector = world.resource_mut::<BehaviorInspector<T>>();
                behavior_inspector.behaviors.insert(
                    file_id.clone(),
                    BehaviorInspectorItem {
                        entity: None,
                        name: file_name,
                        state: BehaviorInspectorState::New,
                        collapsed: false,
                        behavior: None,
                        instances: vec![],
                        orphans: vec![],
                        start_option: StartOption::Spawn,
                        stop_option: StopOption::Despawn,
                        modified: true,
                    },
                );
                behavior_inspector.selected = Some(file_id.clone());
                refresh_orphans = true;
            }

            let mut behavior_inspector = world.resource_mut::<BehaviorInspector<T>>();
            if let Some(file_id) = behavior_inspector.selected.clone() {
                if let Some(behavior_inspector_item) =
                    behavior_inspector.behaviors.get_mut(&file_id)
                {
                    if let BehaviorInspectorState::Saving(_) = behavior_inspector_item.state {
                    } else {
                        if ui.add(egui::Button::new("💾 Save")).clicked() {
                            behavior_inspector_item.state = BehaviorInspectorState::Save;
                            warn!("Saving behavior {:?}", file_id);
                        }
                    }
                }
            }
        });

        let behavior_inspector = world.resource_mut::<BehaviorInspector<T>>();
        let mut selected_behavior = behavior_inspector.selected.clone();

        let mut selected_label_prefix = "".to_string();
        if let Some(behavior_inspector_item) = selected_behavior
            .as_ref()
            .and_then(|behavior| behavior_inspector.behaviors.get(behavior))
        {
            if let BehaviorInspectorState::Loading(started) = behavior_inspector_item.state {
                let elapsed = elapsed - started;
                let choose = if elapsed.as_millis() % 200 < 100 {
                    "⌛"
                } else {
                    "⏳"
                };
                selected_label_prefix = format!("{} ", choose);
            }
        }
        let selected_label = format!(
            "{}{}",
            selected_label_prefix,
            utils::get_label_from_file_id(&selected_behavior, &behavior_inspector)
        );

        egui::ComboBox::from_id_source("Behavior Selector")
            .width(250.0)
            .selected_text(selected_label)
            .show_ui(ui, |ui| {
                let mut selectable_behaviors: Vec<Option<BehaviorFileId>> = {
                    let mut keys: Vec<BehaviorFileId> =
                        behavior_inspector.behaviors.keys().cloned().collect();
                    keys.sort_by(|a, b| {
                        let name_a = &behavior_inspector.behaviors[a].name;
                        let name_b = &behavior_inspector.behaviors[b].name;
                        name_a.cmp(&name_b)
                    });
                    keys.iter().map(|key| Some(key.clone())).collect()
                };
                selectable_behaviors.insert(0, None);
                for selectable_behavior in selectable_behaviors {
                    let label =
                        utils::get_label_from_file_id(&selectable_behavior, &behavior_inspector);
                    let selectable_label = egui::SelectableLabel::new(
                        behavior_inspector.selected == selectable_behavior,
                        label,
                    );
                    if ui.add(selectable_label).clicked() {
                        info!("Selected: {:?}", selectable_behavior);
                        selected_behavior = selectable_behavior.clone();
                        refresh_instances = true;
                        refresh_orphans = true;
                    }
                }
            });
        // update selected behavior
        let mut behavior_inspector = world.resource_mut::<BehaviorInspector<T>>();
        behavior_inspector.selected = selected_behavior.clone();

        // if seleted belavior is only listed, load it
        if let Some(selected_behavior) = &selected_behavior {
            if let Some(behavior_inspector_item) =
                behavior_inspector.behaviors.get_mut(selected_behavior)
            {
                if let BehaviorInspectorState::Listing = behavior_inspector_item.state {
                    behavior_inspector_item.state = BehaviorInspectorState::Load;
                }
            }
        }

        if let Some(selected_behavior) = &selected_behavior {
            if refresh_instances || refresh_orphans {
                let behavior_client = world.get_resource::<BehaviorClient<T>>();
                if let Some(behavior_client) = behavior_client {
                    if refresh_instances {
                        behavior_client
                            .sender
                            .send(BehaviorProtocolClient::Instances(selected_behavior.clone()))
                            .unwrap();
                    }
                    if refresh_orphans {
                        behavior_client
                            .sender
                            .send(BehaviorProtocolClient::Orphans(selected_behavior.clone()))
                            .unwrap();
                    }
                }
            }
        }
    });
}
