use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiContext, *};
use egui_node_graph::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

pub struct DecisionPlugin;

impl Plugin for DecisionPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DecisionGraphState>()
            .add_system(egui_update);
    }
}

// ========= First, define your user data types =============

/// The NodeData holds a custom data struct inside each node. It's useful to
/// store additional information that doesn't live in parameters. For this
/// example, the node data stores the template (i.e. the "type") of the node.
#[derive(Serialize, Deserialize)]
pub struct DecisionNodeData {
    template: DecisionNodeTemplate,
}

/// `DataType`s are what defines the possible range of connections when
/// attaching two ports together. The graph UI will make sure to not allow
/// attaching incompatible datatypes.
#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionDataType {
    Scalar,
    Vec2,
    Flow,
}

/// In the graph, input parameters can optionally have a constant value. This
/// value can be directly edited in a widget inside the node itself.
///
/// There will usually be a correspondence between DataTypes and ValueTypes. But
/// this library makes no attempt to check this consistency. For instance, it is
/// up to the user code in this example to make sure no parameter is created
/// with a DataType of Scalar and a ValueType of Vec2.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum DecisionValueType {
    Vec2 { value: egui::Vec2 },
    Scalar { value: f32 },
    Connection,
}

impl DecisionValueType {
    /// Tries to downcast this value type to a vector
    pub fn try_to_vec2(self) -> anyhow::Result<egui::Vec2> {
        if let DecisionValueType::Vec2 { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to vec2", self)
        }
    }

    /// Tries to downcast this value type to a scalar
    pub fn try_to_scalar(self) -> anyhow::Result<f32> {
        if let DecisionValueType::Scalar { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to scalar", self)
        }
    }
}

/// NodeTemplate is a mechanism to define node templates. It's what the graph
/// will display in the "new node" popup. The user code needs to tell the
/// library how to convert a NodeTemplate into a Node.
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum DecisionNodeTemplate {
    // MakeVector,
    // MakeScalar,
    // AddScalar,
    // SubtractScalar,
    // VectorTimesScalar,
    // AddVector,
    // SubtractVector,
    Sequence,
    Selector,
    Behavior,
}

/// The response type is used to encode side-effects produced when drawing a
/// node in the graph. Most side-effects (creating new nodes, deleting existing
/// nodes, handling connections...) are already handled by the library, but this
/// mechanism allows creating additional side effects from user code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecisionGraphResponse {
    SetActiveNode(NodeId),
    ClearActiveNode,
    AddPin(NodeId),
    RemovePin(NodeId),
}

/// The graph 'global' state. This state struct is passed around to the node and
/// parameter drawing callbacks. The contents of this struct are entirely up to
/// the user. For this example, we use it to keep track of the 'active' node.
#[derive(Default, Component, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct DecisionGraphState {
    #[reflect(ignore)]
    pub active_node: Option<NodeId>,
}

// =========== Then, you need to implement some traits ============

// A trait for the data types, to tell the library how to display them
impl DataTypeTrait<DecisionGraphState> for DecisionDataType {
    fn data_type_color(&self, _user_state: &mut DecisionGraphState) -> egui::Color32 {
        match self {
            DecisionDataType::Scalar => egui::Color32::from_rgb(38, 109, 211),
            DecisionDataType::Vec2 => egui::Color32::from_rgb(238, 207, 109),
            DecisionDataType::Flow => egui::Color32::from_rgb(238, 255, 109),
        }
    }

    fn name(&self) -> Cow<'_, str> {
        match self {
            DecisionDataType::Scalar => Cow::Borrowed("scalar"),
            DecisionDataType::Vec2 => Cow::Borrowed("2d vector"),
            DecisionDataType::Flow => Cow::Borrowed("Flow"),
        }
    }
}

// A trait for the node kinds, which tells the library how to build new nodes
// from the templates in the node finder
impl NodeTemplateTrait for DecisionNodeTemplate {
    type NodeData = DecisionNodeData;
    type DataType = DecisionDataType;
    type ValueType = DecisionValueType;
    type UserState = DecisionGraphState;

