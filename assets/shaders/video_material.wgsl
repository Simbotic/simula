struct VideoMaterial {
    color: vec4<f32>,
    alpha_scaler: f32,
};

@group(1) @binding(0)
var<uniform> material: VideoMaterial;
@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let x_flipped_uv = vec2<f32>(1.0 - uv.x, uv.y);
    let tex_color = textureSample(base_color_texture, base_color_sampler, x_flipped_uv);
    let color = material.color.rgb * tex_color.rgb;
    let alpha = material.color.a * tex_color.a * material.alpha_scaler;
    return vec4<f32>(color.rgb, alpha);
}
