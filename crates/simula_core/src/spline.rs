use bevy::prelude::*;

#[derive(Component)]
pub struct Spline {
    pub segments: Vec<Curve>,
}

impl Default for Spline {
    fn default() -> Self {
        Self::new()
    }
}

impl Spline {
    pub fn new() -> Self {
        Self {
            segments: vec![Curve::new(
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.3, 0.0, 0.0),
                Vec3::new(0.7, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
            )],
        }
    }

    pub fn from_points(points: Vec<Vec3>) -> Self {
        let mut segments = vec![];
        for i in (0..points.len()).step_by(3) {
            if i + 3 >= points.len() {
                break;
            }
            segments.push(Curve::new(
                points[i],
                points[i + 1],
                points[i + 2],
                points[i + 3],
            ));
        }
        Self { segments }
    }

    // TODO: align and mirror functions

    pub fn add_segment(&mut self, segment: Curve) {
        self.segments.push(segment);
    }

    // TODO: Handle case where there are no segments
    pub fn get_segment(&self, t: f32) -> usize {
        let segment = (t * self.segments.len() as f32).floor() as usize;
        if segment >= self.segments.len() {
            if self.segments.len() == 1 {
                return 0;
            } else {
                return self.segments.len() - 1;
            }
        }
        segment
    }

    pub fn get_point(&self, t: f32) -> Vec3 {
        let segment = self.get_segment(t);
        let t = t * self.segments.len() as f32 - segment as f32;
        self.segments[segment].get_point(t)
    }

    pub fn get_tangent(&self, t: f32) -> Vec3 {
        let segment = self.get_segment(t);
        let t = t * self.segments.len() as f32 - segment as f32;
        self.segments[segment].get_tangent(t)
    }

    pub fn get_normal(&self, t: f32) -> Vec3 {
        let segment = self.get_segment(t);
        let t = t * self.segments.len() as f32 - segment as f32;
        self.segments[segment].get_normal(t)
    }

    pub fn get_binormal(&self, t: f32) -> Vec3 {
        let segment = self.get_segment(t);
        let t = t * self.segments.len() as f32 - segment as f32;
        self.segments[segment].get_binormal(t)
    }

    pub fn get_frame(&self, t: f32) -> Mat4 {
        let segment = self.get_segment(t);
        let t = t * self.segments.len() as f32 - segment as f32;
        self.segments[segment].get_frame(t)
    }

    pub fn get_length(&self) -> f32 {
        let mut length = 0.0;
        for segment in self.segments.iter() {
            length += segment.get_length();
        }
        length
    }

    pub fn get_length_at(&self, t: f32) -> f32 {
        let segment = self.get_segment(t);
        let t = t * self.segments.len() as f32 - segment as f32;
        let mut length = 0.0;
        for i in 0..segment {
            length += self.segments[i].get_length();
        }
        length += self.segments[segment].get_length_at(t);
        length
    }

    pub fn get_t_at_length(&self, length: f32) -> f32 {
        let mut length = length;
        for i in 0..self.segments.len() {
            let segment_length = self.segments[i].get_length();
            if length < segment_length {
                return i as f32 / self.segments.len() as f32
                    + length / segment_length / self.segments.len() as f32;
            }
            length -= segment_length;
        }
        1.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Curve {
    pub p0: Vec3,
    pub p1: Vec3,
    pub p2: Vec3,
    pub p3: Vec3,
}

impl Curve {
    pub fn new(point_a: Vec3, point_b: Vec3, point_c: Vec3, point_d: Vec3) -> Self {
        Self {
            p0: point_a,
            p1: point_b,
            p2: point_c,
            p3: point_d,
        }
    }

    pub fn get_point(&self, t: f32) -> Vec3 {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        self.p0 * mt3 + self.p1 * 3.0 * mt2 * t + self.p2 * 3.0 * mt * t2 + self.p3 * t3
    }

    pub fn get_tangent(&self, t: f32) -> Vec3 {
        let t2 = t * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        self.p0 * -3.0 * mt2
            + self.p1 * (3.0 * mt2 - 6.0 * mt * t)
            + self.p2 * (6.0 * mt * t - 3.0 * t2)
            + self.p3 * 3.0 * t2
    }

    pub fn get_normal(&self, t: f32) -> Vec3 {
        let tangent = self.get_tangent(t);
        let normal = tangent.cross(Vec3::Y);
        normal.normalize()
    }

    pub fn get_binormal(&self, t: f32) -> Vec3 {
        let tangent = self.get_tangent(t);
        let normal = self.get_normal(t);
        let binormal = normal.cross(tangent);
        binormal.normalize()
    }

    pub fn get_frame(&self, t: f32) -> Mat4 {
        let tangent = -self.get_tangent(t).normalize();
        let normal = self.get_normal(t).normalize();
        let binormal = self.get_binormal(t).normalize();
        let point = self.get_point(t);
        Mat4::from_cols(
            Vec4::new(normal.x, normal.y, normal.z, 0.0),
            Vec4::new(binormal.x, binormal.y, binormal.z, 0.0),
            Vec4::new(tangent.x, tangent.y, tangent.z, 0.0),
            Vec4::new(point.x, point.y, point.z, 1.0),
        )
    }

    pub fn get_length(&self) -> f32 {
        let mut length = 0.0;
        let mut last_point = self.get_point(0.0);
        for i in 1..=100 {
            let t = i as f32 / 100.0;
            let point = self.get_point(t);
            length += (point - last_point).length();
            last_point = point;
        }
        length
    }

    pub fn get_length_at(&self, t: f32) -> f32 {
        let mut length = 0.0;
        let mut last_point = self.get_point(0.0);
        for i in 1..=(t * 100.0) as usize {
            let t = i as f32 / 100.0;
            let point = self.get_point(t);
            length += (point - last_point).length();
            last_point = point;
        }
        length
    }

    pub fn get_t_at_length(&self, length: f32) -> f32 {
        let mut current_length = 0.0;
        let mut last_point = self.get_point(0.0);
        for i in 1..=100 {
            let t = i as f32 / 100.0;
            let point = self.get_point(t);
            current_length += (point - last_point).length();
            if current_length >= length {
                return t;
            }
            last_point = point;
        }
        1.0
    }

    pub fn get_t_at_point(&self, point: Vec3) -> f32 {
        let mut closest_t = 0.0;
        let mut closest_distance = std::f32::MAX;
        for i in 0..=100 {
            let t = i as f32 / 100.0;
            let curve_point = self.get_point(t);
            let distance = (curve_point - point).length();
            if distance < closest_distance {
                closest_distance = distance;
                closest_t = t;
            }
        }
        closest_t
    }

    pub fn get_speed_at(&self, t: f32) -> f32 {
        let tangent = self.get_tangent(t);
        tangent.length()
    }
}
