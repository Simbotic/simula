//! A Rust implementation of the force-directed graph algorithm from [Graphoon](https://github.com/rm-code/Graphoon/).
//!
//! # Example
//!
//! ```
//! use bevy::math::Vec3;
//! use std::time::Duration;
//! use simula_core::force_graph::{ForceGraph, Node, NodeData};
//!
//! // create a force graph with default parameters
//! let mut graph = <ForceGraph>::new(Default::default());
//!
//! // create nodes
//! let n1_idx = graph.add_node(NodeData {
//!     position: Vec3::new(250.0, 250.0, 250.0),
//!     ..Default::default()
//! });
//! let n2_idx = graph.add_node(NodeData {
//!     position: Vec3::new(750.0, 250.0, 250.0),
//!     ..Default::default()
//! });
//! let n3_idx = graph.add_node(NodeData {
//!     position: Vec3::new(250.0, 750.0, 250.0),
//!     ..Default::default()
//! });
//! let n4_idx = graph.add_node(NodeData {
//!     position: Vec3::new(250.0, 250.0, 750.0),
//!     ..Default::default()
//! });
//! let n5_idx = graph.add_node(NodeData {
//!     position: Vec3::new(500.0, 500.0, 500.0),
//!     is_anchor: true,
//!     ..Default::default()
//! });
//!
//! // set up links between nodes
//! graph.add_edge(n1_idx, n5_idx, Default::default());
//! graph.add_edge(n2_idx, n5_idx, Default::default());
//! graph.add_edge(n3_idx, n5_idx, Default::default());
//! graph.add_edge(n4_idx, n5_idx, Default::default());
//!
//! // --- your game loop would start here ---
//!
//! // draw edges with your own drawing function
//! fn draw_edge(x1: f32, y1: f32, x2: f32, y2: f32) {}
//!
//! graph.visit_edges(|node1, node2, _edge| {
//!     draw_edge(node1.position().x, node1.position().y, node2.position().x, node2.position().y);
//! });
//!
//! // draw nodes with your own drawing function
//! fn draw_node(x: f32, y: f32) {}
//!
//! graph.visit_nodes(|node| {
//!     draw_node(node.position().x, node.position().y);
//! });
//!
//! // calculate dt with your own timing function
//! let dt = Duration::new(0, 5);
//! graph.update(dt);
//!
//! // --- your game loop would repeat here ---
//!
//! ```

use bevy::prelude::*;
use petgraph::{
    stable_graph::StableUnGraph,
    visit::{EdgeRef, IntoEdgeReferences},
};
use std::{
    collections::BTreeSet,
    ops::{Deref, DerefMut},
    time::Duration,
};

#[derive(PartialEq, PartialOrd, Ord, Eq, Clone, Copy)]
pub struct NodeIndex(pub petgraph::stable_graph::NodeIndex<petgraph::stable_graph::DefaultIx>);