    fn node_finder_label(&self) -> &str {
        match self {
            // MyNodeTemplate::MakeVector => "New vector",
            // MyNodeTemplate::MakeScalar => "New scalar",
            // MyNodeTemplate::AddScalar => "Scalar add",
            // MyNodeTemplate::SubtractScalar => "Scalar subtract",
            // MyNodeTemplate::AddVector => "Vector add",
            // MyNodeTemplate::SubtractVector => "Vector subtract",
            // MyNodeTemplate::VectorTimesScalar => "Vector times scalar",
            DecisionNodeTemplate::Sequence => "Sequence",
            DecisionNodeTemplate::Selector => "Selector",
            DecisionNodeTemplate::Behavior => "Behavior",
        }
    }

    fn node_graph_label(&self) -> String {
        // It's okay to delegate this to node_finder_label if you don't want to
        // show different names in the node finder and the node itself.
        self.node_finder_label().into()
    }

    fn user_data(&self) -> Self::NodeData {
        DecisionNodeData { template: *self }
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        // The nodes are created empty by default. This function needs to take
        // care of creating the desired inputs and outputs based on the template

        match self {
            // MyNodeTemplate::AddScalar => {
            //     // The first input param doesn't use the closure so we can comment
            //     // it in more detail.
            //     graph.add_input_param(
            //         node_id,
            //         // This is the name of the parameter. Can be later used to
            //         // retrieve the value. Parameter names should be unique.
            //         "A".into(),
            //         // The data type for this input. In this case, a scalar
            //         MyDataType::Scalar,
            //         // The value type for this input. We store zero as default
            //         MyValueType::Scalar { value: 0.0 },
            //         // The input parameter kind. This allows defining whether a
            //         // parameter accepts input connections and/or an inline
            //         // widget to set its value.
            //         InputParamKind::ConnectionOrConstant,
            //         true,
            //     );
            //     input_scalar(graph, "B");
            //     output_scalar(graph, "out");
            // }
            // MyNodeTemplate::SubtractScalar => {
            //     input_scalar(graph, "A");
            //     input_scalar(graph, "B");
            //     output_scalar(graph, "out");
            // }
            // MyNodeTemplate::VectorTimesScalar => {
            //     input_scalar(graph, "scalar");
            //     input_vector(graph, "vector");
            //     output_vector(graph, "out");
            // }
            // MyNodeTemplate::AddVector => {
            //     input_vector(graph, "v1");
            //     input_vector(graph, "v2");
            //     output_vector(graph, "out");
            // }
            // MyNodeTemplate::SubtractVector => {
            //     input_vector(graph, "v1");
            //     input_vector(graph, "v2");
            //     output_vector(graph, "out");
            // }
            // MyNodeTemplate::MakeVector => {
            //     input_scalar(graph, "x");
            //     input_scalar(graph, "y");
            //     output_vector(graph, "out");
            // }
            // MyNodeTemplate::MakeScalar => {
            //     input_scalar(graph, "value");
            //     output_scalar(graph, "out");
            // }
            DecisionNodeTemplate::Sequence => {
                graph.add_input_param(
                    node_id,
                    "".to_string(),
                    DecisionDataType::Flow,
                    DecisionValueType::Connection,
                    InputParamKind::ConnectionOrConstant,
                    true,
                );

                graph.add_output_param(node_id, "".to_string(), DecisionDataType::Flow);
                graph.add_output_param(node_id, "".to_string(), DecisionDataType::Flow);
            }
            DecisionNodeTemplate::Selector => {
                graph.add_input_param(
                    node_id,
                    "".to_string(),
                    DecisionDataType::Flow,
                    DecisionValueType::Connection,
                    InputParamKind::ConnectionOrConstant,
                    true,
                );

                graph.add_output_param(node_id, "".to_string(), DecisionDataType::Flow);
                graph.add_output_param(node_id, "".to_string(), DecisionDataType::Flow);
            }
            DecisionNodeTemplate::Behavior => {
                graph.add_input_param(
                    node_id,
                    "".to_string(),
                    DecisionDataType::Flow,
                    DecisionValueType::Connection,
                    InputParamKind::ConnectionOrConstant,
                    true,
                );
            }
        }
    }
}

pub struct AllMyNodeTemplates;
impl NodeTemplateIter for AllMyNodeTemplates {
    type Item = DecisionNodeTemplate;

