use bevy::{prelude::*, utils::HashSet};
use bevy_inspector_egui::{bevy_egui::EguiContext, *};
use egui_node_graph::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
// use crate::BehaviorState;

#[derive(Bundle)]
pub struct BehaviorGraphBundle {
    pub editor_state: BehaviorEditorState,
    pub graph_state: BehaviorGraphState,
    pub name: Name,
}

/// The NodeData holds a custom data struct inside each node. It's useful to
/// store additional information that doesn't live in parameters. For this
/// example, the node data stores the template (i.e. the "type") of the node.
#[derive(Serialize, Deserialize)]
pub struct BehaviorNodeData {
    template: BehaviorNodeTemplate,
    behavior: Option<String>,
    name: String,
}

/// `DataType`s are what defines the possible range of connections when
/// attaching two ports together. The graph UI will make sure to not allow
/// attaching incompatible datatypes.
#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub enum BehaviorDataType {
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
pub enum BehaviorValueType {
    Vec2 { value: egui::Vec2 },
    Scalar { value: f32 },
    Connection,
}

impl BehaviorValueType {
    /// Tries to downcast this value type to a vector
    pub fn try_to_vec2(self) -> anyhow::Result<egui::Vec2> {
        if let BehaviorValueType::Vec2 { value } = self {
            Ok(value)
        } else {
            anyhow::bail!("Invalid cast from {:?} to vec2", self)
        }
    }

    /// Tries to downcast this value type to a scalar
    pub fn try_to_scalar(self) -> anyhow::Result<f32> {
        if let BehaviorValueType::Scalar { value } = self {
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
pub enum BehaviorNodeTemplate {
    // MakeVector,
    // MakeScalar,
    // AddScalar,
    // SubtractScalar,
    VectorTimesScalar,
    // AddVector,
    // SubtractVector,
    Sequence,
    Selector,
    Action,
}

/// The response type is used to encode side-effects produced when drawing a
/// node in the graph. Most side-effects (creating new nodes, deleting existing
/// nodes, handling connections...) are already handled by the library, but this
/// mechanism allows creating additional side effects from user code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BehaviorGraphResponse {
    SetActiveNode(NodeId),
    ClearActiveNode,
    AddPin(NodeId),
    RemovePin(NodeId),
    SelectBehavior(NodeId, Option<String>),
    EditingNodeName(NodeId, String),
    NodeNameChange(NodeId, String),
}

/// The graph 'global' state. This state struct is passed around to the node and
/// parameter drawing callbacks. The contents of this struct are entirely up to
/// the user. For this example, we use it to keep track of the 'active' node.
#[derive(Default, Component, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct BehaviorGraphState {
    #[reflect(ignore)]
    pub active_node: Option<NodeId>,
    #[reflect(ignore)]
    #[serde(skip)]
    pub behaviors: HashSet<String>,
    #[reflect(ignore)]
    #[serde(skip)]
    pub editing_node_name: Option<(NodeId, String)>,
}

// A trait for the data types, to tell the library how to display them
impl DataTypeTrait<BehaviorGraphState> for BehaviorDataType {
    fn data_type_color(&self, _user_state: &mut BehaviorGraphState) -> egui::Color32 {
        match self {
            BehaviorDataType::Scalar => egui::Color32::from_rgb(38, 109, 211),
            BehaviorDataType::Vec2 => egui::Color32::from_rgb(238, 207, 109),
            BehaviorDataType::Flow => color_hex_utils::color_from_hex("#555555").unwrap(),
        }
    }

    fn name(&self) -> Cow<'_, str> {
        match self {
            BehaviorDataType::Scalar => Cow::Borrowed("scalar"),
            BehaviorDataType::Vec2 => Cow::Borrowed("2d vector"),
            BehaviorDataType::Flow => Cow::Borrowed("Flow"),
        }
    }
}

// A trait for the node kinds, which tells the library how to build new nodes
// from the templates in the node finder
impl NodeTemplateTrait for BehaviorNodeTemplate {
    type NodeData = BehaviorNodeData;
    type DataType = BehaviorDataType;
    type ValueType = BehaviorValueType;
    type UserState = BehaviorGraphState;

