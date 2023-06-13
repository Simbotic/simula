use crate::{
    inspector::graph::{
        BehaviorData, BehaviorDataType, BehaviorEditorState, BehaviorGraphState, BehaviorNodeData,
        BehaviorNodeTemplate, BehaviorNodeTemplates, BehaviorResponse,
    },
    protocol::{
        BehaviorClient, BehaviorFileId, BehaviorFileName, BehaviorProtocolClient,
        BehaviorProtocolServer, BehaviorServer, RemoteEntity,
    },
    Behavior, BehaviorFactory, BehaviorType,
};
use bevy::{prelude::*, utils::HashMap, window::PrimaryWindow};
use crossbeam_channel::unbounded;
use egui_node_graph::{NodeResponse, NodeTemplateTrait};
use serde::{Deserialize, Serialize};
use simula_inspector::{egui, Inspector, Inspectors};
use std::borrow::Cow;
use std::time::Duration;

pub mod graph;
pub mod utils;

#[derive(Default)]
pub struct BehaviorInspectorPlugin<T: BehaviorFactory>(pub std::marker::PhantomData<T>);

impl<T> Plugin for BehaviorInspectorPlugin<T>
where
    T: BehaviorFactory + Serialize + for<'de> Deserialize<'de>,
    <T as BehaviorFactory>::Attributes: BehaviorInspectable<T>,
{
    fn build(&self, app: &mut App) {
        // Setup bi-directional communication channels
        let (protocol_client_sender, protocol_client_receiver) = unbounded();
        let (protocol_server_sender, protocol_server_receiver) = unbounded();

        let client = BehaviorClient::<T> {
            sender: protocol_client_sender,
            receiver: protocol_server_receiver,
        };

        let server = BehaviorServer::<T> {
            sender: protocol_server_sender,
            receiver: protocol_client_receiver,
        };

        app.insert_resource(client)
            .insert_resource(server)
            .insert_resource(BehaviorInspector::<T>::default())
            .add_startup_system(setup::<T>)
            .add_system(update::<T>);
    }
}

pub trait BehaviorInspectable<T: BehaviorFactory> {
    fn get_pos(&self) -> Vec2;
    fn set_pos(&mut self, pos: Vec2);
}

#[derive(Clone)]
enum StartOption {
    Spawn,
    Attach(RemoteEntity),
}

#[derive(Clone)]
enum BehaviorInspectorState {
    New,
    Listing,
    Editing,
    Load,
    Loading(Duration),
    Save,
    Saving(Duration),
    Start(StartOption),
    Starting(StartOption, Duration),
    Running(StartOption),
    Stop(StartOption),
    Stopping(Duration),
}

#[derive(Clone)]
struct BehaviorInspectorItem<T: BehaviorFactory> {
    pub entity: Option<Entity>,
    pub name: BehaviorFileName,
    pub state: BehaviorInspectorState,
    pub collapsed: bool,
    pub behavior: Option<Behavior<T>>,
    pub instances: Vec<RemoteEntity>,
}

#[derive(Default, Clone, Resource)]
struct BehaviorInspector<T: BehaviorFactory> {
    pub selected: Option<BehaviorFileId>,
    pub behaviors: HashMap<BehaviorFileId, BehaviorInspectorItem<T>>,
}

