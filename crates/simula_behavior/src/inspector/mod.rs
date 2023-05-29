use crate::{
    inspector::graph::{
        AllMyNodeTemplates, MyDataType, MyEditorState, MyGraphState, MyNodeTemplate, MyResponse,
    },
    protocol::{
        BehaviorClient, BehaviorFileId, BehaviorFileName, BehaviorProtocolClient,
        BehaviorProtocolServer, BehaviorServer,
    },
    BehaviorFactory, BehaviorType,
};
use bevy::{prelude::*, utils::HashMap};
use crossbeam_channel::unbounded;
use egui_node_graph::NodeResponse;
use serde::{Deserialize, Serialize};
use simula_inspector::{egui, Inspector, Inspectors};

pub mod graph;

#[derive(Default)]
pub struct BehaviorInspectorPlugin<T: BehaviorFactory>(pub std::marker::PhantomData<T>);

impl<T> Plugin for BehaviorInspectorPlugin<T>
where
    T: BehaviorFactory + Serialize + for<'de> Deserialize<'de>,
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
            .insert_resource(BehaviorInspector::default())
            .add_startup_system(setup::<T>)
            .add_system(update::<T>);
    }
}

#[derive(Reflect, FromReflect, Clone)]
enum BehaviorInspectorState {
    New,
    Unloaded,
    Loading,
    Loaded(Entity),
}

#[derive(Reflect, FromReflect, Clone)]
struct BehaviorInspectorItem {
    pub id: BehaviorFileId,
    pub name: BehaviorFileName,
    pub state: BehaviorInspectorState,
}

#[derive(Default, Clone, Resource, Reflect)]
struct BehaviorInspector {
    pub selected: Option<BehaviorFileId>,
    pub behaviors: HashMap<BehaviorFileId, BehaviorInspectorItem>,
}

fn get_label_from_id(
    file_id: &Option<BehaviorFileId>,
    behavior_inspector: &BehaviorInspector,
) -> String {
    match file_id {
        None => "None".to_string(),
        Some(behavior_file_id) => {
            let behavior_inspector_item = &behavior_inspector.behaviors[behavior_file_id];
            let name = behavior_inspector_item.name.clone();
            (*name).clone()
        }
    }
}

fn menu_ui<T: BehaviorFactory + Serialize + for<'de> Deserialize<'de>>(
    ui: &mut egui::Ui,
    world: &mut World,
) {
    egui::menu::menu_button(ui, "🏃 Behaviors", |ui| {
        if ui.add(egui::Button::new("New")).clicked() {
            let file_id = BehaviorFileId::new();
            let file_name = BehaviorFileName(format!("bt_{}", *file_id).into());
            let mut behavior_inspector = world.resource_mut::<BehaviorInspector>();
            behavior_inspector.behaviors.insert(
                file_id.clone(),
                BehaviorInspectorItem {
                    id: file_id.clone(),
                    name: file_name,
                    state: BehaviorInspectorState::New,
                },
            );
            behavior_inspector.selected = Some(file_id.clone());
        };
    });

    let behavior_inspector = world.resource_mut::<BehaviorInspector>();
    let mut new_selected = behavior_inspector.selected.clone();

    egui::ComboBox::from_id_source("Behavior Inspector Selector")
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
                if ui
                    .selectable_label(
                        behavior_inspector.selected == selectable_behavior,
                        get_label_from_id(&selectable_behavior, &behavior_inspector),
                    )
                    .clicked()
                {
                    println!("Selected: {:?}", selectable_behavior);
                    new_selected = selectable_behavior.clone();
                }
            }
        });

    let mut behavior_inspector = world.resource_mut::<BehaviorInspector>();
    behavior_inspector.selected = new_selected;
}

