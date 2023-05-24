use crate::prelude::*;
use bevy::{prelude::*, reflect::TypeRegistryArc};
use bevy_inspector_egui::{
    egui::{self},
    reflect_inspector,
};
use egui_node_graph::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// The NodeData holds a custom data struct inside each node. It's useful to
/// store additional information that doesn't live in parameters. For this
/// example, the node data stores the template (i.e. the "type") of the node.
#[derive(Serialize, Debug)]
pub struct MyNodeData<T: BehaviorSpawner> {
    pub data: Option<T>,
}

/// `DataType`s are what defines the possible range of connections when
/// attaching two ports together. The graph UI will make sure to not allow
/// attaching incompatible datatypes.
#[derive(PartialEq, Eq, Deserialize, Serialize)]
pub enum MyDataType {
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
pub enum MyValueType<T> {
    // Data(T),
    Flow,
    Marker {
        _marker: std::marker::PhantomData<T>,
    },
}

impl<T> Default for MyValueType<T> {
    fn default() -> Self {
        // NOTE: This is just a dummy `Default` implementation. The library
        // requires it to circumvent some internal borrow checker issues.
        Self::Flow
    }
}

/// NodeTemplate is a mechanism to define node templates. It's what the graph
/// will display in the "new node" popup. The user code needs to tell the
/// library how to convert a NodeTemplate into a Node.
#[derive(Clone, Copy, Serialize)]
pub enum MyNodeTemplate<T: BehaviorSpawner> {
    Root,
    Behavior(T),
}

/// The response type is used to encode side-effects produced when drawing a
/// node in the graph. Most side-effects (creating new nodes, deleting existing
/// nodes, handling connections...) are already handled by the library, but this
/// mechanism allows creating additional side effects from user code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MyResponse<T: BehaviorSpawner> {
    SetActiveNode(NodeId),
    ClearActiveNode,
    NodeEdited(NodeId, T),
}

/// The graph 'global' state. This state struct is passed around to the node and
/// parameter drawing callbacks. The contents of this struct are entirely up to
/// the user. For this example, we use it to keep track of the 'active' node.
#[derive(Default, Component, Serialize, Deserialize)]
pub struct MyGraphState {
    pub active_node: Option<NodeId>,
    #[serde(skip)]
    pub type_registry: TypeRegistryArc,
}

// =========== Then, you need to implement some traits ============

// A trait for the data types, to tell the library how to display them
impl DataTypeTrait<MyGraphState> for MyDataType {
    fn data_type_color(&self, _user_state: &mut MyGraphState) -> egui::Color32 {
        match self {
            MyDataType::Flow => egui::Color32::from_rgb(238, 207, 109),
        }
    }

    fn name(&self) -> Cow<'_, str> {
        match self {
            MyDataType::Flow => Cow::Borrowed("flow"),
        }
    }
}

// A trait for the node kinds, which tells the library how to build new nodes
// from the templates in the node finder
impl<T> NodeTemplateTrait for MyNodeTemplate<T>
where
    T: BehaviorSpawner,
{
    type NodeData = MyNodeData<T>;
    type DataType = MyDataType;
    type ValueType = MyValueType<T>;
    type UserState = MyGraphState;
    type CategoryType = &'static str;

    fn node_finder_label(&self, _user_state: &mut Self::UserState) -> Cow<'_, str> {
        match self {
            MyNodeTemplate::Root => Cow::Borrowed("Root"),
            MyNodeTemplate::Behavior(behavior) => Cow::Borrowed(behavior.label()),
        }
    }

    // this is what allows the library to show collapsible lists in the node finder.
    fn node_finder_categories(&self, _user_state: &mut Self::UserState) -> Vec<&'static str> {
        match self {
            MyNodeTemplate::Root => vec!["Root"],
            MyNodeTemplate::Behavior(behavior) => behavior.categories(),
        }
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        // It's okay to delegate this to node_finder_label if you don't want to
        // show different names in the node finder and the node itself.
        self.node_finder_label(user_state).into()
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        let data = match self {
            MyNodeTemplate::Root => None,
            MyNodeTemplate::Behavior(behavior) => Some(behavior.clone()),
        };
        MyNodeData { data }
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        match self {
            MyNodeTemplate::Root => {
                graph.add_output_param(node_id, "0".into(), MyDataType::Flow);
            }
            MyNodeTemplate::Behavior(behavior) => match behavior.typ() {
                BehaviorType::Action => {
                    graph.add_input_param(
                        node_id,
                        "".into(),
                        MyDataType::Flow,
                        MyValueType::Flow,
                        InputParamKind::ConnectionOnly,
                        true,
                    );
                }
                BehaviorType::Decorator => {
                    graph.add_input_param(
                        node_id,
                        "".into(),
                        MyDataType::Flow,
                        MyValueType::Flow,
                        InputParamKind::ConnectionOnly,
                        true,
                    );
                    graph.add_output_param(node_id, "".into(), MyDataType::Flow);
                }
                BehaviorType::Composite => {
                    graph.add_input_param(
                        node_id,
                        "".into(),
                        MyDataType::Flow,
                        MyValueType::Flow,
                        InputParamKind::ConnectionOnly,
                        true,
                    );

                    graph.add_output_param(node_id, "".into(), MyDataType::Flow);
                    graph.add_output_param(node_id, "".into(), MyDataType::Flow);
                    graph.add_output_param(node_id, "".into(), MyDataType::Flow);
                }
            },
        }
    }
}