impl Deref for NodeIndex {
    type Target = petgraph::stable_graph::NodeIndex<petgraph::stable_graph::DefaultIx>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NodeIndex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for NodeIndex {
    fn default() -> Self {
        NodeIndex(Default::default())
    }
}

/// Parameters to control the simulation of the force graph.
#[derive(Reflect, Clone, Debug)]
pub struct SimulationParameters {
    pub force_charge: f32,
    pub force_spring: f32,
    pub force_max: f32,
    pub node_speed: Vec3,
    pub damping_factor: f32,
    pub iterations: usize,
    pub max_dt: f32,
}

impl Default for SimulationParameters {
    fn default() -> Self {
        SimulationParameters {
            force_charge: 12000.0,
            force_spring: 0.3,
            force_max: 280.0,
            node_speed: Vec3::splat(500.0),
            damping_factor: 0.95,
            iterations: 10,
            max_dt: 0.03,
        }
    }
}

#[derive(Clone)]
/// Stores data associated with a node that can be modified by the user.
pub struct NodeData<UserNodeData = ()> {
    /// The position of the node.
    pub position: Vec3,
    /// The mass of the node.
    ///
    /// Increasing the mass of a node increases the force with which it repels other nearby nodes.
    pub mass: f32,
    /// Whether the node is fixed to its current position.
    pub is_anchor: bool,
    /// Arbitrary user data.
    ///
    /// Defaults to `()` if not specified.
    pub user_data: UserNodeData,
}

impl<UserNodeData> Default for NodeData<UserNodeData>
where
    UserNodeData: Default,
{
    fn default() -> Self {
        NodeData {
            position: Vec3::ZERO,
            mass: 10.0,
            is_anchor: false,
            user_data: Default::default(),
        }
    }
}

/// Stores data associated with an edge that can be modified by the user.
pub struct EdgeData<UserEdgeData = ()> {
    /// Arbitrary user data.
    ///
    /// Defaults to `()` if not specified.
    pub user_data: UserEdgeData,
}

impl<UserEdgeData> Default for EdgeData<UserEdgeData>
where
    UserEdgeData: Default,
{
    fn default() -> Self {
        EdgeData {
            user_data: Default::default(),
        }
    }
}

/// The main force graph structure.
pub struct ForceGraph<UserNodeData = (), UserEdgeData = ()> {
    pub parameters: SimulationParameters,
    graph: StableUnGraph<Node<UserNodeData>, EdgeData<UserEdgeData>>,
    node_indices: BTreeSet<NodeIndex>,
}

impl<UserNodeData, UserEdgeData> ForceGraph<UserNodeData, UserEdgeData> {
    /// Constructs a new force graph.
    ///
    /// Use the following syntax to create a graph with default parameters:
    /// ```
    /// use simula_core::force_graph::ForceGraph;
    /// let graph = <ForceGraph>::new(Default::default());
    /// ```
    pub fn new(parameters: SimulationParameters) -> Self {
        ForceGraph {
            parameters,
            graph: StableUnGraph::default(),
            node_indices: Default::default(),
        }
    }

    /// Provides access to the raw graph structure if required.
    pub fn get_graph(&self) -> &StableUnGraph<Node<UserNodeData>, EdgeData<UserEdgeData>> {
        &self.graph
    }

    /// Provides access to the raw graph structure if required.
    pub fn get_graph_mut(
        &mut self,
    ) -> &mut StableUnGraph<Node<UserNodeData>, EdgeData<UserEdgeData>> {
        &mut self.graph
    }

    /// Adds a new node and returns an index that can be used to reference the node.
    pub fn add_node(&mut self, node_data: NodeData<UserNodeData>) -> NodeIndex {
        let inner_idx = self.graph.add_node(Node {
            data: node_data,
            index: Default::default(),
            velocity: Vec3::ZERO,
            accel: Vec3::ZERO,
        });
        let idx = NodeIndex(inner_idx);
        self.graph[inner_idx].index = idx;
        self.node_indices.insert(idx);
        idx
    }

    /// Removes a node by index.
    pub fn remove_node(&mut self, idx: NodeIndex) {
        self.graph.remove_node(*idx);
        self.node_indices.remove(&idx);
    }

    /// Adds or updates an edge connecting two nodes by index.
    pub fn add_edge(&mut self, n1_idx: NodeIndex, n2_idx: NodeIndex, edge: EdgeData<UserEdgeData>) {
        self.graph.update_edge(*n1_idx, *n2_idx, edge);
    }

    /// Removes all nodes from the force graph.
    pub fn clear(&mut self) {
        self.graph.clear();
        self.node_indices.clear();
    }