    fn node_finder_label(&self) -> &str {
        match self {
            // MyNodeTemplate::MakeVector => "New vector",
            // MyNodeTemplate::MakeScalar => "New scalar",
            // MyNodeTemplate::AddScalar => "Scalar add",
            // MyNodeTemplate::SubtractScalar => "Scalar subtract",
            // MyNodeTemplate::AddVector => "Vector add",
            // MyNodeTemplate::SubtractVector => "Vector subtract",
            BehaviorNodeTemplate::VectorTimesScalar => "Vector times scalar",
            BehaviorNodeTemplate::Sequence => "Sequence",
            BehaviorNodeTemplate::Selector => "Selector",
            BehaviorNodeTemplate::Action => "Action",
        }
    }

    fn node_graph_label(&self) -> String {
        // It's okay to delegate this to node_finder_label if you don't want to
        // show different names in the node finder and the node itself.
        // self.node_finder_label().into()
        // self.user_data().name
        self.node_finder_label().into()
    }

    fn user_data(&self) -> Self::NodeData {
        BehaviorNodeData {
            template: *self,
            behavior: None,
            name: self.node_finder_label().into(),
        }
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        // The nodes are created empty by default. This function needs to take
        // care of creating the desired inputs and outputs based on the template

        let input_scalar = |graph: &mut BehaviorGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                BehaviorDataType::Scalar,
                BehaviorValueType::Scalar { value: 0.0 },
                InputParamKind::ConnectionOrConstant,
                true,
            );
        };
        let input_vector = |graph: &mut BehaviorGraph, name: &str| {
            graph.add_input_param(
                node_id,
                name.to_string(),
                BehaviorDataType::Vec2,
                BehaviorValueType::Vec2 {
                    value: egui::vec2(0.0, 0.0),
                },
                InputParamKind::ConnectionOrConstant,
                true,
            );
        };

        let _output_scalar = |graph: &mut BehaviorGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), BehaviorDataType::Scalar);
        };
        let output_vector = |graph: &mut BehaviorGraph, name: &str| {
            graph.add_output_param(node_id, name.to_string(), BehaviorDataType::Vec2);
        };

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
            // BehaviorNodeTemplate::SubtractScalar => {
            //     input_scalar(graph, "A");
            //     input_scalar(graph, "B");
            //     output_scalar(graph, "out");
            // }
            BehaviorNodeTemplate::VectorTimesScalar => {
                input_scalar(graph, "scalar");
                input_vector(graph, "vector");
                output_vector(graph, "out");
            }
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
            BehaviorNodeTemplate::Sequence => {
                graph.add_input_param(
                    node_id,
                    "".to_string(),
                    BehaviorDataType::Flow,
                    BehaviorValueType::Connection,
                    InputParamKind::ConnectionOrConstant,
                    true,
                );

                graph.add_output_param(node_id, "".to_string(), BehaviorDataType::Flow);
                graph.add_output_param(node_id, "".to_string(), BehaviorDataType::Flow);
            }
            BehaviorNodeTemplate::Selector => {
                graph.add_input_param(
                    node_id,
                    "".to_string(),
                    BehaviorDataType::Flow,
                    BehaviorValueType::Connection,
                    InputParamKind::ConnectionOrConstant,
                    true,
                );

                graph.add_output_param(node_id, "".to_string(), BehaviorDataType::Flow);
                graph.add_output_param(node_id, "".to_string(), BehaviorDataType::Flow);
            }
            BehaviorNodeTemplate::Action => {
                graph.add_input_param(
                    node_id,
                    "".to_string(),
                    BehaviorDataType::Flow,
                    BehaviorValueType::Connection,
                    InputParamKind::ConnectionOrConstant,
                    true,
                );
            }
        }
    }
}

pub struct AllMyNodeTemplates;
impl NodeTemplateIter for AllMyNodeTemplates {
    type Item = BehaviorNodeTemplate;

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
            BehaviorNodeTemplate::VectorTimesScalar,
            BehaviorNodeTemplate::Sequence,
            BehaviorNodeTemplate::Selector,
            BehaviorNodeTemplate::Action,
        ]
    }
}

