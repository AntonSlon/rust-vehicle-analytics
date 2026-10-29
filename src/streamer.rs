use gstreamer::glib::object::CastNone;
use gstreamer::prelude::{Cast, ElementExt, GstBinExt};
use gstreamer::{glib, Pipeline, State};
use gstreamer_app::{AppSink, AppSrc};
use std::sync::mpsc::Receiver;

pub enum StreamMessage {
    Sample(gstreamer::Sample),
    Eos,
}

pub struct VideoStreamer {
    pipeline: Pipeline,
    pub receiver: Receiver<StreamMessage>,
}

impl VideoStreamer {
    pub fn new(pipeline_str: &str) -> Result<Self, glib::Error> {
        gstreamer::init()?;
        let (tx, rx) = std::sync::mpsc::sync_channel::<StreamMessage>(1);

        let pipeline = gstreamer::parse::launch(pipeline_str)?
            .downcast::<Pipeline>()
            .expect("failed downcast pipeline");

        let appsink: AppSink = pipeline
            .by_name("mysink")
            .expect("Appsink not found")
            .downcast::<AppSink>()
            .unwrap();

        appsink.set_max_buffers(1);
        appsink.set_drop(true);

        // sender eos message
        let tx_eos = tx.clone();

        appsink.set_callbacks(
            gstreamer_app::AppSinkCallbacks::builder()
                .eos(move |_| {
                    tx_eos.send(StreamMessage::Eos).unwrap();
                })
                .new_sample(move |appsink| {
                    let sample = appsink
                        .pull_sample()
                        .map_err(|_| gstreamer::FlowError::Eos)?;
                    tx.send(StreamMessage::Sample(sample)).unwrap();
                    Result::Ok(gstreamer::FlowSuccess::Ok)
                })
                .build(),
        );

        Result::Ok(Self {
            receiver: rx,
            pipeline,
        })
    }

    pub fn start(&self) {
        self.pipeline
            .set_state(State::Playing)
            .expect("State playing error");
    }

    pub fn stop(&self) {
        self.pipeline
            .set_state(State::Null)
            .expect("State null error");
    }

    pub fn get_app_src(&self, w: u32, h: u32, fps: f32) -> Result<(Pipeline, AppSrc), glib::Error> {
        let pipline_for_stream: &str = &format!(
            "appsrc name=processed is-live=true format=time do-timestamp=true \
        max-buffers=1 leaky-type=downstream \
        caps=video/x-raw,format=RGB,width={w},height={h},framerate={}/1 \
        ! videoconvert ! autovideosink sync=false", fps as u32);

        let pipline = gstreamer::parse::launch(pipline_for_stream)
            .unwrap()
            .downcast::<gstreamer::Pipeline>()
            .unwrap();

        let appsrc = pipline
            .by_name("processed")
            .and_downcast::<AppSrc>()
            .unwrap();

        Result::Ok((pipline, appsrc))
    }
}