fn get_label_from_id<T: BehaviorFactory>(
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

fn menu_ui<T: BehaviorFactory + Serialize + for<'de> Deserialize<'de>>(
    ui: &mut egui::Ui,
    world: &mut World,
) {
    egui::menu::menu_button(ui, "🏃 Behaviors", |ui| {
        if ui.add(egui::Button::new("✚ New")).clicked() {
            let file_id = BehaviorFileId::new();
            let file_name = BehaviorFileName(format!("bhts/u/bt_{}", *file_id).into());
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
                },
            );
            behavior_inspector.selected = Some(file_id.clone());
        }

        let mut behavior_inspector = world.resource_mut::<BehaviorInspector<T>>();
        if let Some(file_id) = behavior_inspector.selected.clone() {
            if let Some(behavior_inspector_item) = behavior_inspector.behaviors.get_mut(&file_id) {
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
    let mut refresh_instances = false;

    egui::ComboBox::from_id_source("Behavior Inspector Selector")
        .width(250.0)
        .selected_text(get_label_from_id(
            &behavior_inspector.selected,
            &behavior_inspector,
        ))
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
                let selectable_label = egui::SelectableLabel::new(
                    behavior_inspector.selected == selectable_behavior,
                    get_label_from_id(&selectable_behavior, &behavior_inspector),
                );
                if ui.add(selectable_label).clicked() {
                    info!("Selected: {:?}", selectable_behavior);
                    selected_behavior = selectable_behavior.clone();
                    refresh_instances = true;
                }
            }
        });

    // update selected behavior
    let mut behavior_inspector = world.resource_mut::<BehaviorInspector<T>>();
    behavior_inspector.selected = selected_behavior.clone();

    // if seleted belavior is only listed, load it
    if let Some(selected_behavior) = &selected_behavior {
        if let Some(behavior_inspector_item) =
            behavior_inspector.behaviors.get_mut(&selected_behavior)
        {
            if let BehaviorInspectorState::Listing = behavior_inspector_item.state {
                behavior_inspector_item.state = BehaviorInspectorState::Load;
            }
        }
    }

    if let Some(selected_behavior) = &selected_behavior {
        if refresh_instances {
            let behavior_client = world.get_resource::<BehaviorClient<T>>();
            if let Some(behavior_client) = behavior_client {
                behavior_client
                    .sender
                    .send(BehaviorProtocolClient::Instances(selected_behavior.clone()))
                    .unwrap();
            }
        }
    }
}

fn window_ui<T: BehaviorFactory>(context: &mut egui::Context, world: &mut World) {
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
        BehaviorInspectorState::Start(_) => {}
        BehaviorInspectorState::Starting(_, _) => {}
        BehaviorInspectorState::Running(_) => {}
        BehaviorInspectorState::Stop(_) => {}
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

    let window = world
        .query_filtered::<&Window, With<PrimaryWindow>>()
        .single(world);
    let default_size = egui::vec2(window.width() * 0.7, window.height() * 0.7);

    let mut open = true;
    let mut window_name = format!("{}", *file_name);
    egui::Window::new(&format!("BHI:[{}]", *selected_behavior))
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
                let inspector_item = behavior_inspector
                    .behaviors
                    .get_mut(&selected_behavior)
                    .unwrap();

                ui.horizontal(|ui| {
                    egui::menu::bar(ui, |ui| {
                        if inspector_item.collapsed {
                            if ui.add(egui::Button::new("▶").frame(false)).clicked() {
                                inspector_item.collapsed = false;
                            }
                        } else {
                            if ui.add(egui::Button::new("▼").frame(false)).clicked() {
                                inspector_item.collapsed = true;
                            }
                        }

                        if let BehaviorInspectorState::Saving(_) = inspector_item_state {
                            ui.add_enabled(false, egui::Label::new("💾"));
                        } else if let BehaviorInspectorState::Editing = inspector_item_state {
                            if ui.add(egui::Button::new("💾")).clicked() {
                                inspector_item.state = BehaviorInspectorState::Save;
                            }
                        }

                        if ui
                            .add_enabled(pan_length > 1000.0, egui::Button::new("⨀").frame(true))
                            .clicked()
                        {
                            pan_reset = true;
                        }

                        ui.add_space(20.0);

                        if let BehaviorInspectorState::Editing = inspector_item_state {
                            if ui.add(egui::Button::new("⏵").frame(true)).clicked() {
                                inspector_item.state =
                                    BehaviorInspectorState::Start(StartOption::Spawn);
                            }
                        }

                        if let BehaviorInspectorState::Running(options) = inspector_item_state {
                            if ui.add(egui::Button::new("⏹").frame(true)).clicked() {
                                inspector_item.state = BehaviorInspectorState::Stop(options);
                            }
                        }

                        ui.style_mut().visuals.extreme_bg_color =
                            egui::Color32::from_rgba_premultiplied(0, 0, 0, 100);
                        if ui
                            .add(
                                egui::TextEdit::singleline(&mut window_name)
                                    .desired_width(50.0)
                                    .clip_text(false),
                            )
                            .changed()
                        {
                            inspector_item.name = BehaviorFileName(window_name.clone().into());
                        }

                        // Space for the little cross icon
                        ui.add_space(8.0);
                        ui.add_space(ui.available_width() - 12.0);
                        if utils::close_button(ui, ui.available_rect_before_wrap()).clicked() {
                            open = false;
                        }
                    });
                });

                if !inspector_item.collapsed {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgba_unmultiplied(52, 50, 55, 140))
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

                            let graph_response = editor_state.draw_graph_editor(
                                ui,
                                BehaviorNodeTemplates::<T>::default(),
                                &mut graph_state,
                                Vec::default(),
                            );

                            for response in graph_response.node_responses {
                                trace!("response: {:?}", response);
                                match response {
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
                                    NodeResponse::User(BehaviorResponse::NodeEdited(
                                        node_id,
                                        data,
                                    )) => {
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
            });
        });

    if !open {
        let mut behavior_inspector = world.resource_mut::<BehaviorInspector<T>>();
        behavior_inspector.selected = None;
    }
}