fn window_ui<T: BehaviorFactory>(context: &mut egui::Context, world: &mut World) {
    let behavior_inspector = world.resource_mut::<BehaviorInspector>().clone();

    let Some(selected_behavior) = behavior_inspector.selected else { return;};

    if let Some(behavior_inspector_item) = behavior_inspector.behaviors.get(&selected_behavior) {
        if let BehaviorInspectorState::Loaded(entity) = behavior_inspector_item.state {
            let mut behavior_graphs = world.query::<(
                Entity,
                Option<&Name>,
                &mut MyGraphState,
                &mut MyEditorState<T>,
            )>();

            egui::Window::new(format!("Behavior: {}", *selected_behavior))
                .title_bar(true)
                .resizable(true)
                .collapsible(true)
                .scroll2([true, true])
                .show(context, |ui| {
                    let Ok((_, _, mut graph_state, mut editor_state)) = behavior_graphs.get_mut(world, entity) else {
                        return;};

                        let graph_response = editor_state.draw_graph_editor(
                                            ui,
                                            AllMyNodeTemplates::<T>::default(),
                                            &mut graph_state,
                                            Vec::default(),
                                        );

                        for response in graph_response.node_responses {
                            println!("response: {:?}", response);
                            match response {
                                NodeResponse::ConnectEventEnded { output: output_id, input: input_id } => {
                                    // Check if output is already connected, and if so, remove the previous connection
                                    let mut removes = vec![];
                                    for (other_input, other_output) in editor_state.graph.connections.iter() {
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
                                    if let MyNodeTemplate::Behavior(behavior) = &node.user_data.data {
                                        if behavior.typ() == BehaviorType::Composite {
                                            // Get all unused outputs
                                            let mut unused_outputs = vec![];
                                            node.outputs.iter().for_each(|(_, output_id)| {
                                                let connected = editor_state.graph.connections.iter().filter(|(_, other_output)| {
                                                    println!("Output: {:#?} == {:#?}", output_id, *other_output);
                                                    *other_output == output_id
                                                }).count() > 0;
                                                if !connected {
                                                    unused_outputs.push(*output_id);
                                                }
                                            });

                                            // If there are no unused outputs, add a new output
                                            if unused_outputs.len() == 0 {
                                                editor_state.graph.add_output_param(node_id, "B".into(), MyDataType::Flow);
                                            }

                                            // Remove all but one unused output
                                            while unused_outputs.len() > 1 {
                                                if let Some(output_id) = unused_outputs.pop() {
                                                    editor_state.graph.remove_output_param(output_id);
                                                }
                                            }
                                        }
                                    }
                                }
                                NodeResponse::User(MyResponse::NodeEdited(node_id, data)) => {
                                    if let Some(node) = editor_state.graph.nodes.get_mut(node_id) {
                                        node.user_data.data = MyNodeTemplate::Behavior(data);
                                    }
                                }
                                NodeResponse::User(MyResponse::NameEdited(node_id, name)) => {
                                    if let Some(node) = editor_state.graph.nodes.get_mut(node_id) {
                                        node.user_data.name = name;
                                    }
                                }
                                _ => {}
                            }
                        }
                });
        }
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
    type_registry: Res<AppTypeRegistry>,
    mut behavior_inspector: ResMut<BehaviorInspector>,
    behavior_client: Res<BehaviorClient<T>>,
) where
    T: BehaviorFactory + Serialize + for<'de> Deserialize<'de>,
{
    if let Some(selected_behavior) = behavior_inspector.selected.clone() {
        if let Some(behavior_inspector_item) =
            behavior_inspector.behaviors.get_mut(&selected_behavior)
        {
            // If selected behavior is unloaded, load it
            if let BehaviorInspectorState::Unloaded = behavior_inspector_item.state {
                info!("Loading behavior: {}", *selected_behavior);
                behavior_inspector_item.state = BehaviorInspectorState::Loading;
                behavior_client
                    .sender
                    .send(BehaviorProtocolClient::LoadFile(selected_behavior))
                    .unwrap();
            }
        }
    }
    if let Ok(server_msg) = behavior_client.receiver.try_recv() {
        match server_msg {
            // Receive list of behaviors
            BehaviorProtocolServer::BehaviorFileNames(behaviors) => {
                for (file_id, file_name) in &behaviors {
                    if !behavior_inspector.behaviors.contains_key(&file_id) {
                        behavior_inspector.behaviors.insert(
                            file_id.clone(),
                            BehaviorInspectorItem {
                                id: file_id.clone(),
                                name: file_name.clone(),
                                state: BehaviorInspectorState::Unloaded,
                            },
                        );
                    }
                }
            }
            // Receive behavior data
            BehaviorProtocolServer::BehaviorFile((file_id, file_data)) => {
                if let Some(behavior_inspector_item) =
                    behavior_inspector.behaviors.get_mut(&file_id)
                {
                    if let BehaviorInspectorState::Loading = behavior_inspector_item.state {
                        let entity = commands
                            .spawn(Name::new(format!("BT: {}", *file_id)))
                            .insert(MyGraphState {
                                type_registry: type_registry.0.clone(),
                                ..Default::default()
                            })
                            .insert(ron::de::from_str::<MyEditorState<T>>(&file_data).unwrap())
                            .id();
                        behavior_inspector_item.state = BehaviorInspectorState::Loaded(entity);
                    }
                }
            }
            _ => {
                panic!("Unexpected message from server");
            }
        }
    }
}
