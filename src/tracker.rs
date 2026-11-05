use crate::detection::Detection;
    use crate::draw::{draw_deteciton, draw_fps, draw_vector};
use crate::traffic_counter::Report;
use crate::util::{get_type_and_color, raw_bytes2img, raw_bytes2source, results2byte_track_boxes};
use ab_glyph::FontArc;
use edgefirst_tracker::{ByteTrack, ByteTrackBuilder, DetectionBox, Tracker, Uuid};
use image::DynamicImage ;
use std::collections::HashMap;
use std::time::Instant;
use ultralytics_inference::{Results, SourceMeta, YOLOModel};

pub enum TimingSource {
    Video,
    RealTime,
}

pub struct YoloTracker {
    model: YOLOModel,
    byte_track: ByteTrack<Detection>,
    start_time: Instant,
    display_ids: HashMap<Uuid, (u32, [u32; 2])>,
    next_id: u32,
    font: FontArc,
    pub fps: f32,
    pub traffic_report: Report,
    pub frame_cnt: u32
}

impl YoloTracker {
    pub fn new(model_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let byte_track = ByteTrackBuilder::new()
            .track_high_conf(0.6) // 0.6
            .track_iou(0.2)
            .track_update(0.25)
            .track_extra_lifespan(1_000_000_100) // 500_000_000
            .build();
        std::result::Result::Ok(Self {
            model: YOLOModel::load(model_name)?,
            byte_track: byte_track,
            start_time: Instant::now(),
            display_ids: HashMap::new(),
            next_id: 0,
            fps: 1.0,
            traffic_report: Report::new(),
            frame_cnt: 0,
            font: FontArc::try_from_slice(include_bytes!("DejaVuSans.ttf"))?
        })
    }

    pub fn run(
        &mut self,
        w: u32,
        h: u32,
        raw_img: &[u8],
        image_time: Option<f32>
    ) -> Result<DynamicImage, Box<dyn std::error::Error>> {
        //let timestamp = Instant::now();
        let source = raw_bytes2source(w, h, raw_img)?;
        //println!("raw_bytes2source: {}", timestamp.elapsed().as_secs_f32());

        // YOLO inference
        //let instant_yolo_time = Instant::now();
        let results: Vec<(SourceMeta, Results)> = self.model.predict_source(source, None)?;
        //println!("yolo inference: {}", instant_yolo_time.elapsed().as_secs_f32());
        let detections = results2byte_track_boxes(&results);    

        // ByteTrack

        let timestamp = self.start_time.elapsed().as_nanos() as u64;

        //let instant_bytetrack_time = Instant::now();
        let results: Vec<Option<edgefirst_tracker::TrackInfo>> =
            self.byte_track.update(&detections, timestamp);
        //println!("bytetrack inference {}", instant_bytetrack_time.elapsed().as_secs_f32());

        //let timestamp = Instant::now();
        let mut img = raw_bytes2img(w, h, raw_img)?;
        //println!("raw_bytes2img: {}", timestamp.elapsed().as_secs_f32());

        for (track, detection) in results.iter().zip(detections.iter()) {
            if let Some(track) = track {
                let uuid = track.uuid;
                let bbox = track.tracked_location;
                let x = bbox[0] as u32;
                let y = bbox[1] as u32;
                let w = bbox[2] as u32 - bbox[0] as u32;
                let h = bbox[3] as u32 - bbox[1] as u32;
                let cx = x as f32 + w as f32 / 2.0;
                let cy = y as f32 + h as f32 / 2.0;

                let class_id = detection.class_id;
                let score = detection.score();
                let (_, color) = get_type_and_color(&class_id);

                let id = if let Some(id_cxcy) = self.display_ids.get(&uuid) {
                    let cx = x as f32 + w as f32 / 2.0;
                    let cy = y as f32 + h as f32 / 2.0;
                    let dx = cx - id_cxcy.1[0] as f32;
                    let dy = cy - id_cxcy.1[1] as f32;

                    draw_vector(&mut img, (cx, cy), (cx + dx * 10.0, cy + dy * 10.0), color);

                    id_cxcy.0
                }
                else {
                    self.next_id += 1;
                    self.next_id
                };

                self.display_ids.insert(uuid, (id, [cx as u32, cy as u32]));
                //let time: Option<f32> = image_time;

                self.traffic_report.add_vehicle(
                    id,
                    uuid.to_string(),
                    class_id as u32,
                    image_time,
                    score,
                );

                img = draw_deteciton(&[x, y, w, h], &id, img, &class_id, &self.font);
            }
        };

        
        img = draw_fps(img, self.fps as u64, &self.font);
        self.frame_cnt += 1;

        std::result::Result::Ok(img)
    }
}
