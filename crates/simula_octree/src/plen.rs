use bevy::prelude::*;

// PlenOctreeNode represents a node in the PlenOctree data structure.
pub struct PlenOctreeNode {
    // Optional array of child nodes.
    children: Option<[Box<PlenOctreeNode>; 8]>,
    // Optional Vec3 data contained in the node.
    data: Option<Vec3>,
    // Axis-aligned bounding box of the node.
    bounds: Aabb,
}

// Aabb represents an axis-aligned bounding box.
#[derive(Clone, Copy, Debug, Default)]
pub struct Aabb {
    pub center: Vec3,
    pub half_extents: Vec3,
}

impl Aabb {
    // Creates a new Aabb with the given center and half_extents.
    pub fn new(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            center,
            half_extents,
        }
    }

    // Calculates the minimum corner of the bounding box.
    pub fn min(&self) -> Vec3 {
        self.center - self.half_extents
    }

    // Calculates the maximum corner of the bounding box.
    pub fn max(&self) -> Vec3 {
        self.center + self.half_extents
    }

    // Checks if the bounding box intersects another bounding box.
    pub fn intersects_aabb(&self, other: &Aabb) -> bool {
        let a_min = self.min();
        let a_max = self.max();
        let b_min = other.min();
        let b_max = other.max();

        a_min.x < b_max.x
            && a_max.x > b_min.x
            && a_min.y < b_max.y
            && a_max.y > b_min.y
            && a_min.z < b_max.z
            && a_max.z > b_min.z
    }

    // Checks if the bounding box contains a point.
    pub fn contains_point(&self, point: &Vec3) -> bool {
        let min = self.min();
        let max = self.max();

        point.x >= min.x
            && point.x <= max.x
            && point.y >= min.y
            && point.y <= max.y
            && point.z >= min.z
            && point.z <= max.z
    }
}

impl PlenOctreeNode {
    // Create a new PlenOctreeNode
    pub fn new(bounds: Aabb) -> Self {
        Self {
            children: None,
            data: None,
            bounds,
        }
    }

    // Inserts a Vec3 data point into the PlenOctreeNode.
    pub fn insert(&mut self, data: Vec3) {
        let octant = self.calculate_octant(&data);
        if let Some(children) = &mut self.children {
            children[octant as usize].insert(data);
        } else if let Some(existing_data) = self.data {
            if data != existing_data {
                self.split();
                self.insert(existing_data.clone());
                self.insert(data);
            }
        } else {
            self.data = Some(data);
        }
    }

    // Splits the node into 8 child nodes.
    fn split(&mut self) {
        let children_bounds = self.calculate_child_bounds();
        let children = children_bounds
            .iter()
            .map(|&bounds| Box::new(PlenOctreeNode::new(bounds)))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_else(|_| panic!("Failed to create child nodes."));
        self.children = Some(children);
        self.data = None;
    }

    // Calculates the octant of a Vec3 data point.
    fn calculate_octant(&self, data: &Vec3) -> u8 {
        let center = self.bounds.center;
        let mut octant = 0;
        if data.x >= center.x {
            octant |= 1;
        }
        if data.y >= center.y {
            octant |= 2;
        }
        if data.z >= center.z {
            octant |= 4;
        }
        octant
    }

    // Calculates the bounds of the 8 child nodes.
    fn calculate_child_bounds(&self) -> [Aabb; 8] {
        let min = self.bounds.min();
        let center = self.bounds.center;
        let mut child_bounds = [Aabb::default(); 8];
        for i in 0..8 {
            let x = if i & 1 == 0 { min.x } else { center.x };
            let y = if i & 2 == 0 { min.y } else { center.y };
            let z = if i & 4 == 0 { min.z } else { center.z };
            let child_min = Vec3::new(x, y, z);
            let child_max = child_min + (center - min);
            child_bounds[i] = Aabb::new(child_min, child_max);
        }
        child_bounds
    }

    // Queries the PlenOctreeNode for data points within a range.
    pub fn query_range(&self, query_bounds: &Aabb, results: &mut Vec<Vec3>) {
        if !self.bounds.intersects_aabb(query_bounds) {
            return;
        }
        if let Some(data) = &self.data {
            if query_bounds.contains_point(data) {
                results.push(data.clone());
            }
        } else if let Some(children) = &self.children {
            for child in children.iter() {
                child.query_range(query_bounds, results);
            }
        }
    }
}

pub struct PlenOctree {
    root: PlenOctreeNode,
}

impl PlenOctree {
    pub fn new(bounds: Aabb) -> Self {
        Self {
            root: PlenOctreeNode::new(bounds),
        }
    }

    pub fn insert(&mut self, data: Vec3) {
        self.root.insert(data);
    }

    pub fn query_range(&self, query_bounds: &Aabb) -> Vec<Vec3> {
        let mut results = Vec::new();
        self.root.query_range(query_bounds, &mut results);
        results
    }
}
