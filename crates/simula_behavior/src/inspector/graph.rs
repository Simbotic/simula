use crate::{prelude::*, protocol::BehaviorState};
use bevy::{
    prelude::{default, Color, Component, Deref, DerefMut, Time},
    reflect::TypeRegistryArc,
};
use bevy_inspector_egui::{
    egui::{self, Widget},
    reflect_inspector,
};
use egui_node_graph::{
    AnyParameterId, DataTypeTrait, Graph, GraphEditorState, InputParamKind, NodeDataTrait, NodeId,
    NodeResponse, NodeTemplateIter, NodeTemplateTrait, UserResponseTrait, WidgetValueTrait,
};
use serde::{Deserialize, Serialize};
use simula_core::signal::{SignalFunction, SignalGenerator};
use std::borrow::Cow;

/// The NodeData holds a custom data struct inside each node. It's useful to
/// store additional information that doesn't live in parameters. For this
/// example, the node data stores the template (i.e. the "type") of the node.
#[derive(Deserialize, Serialize, Debug)]
pub struct BehaviorNodeData<T: BehaviorFactory> {
    pub data: BehaviorNodeTemplate<T>,
    #[serde(skip)]
    pub state: Option<BehaviorState>,
}

/// `DataType`s are what defines the possible range of connections when
/// attaching two ports together. The graph UI will make sure to not allow
/// attaching incompatible datatypes.
#[derive(PartialEq, Eq, Deserialize, Serialize)]
pub enum BehaviorDataType {
    Flow,
}

/// In the graph, input parameters can optionally have a constant value. This
/// value can be directly edited in a widget inside the node itself.
///
/// There will usually be a correspondence between DataTypes and ValueTypes. But
/// this library makes no attempt to check this consistency. For instance, it is
/// up to the user code in this example to make sure no parameter is created
/// with a DataType of Scalar and a ValueType of Vec2.
#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum BehaviorValueType<T> {
    // Data(T),
    Flow,
    Marker {
        _marker: std::marker::PhantomData<T>,
    },
}

impl<T> Default for BehaviorValueType<T> {
    fn default() -> Self {
        // NOTE: This is just a dummy `Default` implementation. The library
        // requires it to circumvent some internal borrow checker issues.
        Self::Flow
    }
}

/// NodeTemplate is a mechanism to define node templates. It's what the graph
/// will display in the "new node" popup. The user code needs to tell the
/// library how to convert a NodeTemplate into a Node.
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum BehaviorNodeTemplate<T: BehaviorFactory> {
    Root,
    Behavior(T),
}

/// The response type is used to encode side-effects produced when drawing a
/// node in the graph. Most side-effects (creating new nodes, deleting existing
/// nodes, handling connections...) are already handled by the library, but this
/// mechanism allows creating additional side effects from user code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BehaviorResponse<T: BehaviorFactory> {
    NodeEdited(NodeId, T),
    NameEdited(NodeId, String),
    NodeEditDone(NodeId),
}

/// The graph 'global' state. This state struct is passed around to the node and
/// parameter drawing callbacks. The contents of this struct are entirely up to
/// the user. For this example, we use it to keep track of the 'active' node.
#[derive(Component)]
pub struct BehaviorGraphState {
    pub active_node: Option<NodeId>,
    pub editing_name: Option<NodeId>,
    pub type_registry: TypeRegistryArc,
    pub time: Time,
    pub blinker: SignalGenerator,
    pub root_node: Option<NodeId>,
}

impl Default for BehaviorGraphState {
    fn default() -> Self {
        Self {
            active_node: None,
            type_registry: TypeRegistryArc::default(),
            editing_name: None,
            time: Time::default(),
            blinker: SignalGenerator {
                func: SignalFunction::Triangle,
                frequency: 1.0,
                amplitude: 0.5,
                offset: 0.5,
                ..default()
            },
            root_node: None,
        }
    }
}

// A trait for the data types, to tell the library how to display them
impl<T>
    DataTypeTrait<BehaviorNodeData<T>, BehaviorDataType, BehaviorValueType<T>, BehaviorGraphState>
    for BehaviorDataType
