use crate::{
    inspector::{
        graph::{
            BehaviorData, BehaviorDataType, BehaviorEditorState, BehaviorGraphState,
            BehaviorNodeData, BehaviorNodeTemplate, BehaviorValueType,
        },
        BehaviorInspectable, BehaviorInspector, BehaviorNodeInspectable,
    },
    protocol::{BehaviorFileId, BehaviorTelemetry, RemoteEntity, StartOption, StopOption},
    Behavior, BehaviorFactory, BehaviorType,
};
use bevy::prelude::*;
use egui_node_graph::{Graph, InputId, NodeId, NodeTemplateTrait, OutputId};
use simula_inspector::egui;
use std::borrow::Cow;

fn get_root_child<T: BehaviorFactory>(
    graph: &Graph<BehaviorNodeData<T>, BehaviorDataType, BehaviorValueType<T>>,
) -> Option<NodeId> {
    for node in graph.nodes.values() {
        if let BehaviorData::Root = &node.user_data.data {
            // All this just to get first child of root node
            let root_child_id: Option<NodeId> = node
                .outputs
                .iter()
                .filter_map(|(_, output_id)| {
                    graph
                        .connections
                        .iter()
                        .find(|(input_id, rhs_output_id)| {
                            output_id == *rhs_output_id
                                && graph.inputs[*input_id].typ == BehaviorDataType::Flow
                        })
                        .and_then(|(input_id, _)| Some(graph.inputs[input_id].node))
                })
                .next();
            return root_child_id;
        }
    }
    None
}

// Recursively build behavior from graph
pub fn graph_to_behavior<T>(
    editor: &BehaviorEditorState<T>,
    node_id: Option<NodeId>,
) -> Result<Behavior<T>, String>
where
    T: BehaviorFactory,
    <T as BehaviorFactory>::Attributes: BehaviorNodeInspectable<T>,
{
    let Some(node_id) = node_id else {
        let root_child_id = get_root_child(&editor.graph);
        if let Some(root_child_id) = root_child_id {
            return graph_to_behavior(editor, Some(root_child_id));
        } else {
            return Err("No root child".to_owned());
        }
    };

    let node: &egui_node_graph::Node<BehaviorNodeData<T>> = &editor.graph.nodes[node_id];
    let BehaviorData::Behavior(behavior) = &node.user_data.data else { return Err("Expected behavior node".to_owned()) };
    let mut attribs = <T as BehaviorFactory>::Attributes::default();
    editor
        .node_positions
        .get(node_id)
        .map(|pos| attribs.set_pos(Vec2::new(pos.x, pos.y)));
    let mut behavior = Behavior::new(
        node.label.to_owned(),
        behavior.clone(),
        attribs,
        Default::default(),
    );
    for (_, output_id) in node.outputs.iter() {
        let child_id = editor
            .graph
            .connections
            .iter()
            .find(|(input_id, rhs_output_id)| {
                output_id == *rhs_output_id
                    && editor.graph.inputs[*input_id].typ == BehaviorDataType::Flow
            })
            .and_then(|(input_id, _)| Some(editor.graph.inputs[input_id].node));
        if let Some(child_id) = child_id {
            if let Ok(child) = graph_to_behavior(editor, Some(child_id)) {
                behavior.nodes_mut().push(child);
            }
        }
    }

    Ok(behavior)
}

// Recursively update graph from behavior
pub fn behavior_to_graph<T>(
    editor: &mut BehaviorEditorState<T>,
    node_id: Option<NodeId>,
    behavior: &Behavior<T>,
) -> Result<(), String>
where
    T: BehaviorFactory,
    <T as BehaviorFactory>::Attributes: BehaviorNodeInspectable<T>,
{
    let Some(node_id) = node_id else {
        let root_child_id = get_root_child(&editor.graph);
        if let Some(root_child_id) = root_child_id {
            return behavior_to_graph(editor, Some(root_child_id), behavior);
        } else {
            return Err("No root child".to_owned());
        }
    };

    let graph = &mut editor.graph;

    // Update graph node with behavior data
    let node: &mut egui_node_graph::Node<BehaviorNodeData<T>> = &mut graph.nodes[node_id];
    node.user_data.data = BehaviorData::Behavior(behavior.data().clone());
    node.user_data.state = None;

    // Get node children
    let node_children: Vec<NodeId> = node
        .outputs
        .iter()
        .filter_map(|(_, output_id)| {
            graph
                .connections
                .iter()
                .find(|(input_id, rhs_output_id)| {
                    output_id == *rhs_output_id
                        && graph.inputs[*input_id].typ == BehaviorDataType::Flow
                })
                .and_then(|(input_id, _)| Some(graph.inputs[input_id].node))
        })
        .collect();

    // Zip and iterate over children
    let node_children = node_children.iter().cloned();
    let behavior_children = behavior.nodes().iter();
    for (node_child, behavior_child) in node_children.zip(behavior_children) {
        behavior_to_graph(editor, Some(node_child), behavior_child)?;
    }

    Ok(())
}

