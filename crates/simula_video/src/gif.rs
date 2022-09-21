use crate::VideoPlayer;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    utils::{BoxedFuture, HashMap},
};
use image::{codecs::gif::GifDecoder, AnimationDecoder, ImageBuffer, Rgba};

#[derive(Debug, TypeUuid)]
#[uuid = "ea16c586-3962-11ed-bc41-02a179e5df2c"]
pub struct GifAsset {
    pub path: Box<std::path::Path>,
    pub frames: Vec<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    pub images: HashMap<usize, Handle<Image>>,
}

#[derive(Default)]
pub struct GifAssetLoader;

impl AssetLoader for GifAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let gif = GifDecoder::new(bytes)?;
            let frames = gif.into_frames();
            let frames = frames.collect_frames()?;
            let frames = frames.into_iter().map(|frame| frame.into_buffer());
            let frames = frames.collect::<Vec<_>>();
            let asset = GifAsset {
                path: load_context.path().into(),
                frames,
                images: HashMap::default(),
            };

            load_context.set_default_asset(LoadedAsset::new(asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["gif"]
    }
}

pub fn run(
    mut gifs: ResMut<Assets<GifAsset>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    videos: Query<(&VideoPlayer, &Handle<GifAsset>, &Handle<StandardMaterial>)>,
) {
    for (video, asset, material) in videos.iter() {
        if let Some(material) = materials.get_mut(&material) {
            if let Some(gif) = gifs.get_mut(asset) {
                if let Some(image) = gif.images.get(&video.current_frame) {
                    material.base_color_texture = Some(image.clone());
                } else {
                    if video.current_frame <= gif.frames.len() {
                        debug!("video image: {:?} {:?}", gif.path, video);
                        let frame = gif.frames[video.current_frame].clone();
                        let image = images.add(Image::new_fill(
                            Extent3d {
                                width: frame.width(),
                                height: frame.height(),
                                depth_or_array_layers: 1,
                            },
                            TextureDimension::D2,
                            &frame.into_raw(),
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