fn setup<T>(mut inspectors: ResMut<Inspectors>)
where
    T: BehaviorFactory + Serialize + for<'de> Deserialize<'de>,
{
    inspectors.inspectors.push(Inspector {
        menu_ui: menu_ui::<T>,
        window_ui: window_ui::<T>,
    });
}

fn update<T>(
    mut commands: Commands,
    time: Res<Time>,
    type_registry: Res<AppTypeRegistry>,
    mut behavior_inspector: ResMut<BehaviorInspector<T>>,
    behavior_client: Res<BehaviorClient<T>>,
    mut graph_states: Query<&mut BehaviorGraphState>,
    mut editor_states: Query<&mut BehaviorEditorState<T>>,
) where
    T: BehaviorFactory + Serialize + for<'de> Deserialize<'de>,
    <T as BehaviorFactory>::Attributes: BehaviorInspectable<T>,
{
    // Get now
    let now = time.elapsed();

    // provide time for all graph states
    for mut graph_state in graph_states.iter_mut() {
        graph_state.time = time.clone();
    }

    for (file_id, behavior_inspector_item) in behavior_inspector.behaviors.iter_mut() {
        match behavior_inspector_item.state.clone() {
            // behavior is only listed, no need to do anything
            BehaviorInspectorState::Listing => {}
            // behavior is editing, no need to do anything
            BehaviorInspectorState::Editing => {}
            // If behavior item is Load, load it
            BehaviorInspectorState::Load => {
                info!("Loading behavior: {}", *behavior_inspector_item.name);
                behavior_inspector_item.state = BehaviorInspectorState::Loading(now);
                behavior_client
                    .sender
                    .send(BehaviorProtocolClient::LoadFile(file_id.clone()))
                    .unwrap();
            }
            // If behavior item is Loading, check to see if it timed out
            BehaviorInspectorState::Loading(started) => {
                if now - started > Duration::from_secs(5) {
                    warn!(
                        "Loading behavior timed out: {}",
                        *behavior_inspector_item.name
                    );
                    behavior_inspector_item.state = BehaviorInspectorState::Listing;
                }
            }
            // if behavior item is New, create it
            BehaviorInspectorState::New => {
                info!("Creating behavior: {}", *behavior_inspector_item.name);

                let mut graph_state = BehaviorGraphState {
                    type_registry: type_registry.0.clone(),
                    ..Default::default()
                };
                let mut editor_state = BehaviorEditorState::<T>::default();

                // create the only root node allowed
                let root_node_data = BehaviorNodeData {
                    data: BehaviorData::Root,
                    state: None,
                };
                let root_node =
                    editor_state
                        .graph
                        .add_node("Root".into(), root_node_data, |graph, node_id| {
                            BehaviorNodeTemplate::Root.build_node(graph, &mut graph_state, node_id)
                        });

                editor_state
                    .node_positions
                    .insert(root_node, egui::Pos2::new(0.0, 0.0));
                editor_state.node_order.push(root_node);

                let entity = commands
                    .spawn(Name::new(format!("BHI: {}", *behavior_inspector_item.name)))
                    .insert(graph_state)
                    .insert(editor_state)
                    .id();
                behavior_inspector_item.entity = Some(entity);
                behavior_inspector_item.state = BehaviorInspectorState::Editing;
            }
            // if behavior item is Save, save it
            BehaviorInspectorState::Save => {
                info!("Saving behavior: {}", *behavior_inspector_item.name);
                // set Editing in case anything goes wrong
                behavior_inspector_item.state = BehaviorInspectorState::Editing;
                // if we have an entity, we can save
                if let Some(entity) = behavior_inspector_item.entity {
                    if let Ok(editor_state) = editor_states.get(entity) {
                        let behavior = utils::graph_to_behavior(&editor_state, None);
                        if let Ok(behavior) = behavior {
                            debug!("behavior: {:#?}", behavior);
                            behavior_inspector_item.behavior = Some(behavior.clone());
                            behavior_inspector_item.state = BehaviorInspectorState::Saving(now);
                            behavior_client
                                .sender
                                .send(BehaviorProtocolClient::SaveFile(
                                    file_id.clone(),
                                    behavior_inspector_item.name.clone(),
                                    behavior,
                                ))
                                .unwrap();
                        } else if let Err(e) = behavior {
                            error!("{} for behavior: {}", e, *behavior_inspector_item.name);
                        }
                    } else {
                        error!(
                            "No editor state for behavior: {}",
                            *behavior_inspector_item.name
                        );
                    }
                }
                // if we don't have an entity, lets try to New this behavior
                else {
                    error!("No entity for behavior: {}", *behavior_inspector_item.name);
                    behavior_inspector_item.state = BehaviorInspectorState::New;
                }
            }
            // if behavior item is Saving, check to see if it timed out
            BehaviorInspectorState::Saving(started) => {
                if now - started > Duration::from_secs(5) {
                    warn!(
                        "Saving behavior timed out: {}",
                        *behavior_inspector_item.name
                    );
                    behavior_inspector_item.state = BehaviorInspectorState::Editing;
                }
            }
            // if behavior item should Start, start it
            BehaviorInspectorState::Start(options) => {
                info!("Run behavior: {}", *behavior_inspector_item.name);
                // set Editing in case anything goes wrong
                behavior_inspector_item.state = BehaviorInspectorState::Editing;
                // if we have an entity, we can run
                if let Some(entity) = behavior_inspector_item.entity {
                    if let Ok(editor_state) = editor_states.get(entity) {
                        let behavior = utils::graph_to_behavior(&editor_state, None);
                        if let Ok(behavior) = behavior {
                            debug!("behavior: {:#?}", behavior);
                            behavior_inspector_item.behavior = Some(behavior.clone());
                            behavior_inspector_item.state =
                                BehaviorInspectorState::Starting(options.clone(), now);
                            behavior_client
                                .sender
                                .send(BehaviorProtocolClient::Start(
                                    file_id.clone(),
                                    behavior_inspector_item.name.clone(),
                                    behavior,
                                ))
                                .unwrap();
                            behavior_client
                                .sender
                                .send(BehaviorProtocolClient::Telemetry(file_id.clone(), true))
                                .unwrap();
                        } else if let Err(e) = behavior {
                            error!("{} for behavior: {}", e, *behavior_inspector_item.name);
                        }
                    } else {
                        error!(
                            "No editor state for behavior: {}",
                            *behavior_inspector_item.name
                        );
                    }
                }
                // if we don't have an entity, lets try to New this behavior
                else {
                    error!("No entity for behavior: {}", *behavior_inspector_item.name);
                    behavior_inspector_item.state = BehaviorInspectorState::New;
                }
            }
            // if behavior item is Running, no need to do anything
            BehaviorInspectorState::Running(_) => {}
            // if behavior item is Starting, check to see if it timed out
            BehaviorInspectorState::Starting(_options, started) => {
                if now - started > Duration::from_secs(5) {
                    warn!(
                        "Starting behavior timed out: {}",
                        *behavior_inspector_item.name
                    );
                    behavior_inspector_item.state = BehaviorInspectorState::Editing;
                }
            }
            // if behavior item should Stop, stop it
            BehaviorInspectorState::Stop(options) => {
                info!("Stop behavior: {}", *behavior_inspector_item.name);
                // ask server to stop behavior only if it was spawned by inspector
                if let StartOption::Spawn = options {
                    behavior_inspector_item.state = BehaviorInspectorState::Stopping(now);
                    behavior_client
                        .sender
                        .send(BehaviorProtocolClient::Stop(file_id.clone()))
                        .unwrap();
                }
                // else, just detach
                else {
                    behavior_inspector_item.state = BehaviorInspectorState::Editing;
                }
            }
            // if behavior item is Stopping, check to see if it timed out
            BehaviorInspectorState::Stopping(started) => {
                if now - started > Duration::from_secs(5) {
                    warn!(
                        "Stopping behavior timed out: {}",
                        *behavior_inspector_item.name
                    );
                    behavior_inspector_item.state = BehaviorInspectorState::Editing;
                }
            }
        }
    }

    while let Ok(server_msg) = behavior_client.receiver.try_recv() {
        match server_msg {
            // Receive behavior file name
            BehaviorProtocolServer::FileName(file_id, file_name) => {
                info!("Received FileName: {:?} {}", file_id, *file_name);
                if !behavior_inspector.behaviors.contains_key(&file_id) {
                    behavior_inspector.behaviors.insert(
                        file_id.clone(),
                        BehaviorInspectorItem {
                            entity: None,
                            name: file_name.clone(),
                            state: BehaviorInspectorState::Listing,
                            collapsed: false,
                            behavior: None,
                            instances: vec![],
                        },
                    );
                }
            }
            // Receive instance running behavior
            BehaviorProtocolServer::Instances(file_id, remote_entities) => {
                info!("Received Instances: {:?}", file_id);
                if let Some(behavior_inspector_item) =
                    behavior_inspector.behaviors.get_mut(&file_id)
                {
                    debug!("{:?}", remote_entities);
                    behavior_inspector_item.instances = remote_entities;
                }
            }
            // Receive behavior data
            BehaviorProtocolServer::FileLoaded(file_id, behavior) => {
                info!("Received FileLoaded: {:?}", file_id);
                if let Some(behavior_inspector_item) =
                    behavior_inspector.behaviors.get_mut(&file_id)
                {
                    if let BehaviorInspectorState::Loading(_) = behavior_inspector_item.state {
                        info!("Loading behavior: {}", *behavior_inspector_item.name);

                        let mut graph_state = BehaviorGraphState {
                            type_registry: type_registry.0.clone(),
                            ..Default::default()
                        };
                        let mut editor_state = BehaviorEditorState::<T>::default();

                        // create the only root node allowed
                        let root_node_data = BehaviorNodeData {
                            data: BehaviorData::Root,
                            state: None,
                        };
                        let root_node = editor_state.graph.add_node(
                            "Root".into(),
                            root_node_data,
                            |graph, node_id| {
                                BehaviorNodeTemplate::Root.build_node(
                                    graph,
                                    &mut graph_state,
                                    node_id,
                                )
                            },
                        );

                        editor_state
                            .node_positions
                            .insert(root_node, egui::Pos2::new(0.0, 0.0));
                        editor_state.node_order.push(root_node);

                        utils::behavior_into_graph(
                            &mut editor_state,
                            &mut graph_state,
                            root_node,
                            &behavior,
                        );

                        let entity = commands
                            .spawn(Name::new(format!("BHI: {}", *behavior_inspector_item.name)))
                            .insert(graph_state)
                            .insert(editor_state)
                            .id();
                        behavior_inspector_item.entity = Some(entity);
                        behavior_inspector_item.state = BehaviorInspectorState::Editing;
                    }
                }
            }
            // Receive file saved
            BehaviorProtocolServer::FileSaved(file_id) => {
                info!("Received FileSaved: {:?}", file_id);
                if let Some(behavior_inspector_item) =
                    behavior_inspector.behaviors.get_mut(&file_id)
                {
                    if let BehaviorInspectorState::Saving(_) = behavior_inspector_item.state {
                        behavior_inspector_item.state = BehaviorInspectorState::Editing;
                    }
                } else {
                    error!("Unexpected file saved: {:?}", file_id);
                }
            }
            // Behavior started
            BehaviorProtocolServer::Started(file_id) => {
                info!("Received Started: {:?}", file_id);
                if let Some(behavior_inspector_item) =
                    behavior_inspector.behaviors.get_mut(&file_id)
                {
                    if let BehaviorInspectorState::Starting(options, _) =
                        &behavior_inspector_item.state
                    {
                        behavior_inspector_item.state =
                            BehaviorInspectorState::Running(options.clone());
                    }
                } else {
                    error!("Unexpected behavior started: {:?}", file_id);
                }
            }
            // Behavior stopped
            BehaviorProtocolServer::Stopped(file_id) => {
                info!("Received Stopped: {:?}", file_id);
                if let Some(behavior_inspector_item) =
                    behavior_inspector.behaviors.get_mut(&file_id)
                {
                    if let Some(behavior) = behavior_inspector_item.behavior.clone() {
                        if let Some(entity) = behavior_inspector_item.entity {
                            if let Ok(mut editor_state) = editor_states.get_mut(entity) {
                                if let Err(e) =
                                    utils::behavior_to_graph(&mut editor_state, None, &behavior)
                                {
                                    error!("Failed to restore behavior: {}", e);
                                }
                            }
                        }
                    }
                    if let BehaviorInspectorState::Stopping(_) = behavior_inspector_item.state {
                        behavior_inspector_item.state = BehaviorInspectorState::Editing;
                    }
                } else {
                    error!("Unexpected behavior stopped: {:?}", file_id);
                }
            }
            // Behavior telemetry
            BehaviorProtocolServer::Telemetry(file_id, telemetry) => {
                trace!("Received Telemetry: {:#?}", telemetry);
                if let Some(behavior_inspector_item) =
                    behavior_inspector.behaviors.get_mut(&file_id)
                {
                    if let BehaviorInspectorState::Running(_) = behavior_inspector_item.state {
                        if let Some(entity) = behavior_inspector_item.entity {
                            if let Ok(mut editor_state) = editor_states.get_mut(entity) {
                                if let Err(e) = utils::behavior_telemerty_to_graph(
                                    &mut editor_state.graph,
                                    None,
                                    &telemetry,
                                ) {
                                    error!("Failed to apply telemetry: {}", e);
                                }
                            }
                        }
                    }
                } else {
                    error!("Unexpected behavior telemetry: {:?}", file_id);
                }
            }
        }
    }
}
