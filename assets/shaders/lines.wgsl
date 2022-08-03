#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_types

// struct LinesMaterial;
// [[group(1), binding(0)]]
// var<uniform> material: LinesMaterial;

@group(2) @binding(0)
var<uniform> mesh: Mesh;

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let world_position = mesh.model * vec4<f32>(vertex.position, 1.0);
    out.clip_position = view.view_proj * world_position;

    //let color = vec4<f32>((vec4<u32>(vertex.color) >> vec4<u32>(0u, 8u, 16u, 24u)) & vec4<u32>(255u)) / 255.0;
    let color = vertex.color;
    out.color = color;

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color.rgb, 1.0);
}
