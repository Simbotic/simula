#import bevy_pbr::mesh_view_bind_group
#import bevy_pbr::mesh_struct

// struct VoxelsMaterial;
// [[group(1), binding(0)]]
// var<uniform> material: VoxelsMaterial;

[[group(2), binding(0)]]
var<uniform> mesh: Mesh;

struct Vertex {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
    [[location(3)]] color: u32;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] world_normal: vec3<f32>;
    [[location(2)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vertex(vertex: Vertex) -> VertexOutput {
    let world_position = mesh.model * vec4<f32>(vertex.position, 1.0);
    var out: VertexOutput;
    out.world_position = world_position;
    out.clip_position = view.view_proj * world_position;

    let world_normal = mat3x3<f32>(
        mesh.inverse_transpose_model[0].xyz,
        mesh.inverse_transpose_model[1].xyz,
        mesh.inverse_transpose_model[2].xyz
    ) * vertex.normal;
    out.world_normal = world_normal;

    var color = vec4<f32>((vec4<u32>(vertex.color) >> vec4<u32>(0u, 8u, 16u, 24u)) & vec4<u32>(255u)) / 255.0;
    color = vec4<f32>(color.rgb * (dot(world_normal, normalize(vec3<f32>(0.2, 1.0, 0.1))) * 0.25 + 0.75), color.a);
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