use crate::VideoPlayer;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    utils::{BoxedFuture, HashMap},
};
use image::{codecs::webp::WebPDecoder, AnimationDecoder, ImageBuffer, Rgba};

#[derive(Debug, TypeUuid)]
#[uuid = "3747c102-39bd-11ed-a320-02a179e5df2c"]
pub struct WebPAsset {
    pub path: String,
    pub frames: Vec<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    pub images: HashMap<usize, Handle<Image>>,
}

#[derive(Default)]
pub struct WebPAssetLoader;

impl AssetLoader for WebPAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let gif = WebPDecoder::new(bytes)?;
            let frames = gif.into_frames();
            let frames = frames.collect_frames()?;
            let frames = frames.into_iter().map(|frame| frame.into_buffer());
            let frames = frames.collect::<Vec<_>>();
            let asset = WebPAsset {
                path: load_context.path().display().to_string(),
                frames,
                images: HashMap::default(),
            };
            load_context.set_default_asset(LoadedAsset::new(asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["webp"]
    }
}

pub fn run(
    mut gifs: ResMut<Assets<WebPAsset>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    videos: Query<(
        &VideoPlayer,
        &Handle<WebPAsset>,
        &Handle<StandardMaterial>,
        &ComputedVisibility,
    )>,
) {
    for (video, asset, material, visibility) in videos.iter() {
        if !visibility.is_visible() {
            continue;
        }
        if let Some(material) = materials.get_mut(&material) {
            if let Some(gif) = gifs.get_mut(asset) {
                if let Some(image) = gif.images.get(&video.current_frame) {
                    material.base_color_texture = Some(image.clone());
                } else {
                    if video.current_frame <= gif.frames.len() {
                        debug!("video image: {:?} {:?}", gif.path, video);
                        let frame = gif.frames[video.current_frame].clone();
                        let image = images.add(Image::new(
                            Extent3d {
                                width: frame.width(),
                                height: frame.height(),
                                depth_or_array_layers: 1,
                            },
                            TextureDimension::D2,
                            frame.into_raw(),
                            TextureFormat::Rgba8UnormSrgb,
                        ));
                        gif.images.insert(video.current_frame, image.clone());
                        material.base_color_texture = Some(image);
                    }
                }
            }
        }
    }
}