impl WidgetValueTrait for BehaviorValueType {
    type Response = BehaviorGraphResponse;
    fn value_widget(&mut self, param_name: &str, ui: &mut egui::Ui) -> Vec<BehaviorGraphResponse> {
        // This trait is used to tell the library which UI to display for the
        // inline parameter widgets.
        match self {
            BehaviorValueType::Vec2 { value } => {
                ui.label(param_name);
                ui.horizontal(|ui| {
                    ui.label("x");
                    ui.add(egui::DragValue::new(&mut value.x));
                    ui.label("y");
                    ui.add(egui::DragValue::new(&mut value.y));
                });
            }
            BehaviorValueType::Scalar { value } => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.add(egui::DragValue::new(value));
                });
            }
            BehaviorValueType::Connection { .. } => {
                ui.label(param_name);
                ui.label("Connection");
            }
        }
        // This allows you to return your responses from the inline widgets.
        Vec::new()
    }
}

impl UserResponseTrait for BehaviorGraphResponse {}
impl NodeDataTrait for BehaviorNodeData {
    type Response = BehaviorGraphResponse;
    type UserState = BehaviorGraphState;
    type DataType = BehaviorDataType;
    type ValueType = BehaviorValueType;

    // Change titlebar color
    fn titlebar_color(
        &self,
        _ui: &egui::Ui,
        node_id: NodeId,
        graph: &Graph<BehaviorNodeData, BehaviorDataType, BehaviorValueType>,
        _user_state: &mut Self::UserState,
    ) -> Option<egui::Color32> {
        let template = graph.nodes[node_id].user_data.template;
        match template {
            BehaviorNodeTemplate::Sequence => color_hex_utils::color_from_hex("#328900").ok(),
            BehaviorNodeTemplate::Selector => color_hex_utils::color_from_hex("#EE0000").ok(),
            BehaviorNodeTemplate::Action => color_hex_utils::color_from_hex("#0085D4").ok(),
            BehaviorNodeTemplate::VectorTimesScalar => {
                color_hex_utils::color_from_hex("#0085D4").ok()
            }
        }
    }

    fn titlebar_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        graph: &Graph<BehaviorNodeData, BehaviorDataType, BehaviorValueType>,
        user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<BehaviorGraphResponse, BehaviorNodeData>>
    where
        BehaviorGraphResponse: UserResponseTrait,
    {
        let mut responses = Vec::new();

        let text_color = color_hex_utils::color_from_hex("#fefefe").unwrap();
        ui.visuals_mut().widgets.noninteractive.fg_stroke = egui::Stroke::new(2.0, text_color);

        egui::Grid::new("some_unique_id").show(ui, |ui| {
            // ui.small_button("+");

            if let Some(name) = user_state
                .editing_node_name
                .as_ref()
                .and_then(|(id, name)| if *id == node_id { Some(name) } else { None })
            {
                let mut name = name.clone();
                if ui.add(egui::TextEdit::singleline(&mut name)).lost_focus() {
                    user_state.editing_node_name = None;
                    responses.push(NodeResponse::User(BehaviorGraphResponse::NodeNameChange(
                        node_id, name,
                    )));
                } else {
                    responses.push(NodeResponse::User(BehaviorGraphResponse::EditingNodeName(
                        node_id, name,
                    )));
                }
            } else {
                if ui
                    .add(
                        egui::Label::new(
                            egui::RichText::new(&graph[node_id].user_data.name)
                                .text_style(egui::TextStyle::Button)
                                .strong()
                                .color(text_color),
                        )
                        .sense(egui::Sense::click()),
                    )
                    .double_clicked()
                {
                    responses.push(NodeResponse::User(BehaviorGraphResponse::EditingNodeName(
                        node_id,
                        graph[node_id].user_data.name.clone(),
                    )));
                }
                ui.add_space(8.0); // The size of the little cross icon
            }

            // ui.small_button("x");
        });

        responses
    }