where
    T: BehaviorFactory,
{
    // type DataType = BehaviorDataType;
    // type ValueType = BehaviorValueType<T>;
    // type UserState = BehaviorGraphState;

    fn data_type_color(
        &self,
        node_id: NodeId,
        graph: &Graph<BehaviorNodeData<T>, BehaviorDataType, BehaviorValueType<T>>,
        user_state: &mut BehaviorGraphState,
    ) -> egui::Color32 {
        match self {
            BehaviorDataType::Flow => {
                if let Some(node) = graph.nodes.get(node_id) {
                    // Draw circle state
                    if let Some(state) = node.user_data.state {
                        let blink = (user_state
                            .blinker
                            .sample(user_state.time.elapsed())
                            .clamp(0.0, 1.0)
                            * 255.0) as u8;
                        match state {
                            BehaviorState::Cursor => {
                                egui::Color32::from_rgba_unmultiplied(0, 255, 0, blink)
                            }
                            BehaviorState::Running => egui::Color32::GREEN,
                            BehaviorState::Success => egui::Color32::DARK_GREEN,
                            BehaviorState::Failure => egui::Color32::RED,
                            _ => egui::Color32::GRAY,
                        }
                    } else {
                        egui::Color32::DARK_GRAY
                    }
                } else {
                    egui::Color32::DARK_GRAY
                }
            }
        }
    }

    fn name(&self) -> Cow<'_, str> {
        match self {
            BehaviorDataType::Flow => Cow::Borrowed("flow"),
        }
    }
}

// A trait for the node kinds, which tells the library how to build new nodes
// from the templates in the node finder
impl<T> NodeTemplateTrait for BehaviorNodeTemplate<T>
where
    T: BehaviorFactory,
{
    type NodeData = BehaviorNodeData<T>;
    type DataType = BehaviorDataType;
    type ValueType = BehaviorValueType<T>;
    type UserState = BehaviorGraphState;
    type CategoryType = &'static str;

    fn node_finder_label(&self, _user_state: &mut Self::UserState) -> Cow<'_, str> {
        match self {
            BehaviorNodeTemplate::Root => Cow::Borrowed("Root"),
            BehaviorNodeTemplate::Behavior(behavior) => Cow::Borrowed(behavior.label()),
        }
    }

    // this is what allows the library to show collapsible lists in the node finder.
    fn node_finder_categories(&self, _user_state: &mut Self::UserState) -> Vec<&'static str> {
        match self {
            BehaviorNodeTemplate::Root => vec!["Root"],
            BehaviorNodeTemplate::Behavior(behavior) => behavior.categories(),
        }
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        // It's okay to delegate this to node_finder_label if you don't want to
        // show different names in the node finder and the node itself.
        self.node_finder_label(user_state).into()
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        BehaviorNodeData {
            data: self.clone(),
            state: None,
        }
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        match self {
            BehaviorNodeTemplate::Root => {
                println!("Building root node");
                graph.add_output_param(node_id, "".into(), BehaviorDataType::Flow);
            }
            BehaviorNodeTemplate::Behavior(behavior) => match behavior.typ() {
                BehaviorType::Action => {
                    graph.add_input_param(
                        node_id,
                        "A".into(),
                        BehaviorDataType::Flow,
                        BehaviorValueType::Flow,
                        InputParamKind::ConnectionOnly,
                        true,
                    );
                }
                BehaviorType::Decorator => {
                    graph.add_input_param(
                        node_id,
                        "A".into(),
                        BehaviorDataType::Flow,
                        BehaviorValueType::Flow,
                        InputParamKind::ConnectionOnly,
                        true,
                    );
                    graph.add_output_param(node_id, "".into(), BehaviorDataType::Flow);
                }
                BehaviorType::Composite => {
                    graph.add_input_param(
                        node_id,
                        "A".into(),
                        BehaviorDataType::Flow,
                        BehaviorValueType::Flow,
                        InputParamKind::ConnectionOnly,
                        true,
                    );

                    graph.add_output_param(node_id, "B".into(), BehaviorDataType::Flow);
                }
            },
        }
    }
}