    fn all_kinds(&self) -> Vec<Self::Item> {
        // This function must return a list of node kinds, which the node finder
        // will use to display it to the user. Crates like strum can reduce the
        // boilerplate in enumerating all variants of an enum.
        vec![
            // MyNodeTemplate::MakeScalar,
            // MyNodeTemplate::MakeVector,
            // MyNodeTemplate::AddScalar,
            // MyNodeTemplate::SubtractScalar,
            // MyNodeTemplate::AddVector,
            // MyNodeTemplate::SubtractVector,
            // MyNodeTemplate::VectorTimesScalar,
            DecisionNodeTemplate::Sequence,
            DecisionNodeTemplate::Selector,
            DecisionNodeTemplate::Behavior,
        ]
    }
}

impl WidgetValueTrait for DecisionValueType {
    type Response = DecisionGraphResponse;
    fn value_widget(&mut self, param_name: &str, ui: &mut egui::Ui) -> Vec<DecisionGraphResponse> {
        // This trait is used to tell the library which UI to display for the
        // inline parameter widgets.
        match self {
            DecisionValueType::Vec2 { value } => {
                ui.label(param_name);
                ui.horizontal(|ui| {
                    ui.label("x");
                    ui.add(egui::DragValue::new(&mut value.x));
                    ui.label("y");
                    ui.add(egui::DragValue::new(&mut value.y));
                });
            }
            DecisionValueType::Scalar { value } => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.add(egui::DragValue::new(value));
                });
            }
            DecisionValueType::Connection { .. } => {
                // ui.label(param_name);
                // ui.label("Connection");
            }
        }
        // This allows you to return your responses from the inline widgets.
        Vec::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Behavior {
    BehaviorA,
    BehaviorB,
    BehaviorC,
}

impl UserResponseTrait for DecisionGraphResponse {}
impl NodeDataTrait for DecisionNodeData {
    type Response = DecisionGraphResponse;
    type UserState = DecisionGraphState;
    type DataType = DecisionDataType;
    type ValueType = DecisionValueType;

    // This method will be called when drawing each node. This allows adding
    // extra ui elements inside the nodes. In this case, we create an "active"
    // button which introduces the concept of having an active node in the
    // graph. This is done entirely from user code with no modifications to the
    // node graph library.
    fn bottom_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        graph: &Graph<DecisionNodeData, DecisionDataType, DecisionValueType>,
        user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<DecisionGraphResponse, DecisionNodeData>>
    where
        DecisionGraphResponse: UserResponseTrait,
    {
        let mut responses = Vec::new();

        // On hover, we show a tooltip.

        if let DecisionNodeTemplate::Behavior = graph.nodes[node_id].user_data.template {
            let mut selected = Behavior::BehaviorA;
            egui::ComboBox::from_label("Select one!")
                .selected_text(format!("{:?}", selected))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut selected, Behavior::BehaviorA, "BehaviorA");
                    ui.selectable_value(&mut selected, Behavior::BehaviorB, "BehaviorB");
                    ui.selectable_value(&mut selected, Behavior::BehaviorC, "BehaviorC");
                });
        } else {
            // ui.collapsing("...", |ui| {
            //     let button = egui::Button::new(egui::RichText::new("➕ Pin"));
            //     if ui.add(button).clicked() {
            //         responses.push(NodeResponse::User(MyResponse::AddPin(node_id)));
            //     }
            //     let button = egui::Button::new(egui::RichText::new("➖ Pin"));
            //     if ui.add(button).clicked() {
            //         responses.push(NodeResponse::User(MyResponse::DelPin(node_id)));
            //     }
            // })
            // .header_response
            // .on_hover_text("Add or remove pins");

            ui.horizontal(|ui| {
                let button = egui::Button::new(egui::RichText::new("➕"));
                if ui.add(button).clicked() {
                    responses.push(NodeResponse::User(DecisionGraphResponse::AddPin(node_id)));
                }
                let button = egui::Button::new(egui::RichText::new("➖"));
                if ui.add(button).clicked() {
                    responses.push(NodeResponse::User(DecisionGraphResponse::RemovePin(
                        node_id,
                    )));
                }
            })
            .response
            .on_hover_text("Add or remove pins");
        }

        // // This logic is entirely up to the user. In this case, we check if the
        // // current node we're drawing is the active one, by comparing against
        // // the value stored in the global user state, and draw different button
        // // UIs based on that.

        // let mut responses = vec![];
        // let is_active = user_state
        //     .active_node
        //     .map(|id| id == node_id)
        //     .unwrap_or(false);

        // // Pressing the button will emit a custom user response to either set,
        // // or clear the active node. These responses do nothing by themselves,
        // // the library only makes the responses available to you after the graph
        // // has been drawn. See below at the update method for an example.
        // if !is_active {
        //     if ui.button("👁 Set active").clicked() {
        //         responses.push(NodeResponse::User(MyResponse::SetActiveNode(node_id)));
        //     }
        // } else {
        //     let button =
        //         egui::Button::new(egui::RichText::new("👁 Active").color(egui::Color32::BLACK))
        //             .fill(egui::Color32::GOLD);
        //     if ui.add(button).clicked() {
        //         responses.push(NodeResponse::User(MyResponse::ClearActiveNode));
        //     }
        // }

        responses
    }
}