    // This method will be called when drawing each node. This allows adding
    // extra ui elements inside the nodes. In this case, we create an "active"
    // button which introduces the concept of having an active node in the
    // graph. This is done entirely from user code with no modifications to the
    // node graph library.
    fn bottom_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: NodeId,
        graph: &Graph<BehaviorNodeData, BehaviorDataType, BehaviorValueType>,
        user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<BehaviorGraphResponse, BehaviorNodeData>>
    where
        BehaviorGraphResponse: UserResponseTrait,
    {
        let mut responses = Vec::new();

        // On hover, we show a tooltip.

        let template = graph.nodes[node_id].user_data.template;
        let user_data = &graph.nodes[node_id].user_data;

        if let BehaviorNodeTemplate::Action = template {
            let mut selected = user_data
                .behavior
                .clone()
                .unwrap_or("Select Behavior".to_string());
            let mut changed = false;
            egui::ComboBox::from_label("")
                .selected_text(format!("{}", selected))
                .show_ui(ui, |ui| {
                    for behavior_name in &user_state.behaviors {
                        changed = changed
                            || ui
                                .selectable_value(
                                    &mut selected,
                                    behavior_name.clone(),
                                    behavior_name,
                                )
                                .changed();
                    }
                });
            if changed {
                // println!("Changed to {:?}", selected);
                responses.push(NodeResponse::User(BehaviorGraphResponse::SelectBehavior(
                    node_id,
                    Some(selected),
                )))
            }
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
                let button = egui::Button::new(egui::RichText::new("➕")).small();
                if ui.add(button).clicked() {
                    responses.push(NodeResponse::User(BehaviorGraphResponse::AddPin(node_id)));
                }
                let button = egui::Button::new(egui::RichText::new("➖")).small();
                if ui.add(button).clicked() {
                    responses.push(NodeResponse::User(BehaviorGraphResponse::RemovePin(
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

type BehaviorGraph = Graph<BehaviorNodeData, BehaviorDataType, BehaviorValueType>;
// type MyEditorState =
//     GraphEditorState<MyNodeData, MyDataType, MyValueType, MyNodeTemplate, MyGraphState>;

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct BehaviorEditorState {
    pub show: bool,
    #[reflect(ignore)]
    pub state: GraphEditorState<
        BehaviorNodeData,
        BehaviorDataType,
        BehaviorValueType,
        BehaviorNodeTemplate,
        BehaviorGraphState,
    >,
}

pub fn egui_update(
    mut egui_context: ResMut<EguiContext>,
    mut states: Query<(
        Entity,
        &mut BehaviorEditorState,
        &mut BehaviorGraphState,
        &Name,
    )>,
    behaviors: Query<(&Parent, &Name)>,
) {
    for (entity, mut editor_state, mut user_state, name) in states.iter_mut() {
        if !editor_state.show {
            continue;
        }

        for (parent, name) in behaviors.iter() {
            if parent.get() == entity {
                user_state.behaviors.insert(name.to_string());
            }
        }

        let ctx = egui_context.ctx_mut();
        // egui::TopBottomPanel::top("top").show(ctx, |ui| {
        //     egui::menu::bar(ui, |ui| {
        //         egui::widgets::global_dark_light_mode_switch(ui);
        //     });
        // });

        let graph_response = egui::Window::new(name.as_str())
            .id(egui::Id::new(entity))
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.small_button("Print").on_hover_text("Print to console").clicked().then(|| {
                        let graph_str = ron::ser::to_string(&editor_state.state.graph).unwrap();
                        println!("Graph: {}", graph_str);
                    });
                });
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
                        BehaviorGraphResponse::SetActiveNode(node) => {
                            user_state.active_node = Some(node)
                        }
                        BehaviorGraphResponse::ClearActiveNode => user_state.active_node = None,
                        BehaviorGraphResponse::AddPin(node_id) => {
                            let graph = &mut editor_state.state.graph;
                            graph.add_output_param(node_id, "".to_string(), BehaviorDataType::Flow);
                        }
                        BehaviorGraphResponse::RemovePin(node_id) => {
                            let graph = &mut editor_state.state.graph;
                            if let Some(output_id) = graph.nodes[node_id].output_ids().last() {
                                graph.remove_output_param(output_id);
                            }
                        }
                        BehaviorGraphResponse::SelectBehavior(node_id, behavior) => {
                            println!("Select behavior: {:?}", behavior);
                            let graph = &mut editor_state.state.graph;
                            graph.nodes[node_id].user_data.behavior = behavior;
                        }
                        BehaviorGraphResponse::EditingNodeName(node_id, name) => {
                            user_state.editing_node_name = Some((node_id, name));
                            // let graph = &mut editor_state.state.graph;
                            // graph.nodes[node_id].user_data.editing_name = true;
                        }
                        BehaviorGraphResponse::NodeNameChange(node_id, name) => {
                            println!("Node name change: {:?}", name);
                            let graph = &mut editor_state.state.graph;
                            graph.nodes[node_id].user_data.name = name;
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
