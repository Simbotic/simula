use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
};

impl Material for VideoMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/video_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

#[derive(Asset, TypePath, AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "afbf5e64-6f79-11ed-9551-02a179e5df2a"]
pub struct VideoMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub alpha_scaler: f32,
    #[texture(1)]
    #[sampler(2)]
    pub video_texture: Option<Handle<Image>>,
    pub alpha_mode: AlphaMode,
}

impl Default for VideoMaterial {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            alpha_scaler: 0.0,
            video_texture: None,
            alpha_mode: AlphaMode::Opaque,
        }
    }
}
