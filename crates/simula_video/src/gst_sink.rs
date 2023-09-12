use crate::VideoMaterial;
use bevy::{
    asset::Error,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use crossbeam_channel::{bounded, Receiver, Sender};
use derive_more::{Display, Error};
use gst::element_error;
use gst::prelude::*;
use gstreamer as gst;
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;

#[derive(Component, Clone)]
pub struct GstSink {
    pub pipeline: String,
    pub size: UVec2,
}

impl Default for GstSink {
    fn default() -> Self {
        Self {
            pipeline: "videotestsrc ! appsink name=simula".to_string(),
            size: UVec2::new(512, 512),
        }
    }
}

#[derive(Component)]
pub struct GstSinkProcess {
    pub process: std::thread::JoinHandle<()>,
    receiver: Receiver<Vec<u8>>,
}

#[derive(Debug, Display, Error)]
#[display(fmt = "Missing element {}", _0)]
struct MissingElement(#[error(not(source))] &'static str);

#[derive(Debug, Display, Error)]
#[display(fmt = "Received error from {}: {} (debug: {:?})", src, error, debug)]
struct ErrorMessage {
    src: String,
    error: String,
    debug: Option<glib::GString>,
}

pub(crate) fn setup_gst_sink() {}

pub(crate) fn stream_gst_sinks(
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<VideoMaterial>>,
    videos: Query<(
        &GstSinkProcess,
        &Handle<VideoMaterial>,
        &ComputedVisibility,
        &GstSink,
    )>,
) {
    for (process, material, visibility, sink) in videos.iter() {
        if !visibility.is_visible() {
            continue;
        }
        if let Ok(data) = process.receiver.try_recv() {
            let mut material = materials.get_mut(&material).unwrap();
            let image = Image::new(
                Extent3d {
                    width: sink.size.x,
                    height: sink.size.y,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                data,
                TextureFormat::Rgba8UnormSrgb,
            );
            let image_handle = images.add(image);
            material.alpha_scaler = 1.0;
            material.video_texture = Some(image_handle);
        }
    }
}

pub(crate) fn launch_gst_sinks(
    mut commands: Commands,
    sinks: Query<(Entity, &GstSink), Without<GstSinkProcess>>,
) {
    for (entity, sink) in sinks.iter() {
        let (sender, receiver) = bounded(1);
        let sink = sink.clone();
        let launch_handle = std::thread::spawn(move || {
            match create_pipeline(sink, sender).and_then(pipeline_loop) {
                Ok(r) => r,
                Err(e) => eprintln!("Error! {}", e),
            }
        });
        commands.entity(entity).insert(GstSinkProcess {
            process: launch_handle,
            receiver,
        });
    }
}

fn create_pipeline(sink: GstSink, sender: Sender<Vec<u8>>) -> Result<gst::Pipeline, Error> {
    gst::init()?;

    let mut context = gst::ParseContext::new();
    let pipeline = gst::parse_launch_full(
        &sink.pipeline.clone(),
        Some(&mut context),
        gst::ParseFlags::empty(),
    )?;

    let pipeline = pipeline.dynamic_cast::<gst::Pipeline>().unwrap();

    let appsink = pipeline
        .by_name("simula")
        .ok_or_else(|| MissingElement("simula"))?
        .dynamic_cast::<gst_app::AppSink>()
        .unwrap();

    let width = sink.size.x as i32;
    let height = sink.size.y as i32;

    // set video caps
    let caps = gst::Caps::builder("video/x-raw")
        .field("format", &"RGBA")
        .field("width", &width)
        .field("height", &height)
        .build();
    appsink.set_caps(Some(&caps));

    // create app sink callbacks
    let callbacks = gst_app::AppSinkCallbacks::builder()
        .new_sample(move |appsink| {
            let sample = appsink.pull_sample().unwrap();
            let buffer = sample.buffer().unwrap();

            let caps = sample.caps().expect("Sample without caps");
            let video_info = gst_video::VideoInfo::from_caps(caps).expect("Failed to parse caps");

            // At this point, buffer is only a reference to an existing memory region somewhere.
            // When we want to access its content, we have to map it while requesting the required
            // mode of access (read, read/write).
            // This type of abstraction is necessary, because the buffer in question might not be
            // on the machine's main memory itself, but rather in the GPU's memory.
            // So mapping the buffer makes the underlying memory region accessible to us.
            // See: https://gstreamer.freedesktop.org/documentation/plugin-development/advanced/allocation.html
            let frame = gst_video::VideoFrameRef::from_buffer_ref_readable(buffer, &video_info)
                .map_err(|_| {
                    element_error!(
                        appsink,
                        gst::ResourceError::Failed,
                        ("Failed to map buffer readable")
                    );

                    gst::FlowError::Error
                })?;

            // Now we can access the buffer's content.
            // The frame's data is a slice of planes, each plane being a slice of bytes.
            // The data is laid out in memory in the order of the planes.
            // For a RGBA frame, the first plane is the RGBA plane.
            // See: https://gstreamer.freedesktop.org/documentation/plugin-development/advanced/allocation.html
            let data = frame.plane_data(0).unwrap();
            sender.send(data.to_vec()).unwrap();

            Ok(gst::FlowSuccess::Ok)
        })
        .build();

    appsink.set_callbacks(callbacks);

    Ok(pipeline)
}

fn pipeline_loop(pipeline: gst::Pipeline) -> Result<(), Error> {
    pipeline.set_state(gst::State::Playing)?;

    let bus = pipeline
        .bus()
        .expect("Pipeline without bus. Shouldn't happen!");

    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;
        eprintln!("{:?} \n", msg );

        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                pipeline.set_state(gst::State::Null)?;
                return Err(ErrorMessage {
                    src: msg
                        .src()
                        .map(|s| String::from(s.path_string()))
                        .unwrap_or_else(|| String::from("None")),
                    error: err.error().to_string(),
                    debug: err.debug(),
                }
                .into());
            }
            _ => (),
        }
    }

    pipeline.set_state(gst::State::Null)?;

    Ok(())
}