#[derive(Default)]
pub struct BehaviorNodeTemplates<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> NodeTemplateIter for BehaviorNodeTemplates<T>
where
    T: BehaviorFactory,
{
    type Item = BehaviorNodeTemplate<T>;

    fn all_kinds(&self) -> Vec<Self::Item> {
        // This function must return a list of node kinds, which the node finder
        // will use to display it to the user. Crates like strum can reduce the
        // boilerplate in enumerating all variants of an enum.
        let kinds: Vec<BehaviorNodeTemplate<T>> = T::list()
            .into_iter()
            .map(|t| BehaviorNodeTemplate::Behavior(t))
            .collect();
        // Do not add Root node to kinds
        // kinds.extend(vec![BehaviorNodeTemplate::Root]);
        kinds
    }
}

impl<T> WidgetValueTrait for BehaviorValueType<T>
where
    T: BehaviorFactory,
{
    type Response = BehaviorResponse<T>;
    type UserState = BehaviorGraphState;
    type NodeData = BehaviorNodeData<T>;

    fn value_widget(
        &mut self,
        _param_name: &str,
        _node_id: NodeId,
        ui: &mut egui::Ui,
        _user_state: &mut BehaviorGraphState,
        _node_data: &BehaviorNodeData<T>,
    ) -> Vec<BehaviorResponse<T>> {
        ui.label("");
        default()
    }

    fn value_widget_connected(
        &mut self,
        _param_name: &str,
        _node_id: NodeId,
        ui: &mut egui::Ui,
        _user_state: &mut Self::UserState,
        _node_data: &Self::NodeData,
    ) -> Vec<Self::Response> {
        ui.label("");
        default()
    }
}

fn to_bytes(color: &Color) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(
        (color.r() * 255.0) as u8,
        (color.g() * 255.0) as u8,
        (color.b() * 255.0) as u8,
        (color.a() * 255.0) as u8,
    )
}

impl<T> UserResponseTrait for BehaviorResponse<T> where T: BehaviorFactory {}