// Recursively create graph from behavior
pub fn behavior_into_graph<T>(
    editor: &mut BehaviorEditorState<T>,
    graph_state: &mut BehaviorGraphState,
    parent_node_id: NodeId,
    behavior: &Behavior<T>,
) where
    T: BehaviorFactory + BehaviorInspectable,
    <T as BehaviorFactory>::Attributes: BehaviorNodeInspectable<T>,
{
    // Create graph node with behavior data
    let behavior_data = BehaviorData::Behavior(behavior.data().clone());
    let behavior_template = BehaviorNodeTemplate::Behavior(behavior.data().clone());
    let node_data = BehaviorNodeData {
        data: behavior_data.clone(),
        state: None,
        entity: None,
    };
    let node_id = editor
        .graph
        .add_node(behavior.name().into(), node_data, |graph, node_id| {
            behavior_template.build_node(graph, graph_state, node_id)
        });
    let node_pos = behavior.attrs().get_pos();
    editor
        .node_positions
        .insert(node_id, egui::pos2(node_pos.x, node_pos.y));
    editor.node_order.push(node_id);

    // If parent node is a composite, add an extra output
    if let BehaviorData::Behavior(behavior) =
        editor.graph.nodes[parent_node_id].user_data.data.clone()
    {
        if let BehaviorType::Composite = behavior.typ() {
            editor
                .graph
                .add_output_param(parent_node_id, "".into(), BehaviorDataType::Flow);
        }
    }

    // Find available output from parent node
    let parent_node = &editor.graph.nodes[parent_node_id];
    let output_id: Option<OutputId> = parent_node.output_ids().find(|output_id| {
        editor
            .graph
            .connections
            .iter()
            .find(|(_, rhs_output_id)| output_id == *rhs_output_id)
            .is_none()
    });

    // Find input from node
    let node = &editor.graph.nodes[node_id];
    let input_id: Option<InputId> = node.input_ids().find(|input_id| {
        editor
            .graph
            .connections
            .iter()
            .find(|(lhs_input_id, _)| input_id == lhs_input_id)
            .is_none()
    });

    // Connect parent and child node
    if let (Some(output_id), Some(input_id)) = (output_id, input_id) {
        editor.graph.add_connection(output_id, input_id);
    } else {
        error!("Failed to connect {:?} to {:?}", parent_node_id, node_id);
    }

    // Recursively apply to children
    for child in behavior.nodes() {
        behavior_into_graph(editor, graph_state, node_id, child);
    }
}

// Recursively update graph from behavior telemetry
pub fn behavior_telemerty_to_graph<T>(
    graph: &mut Graph<BehaviorNodeData<T>, BehaviorDataType, BehaviorValueType<T>>,
    node_id: Option<NodeId>,
    telemetry: &BehaviorTelemetry<T>,
) -> Result<(), String>
where
    T: BehaviorFactory,
{
    let Some(node_id) = node_id else {
        let root_child_id = get_root_child(&graph);
        if let Some(root_child_id) = root_child_id {
            return behavior_telemerty_to_graph(graph, Some(root_child_id), telemetry);
        } else {
            return Err("No root child".to_owned());
        }
    };

    // Update graph node with behavior telemetry
    let node: &mut egui_node_graph::Node<BehaviorNodeData<T>> = &mut graph.nodes[node_id];
    if let BehaviorTelemetry(entity, state, Some(behavior), _) = telemetry {
        node.user_data.data = BehaviorData::Behavior(behavior.clone());
        node.user_data.state = Some(*state);
        node.user_data.entity = entity.clone();
    }

    // Get node children
    let node_children: Vec<NodeId> = node
        .outputs
        .iter()
        .filter_map(|(_, output_id)| {
            graph
                .connections
                .iter()
                .find(|(input_id, rhs_output_id)| {
                    output_id == *rhs_output_id
                        && graph.inputs[*input_id].typ == BehaviorDataType::Flow
                })
                .and_then(|(input_id, _)| Some(graph.inputs[input_id].node))
        })
        .collect();

    // children iterators
    let mut node_children = node_children.iter();
    let mut telemetry_children = telemetry.3.iter();

    let mut node_child = node_children.next();
    let mut behavior_child = telemetry_children.next();

    loop {
        match (node_child, behavior_child) {
            (Some(node_child), Some(behavior_child)) => {
                behavior_telemerty_to_graph(graph, Some(*node_child), behavior_child)?;
            }
            (Some(_node_child), None) => {
                warn!("More graph nodes then behaviors");
            }
            (None, Some(_behavior_child)) => {
                warn!("More behaviors then graph nodes");
            }
            (None, None) => {
                break;
            }
        }

        node_child = node_children.next();
        behavior_child = telemetry_children.next();
    }

    Ok(())
}

