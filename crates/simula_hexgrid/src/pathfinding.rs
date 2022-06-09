// //use crate::helpers::node_distance;
// //use crate::helpers::node_neighbours_offset;
// //use crate::helpers::offset_to_cubic;
// //use crate::HexOrientation;
// use ::std::collections::HashMap;
// use core::panic;
// use bevy::prelude::*;

pub enum HexOrientation {
    FlatTopOddUp,
}

/// Determines a score to rank a chosen path, lower scores are better
pub fn a_star_score(complexity: f32, weighting: f32) -> f32 {
    complexity + weighting
}

/// Finds a nodes weight based on the number of 'jumps' you'd have to make from
/// your current node to the end node. For the Offset grid we cannot compute the
/// number of jumps directly, instead we have to convert the Offset coordinates
/// of our nodes to the Cubic based coordinate system.
pub fn calculate_node_weight(
    current_node: &(i32, i32),
    end_node: &(i32, i32),
    orientation: &HexOrientation,
) -> f32 {
    let cubic_start = offset_to_cubic((current_node.0, current_node.1), orientation);
    let cubic_end = offset_to_cubic((end_node.0, end_node.1), orientation);
    // by finding the distance between nodes we're effectively finding the 'ring' it sits on which is the number of jumps to it
    node_distance(cubic_start, cubic_end) as f32
}

pub fn offset_to_cubic(node_coords: (i32, i32), orientation: &HexOrientation) -> (i32, i32, i32) {
    match orientation {
        HexOrientation::FlatTopOddUp => {
            let x: i32 = node_coords.0;
            let z: i32 = node_coords.1 - (node_coords.0 - (node_coords.0 & 1)) / 2;
            let y: i32 = -x - z;
            (x, y, z)
        }
    }
}

/// The distance between two nodes by using cubic coordinates
pub fn node_distance(start: (i32, i32, i32), end: (i32, i32, i32)) -> i32 {
    ((start.0 - end.0).abs() + (start.1 - end.1).abs() + (start.2 - end.2).abs()) / 2
}

pub fn node_neighbours_offset(
    source: (i32, i32),
    _orientation: &HexOrientation,
    min_column: i32,
    max_column: i32,
    min_row: i32,
    max_row: i32,
) -> Vec<(i32, i32)> {
    let mut neighbours = Vec::new();
    // starting from north round a tile clockwise
    // even column
    if source.0 & 1 == 0 {
        // north
        if source.1 + 1 < max_row {
            neighbours.push((source.0, source.1 + 1));
        };
        // north-east
        if source.0 + 1 < max_column {
            neighbours.push((source.0 + 1, source.1));
        };
        // south-east
        if source.0 + 1 < max_column && source.1 - 1 > min_row {
            neighbours.push((source.0 + 1, source.1 - 1));
        };
        // south
        if source.1 - 1 > min_row {
            neighbours.push((source.0, source.1 - 1));
        };
        // south-west
        if source.0 - 1 > min_column && source.1 - 1 > min_row {
            neighbours.push((source.0 - 1, source.1 - 1));
        }
        // north-west
        if source.0 - 1 > min_column {
            neighbours.push((source.0 - 1, source.1));
        }
    } else {
        // odd column
        // north
        if source.1 + 1 < max_row {
            neighbours.push((source.0, source.1 + 1));
        }
        // north-east
        if source.0 + 1 < max_column && source.1 + 1 < max_row {
            neighbours.push((source.0 + 1, source.1 + 1))
        }
        // south-east
        if source.0 + 1 < max_column {
            neighbours.push((source.0 + 1, source.1));
        }
        // south
        if source.1 - 1 > min_row {
            neighbours.push((source.0, source.1 - 1));
        }
        // south-west
        if source.0 - 1 < max_column {
            neighbours.push((source.0 - 1, source.1));
        }
        // north-east
        if source.0 - 1 < max_column && source.1 + 1 < max_row {
            neighbours.push((source.0 - 1, source.1 + 1))
        }
    }
    neighbours
}