#[derive(Default)]
pub struct AllMyNodeTemplates<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> NodeTemplateIter for AllMyNodeTemplates<T>
where
    T: BehaviorSpawner,
{
    type Item = MyNodeTemplate<T>;

    fn all_kinds(&self) -> Vec<Self::Item> {
        // This function must return a list of node kinds, which the node finder
        // will use to display it to the user. Crates like strum can reduce the
        // boilerplate in enumerating all variants of an enum.
        let mut kinds: Vec<MyNodeTemplate<T>> = T::list()
            .into_iter()
            .map(|t| MyNodeTemplate::Behavior(t))
            .collect();
        kinds.extend(vec![MyNodeTemplate::Root]);
        kinds
    }
}

impl<T> WidgetValueTrait for MyValueType<T>
where
    T: BehaviorSpawner,
{
    type Response = MyResponse<T>;
    type UserState = MyGraphState;
    type NodeData = MyNodeData<T>;
    fn value_widget(
        &mut self,
        _param_name: &str,
        _node_id: NodeId,
        _ui: &mut egui::Ui,
        _user_state: &mut MyGraphState,
        _node_data: &MyNodeData<T>,
    ) -> Vec<MyResponse<T>> {
        // This trait is used to tell the library which UI to display for the
        // inline parameter widgets.
        match self {
            MyValueType::Flow => {}
            MyValueType::Marker { .. } => {}
        }
        // This allows you to return your responses from the inline widgets.
        Vec::new()
    }
}

impl<T> UserResponseTrait for MyResponse<T> where T: BehaviorSpawner {}

impl<T> NodeDataTrait for MyNodeData<T>
where
    T: BehaviorSpawner,
{
    type Response = MyResponse<T>;
    type UserState = MyGraphState;
    type DataType = MyDataType;
    type ValueType = MyValueType<T>;

    // This method will be called when drawing each node. This allows adding
    // extra ui elements inside the nodes. In this case, we create an "active"
    // button which introduces the concept of having an active node in the
    // graph. This is done entirely from user code with no modifications to the
    // node graph library.
    fn bottom_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        graph: &Graph<MyNodeData<T>, MyDataType, MyValueType<T>>,
        user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<MyResponse<T>, MyNodeData<T>>>
    where
        T: BehaviorSpawner,
        MyResponse<T>: UserResponseTrait,
    {
        let mut responses = vec![];

        if let Some(node) = graph.nodes.get(node_id) {
            match &node.user_data.data {
                Some(behavior) => {
                    let mut behavior = behavior.clone();
                    let type_registry = user_state.type_registry.read();
                    if reflect_inspector::ui_for_value(behavior.reflect(), ui, &type_registry) {
                        responses.push(NodeResponse::User(MyResponse::NodeEdited(
                            node_id, behavior,
                        )));
                    }
                }
                None => {}
            }
        }

        responses
    }
}

#[derive(Default, Component, Deref, DerefMut)]
pub struct MyEditorState<T: BehaviorSpawner>(
    pub GraphEditorState<MyNodeData<T>, MyDataType, MyValueType<T>, MyNodeTemplate<T>, MyGraphState>,
);