pub fn layout_graph<T>(
    editor: &mut BehaviorEditorState<T>,
    node_id: Option<NodeId>,
    depth: usize,
    child: &mut usize,
) where
    T: BehaviorFactory,
{
    // TODO: Make these dynamic
    const NODE_WIDTH: f32 = 200.0;
    const NODE_HEIGHT: f32 = 150.0;

    let Some(node_id) = node_id else {
            let root_child_id = get_root_child(&editor.graph);
            if let Some(root_child_id) = root_child_id {
                layout_graph(editor, Some(root_child_id), 1, child);
            } else {
                error!("No root child");
            }
            return;
        };

    editor.node_positions[node_id] =
        egui::pos2((depth as f32) * NODE_WIDTH, (*child as f32) * NODE_HEIGHT);

    // Get node children
    let graph = &mut editor.graph;
    let node: &mut egui_node_graph::Node<BehaviorNodeData<T>> = &mut graph.nodes[node_id];
    let node_children: Vec<NodeId> = node
        .outputs
        .iter()
        .filter_map(|(_, output_id)| {
            graph
                .connections
                .iter()
                .find(|(input_id, rhs_output_id)| {
                    output_id == *rhs_output_id
                        && graph.inputs[*input_id].typ == BehaviorDataType::Flow
                })
                .and_then(|(input_id, _)| Some(graph.inputs[input_id].node))
        })
        .collect();

    // Zip and iterate over children
    let node_children = node_children.iter().enumerate();
    for (idx, node_child) in node_children {
        if idx > 0 {
            *child = *child + 1;
        }
        layout_graph(editor, Some(*node_child), depth + 1, child);
    }
}

// For use with world.get_entity_component_reflect
fn _components_of_entity(
    world: &mut World,
    entity: Entity,
) -> Vec<(String, bevy::ecs::component::ComponentId, core::any::TypeId)> {
    let entity_ref = world.get_entity(entity).unwrap();
    let archetype = entity_ref.archetype();
    let mut components: Vec<_> = archetype
        .components()
        .filter_map(|component_id| {
            let info = world.components().get_info(component_id).unwrap();
            let name = pretty_type_name::pretty_type_name_str(info.name());
            Some((name, component_id, info.type_id().unwrap()))
        })
        .collect();
    components.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));
    components
}

pub fn close_button(ui: &mut egui::Ui, node_rect: egui::Rect) -> egui::Response {
    // Measurements
    let margin = 6.0;
    let size = 6.0;
    let stroke_width = 2.0;
    let offs = margin + size / 2.0;

    let position = egui::pos2(node_rect.right() - offs, node_rect.top() + offs);
    let rect = egui::Rect::from_center_size(position, egui::vec2(size, size));
    let resp = ui.allocate_rect(rect, egui::Sense::click());

    let color = if resp.clicked() {
        egui::Color32::WHITE
    } else if resp.hovered() {
        egui::Color32::GRAY
    } else {
        egui::Color32::DARK_GRAY
    };
    let stroke = egui::Stroke {
        width: stroke_width,
        color,
    };

    ui.painter()
        .line_segment([rect.left_top(), rect.right_bottom()], stroke);
    ui.painter()
        .line_segment([rect.right_top(), rect.left_bottom()], stroke);

    resp
}

pub(super) fn get_label_from_file_id<T: BehaviorFactory>(
    file_id: &Option<BehaviorFileId>,
    behavior_inspector: &BehaviorInspector<T>,
) -> Cow<'static, str> {
    match file_id {
        None => Cow::Borrowed("None"),
        Some(behavior_file_id) => {
            let behavior_inspector_item = &behavior_inspector.behaviors[behavior_file_id];
            (*behavior_inspector_item.name).clone()
        }
    }
}

pub(super) fn get_label_from_start_option(start_option: &StartOption) -> Cow<'static, str> {
    match start_option {
        StartOption::Spawn => Cow::Borrowed("Spawn"),
        StartOption::Attach(RemoteEntity { bits, name }) => {
            format!("Attach: [{}] {}", bits, name).into()
        }
        StartOption::Insert(RemoteEntity { bits, name }) => {
            format!("Insert: [{}] {}", bits, name).into()
        }
    }
}

pub(super) fn get_label_from_stop_option(
    start_option: &StartOption,
    stop_option: &StopOption,
) -> Cow<'static, str> {
    let current_label = match start_option {
        StartOption::Spawn => Cow::Borrowed(""),
        StartOption::Attach(RemoteEntity { bits, name }) => format!(": [{}] {}", bits, name).into(),
        StartOption::Insert(RemoteEntity { bits, name }) => format!(": [{}] {}", bits, name).into(),
    };
    match stop_option {
        StopOption::Despawn => format!("Despawn{}", current_label).into(),
        StopOption::Detach => format!("Detach{}", current_label).into(),
        StopOption::Remove => format!("Remove{}", current_label).into(),
    }
}
