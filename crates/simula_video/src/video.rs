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
    let video = web_sys::window()
        .and_then(|window| {
            window
                .document()
                .and_then(|document| document.create_element("video").ok())
                .and_then(|video| video.dyn_into::<web_sys::HtmlVideoElement>().ok())
        })
        .and_then(|video| {
            video.set_id(&video_id.to_string());
            video.set_autoplay(true);
            video.set_loop(true);
            video.set_src(&src.src);
            video.set_controls(true);
            Some(video)
        });

    let canvas = web_sys::window()
        .and_then(|window| {
            window
                .document()
                .and_then(|document| document.create_element("canvas").ok())
                .and_then(|canvas| canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok())
        })
        .and_then(|canvas| {
            canvas.set_id(&canva_id.to_string());
            canvas.set_width(src.size.x);
            canvas.set_height(src.size.y);
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
    let mut tags = vec![];
    let mut videos = world.query_filtered::<(Entity, &VideoSrc), Without<VideoTag>>();
    for (entity, src) in videos.iter(world) {
        if let Some(video_canvas) = video_canvas(src) {
            tags.push((entity, video_canvas));
        }
    }

    for (entity, video_canvas) in tags {
        world.entity_mut(entity).insert(VideoTag);
        let mut video_res = world.get_non_send_resource_mut::<VideoResource>().unwrap();

        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .append_child(&video_canvas.video)
            .unwrap();

        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .body()
            .unwrap()
            .append_child(&video_canvas.canvas)
            .unwrap();

        video_res.videos.insert(entity, video_canvas);
    }
}

pub(crate) fn blit_videos_to_canvas(world: &mut World) {
    let mut videos = world.query_filtered::<(Entity, &VideoSrc), With<VideoTag>>();

    let videos: Vec<(Entity, UVec2)> = videos
        .iter(world)
        .map(|(entity, src)| (entity, src.size))
        .collect();

    for (entity, size) in videos {
        let videos = world.get_non_send_resource::<VideoResource>().unwrap();
        let video_canvas = videos.videos.get(&entity).unwrap();

        let mut attribs = web_sys::ContextAttributes2d::new();
        attribs.will_read_frequently(true);

        let ctx = video_canvas
            .canvas
            .get_context_with_context_options("2d", &attribs)
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        ctx.draw_image_with_html_video_element(&video_canvas.video, 0.0, 0.0)
            .unwrap();

        let img_data = ctx
            .get_image_data(0.0, 0.0, size.x as f64, size.y as f64)
            .unwrap();
        let data = img_data.data();

        let mut images = world.get_resource_mut::<Assets<Image>>().unwrap();
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

        let material = world
            .get::<Handle<StandardMaterial>>(entity)
            .unwrap()
            .clone();
        let mut materials = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        let material = materials.get_mut(&material).unwrap();
        material.base_color_texture = Some(image.clone());
    }
}
