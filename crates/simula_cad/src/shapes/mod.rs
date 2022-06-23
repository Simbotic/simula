use bevy::prelude::*;

mod star;

pub use star::*;

pub trait ShapeMesh {
    fn to_mesh(&self) -> Mesh;
}