    /// Applies the next step of the force graph simulation.
    ///
    /// The number of seconds that have elapsed since the previous update must be calculated and
    /// provided by the user as `dt`.
    pub fn update(&mut self, dt: Duration) {
        if self.graph.node_count() == 0 {
            return;
        }

        let dt = dt.as_secs_f32().min(self.parameters.max_dt);
        let dt = dt / self.parameters.iterations as f32;
        if !(dt > 0.0) {
            return;
        }

        for _ in 0..self.parameters.iterations {
            for (n1_idx_i, n1_idx) in self.node_indices.iter().enumerate() {
                let mut edges = self.graph.neighbors(**n1_idx).detach();
                while let Some(n2_idx) = edges.next_node(&self.graph) {
                    let (n1, n2) = self.graph.index_twice_mut(**n1_idx, n2_idx);
                    let f = attract_nodes(n1, n2, &self.parameters);
                    n1.apply_force(f, dt, &self.parameters);
                }

                for n2_idx in self.node_indices.iter().skip(n1_idx_i + 1) {
                    let (n1, n2) = self.graph.index_twice_mut(**n1_idx, **n2_idx);
                    let f = repel_nodes(n1, n2, &self.parameters);
                    if !n1.data.is_anchor {
                        n1.apply_force(f, dt, &self.parameters);
                    }
                    if !n2.data.is_anchor {
                        n2.apply_force(-f, dt, &self.parameters);
                    }
                }

                let n1 = &mut self.graph[**n1_idx];
                if !n1.data.is_anchor {
                    n1.update(dt, &self.parameters);
                }
            }
        }
    }

    /// Processes each node with a user-defined callback `cb`.
    pub fn visit_nodes<F: FnMut(&Node<UserNodeData>)>(&self, mut cb: F) {
        for n_idx in self.graph.node_indices() {
            cb(&self.graph[n_idx]);
        }
    }

    /// Mutates each node with a user-defined callback `cb`.
    pub fn visit_nodes_mut<F: FnMut(&mut Node<UserNodeData>)>(&mut self, mut cb: F) {
        for node in self.graph.node_weights_mut() {
            cb(node);
        }
    }

    /// Processes each edge and its associated nodes with a user-defined callback `cb`.
    pub fn visit_edges<
        F: FnMut(&Node<UserNodeData>, &Node<UserNodeData>, &EdgeData<UserEdgeData>),
    >(
        &self,
        mut cb: F,
    ) {
        for edge_ref in self.graph.edge_references() {
            let source = &self.graph[edge_ref.source()];
            let target = &self.graph[edge_ref.target()];
            let edge_data = edge_ref.weight();
            cb(source, target, edge_data);
        }
    }
}

/// References a node in the [ForceGraph]. Can not be constructed by the user.
pub struct Node<UserNodeData = ()> {
    /// The node data provided by the user.
    pub data: NodeData<UserNodeData>,
    index: NodeIndex,
    velocity: Vec3,
    accel: Vec3,
}

impl<UserNodeData> Node<UserNodeData> {
    /// The position of the node.
    pub fn position(&self) -> Vec3 {
        self.data.position
    }

    /// Set position of the node.
    pub fn set_position(&mut self, position: Vec3) {
        self.data.position = position;
    }

    /// The index used to reference the node in the [ForceGraph].
    pub fn index(&self) -> NodeIndex {
        self.index
    }

    fn apply_force(&mut self, force: Vec3, dt: f32, parameters: &SimulationParameters) {
        self.accel += force
            .max(Vec3::splat(-parameters.force_max))
            .min(Vec3::splat(parameters.force_max))
            * dt;
    }

    fn update(&mut self, dt: f32, parameters: &SimulationParameters) {
        self.velocity =
            (self.velocity + self.accel * dt * parameters.node_speed) * parameters.damping_factor;
        self.data.position += self.velocity * dt;
        self.accel = Vec3::ZERO;
    }
}

fn attract_nodes<D>(n1: &Node<D>, n2: &Node<D>, parameters: &SimulationParameters) -> Vec3 {
    let mut d = n2.data.position - n1.data.position;
    let distance = d.length();
    if distance.abs() < f32::EPSILON {
        return Vec3::ONE;
    }
    d /= distance;
    let strength = 1.0 * parameters.force_spring * distance * 0.5;
    d * strength
}

fn repel_nodes<D>(n1: &Node<D>, n2: &Node<D>, parameters: &SimulationParameters) -> Vec3 {
    let mut d = n2.data.position - n1.data.position;
    let distance = d.length();
    if distance.abs() < f32::EPSILON {
        return Vec3::ONE;
    }
    d /= distance;
    let distance_sqrd = distance * distance;
    let strength = -parameters.force_charge * ((n1.data.mass * n2.data.mass) / distance_sqrd);
    d * strength
}
