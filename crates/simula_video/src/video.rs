use crate::material::VideoMaterial;
use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    utils::HashMap,
};
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys;

#[derive(Debug, Component, Clone)]
pub struct VideoSrc {
    pub src: String,
    pub size: UVec2,
    pub playing: bool,
    pub _loop: bool,
}

#[derive(Component)]
pub struct VideoTag;

#[derive(Clone)]
pub struct VideoCanvas {
    pub video: web_sys::HtmlVideoElement,
    pub canvas: web_sys::HtmlCanvasElement,
    pub image: Handle<Image>,
}

#[derive(Default)]
pub struct VideoResource {
    pub videos: HashMap<Entity, VideoCanvas>,
}

fn video_canvas(src: &VideoSrc) -> Option<VideoCanvas> {
    let video_id = Uuid::new_v4();
    let canva_id = Uuid::new_v4();

    // Create video element
    let video = web_sys::window()
        .and_then(|window| {
            window
                .document()
                .and_then(|document| document.create_element("video").ok())
                .and_then(|video| video.dyn_into::<web_sys::HtmlVideoElement>().ok())
        })
        .and_then(|video| {
            // Hide video element
            video.set_hidden(true);
            video
                .style()
                .set_property("display", "none")
                .unwrap_or_default();
            video.set_id(&video_id.to_string());
            video.set_autoplay(false);
            video.set_loop(false);
            video.set_src(&src.src);
            video.set_controls(true);

            // Add video to DOM
            web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| document.body())
                .and_then(|body| body.append_child(&video).ok());

            Some(video)
        });

    // Create canvas element video will render to
    let canvas = web_sys::window()
        .and_then(|window| {
            window
                .document()
                .and_then(|document| document.create_element("canvas").ok())
                .and_then(|canvas| canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        })
        .and_then(|canvas| {
            // Hide canvas element
            canvas.set_hidden(true);
            canvas
                .style()
                .set_property("display", "none")
                .unwrap_or_default();
            canvas.set_id(&canva_id.to_string());
            canvas.set_width(src.size.x);
            canvas.set_height(src.size.y);

            // Add canvas to DOM
            web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| document.body())
                .and_then(|body| body.append_child(&canvas).ok());

            Some(canvas)
        });

    if let (Some(video), Some(canvas)) = (video, canvas) {
        Some(VideoCanvas {
            video,
            canvas,
            image: Handle::default(),
        })
    } else {
        None
    }
}

pub(crate) fn setup(world: &mut World) {
    world.init_non_send_resource::<VideoResource>();
}

pub(crate) fn setup_video_tags(world: &mut World) {
    // Find entities without VideoTag and insert VideoTag
    let mut tags = vec![];
    let mut videos = world.query_filtered::<(Entity, &VideoSrc), Without<VideoTag>>();
    for (entity, src) in videos.iter(world) {
        tags.push((entity, video_canvas(src)));
    }
    for (entity, video_canvas) in tags {
        world.entity_mut(entity).insert(VideoTag);
        if let Some(video_canvas) = video_canvas {
            let video_res = world.get_non_send_resource_mut::<VideoResource>();
            if let Some(mut video_res) = video_res {
                video_res.videos.insert(entity, video_canvas);
            }
        }
    }
}

pub(crate) fn blit_videos_to_canvas(world: &mut World) {
    let mut videos = world
        .query_filtered::<(Entity, &VideoSrc), (With<VideoTag>, With<Handle<VideoMaterial>>)>();

    let videos: Vec<(Entity, UVec2)> = videos
        .iter(world)
        .map(|(entity, src)| (entity, src.size))
        .collect();

    for (entity, size) in videos {
        let videos = world.get_non_send_resource::<VideoResource>();
        if let Some(videos) = videos {
            let video_canvas = videos.videos.get(&entity);
            if let Some(video_canvas) = video_canvas {
                // Get the 2d context of the canvas and make sure it's optimized for fast reading
                let mut attribs = web_sys::ContextAttributes2d::new();
                attribs.will_read_frequently(true);
                let ctx = video_canvas
                    .canvas
                    .get_context_with_context_options("2d", &attribs);

                // Copy the video frame to the canvas
                if let Ok(Some(ctx)) = ctx {
                    let ctx = ctx.dyn_into::<web_sys::CanvasRenderingContext2d>();
                    if let Ok(ctx) = ctx {
                        if ctx
                            .draw_image_with_html_video_element(&video_canvas.video, 0.0, 0.0)
                            .is_err()
                        {
                            error!("Error drawing video to canvas");
                            continue;
                        }

                        // Extract the image data from the canvas
                        let img_data = ctx.get_image_data(0.0, 0.0, size.x as f64, size.y as f64);
                        if let Ok(img_data) = img_data {
                            let data = img_data.data();

                            // Resources should have image assets
                            let mut images = world.get_resource_mut::<Assets<Image>>().unwrap();

                            // Create a new image asset for this video frame
                            let image = images.add(Image::new(
                                Extent3d {
                                    width: size.x,
                                    height: size.y,
                                    depth_or_array_layers: 1,
                                },
                                TextureDimension::D2,
                                data.to_vec(),
                                TextureFormat::Rgba8UnormSrgb,
                            ));

                            // Update the material with the new image, world query assures this exists
                            let material =
                                world.get::<Handle<VideoMaterial>>(entity).unwrap().clone();
                            let materials = world.get_resource_mut::<Assets<VideoMaterial>>();
                            if let Some(mut materials) = materials {
                                let material = materials.get_mut(&material);
                                if let Some(material) = material {
                                    material.video_texture = Some(image.clone());
                                    material.alpha_scaler = 1.0;
                                }
                            }
                        } else {
                            error!("Error getting image data from canvas");
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn update_video_state(world: &mut World) {
    let mut videos = world
        .query_filtered::<(Entity, &VideoSrc), (With<VideoTag>, With<Handle<VideoMaterial>>)>();
    let videos: Vec<(Entity, VideoSrc)> = videos
        .iter(world)
        .map(|(entity, src)| (entity, src.clone()))
        .collect();

    for (entity, src) in videos.iter() {
        let video_res = world.get_non_send_resource_mut::<VideoResource>();
        if let Some(mut video_res) = video_res {
            let video_canvas = video_res.videos.get_mut(&entity);
            if let Some(video_canvas) = video_canvas {
                if src.playing {
                    let _ = video_canvas.video.play();
                } else {
                    let _ = video_canvas.video.pause();
                }

                video_canvas.video.set_loop(src._loop);
            }
        }
    }
}
