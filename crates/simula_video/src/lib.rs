#[cfg(feature = "gst")]
pub use crate::{gst_sink::GstSink, gst_src::GstSrc};
use bevy::prelude::*;
#[cfg(feature = "gif")]
pub use gif::{GifAsset, GifAssetLoader};
pub use material::VideoMaterial;
pub use raw_src::RawSrc;
#[cfg(feature = "video")]
pub use video::VideoSrc;
#[cfg(feature = "webp")]
pub use webp::{WebPAsset, WebPAssetLoader};

#[cfg(feature = "gif")]
mod gif;
#[cfg(feature = "gst")]
mod gst_sink;
#[cfg(feature = "gst")]
mod gst_src;
#[cfg(feature = "video")]
pub mod video;
#[cfg(feature = "webp")]
mod webp;

pub mod material;
pub mod raw_src;
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
        app.add_plugins(MaterialPlugin::<VideoMaterial>::default())
            .register_type::<VideoPlayer>()
            .add_systems(Update, run);

        app.add_systems(Startup, raw_src::setup_raw_src)
            .add_systems(Update, (raw_src::setup_raw_srcs, raw_src::process_raw_srcs));
        raw_src::setup_render_graph(app);

        #[cfg(feature = "gif")]
        app.init_asset::<GifAsset>()
            .init_asset_loader::<GifAssetLoader>()
            .add_systems(Update, gif::run);

        #[cfg(feature = "webp")]
        app.init_asset::<WebPAsset>()
            .init_asset_loader::<WebPAssetLoader>()
            .add_systems(Update, webp::run);

        #[cfg(feature = "gst")]
        {
            app.add_systems(Startup, gst_sink::setup_gst_sink)
                .add_systems(
                    Update,
                    (gst_sink::stream_gst_sinks, gst_sink::launch_gst_sinks),
                );

            app.add_systems(Startup, gst_src::setup_gst_src)
                .add_systems(Update, (gst_src::stream_gst_srcs, gst_src::launch_gst_srcs));
        }

        #[cfg(feature = "video")]
        app.add_systems(Startup, video::setup).add_systems(
            Update,
            (
                video::setup_video_tags,
                video::blit_videos_to_canvas,
                video::update_video_state,
                video::detect_video_removal,
            )
                .chain(),
        );
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
