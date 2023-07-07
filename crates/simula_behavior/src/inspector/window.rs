use crate::{
    inspector::{
        graph::{
            BehaviorData, BehaviorDataType, BehaviorEditorState, BehaviorGraphState,
            BehaviorNodeTemplates, BehaviorResponse,
        },
        utils, BehaviorInspectable, BehaviorInspector, BehaviorInspectorState,
    },
    protocol::{BehaviorFileName, StartOption, StopOption},
    BehaviorFactory, BehaviorType,
};
use bevy::{prelude::*, window::PrimaryWindow};
use egui_node_graph::NodeResponse;
use simula_inspector::egui;

pub fn ui<T: BehaviorFactory + BehaviorInspectable>(
    context: &mut egui::Context,
    world: &mut World,
) {
    let elapsed = world.get_resource::<Time>().unwrap().elapsed();

    let selected_behavior = world
        .resource_mut::<BehaviorInspector<T>>()
        .selected
        .clone();
    let Some(selected_behavior) = selected_behavior else { return;};
    let behavior_inspector = world.resource_mut::<BehaviorInspector<T>>();
    let Some((file_name, inspector_item_state, entity))
        = behavior_inspector.behaviors
        .get(&selected_behavior)
        .and_then(|item| Some((item.name.clone(), item.state.clone(), item.entity))) else { return;};

    match inspector_item_state {
        BehaviorInspectorState::Editing => {}
        BehaviorInspectorState::Save => {}
        BehaviorInspectorState::Saving(_) => {}
        BehaviorInspectorState::Start => {}
        BehaviorInspectorState::Starting(_) => {}
        BehaviorInspectorState::Running => {}
        BehaviorInspectorState::Stop => {}
        BehaviorInspectorState::Stopping(_) => {}
        _ => return,
    }
    let Some(entity) = entity else { return;};

    let mut behavior_graphs = world.query::<(
        Entity,
        Option<&Name>,
        &mut BehaviorGraphState,
        &mut BehaviorEditorState<T>,
    )>();

    if let Ok((_, _, _graph_state, mut editor_state)) = behavior_graphs.get_mut(world, entity) {
        editor_state.editing = false;
        match inspector_item_state {
            BehaviorInspectorState::Editing => {
                editor_state.editing = true;
            }
            BehaviorInspectorState::Save => {}
            BehaviorInspectorState::Saving(_) => {}
            BehaviorInspectorState::Start => {}
            BehaviorInspectorState::Starting(_) => {}
            BehaviorInspectorState::Running => {}
            BehaviorInspectorState::Stop => {}
            BehaviorInspectorState::Stopping(_) => {}
            _ => return,
        }
    }

    let window = world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .single(world);
    let default_size = egui::vec2(window.width() * 0.7, window.height() * 0.7);

    let mut reset_graph_layout = false;

    let mut open = true;
    let mut window_name = format!("{}", *file_name);
    egui::Window::new(&format!("BHI:[{}]", *selected_behavior))
        .id(T::TYPE_UUID.to_string().into())
        .default_size(default_size)
        .title_bar(false)
        .resizable(true)
        .frame(
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_unmultiplied(22, 20, 25, 200))
                .inner_margin(3.0),
        )
        .show(context, |ui| {
            let mut pan_reset = false;
            let mut pan = egui::vec2(0.0, 0.0);
            context.input(|i| {
                pan = i.scroll_delta;
            });
            let mut pan_length = 0.0;
            if let Ok((_, _, _graph_state, editor_state)) = behavior_graphs.get(world, entity) {
                pan_length = editor_state.pan_zoom.pan.length_sq();
            }

            ui.vertical(|ui| {
                let mut behavior_inspector = world.resource_mut::<BehaviorInspector<T>>();
                let behavior_inspector_item = behavior_inspector
                    .behaviors
                    .get_mut(&selected_behavior)
                    .unwrap();

                // if this behavior item has been modified in any way
                // e.g. the graph has been edited, or renamed
                let mut modified = behavior_inspector_item.modified;

                ui.horizontal(|ui| {
                    egui::menu::bar(ui, |ui| {
                        if behavior_inspector_item.collapsed {
                            if ui.add(egui::Button::new("▶").frame(false)).clicked() {
                                behavior_inspector_item.collapsed = false;
                            }
                        } else {
                            if ui.add(egui::Button::new("▼").frame(false)).clicked() {
                                behavior_inspector_item.collapsed = true;
                            }
                        }

                        // add a save button
                        let mut save_enabled = true;
                        if !behavior_inspector_item.modified {
                            save_enabled = false;
                        }
                        if let BehaviorInspectorState::Saving(started) = inspector_item_state {
                            let elapsed = elapsed - started;
                            let waiting = if elapsed.as_millis() % 200 < 100 {
                                "⌛"
                            } else {
                                "⏳"
                            };
                            ui.label(waiting);
                        } else if ui
                            .add_enabled(save_enabled, egui::Button::new("💾"))
                            .clicked()
                        {
                            behavior_inspector_item.state = BehaviorInspectorState::Save;
                        }

                        // enable the center button if the pan is off centered
                        if ui
                            .add_enabled(pan_length > 1000.0, egui::Button::new("⨀").frame(true))
                            .clicked()
                        {
                            pan_reset = true;
                        }

                        // enable the center button if the pan is off centered
                        if ui
                            .add_enabled(true, egui::Button::new("📐").frame(true))
                            .clicked()
                        {
                            reset_graph_layout = true;
                        }

                        ui.add_space(20.0);

                        if let BehaviorInspectorState::Editing = inspector_item_state {
                            if ui.add(egui::Button::new("⏵").frame(true)).clicked() {
                                behavior_inspector_item.state = BehaviorInspectorState::Start;
                            }
                            egui::ComboBox::from_id_source("Behavior Inspector Item StartOption")
                                .width(250.0)
                                .selected_text(utils::get_label_from_start_option(
                                    &behavior_inspector_item.start_option,
                                ))
                                .show_ui(ui, |ui| {
                                    let mut selectables = vec![StartOption::Spawn];
                                    for instance in &behavior_inspector_item.instances {
                                        selectables.push(StartOption::Attach(instance.clone()));
                                    }
                                    for instance in &behavior_inspector_item.orphans {
                                        selectables.push(StartOption::Insert(instance.clone()));
                                    }
                                    for selectable in &selectables {
                                        if ui
                                            .selectable_label(
                                                selectable == &behavior_inspector_item.start_option,
                                                utils::get_label_from_start_option(selectable),
                                            )
                                            .clicked()
                                        {
                                            match selectable {
                                                StartOption::Spawn => {
                                                    behavior_inspector_item.stop_option =
                                                        StopOption::Despawn
                                                }
                                                StartOption::Attach(_) => {
                                                    behavior_inspector_item.stop_option =
                                                        StopOption::Detach
                                                }
                                                StartOption::Insert(_) => {
                                                    behavior_inspector_item.stop_option =
                                                        StopOption::Remove
                                                }
                                            }
                                            behavior_inspector_item.start_option =
                                                selectable.clone();
                                            behavior_inspector_item.state =
                                                BehaviorInspectorState::Start;
                                        }
                                    }
                                });
                        }

                        if let BehaviorInspectorState::Running = inspector_item_state {
                            if ui.add(egui::Button::new("⏹").frame(true)).clicked() {
                                behavior_inspector_item.state = BehaviorInspectorState::Stop;
                            }
                            egui::ComboBox::from_id_source("Behavior Inspector Item StopOption")
                                .width(250.0)
                                .selected_text(utils::get_label_from_stop_option(
                                    &behavior_inspector_item.start_option,
                                    &behavior_inspector_item.stop_option,
                                ))
                                .show_ui(ui, |ui| {
                                    let selectables = vec![
                                        StopOption::Despawn,
                                        StopOption::Detach,
                                        StopOption::Remove,
                                    ];
                                    for selectable in &selectables {
                                        if ui
                                            .selectable_label(
                                                selectable == &behavior_inspector_item.stop_option,
                                                utils::get_label_from_stop_option(
                                                    &behavior_inspector_item.start_option,
                                                    selectable,
                                                ),
                                            )
                                            .clicked()
                                        {
                                            behavior_inspector_item.stop_option =
                                                selectable.clone();
                                            behavior_inspector_item.state =
                                                BehaviorInspectorState::Stop;
                                        }
                                    }
                                });
                        }

                        ui.style_mut().visuals.extreme_bg_color =
                            egui::Color32::from_rgba_premultiplied(0, 0, 0, 100);
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut window_name)
                                    .desired_width(250.0)
                                    .clip_text(false),
                            )
                            .changed()
                        {
                            behavior_inspector_item.name =
                                BehaviorFileName(window_name.clone().into());
                            modified = true;
                        }

                        // Space for the little cross icon
                        ui.add_space(8.0);
                        ui.add_space(ui.available_width() - 12.0);
                        if utils::close_button(ui, ui.available_rect_before_wrap()).clicked() {
                            open = false;
                        }
                    });
                });

                if !behavior_inspector_item.collapsed {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgba_unmultiplied(42, 40, 45, 140))
                        .stroke(egui::Stroke::NONE)
                        .inner_margin(egui::Margin {
                            left: 10.0,
                            right: 10.0,
                            top: 10.0,
                            bottom: 10.0,
                        })
                        .outer_margin(egui::Margin {
                            left: -3.0,
                            right: -3.0,
                            top: 0.0,
                            bottom: -3.0,
                        })
                        .show(ui, |ui| {
                            let (mut graph_state, mut editor_state) =
                                if let Ok((_, _, graph_state, editor_state)) =
                                    behavior_graphs.get_mut(world, entity)
                                {
                                    (graph_state, editor_state)
                                } else {
                                    return;
                                };

                            // keep root node locked
                            if let Some(root_node) = graph_state.root_node {
                                let position =
                                    editor_state.node_positions.get_mut(root_node).unwrap();
                                *position = egui::pos2(0.0, 0.0);
                            } else {
                                for (nodeid, node) in &editor_state.graph.nodes {
                                    if let BehaviorData::Root = &node.user_data.data {
                                        graph_state.root_node = Some(nodeid);
                                        break;
                                    }
                                }
                            }

                            // handle pan
                            let scroll_rect = ui.available_rect_before_wrap();
                            if ui.rect_contains_pointer(scroll_rect) {
                                editor_state.pan_zoom.pan += pan;
                            }
                            if pan_reset {
                                editor_state.pan_zoom.pan = egui::vec2(0.0, 0.0);
                            }

                            // keep graph inside scroll rect
                            let mut clip_rect = ui.available_rect_before_wrap();
                            clip_rect.min.x -= 9.0;
                            clip_rect.max.x += 9.0;
                            clip_rect.min.y -= 9.0;
                            clip_rect.max.y += 9.0;
                            ui.set_clip_rect(clip_rect);

                            // draw node graph
                            let graph_response = ui
                                .push_id(T::TYPE_UUID, |ui| {
                                    editor_state.draw_graph_editor(
                                        ui,
                                        BehaviorNodeTemplates::<T>::default(),
                                        &mut graph_state,
                                        Vec::default(),
                                    )
                                })
                                .inner;

                            for response in graph_response.node_responses {
                                trace!("response: {:?}", response);
                                match response {
                                    NodeResponse::CreatedNode(_) => {
                                        modified = true;
                                    }
                                    NodeResponse::DeleteNodeFull {
                                        node_id: _node_id,
                                        node: _node,
                                    } => {
                                        modified = true;
                                    }
                                    NodeResponse::SelectNode(node_id) => {
                                        graph_state.active_node = Some(node_id);
                                    }
                                    NodeResponse::DeselectNode => {
                                        graph_state.active_node = None;
                                    }
                                    NodeResponse::ConnectEventEnded {
                                        output: output_id,
                                        input: input_id,
                                    } => {
                                        modified = true;

                                        // Check if output is already connected, and if so, remove the previous connection
                                        let mut removes = vec![];
                                        for (other_input, other_output) in
                                            editor_state.graph.connections.iter()
                                        {
                                            if *other_output == output_id {
                                                if other_input != input_id {
                                                    removes.push(other_input);
                                                }
                                            }
                                        }
                                        for other_input in removes {
                                            editor_state.graph.connections.remove(other_input);
                                        }

                                        // If composite type, dynamically adjust outputs of node
                                        let node_id = editor_state.graph.outputs[output_id].node;
                                        let node = editor_state.graph.nodes.get(node_id).unwrap();
                                        if let BehaviorData::Behavior(behavior) =
                                            &node.user_data.data
                                        {
                                            if behavior.typ() == BehaviorType::Composite {
                                                // Get all unused outputs
                                                let mut unused_outputs = vec![];
                                                node.outputs.iter().for_each(|(_, output_id)| {
                                                    let connected = editor_state
                                                        .graph
                                                        .connections
                                                        .iter()
                                                        .filter(|(_, other_output)| {
                                                            *other_output == output_id
                                                        })
                                                        .count()
                                                        > 0;
                                                    if !connected {
                                                        unused_outputs.push(*output_id);
                                                    }
                                                });

                                                // If there are no unused outputs, add a new output
                                                if unused_outputs.len() == 0 {
                                                    editor_state.graph.add_output_param(
                                                        node_id,
                                                        "B".into(),
                                                        BehaviorDataType::Flow,
                                                    );
                                                }

                                                // Remove all but one unused output
                                                while unused_outputs.len() > 1 {
                                                    if let Some(output_id) = unused_outputs.pop() {
                                                        editor_state
                                                            .graph
                                                            .remove_output_param(output_id);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    NodeResponse::DisconnectEvent {
                                        output: _output,
                                        input: _input,
                                    } => {
                                        modified = true;
                                    }
                                    NodeResponse::User(BehaviorResponse::NodeEdited(
                                        node_id,
                                        data,
                                    )) => {
                                        modified = true;
                                        if let Some(node) =
                                            editor_state.graph.nodes.get_mut(node_id)
                                        {
                                            node.user_data.data = BehaviorData::Behavior(data);
                                        }
                                    }
                                    NodeResponse::User(BehaviorResponse::NameEdited(
                                        node_id,
                                        name,
                                    )) => {
                                        modified = true;
                                        if let Some(node) =
                                            editor_state.graph.nodes.get_mut(node_id)
                                        {
                                            node.label = name;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        });
                }

                let mut behavior_inspector = world.resource_mut::<BehaviorInspector<T>>();
                let behavior_inspector_item = behavior_inspector
                    .behaviors
                    .get_mut(&selected_behavior)
                    .unwrap();
                behavior_inspector_item.modified = modified;
            });
        });

    if reset_graph_layout {
        if let Ok((_, _, _graph_state, mut editor_state)) = behavior_graphs.get_mut(world, entity) {
            let mut child = 0;
            utils::layout_graph(&mut editor_state, None, 0, &mut child);
        }
    }

    if !open {
        let mut behavior_inspector = world.resource_mut::<BehaviorInspector<T>>();
        behavior_inspector.selected = None;
    }
}
