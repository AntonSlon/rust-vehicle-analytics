use car_detector::cli::Cli;
use car_detector::constants::{FPS, HEIGHT, WIDTH};
use car_detector::streamer::{StreamMessage, VideoStreamer};
use car_detector::tracker::YoloTracker;
use car_detector::util::update_fps;
use clap::Parser;
use gstreamer::prelude::ElementExt;
use std::time::{Instant};
use ultralytics_inference::io::VideoWriter;

fn main() {
    let args = Cli::parse();

    if let Some(runtime_path) = args.onnx_runtime_path.as_deref() {
        ort::init_from(runtime_path)
            .unwrap_or_else(|error| {
                panic!(
                    "failed to load ONNX Runtime from '{}': {error}",
                    runtime_path.display()
                )
            })
            .commit();
    }

    let pipeline = if args.live_stream {
        "autovideosrc ! videoconvert ! videoscale ! video/x-raw,format=RGB,width=1920,height=1080 ! appsink name=mysink".to_string()
    } else {
        let input_path = args
            .input_video_path
            .as_deref()
            .expect("--input-video-path is required unless --live-stream is used")
            .replace('\\', "\\\\")
            .replace('"', "\\\"");

        format!(
            "filesrc location=\"{input_path}\" ! qtdemux ! h264parse ! avdec_h264 ! videoconvert ! video/x-raw,format=RGB ! appsink name=mysink"
        )
    };

    let streamer = VideoStreamer::new(&pipeline).unwrap();

    streamer.start();
    
    let first_sample = loop {
        match streamer.receiver.recv().unwrap() {
            StreamMessage::Sample(sample) => break sample,
            StreamMessage::Eos => return,
            }
        };

    let caps = first_sample.caps().expect("sample has no caps");
    let structure = caps.structure(0).expect("caps have no structure");
    
    let w = structure.get::<i32>("width").unwrap() as u32;
    let h = structure.get::<i32>("height").unwrap() as u32;
    let fraction = structure.get::<gstreamer::Fraction>("framerate").unwrap();
    let fps = fraction.numer() as f32 / fraction.denom() as f32;
    println!("fps: {fps}");
    println!("{w} {h}");

    let mut model = YoloTracker::new(args.model_name.as_str()).unwrap();
    let mut writer = VideoWriter::new(args.output_video_path, w as usize, h as usize, fps).unwrap();
    
    let (preview_pipeline, app_src) = streamer.get_app_src(w, h, fps).unwrap();
    let state = if args.live_stream {
        gstreamer::State::Playing
    }else {
        gstreamer::State::Null
    };
    preview_pipeline
        .set_state(state)
        .unwrap();

    loop {
        match streamer.receiver.recv() {
            Ok(StreamMessage::Sample(sample)) => {
                if let Some(buffer) = sample.buffer() {
                    if let Ok(buffer_readable) = buffer.map_readable() {
                        let frame_bytes = buffer_readable.as_slice();
                        let structure = sample.caps().unwrap().structure(0).unwrap();

                        let time = if let Some(pts) = buffer.pts() {
                            Some(pts.seconds_f32())
                        }else {
                            None
                        };

                        let w: i32 = structure.get("width").unwrap();
                        let h: i32 = structure.get("height").unwrap();

                        let instant = Instant::now();
                        let img = model.run(w as u32, h as u32, frame_bytes, time).unwrap();
                        let frame_time = instant.elapsed().as_secs_f32();

                        let fps = if model.frame_cnt == 1 {
                            1 as f32 / frame_time
                        }
                        else {
                            let smoothed_fps = model.fps;
                            let current_fps = 1 as f32 / frame_time;
                            update_fps(smoothed_fps, current_fps)
                        };

                        model.fps = fps;

                        //let instant_write_frame = time::Instant::now();

                        //if args.live_stream {
                        writer.write_frame(&img).unwrap();
                        //}

                        // println!(
                        //     "write frame {}",
                        //     instant_write_frame.elapsed().as_secs_f32()
                        // );

                        if state == gstreamer::State::Playing {
                             let slice = img.into_bytes();
                            let gst_img = gstreamer::Buffer::from_mut_slice(slice);
                            app_src.push_buffer(gst_img).unwrap();
                        }
                    }
                }
            }
            Ok(StreamMessage::Eos) => {
                streamer.stop();
                break;
            }
            Err(_) => {
                break;
            }
        }
    }
    println!("End frames");
    streamer.stop();
    writer.finish().unwrap();
    model
        .traffic_counter
        .save_report()
        .expect("report error")
}

// pub fn processe_sample(
//     sample: Sample,
//     writer: Writer
// )