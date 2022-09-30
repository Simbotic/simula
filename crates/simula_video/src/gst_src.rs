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

#[derive(Component)]
pub struct GstSrc {
    pub pipeline: String,
    pub image: Handle<Image>,
}

impl Default for GstSrc {
    fn default() -> Self {
        Self {
            pipeline: "appsrc name=simula ! videoconvert ! autovideosink".to_string(),
            image: Default::default(),
        }
    }
}

#[derive(Component)]
pub struct GstSrcProcess {
    pub process: std::thread::JoinHandle<()>,
    sender: Sender<(Time, Vec<u8>)>,
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

pub fn setup() {}

pub fn stream(
    mut dummy: Local<Handle<Image>>,
    mut commands: Commands,
    time: Res<Time>,
    images: Res<Assets<Image>>,
    videos: Query<(&GstSrcProcess, &GstSrc)>,
    asset_manager: Res<AssetServer>,
) {
    // Copy texture to buffer

    
    // if *dummy == Default::default() {
    //     *dummy = asset_manager.load("textures/metric_512x512.png");
    // }
    // for (process, src) in videos.iter() {
    //     if let Some(image) = images.get(&dummy.clone()) {
            
    //         let data = image.data.clone();
    //         println!("Sending {} bytes", data.len());
    //         let _ = process.sender.try_send((time.clone(), data));
    //     }
    // }
    
    for (process, src) in videos.iter() {
        if let Some(image) = images.get(&src.image) {
            let data = image.data.clone();
            println!("Sending {} bytes", data.len());
            let _ = process.sender.try_send((time.clone(), data));
        }
    }



    // for (process, src) in videos.iter() {
    //     if let Some(image) = images.get(&src.image) {
            
    //         let data = image.data.clone();
    //         println!("Sending {} bytes", data.len());
    //         let _ = process.sender.try_send((time.clone(), data));
    //     }
    // }
}

pub fn launch(mut commands: Commands, srcs: Query<(Entity, &GstSrc), Without<GstSrcProcess>>) {
    for (entity, src) in srcs.iter() {
        let (sender, receiver) = bounded(1);
        let pipeline = src.pipeline.clone();
        let launch_handle = std::thread::spawn(move || {
            match create_pipeline(pipeline, receiver).and_then(pipeline_loop) {
                Ok(r) => r,
                Err(e) => eprintln!("Error! {}", e),
            }
        });
        commands.entity(entity).insert(GstSrcProcess {
            process: launch_handle,
            sender,
        });
    }
}

fn create_pipeline(
    pipeline_str: String,
    receiver: Receiver<(Time, Vec<u8>)>,
) -> Result<gst::Pipeline, Error> {
    gst::init()?;

    let mut context = gst::ParseContext::new();
    let pipeline =
        gst::parse_launch_full(&pipeline_str, Some(&mut context), gst::ParseFlags::empty())?;

    let pipeline = pipeline.dynamic_cast::<gst::Pipeline>().unwrap();

    let appsrc = pipeline
        .by_name("simula")
        .ok_or_else(|| MissingElement("simula"))?
        .dynamic_cast::<gst_app::AppSrc>()
        .unwrap();

    // set video caps
    let caps = gst::Caps::builder("video/x-raw")
        .field("format", &"BGRA")
        .field("width", &512)
        .field("height", &512)
        .field("framerate", &gst::Fraction::new(30, 1))
        .build();
    appsrc.set_caps(Some(&caps));
    appsrc.set_format(gst::Format::Time);
    appsrc.set_is_live(true);

    let video_info = gst_video::VideoInfo::from_caps(&caps).expect("Failed to parse caps");

    let pipeline_time = Time::default();

    // loop {
    //     match receiver.recv() {
    //         Ok((time, data)) => {
    //             let mut buffer = gst::Buffer::with_size(video_info.size()).unwrap();
    //             {
    //                 let buffer = buffer.get_mut().unwrap();

    //                 // let pipeline_elapsed = time.seconds_since_startup() - pipeline_time.seconds_since_startup();

    //                 let ms = ((pipeline_time.seconds_since_startup()) * 1000.0) as u64;
    //                 buffer.set_pts(gst::ClockTime::from_mseconds(ms));
    //                 buffer.set_dts(gst::ClockTime::from_mseconds(ms));
    //                 // buffer.set_duration(gst::ClockTime::from_mseconds(1000 / 60));
    //                 // //
    //                 // // buffer.set_pts(ms * gst::ClockTime::MSECOND);
    //                 let mut map = buffer.map_writable().unwrap();
    //                 let payload = map.as_mut_slice();
    //                 payload.copy_from_slice(&data);
    //             }
    //             // appsrc already handles the error here
    //             println!("Pushing buffer");
    //             let _ = appsrc.push_buffer(buffer);
    //         }
    //         Err(e) => {
    //             element_error!(
    //                 appsrc,
    //                 gst::CoreError::Failed,
    //                 ("Failed to receive data: {}", e)
    //             );
    //             break;
    //         }
    //     }
    // }

    let pipeline_start = std::time::Instant::now();

    let callbacks = gst_app::AppSrcCallbacks::builder()
        .need_data(move |appsrc, _length| {
            let mut buffer = gst::Buffer::with_size(video_info.size()).unwrap();
            match receiver.recv() {
                Ok((time, data)) => {
                    let buffer = buffer.get_mut().unwrap();
                    // let secs_since_pipeline_start =
                    //     time.time_since_startup() - pipeline_start.elapsed();
                    // let ms = (secs_since_pipeline_start * 1000.0) as u64;
                    let ms = pipeline_start.elapsed().as_millis() as u64;
                    buffer.set_pts(gst::ClockTime::from_mseconds(ms));
                    // buffer.set_dts(gst::ClockTime::from_mseconds(ms));
                    let mut map = buffer.map_writable().unwrap();
                    let payload = map.as_mut_slice();
                    payload.copy_from_slice(&data)
                }
                Err(e) => {
                    element_error!(
                        appsrc,
                        gst::CoreError::Failed,
                        ("Failed to receive data: {}", e)
                    );
                }
            }
            // appsrc already handles the error here
            println!("Pushing buffer");
            let _ = appsrc.push_buffer(buffer);
        })
        .build();

    appsrc.set_callbacks(callbacks);

    Ok(pipeline)
}

fn pipeline_loop(pipeline: gst::Pipeline) -> Result<(), Error> {
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
