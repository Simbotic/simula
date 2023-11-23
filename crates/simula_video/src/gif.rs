use std::io::Cursor;

use crate::{VideoMaterial, VideoPlayer};
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    utils::{BoxedFuture, HashMap},
};
use image::{codecs::gif::GifDecoder, AnimationDecoder, ImageBuffer, Rgba};
use thiserror::Error;

#[derive(Asset, TypePath, Debug, TypeUuid)]
#[uuid = "ea16c586-3962-11ed-bc41-02a179e5df2c"]
pub struct GifAsset {
    pub path: String,
    pub frames: Vec<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    pub images: HashMap<usize, Handle<Image>>,
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum GifAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    // An [ImageError](image::ImageError) Error
    #[error("Could not parse asset as image: {0}")]
    Image(#[from] image::ImageError),
}

#[derive(Default)]
pub struct GifAssetLoader;

impl AssetLoader for GifAssetLoader {
    type Asset = GifAsset;
    type Settings = ();
    type Error = GifAssetLoaderError;
    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let cursor: Cursor<Vec<u8>> = Cursor::new(bytes);
            let gif = GifDecoder::new(cursor)?;
            let frames = gif.into_frames();
            let frames = frames.collect_frames()?;
            let frames = frames.into_iter().map(|frame| frame.into_buffer());
            let frames = frames.collect::<Vec<_>>();
            Ok(GifAsset {
                path: load_context.path().display().to_string(),
                frames,
                images: HashMap::default(),
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["gif"]
    }
}

pub fn run(
    mut gifs: ResMut<Assets<GifAsset>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<VideoMaterial>>,
    videos: Query<(
        &VideoPlayer,
        &Handle<GifAsset>,
        &Handle<VideoMaterial>,
        &InheritedVisibility,
        &ViewVisibility,
    )>,
) {
    for (video, asset, material, inherited_visibility, view_visibility) in videos.iter() {
        if !inherited_visibility.get() && !view_visibility.get() {
            continue;
        }
        if let Some(material) = materials.get_mut(material) {
            if let Some(gif) = gifs.get_mut(asset) {
                if let Some(image) = gif.images.get(&video.current_frame) {
                    material.video_texture = Some(image.clone());
                } else if video.current_frame <= gif.frames.len() {
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
                    material.alpha_scaler = 1.0;
                    material.video_texture = Some(image);
                }
            }
        }
    }
}
