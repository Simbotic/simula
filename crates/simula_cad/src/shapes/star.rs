use crate::shapes::ShapeMesh;
use bevy::{prelude::*, render::render_resource::PrimitiveTopology};
use std::f64::consts::PI;

pub struct Star {
    sweep: fj::Sweep,
}

pub fn star(num_points: u64, color: Color) -> Star {
    let r1 = 1.;
    let r2 = 2.;
    let h = 1.;

    // We need to figure out where to generate vertices, depending on the number
    // of points the star is supposed to have. Let's generate an iterator that
    // gives us the angle and radius for each vertex.
    let num_vertices = num_points * 2;
    let vertex_iter = (0..num_vertices).map(|i| {
        let angle = fj::Angle::from_rad(2. * PI / num_vertices as f64 * i as f64);
        let radius = if i % 2 == 0 { r1 } else { r2 };
        (angle, radius)
    });

    // Now that we got that iterator prepared, generating the vertices is just a
    // bit of trigonometry.
    let mut outer = Vec::new();
    let mut inner = Vec::new();
    for (angle, radius) in vertex_iter {
        let (sin, cos) = angle.rad().sin_cos();

        let x = cos * radius;
        let y = sin * radius;

        outer.push([x, y]);
        inner.push([x / 2., y / 2.]);
    }

    let color = color.as_rgba_u32().to_be_bytes();

    let outer = fj::Sketch::from_points(outer).unwrap().with_color(color);
    let inner = fj::Sketch::from_points(inner).unwrap();

    let footprint = fj::Difference2d::from_shapes([outer.into(), inner.into()]);

    let star = fj::Sweep::from_path(footprint.into(), [0., 0., h]);

    Star { sweep: star }
}

impl ShapeMesh for Star {
    fn to_mesh(&self) -> Mesh {
        let shape_processor = fj_operations::shape_processor::ShapeProcessor { tolerance: None };
        let shape: fj::Shape = self.sweep.clone().into();
        let processed_shape = shape_processor.process(&shape).unwrap();

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        let points = processed_shape
            .mesh
            .triangles()
            .map(|triangle| triangle.inner.points())
            .collect::<Vec<_>>();

        let positions: Vec<[f32; 3]> = points
            .iter()
            .flat_map(|points| points.map(|point| point.coords.components.map(|s| s.into_f32())))
            .collect();

        let normals: Vec<[f32; 3]> = points
            .iter()
            .map(|&points| points.into())
            .map(|triangle: fj_math::Triangle<3>| triangle.normal())
            .map(|vector| vector.components.map(|s| s.into_f32()))
            .flat_map(|n| [n, n, n])
            .collect();

        let uvs: Vec<[f32; 2]> = points
            .iter()
            .flat_map(|points| points.map(|point| point.coords.components.map(|s| s.into_f32())))
            .map(|vs| [vs[0], vs[1]])
            .collect();

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        mesh
    }
}
