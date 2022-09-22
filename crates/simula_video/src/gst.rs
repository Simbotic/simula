// This example demonstrates the use of the appsink element.
// It operates the following pipeline:

// {audiotestsrc} - {appsink}

// The application specifies what format it wants to handle. This format
// is applied by calling set_caps on the appsink. Now it's the audiotestsrc's
// task to provide this data format. If the element connected to the appsink's
// sink-pad were not able to provide what we ask them to, this would fail.
// This is the format we request:
// Audio / Signed 16bit / 1 channel / arbitrary sample rate

use gst::element_error;
use gst::prelude::*;
use gstreamer as gst;
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;

use crate::VideoPlayer;
use bevy::{
    asset::Error,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use crossbeam_channel::{bounded, Receiver, Sender};
use derive_more::{Display, Error};

#[derive(Component)]
pub struct GstHolder {
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
    debug: Option<String>,
    source: glib::Error,
}

pub fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let video_material = StandardMaterial {
        base_color: Color::rgb(1.0, 1.0, 1.0),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..Default::default()
    };
    let gst_stuff = gst_main();
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
            material: materials.add(video_material),
            transform: Transform::from_xyz(2.0, 0.5, -3.0)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)).with_scale(Vec3::new(2.0, 1.0, 1.0)),
            ..Default::default()
        })
        .insert(VideoPlayer {
            start_frame: 0,
            end_frame: 80,
            framerate: 20.0,
            playing: true,
            ..Default::default()
        })
        .insert(gst_stuff)
        .insert(Name::new("Video: Gst"));
}

pub fn run(
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    videos: Query<(&GstHolder, &Handle<StandardMaterial>, &ComputedVisibility)>,
) {
    for (video, material, visibility) in videos.iter() {
        if !visibility.is_visible() {
            continue;
        }
        if let Ok(data) = video.receiver.try_recv() {
            let mut material = materials.get_mut(&material).unwrap();
            let image = Image::new_fill(
                Extent3d {
                    width: 512,
                    height: 512,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                &data,
                TextureFormat::Rgba8UnormSrgb,
            );
            let image_handle = images.add(image);
            material.base_color_texture = Some(image_handle);
        }
    }
}

fn create_pipeline(pipeline_str: String, sender: Sender<Vec<u8>>) -> Result<gst::Pipeline, Error> {
    gst::init()?;

    let mut context = gst::ParseContext::new();
    let pipeline =
        gst::parse_launch_full(&pipeline_str, Some(&mut context), gst::ParseFlags::empty())?;

    let pipeline = pipeline.dynamic_cast::<gst::Pipeline>().unwrap();

    let sink = pipeline
        .by_name("simula")
        .ok_or_else(|| MissingElement("simula"))?
        .dynamic_cast::<gst_app::AppSink>()
        .unwrap();

    // set video caps
    let caps = gst::Caps::builder("video/x-raw")
        .field("format", &"RGBA")
        .field("width", &512)
        .field("height", &512)
        .build();
    sink.set_caps(Some(&caps));

    // create app sink callbacks
    let callbacks = gst_app::AppSinkCallbacks::builder()
        .new_sample(move |appsink| {
            let sample = appsink.pull_sample().unwrap();
            let buffer = sample.buffer().unwrap();

            let caps = sample.caps().expect("Sample without caps");
            let info = gst_video::VideoInfo::from_caps(caps).expect("Failed to parse caps");

            // At this point, buffer is only a reference to an existing memory region somewhere.
            // When we want to access its content, we have to map it while requesting the required
            // mode of access (read, read/write).
            // This type of abstraction is necessary, because the buffer in question might not be
            // on the machine's main memory itself, but rather in the GPU's memory.
            // So mapping the buffer makes the underlying memory region accessible to us.
            // See: https://gstreamer.freedesktop.org/documentation/plugin-development/advanced/allocation.html
            let frame = gst_video::VideoFrameRef::from_buffer_ref_readable(buffer, &info).map_err(
                |_| {
                    element_error!(
                        appsink,
                        gst::ResourceError::Failed,
                        ("Failed to map buffer readable")
                    );

                    gst::FlowError::Error
                },
            )?;

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

    sink.set_callbacks(callbacks);

    Ok(pipeline)
}

fn main_loop(pipeline: gst::Pipeline) -> Result<(), Error> {
    pipeline.set_state(gst::State::Playing)?;

    let bus = pipeline
        .bus()
        .expect("Pipeline without bus. Shouldn't happen!");

    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;

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
                    source: err.error(),
                }
                .into());
            }
            _ => (),
        }
    }

    pipeline.set_state(gst::State::Null)?;

    Ok(())
}

pub fn gst_main() -> GstHolder {
    let (sender, receiver) = bounded(1);
    let launch_handle = std::thread::spawn(move || {
        match create_pipeline(
            "filesrc num-buffers=1000 location=assets/videos/sample-01.mkv ! decodebin ! videoconvert ! videoscale ! video/x-raw,format=RGBA,width=512,height=512 ! appsink name=simula".to_string(),
            // "videotestsrc ! video/x-raw,width=512,height=512 ! appsink name=simula".into(),
            sender,
        )
        .and_then(main_loop)
        {
            Ok(r) => r,
            Err(e) => eprintln!("Error! {}", e),
        }
    });

    GstHolder {
        process: launch_handle,
        receiver,
    }
}