impl<T> NodeDataTrait for BehaviorNodeData<T>
where
    T: BehaviorFactory,
{
    type Response = BehaviorResponse<T>;
    type UserState = BehaviorGraphState;
    type DataType = BehaviorDataType;
    type ValueType = BehaviorValueType<T>;

    fn can_delete(
        &self,
        _node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
    ) -> bool {
        match &self.data {
            BehaviorNodeTemplate::Root => false,
            BehaviorNodeTemplate::Behavior(_) => true,
        }
    }

    fn titlebar_color(
        &self,
        _ui: &egui::Ui,
        _node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
    ) -> Option<egui::Color32> {
        match &self.data {
            BehaviorNodeTemplate::Root => None,
            BehaviorNodeTemplate::Behavior(behavior) => Some(to_bytes(&behavior.color())),
        }
    }

    fn top_bar_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        graph: &Graph<Self, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<Self::Response, Self>>
    where
        Self::Response: UserResponseTrait,
    {
        let mut responses = vec![];

        match &self.data {
            BehaviorNodeTemplate::Root => {
                ui.label("Root");
            }
            BehaviorNodeTemplate::Behavior(_behavior) => {
                if let Some(node) = graph.nodes.get(node_id) {
                    match user_state.active_node {
                        Some(active_node_id) if active_node_id == node_id => {
                            let mut name = node.label.clone();
                            ui.style_mut().visuals.extreme_bg_color =
                                egui::Color32::from_rgba_premultiplied(0, 0, 0, 200);
                            if egui::TextEdit::singleline(&mut name)
                                .hint_text("node_name")
                                .text_color(egui::Color32::WHITE)
                                .show(ui)
                                .response
                                .changed()
                            {
                                responses.push(NodeResponse::User(BehaviorResponse::NameEdited(
                                    node_id, name,
                                )));
                            }
                        }
                        _ => {
                            ui.label(&node.label);
                        }
                    }
                }
            }
        }

        responses
    }

    fn separator(
        &self,
        _ui: &mut egui::Ui,
        _node_id: NodeId,
        _param_id: AnyParameterId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
    ) {
        // ui.separator();
    }

    fn output_ui(
        &self,
        ui: &mut egui::Ui,
        _node_id: NodeId,
        _graph: &Graph<Self, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
        _param_name: &str,
    ) -> Vec<NodeResponse<Self::Response, Self>>
    where
        Self::Response: UserResponseTrait,
    {
        ui.label("");
        default()
    }

    fn body_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        graph: &Graph<BehaviorNodeData<T>, BehaviorDataType, BehaviorValueType<T>>,
        user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<BehaviorResponse<T>, BehaviorNodeData<T>>>
    where
        T: BehaviorFactory,
        BehaviorResponse<T>: UserResponseTrait,
    {
        let mut responses = vec![];

        if let Some(node) = graph.nodes.get(node_id) {
            match &node.user_data.data {
                BehaviorNodeTemplate::Behavior(behavior) => {
                    // Small behavior label
                    let label =
                        egui::RichText::new(behavior.label()).color(egui::Color32::DARK_GRAY);
                    egui::Label::new(label).ui(ui);

                    // Reflect behavior properties
                    let type_registry = user_state.type_registry.read();
                    match user_state.active_node {
                        Some(active_node_id) if active_node_id == node_id => {
                            let mut behavior = behavior.clone();
                            if reflect_inspector::ui_for_value(
                                behavior.reflect_mut(),
                                ui,
                                &type_registry,
                            ) {
                                responses.push(NodeResponse::User(BehaviorResponse::NodeEdited(
                                    node_id, behavior,
                                )));
                            }
                        }
                        _ => {
                            reflect_inspector::ui_for_value_readonly(
                                behavior.reflect(),
                                ui,
                                &type_registry,
                            );
                        }
                    }
                }
                BehaviorNodeTemplate::Root => {
                    // if ui.button("⏵").clicked() {
                    //     responses.push(NodeResponse::User(BehaviorResponse::Play(node_id)));
                    // }
                }
            }
        }

        responses
    }

    fn bottom_ui(
        &self,
        _ui: &mut egui::Ui,
        _node_id: NodeId,
        _graph: &Graph<BehaviorNodeData<T>, BehaviorDataType, BehaviorValueType<T>>,
        _user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<BehaviorResponse<T>, BehaviorNodeData<T>>>
    where
        T: BehaviorFactory,
        BehaviorResponse<T>: UserResponseTrait,
    {
        let responses = vec![];

        // let mut name = "test";
        // egui::TextEdit::multiline(&mut name)
        // .code_editor().desired_rows(10).frame(true)
        //     // .min_size(egui::Vec2::new(100.0, 20.0))
        //     .text_color(egui::Color32::WHITE)
        //     // .frame(false)
        //     .show(ui);

        // if let Some(node) = graph.nodes.get(node_id) {
        //     match &node.user_data.data {
        //         MyNodeTemplate::Behavior(behavior) => {
        //             let mut behavior = behavior.clone();
        //             let type_registry = user_state.type_registry.read();
        //             if reflect_inspector::ui_for_value(behavior.reflect(), ui, &type_registry) {
        //                 responses.push(NodeResponse::User(MyResponse::NodeEdited(
        //                     node_id, behavior,
        //                 )));
        //             }
        //         }
        //         MyNodeTemplate::Root => {}
        //     }
        // }

        responses
    }
}

#[derive(Default, Component, Deref, DerefMut, Serialize, Deserialize)]
pub struct BehaviorEditorState<T: BehaviorFactory>(
    pub  GraphEditorState<
        BehaviorNodeData<T>,
        BehaviorDataType,
        BehaviorValueType<T>,
        BehaviorNodeTemplate<T>,
        BehaviorGraphState,
    >,
);
