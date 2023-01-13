use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, Mesh},
        render_resource::PrimitiveTopology,
    },
};

// SDF - Signed Distance Field plugin

struct SDFPlugin;

impl Plugin for SDFPlugin {
    fn build(&self, _app: &mut App) {}
}

trait SDF {
    fn density(&self, p: Vec3) -> f32;

    fn round(&self, p: Vec3, radius: f32) -> f32 {
        self.density(p) - radius
    }

    fn normal(&self, p: Vec3) -> Vec3 {
        let eps = 0.001;
        let x =
            self.density(p + Vec3::new(eps, 0.0, 0.0)) - self.density(p - Vec3::new(eps, 0.0, 0.0));
        let y =
            self.density(p + Vec3::new(0.0, eps, 0.0)) - self.density(p - Vec3::new(0.0, eps, 0.0));
        let z =
            self.density(p + Vec3::new(0.0, 0.0, eps)) - self.density(p - Vec3::new(0.0, 0.0, eps));
        Vec3::new(x, y, z).normalize()
    }

    fn position(&self, p: Vec3) -> Vec3 {
        p - self.normal(p) * self.density(p)
    }

    fn uv(&self, p: Vec3) -> Vec2 {
        let n = self.normal(p);
        Vec2::new(
            0.5 + (n.x.atan2(n.z) / (2.0 * std::f32::consts::PI)),
            0.5 - n.y.acos() / std::f32::consts::PI,
        )
    }

    fn mesh(&self, resolution: Vec3, padding: f32) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut i = 0;
        for x in 0..resolution.x as usize {
            for y in 0..resolution.y as usize {
                for z in 0..resolution.z as usize {
                    let p = Vec3::new(
                        x as f32 / resolution.x as f32 - 0.5,
                        y as f32 / resolution.y as f32 - 0.5,
                        z as f32 / resolution.z as f32 - 0.5,
                    );

                    if self.density(p) < padding {
                        continue;
                    }

                    let n = self.normal(p);
                    let uv = self.uv(p);

                    vertices.push(p);
                    normals.push(n);
                    uvs.push(uv);

                    if x > 0 && y > 0 && z > 0 {
                        let p0 = i
                            - resolution.x as usize * resolution.y as usize
                            - resolution.x as usize
                            - 1;
                        let p1 = i
                            - resolution.x as usize * resolution.y as usize
                            - resolution.x as usize;
                        let p2 = i - resolution.x as usize * resolution.y as usize;
                        let p3 = i - resolution.x as usize * resolution.y as usize - 1;
                        let p4 = i - resolution.x as usize - 1;
                        let p5 = i - resolution.x as usize;
                        let p6 = i;
                        let p7 = i - 1;

                        indices.push(p0 as u32);
                        indices.push(p1 as u32);
                        indices.push(p2 as u32);
                        indices.push(p3 as u32);
                        indices.push(p4 as u32);
                        indices.push(p5 as u32);
                        indices.push(p6 as u32);
                        indices.push(p7 as u32);
                    }

                    i += 1;
                }
            }
        }
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh
    }

    // transformations

    fn translate(&self, p: Vec3, t: Vec3) -> f32 {
        self.density(p - t)
    }

    fn rotate(&self, p: Vec3, r: Quat) -> f32 {
        self.density(r * p)
    }

    fn scale(&self, p: Vec3, s: Vec3) -> f32 {
        self.density(p / s)
    }

    // combinations

    fn union(&self, other: &dyn SDF, p: Vec3) -> f32 {
        self.density(p).min(other.density(p))
    }

    fn subtraction(&self, other: &dyn SDF, p: Vec3) -> f32 {
        self.density(p).max(-other.density(p))
    }

    fn intersection(&self, other: &dyn SDF, p: Vec3) -> f32 {
        self.density(p).max(other.density(p))
    }

    // smooth combinations

    fn smooth_union(&self, other: &dyn SDF, p: Vec3, k: f32) -> f32 {
        let h = (self.density(p).max(other.density(p)) / k).exp();
        (h * self.density(p) + h * other.density(p)) / (h + 1.0)
    }

    fn smooth_subtraction(&self, other: &dyn SDF, p: Vec3, k: f32) -> f32 {
        let h = (self.density(p).max(-other.density(p)) / k).exp();
        (h * self.density(p) - h * other.density(p)) / (h + 1.0)
    }

    fn smooth_intersection(&self, other: &dyn SDF, p: Vec3, k: f32) -> f32 {
        let h = (self.density(p).max(other.density(p)) / k).exp();
        (h * self.density(p) * h * other.density(p)) / (h + 1.0)
    }
}

// impl Into<Mesh> for &dyn SDF {
//     fn into(self) -> Mesh {
//         let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
//         mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new());
//         mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<[f32; 3]>::new());
//         mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<[f32; 2]>::new());
//         mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, Vec::<[f32; 4]>::new());
//         mesh
//     }
// }

pub struct Box {
    pub size: Vec3,
}

impl SDF for Box {
    // float sdBox( vec3 p, vec3 b )
    // {
    //   vec3 q = abs(p) - b;
    //   return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0);
    // }
    fn density(&self, p: Vec3) -> f32 {
        let q = p.abs() - self.size;
        q.length().max(0.0) + q.x.max(q.y.max(q.z)).min(0.0)
    }
}

pub struct Sphere {
    pub radius: f32,
}

impl SDF for Sphere {
    // float sdSphere( vec3 p, float s )
    // {
    //   return length(p)-s;
    // }
    fn density(&self, p: Vec3) -> f32 {
        p.length() - self.radius
    }
}

pub struct CappedCylinder {
    pub radius: f32,
    pub height: f32,
}

impl SDF for CappedCylinder {
    // float sdCappedCylinder( vec3 p, float h, float r )
    // {
    //   vec2 d = abs(vec2(length(p.xz),p.y)) - vec2(r,h);
    //   return min(max(d.x,d.y),0.0) + length(max(d,0.0));
    // }
    fn density(&self, p: Vec3) -> f32 {
        let d = Vec2::new(p.x, p.z).length() - self.radius;
        let d = Vec2::new(d, p.y).abs() - Vec2::new(self.radius, self.height);
        d.x.max(d.y).min(0.0) + d.length().max(0.0)
    }
}
