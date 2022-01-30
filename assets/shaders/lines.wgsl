struct View {
    view_proj: mat4x4<f32>;
    inverse_view: mat4x4<f32>;
    projection: mat4x4<f32>;
    world_position: vec3<f32>;
    near: f32;
    far: f32;
    width: f32;
    height: f32;
};

struct Model {
    model: mat4x4<f32>;
    inverse_transpose_model: mat4x4<f32>;
    flags: u32;
};

[[group(0), binding(0)]]
var<uniform> view: View;

struct Vertex {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] color: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

[[group(2), binding(0)]]
var<uniform> model: Model;

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> VertexOutput {
    let world_position = model.model * vec4<f32>(vertex.position, 1.0);
    var out: VertexOutput;
    out.clip_position = view.view_proj * world_position;
    out.color = vertex.color;
    return out;
}

[[stage(fragment)]]
fn fragment(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(in.color.rgb, 1.0);
}
