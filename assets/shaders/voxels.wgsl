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
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] color: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] color: vec4<f32>;
};

[[group(2), binding(0)]]
var<uniform> model: Model;

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> VertexOutput {
    let world_position = model.model * vec4<f32>(vertex.position, 1.0);
    var out: VertexOutput;
    out.world_position = world_position;
    out.clip_position = view.view_proj * world_position;

    let world_normal = mat3x3<f32>(
        model.inverse_transpose_model[0].xyz,
        model.inverse_transpose_model[1].xyz,
        model.inverse_transpose_model[2].xyz
    ) * vertex.normal;
    out.world_normal = world_normal;

    let color = vec4<f32>(vertex.color.rgb * (dot(world_normal, normalize(vec3<f32>(0.2, 1.0, 0.1))) * 0.25 + 0.75), vertex.color.a);
    out.color = color;

    return out;
}

//  1.0 / 17.0,  9.0 / 17.0,  3.0 / 17.0, 11.0 / 17.0
// 13.0 / 17.0,  5.0 / 17.0, 15.0 / 17.0,  7.0 / 17.0
//  4.0 / 17.0, 12.0 / 17.0,  2.0 / 17.0, 10.0 / 17.0
// 16.0 / 17.0,  8.0 / 17.0, 14.0 / 17.0,  6.0 / 17.0

struct FragmentInput {
    [[builtin(front_facing)]] is_front: bool;
    [[builtin(position)]] frag_coord: vec4<f32>;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] color: vec4<f32>;
};

[[stage(fragment)]]
fn fragment(in: FragmentInput) -> [[location(0)]] vec4<f32> {

    let threshold = array<array<f32, 4>, 4>(
        array<f32, 4>( 1.0 / 17.0,  9.0 / 17.0,  3.0 / 17.0, 11.0 / 17.0),
        array<f32, 4>(13.0 / 17.0,  5.0 / 17.0, 15.0 / 17.0,  7.0 / 17.0),
        array<f32, 4>( 4.0 / 17.0, 12.0 / 17.0,  2.0 / 17.0, 10.0 / 17.0),
        array<f32, 4>(16.0 / 17.0,  8.0 / 17.0, 14.0 / 17.0,  6.0 / 17.0)
    );

    let x = u32(in.frag_coord.x % 4.0);
    let y = u32(in.frag_coord.y % 4.0);
    let alpha = in.color.a - threshold[0][0];
	if (alpha < 0.0) {
		discard;
	}

    return in.color;
}