// type MyGraph = Graph<DecisionNodeData, DecisionDataType, DecisionValueType>;
// type MyEditorState =
//     GraphEditorState<MyNodeData, MyDataType, MyValueType, MyNodeTemplate, MyGraphState>;

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct DecisionEditorState {
    #[reflect(ignore)]
    pub state: GraphEditorState<
        DecisionNodeData,
        DecisionDataType,
        DecisionValueType,
        DecisionNodeTemplate,
        DecisionGraphState,
    >,
}

pub fn egui_update(
    mut egui_context: ResMut<EguiContext>,
    mut states: Query<(
        Entity,
        &mut DecisionEditorState,
        &mut DecisionGraphState,
        &Name,
    )>,
) {
    for (entity, mut editor_state, mut user_state, name) in states.iter_mut() {
        let ctx = egui_context.ctx_mut();
        // egui::TopBottomPanel::top("top").show(ctx, |ui| {
        //     egui::menu::bar(ui, |ui| {
        //         egui::widgets::global_dark_light_mode_switch(ui);
        //     });
        // });

        let graph_response = egui::Window::new(name.as_str())
            .id(egui::Id::new(entity))
            .show(ctx, |ui| {
                editor_state
                    .state
                    .draw_graph_editor(ui, AllMyNodeTemplates, &mut user_state)
            })
            .unwrap()
            .inner;

        if let Some(graph_response) = graph_response {
            for node_response in graph_response.node_responses {
                // Here, we ignore all other graph events. But you may find
                // some use for them. For example, by playing a sound when a new
                // connection is created
                if let NodeResponse::User(user_event) = node_response {
                    match user_event {
                        DecisionGraphResponse::SetActiveNode(node) => {
                            user_state.active_node = Some(node)
                        }
                        DecisionGraphResponse::ClearActiveNode => user_state.active_node = None,
                        DecisionGraphResponse::AddPin(node_id) => {
                            let graph = &mut editor_state.state.graph;
                            graph.add_output_param(node_id, "".to_string(), DecisionDataType::Flow);
                        }
                        DecisionGraphResponse::RemovePin(node_id) => {
                            let graph = &mut editor_state.state.graph;
                            if let Some(output_id) = graph.nodes[node_id].output_ids().last() {
                                graph.remove_output_param(output_id);
                            }
                        }
                    }
                }
            }
        }

        // if let Some(node) = user_state.active_node {
        //     if editor_state.graph.nodes.contains_key(node) {
        //         let text = match evaluate_node(&editor_state.graph, node, &mut HashMap::new()) {
        //             Ok(value) => format!("The result is: {:?}", value),
        //             Err(err) => format!("Execution error: {}", err),
        //         };
        //         ctx.debug_painter().text(
        //             egui::pos2(10.0, 35.0),
        //             egui::Align2::LEFT_TOP,
        //             text,
        //             egui::TextStyle::Button.resolve(&ctx.style()),
        //             egui::Color32::WHITE,
        //         );
        //     } else {
        //         user_state.active_node = None;
        //     }
        // }
    }
}
