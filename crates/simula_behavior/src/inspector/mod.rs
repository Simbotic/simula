use crate::{
    inspector::graph::{
        BehaviorData, BehaviorEditorState, BehaviorGraphState, BehaviorNodeData,
        BehaviorNodeTemplate,
    },
    protocol::{
        BehaviorClient, BehaviorFileId, BehaviorFileName, BehaviorProtocolClient,
        BehaviorProtocolServer, BehaviorServer, RemoteEntity, StartOption, StopOption,
    },
    Behavior, BehaviorFactory,
};
use bevy::{prelude::*, utils::HashMap};
use crossbeam_channel::unbounded;
use egui_node_graph::NodeTemplateTrait;
use serde::{Deserialize, Serialize};
use simula_inspector::{egui, Inspector, Inspectors};
use std::time::Duration;

pub mod graph;
mod menu;
mod utils;
mod window;

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

#[derive(Clone, Copy, PartialEq)]
pub(self) enum BehaviorInspectorState {
    New,
    Listing,
    Editing,
    Load,
    Loading(Duration),
    Save,
    Saving(Duration),
    Start,
    Starting(Duration),
    Running,
    Stop,
    Stopping(Duration),
}

#[derive(Clone)]
pub(self) struct BehaviorInspectorItem<T: BehaviorFactory> {
    pub entity: Option<Entity>,
    pub name: BehaviorFileName,
    pub state: BehaviorInspectorState,
    pub collapsed: bool,
    pub behavior: Option<Behavior<T>>,
    pub instances: Vec<RemoteEntity>,
    pub orphans: Vec<RemoteEntity>,
    pub start_option: StartOption,
    pub stop_option: StopOption,
    pub modified: bool,
}

#[derive(Default, Clone, Resource)]
pub(self) struct BehaviorInspector<T: BehaviorFactory> {
    pub selected: Option<BehaviorFileId>,
    pub behaviors: HashMap<BehaviorFileId, BehaviorInspectorItem<T>>,
}

fn setup<T>(mut inspectors: ResMut<Inspectors>)
where
    T: BehaviorFactory + Serialize + for<'de> Deserialize<'de>,
{
    inspectors.inspectors.push(Inspector {
        menu_ui: menu::ui::<T>,
        window_ui: window::ui::<T>,
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
        match &behavior_inspector_item.state {
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
                if now - *started > Duration::from_secs(5) {
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
                if now - *started > Duration::from_secs(5) {
                    warn!(
                        "Saving behavior timed out: {}",
                        *behavior_inspector_item.name
                    );
                    behavior_inspector_item.state = BehaviorInspectorState::Editing;
                }
            }
            // if behavior item should Start, start it
            BehaviorInspectorState::Start => {
                info!("Run behavior: {}", *behavior_inspector_item.name);
                // set Editing in case anything goes wrong
                behavior_inspector_item.state = BehaviorInspectorState::Editing;
                // if we have an entity, we can run
                if let Some(entity) = behavior_inspector_item.entity {
                    if let Ok(editor_state) = editor_states.get(entity) {
                        let mut behavior_option = None;
                        let mut error = false;
                        // only send a copy of behavior if it has been modified,
                        // is StartOption::Spawn or StartOption::Insert
                        let send_behavior = match behavior_inspector_item.start_option {
                            StartOption::Spawn => false,
                            StartOption::Attach(_) => false,
                            StartOption::Insert(_) => false,
                        };
                        if behavior_inspector_item.modified || send_behavior {
                            let behavior = utils::graph_to_behavior(&editor_state, None);
                            if let Ok(behavior) = behavior {
                                behavior_option = Some(behavior.clone());
                                debug!("behavior: {:#?}", behavior);
                                behavior_inspector_item.behavior = Some(behavior.clone());
                            } else if let Err(e) = behavior {
                                error = true;
                                error!("{} for behavior: {}", e, *behavior_inspector_item.name);
                            }
                        }
                        if !error {
                            behavior_inspector_item.state = BehaviorInspectorState::Starting(now);
                            behavior_client
                                .sender
                                .send(BehaviorProtocolClient::Start(
                                    file_id.clone(),
                                    behavior_inspector_item.name.clone(),
                                    behavior_inspector_item.start_option.clone(),
                                    behavior_option,
                                ))
                                .unwrap();
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
            BehaviorInspectorState::Running => {}
            // if behavior item is Starting, check to see if it timed out
            BehaviorInspectorState::Starting(started) => {
                if now - *started > Duration::from_secs(5) {
                    warn!(
                        "Starting behavior timed out: {}",
                        *behavior_inspector_item.name
                    );
                    behavior_inspector_item.state = BehaviorInspectorState::Editing;
                }
            }
            // if behavior item should Stop, stop it
            BehaviorInspectorState::Stop => {
                info!("Stop behavior: {}", *behavior_inspector_item.name);
                behavior_inspector_item.state = BehaviorInspectorState::Stopping(now);
                behavior_client
                    .sender
                    .send(BehaviorProtocolClient::Stop(
                        file_id.clone(),
                        behavior_inspector_item.stop_option.clone(),
                    ))
                    .unwrap();
            }
            // if behavior item is Stopping, check to see if it timed out
            BehaviorInspectorState::Stopping(started) => {
                if now - *started > Duration::from_secs(5) {
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
                            orphans: vec![],
                            start_option: StartOption::Spawn,
                            stop_option: StopOption::Despawn,
                            modified: false,
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
                    info!("{:?}", remote_entities);
                    behavior_inspector_item.instances = remote_entities;
                    behavior_inspector_item.start_option = StartOption::Spawn;
                    behavior_inspector_item.stop_option = StopOption::Despawn;
                }
            }
            // Receive orphans without behaviors
            BehaviorProtocolServer::Orphans(file_id, remote_entities) => {
                info!("Received Instances: {:?}", file_id);
                if let Some(behavior_inspector_item) =
                    behavior_inspector.behaviors.get_mut(&file_id)
                {
                    info!("{:?}", remote_entities);
                    behavior_inspector_item.orphans = remote_entities;
                    behavior_inspector_item.start_option = StartOption::Spawn;
                    behavior_inspector_item.stop_option = StopOption::Despawn;
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

                        behavior_inspector_item.behavior = Some(behavior.clone());
                        behavior_inspector_item.modified = false;

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
                    if let BehaviorInspectorState::Starting(_) = &behavior_inspector_item.state {
                        behavior_inspector_item.state = BehaviorInspectorState::Running;
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
                    if let BehaviorInspectorState::Running = behavior_inspector_item.state {
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
