#[cfg(feature = "gst")]
pub use crate::{gst_sink::GstSink, gst_src::GstSrc};
use bevy::prelude::*;
#[cfg(feature = "gif")]
pub use gif::{GifAsset, GifAssetLoader};
#[cfg(feature = "webp")]
pub use webp::{WebPAsset, WebPAssetLoader};

#[cfg(feature = "gif")]
mod gif;
#[cfg(feature = "gst")]
mod gst_sink;
#[cfg(feature = "gst")]
mod gst_src;
#[cfg(feature = "webp")]
mod webp;

pub mod rt;

#[derive(Default, Debug, Component, Reflect)]
#[reflect(Component)]
pub struct VideoPlayer {
    pub start_frame: usize,
    pub end_frame: usize,
    pub current_frame: usize,
    pub frame_time: f32,
    pub framerate: f32,
    pub playing: bool,
}

pub struct VideoPlugin;

impl Plugin for VideoPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<VideoPlayer>().add_system(run);

        #[cfg(feature = "gif")]
        app.add_asset::<GifAsset>()
            .init_asset_loader::<GifAssetLoader>()
            .add_system(gif::run);

        #[cfg(feature = "webp")]
        app.add_asset::<WebPAsset>()
            .init_asset_loader::<WebPAssetLoader>()
            .add_system(webp::run);

        #[cfg(feature = "gst")]
        app.add_startup_system(gst_sink::setup)
            .add_system(gst_sink::stream)
            .add_system(gst_sink::launch);

        #[cfg(feature = "gst")]
        app.add_startup_system(gst_src::setup)
            .add_system(gst_src::stream)
            .add_system(gst_src::launch);
    }
}

impl VideoPlayer {
    fn update(&mut self, delta_time: f32) {
        if self.playing {
            self.frame_time += delta_time;
            if self.current_frame < self.start_frame {
                self.current_frame = self.start_frame;
            }
            if self.frame_time >= 1.0 / self.framerate {
                self.frame_time = 0.0;
                self.current_frame += 1;
                if self.current_frame > self.end_frame {
                    self.current_frame = self.start_frame;
                }
            }
        }
    }
}

fn run(time: Res<Time>, mut videos: Query<&mut VideoPlayer>) {
    for mut video in videos.iter_mut() {
        video.update(time.delta_seconds());
    }
}
