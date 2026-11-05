use core::error;
use std::{collections::HashMap, fs::File};

use serde::Serialize;

#[derive(Debug, Clone, Default)]
pub struct ClassFilter {
    class_ids: Vec<u32>,
    scores: Vec<f32>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Vehicles {
    Bicycle,
    Car,
    Motorcycle,
    Bus,
    Truck,
}

impl Vehicles {
    pub fn get_vehicle(id: u32) -> Option<Self> {
        match id {
            1 => Some(Vehicles::Bicycle),
            2 => Some(Vehicles::Car),
            3 => Some(Vehicles::Motorcycle),
            5 => Some(Vehicles::Bus),
            7 => Some(Vehicles::Truck),
            _ => None
        }
    }
}

impl ClassFilter {
    pub fn add(&mut self, class_id: u32, score: f32) {
        self.class_ids.push(class_id);
        self.scores.push(score);
    }

    pub fn filtered_class_id(&self) -> Option<u32> {
        let mut class_scores: HashMap<u32, (f32, usize)> = HashMap::new();

        for (&class_id, &score) in self.class_ids.iter().zip(&self.scores) {
            let score = if score.is_finite() {
                score.max(0.0)
            } else {
                0.0
            };

            let (total_score, observations) = class_scores.entry(class_id).or_default();
            *total_score += score;
            *observations += 1;
        }

        class_scores
            .into_iter()
            .max_by(
                |(class_a, (score_a, observations_a)), (class_b, (score_b, observations_b))| {
                    score_a
                        .total_cmp(score_b)
                        .then_with(|| observations_a.cmp(observations_b))
                        .then_with(|| class_b.cmp(class_a))
                },
            )
            .map(|(class_id, _)| class_id)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TrackReport {
    pub display_id: u32,
    pub class_id: Option<Vehicles>,
    pub first_seen_secs: Option<f32>,
    pub last_seen_secs: Option<f32>,
    pub frames_seen: u32,

    #[serde(skip)]
    class_filter: ClassFilter,
}

impl TrackReport {
    fn new(display_id: u32, class_id: u32, time: Option<f32>, score: f32) -> Self {
        let mut class_filter = ClassFilter::default();
        class_filter.add(class_id, score);

        Self {
            display_id,
            class_id: None,
            first_seen_secs: time,
            last_seen_secs: time,
            frames_seen: 1,
            class_filter,
        }
    }

    fn add_observation(&mut self, class_id: u32, time: Option<f32>, score: f32) {
        self.last_seen_secs = time;
        self.frames_seen += 1;
        self.class_filter.add(class_id, score);

        if let Some(filtered_class_id) = self.class_filter.filtered_class_id() {
            self.class_id = Vehicles::get_vehicle(filtered_class_id);
        }
    }
}

pub struct Report {
    tracks: HashMap<String, TrackReport>,
}

impl Report {
    pub fn new() -> Self {
        Self {
            tracks: HashMap::new(),
        }
    }

    pub fn add_vehicle(
        &mut self,
        display_id: u32,
        uuid: String,
        class_id: u32,
        time: Option<f32>,
        score: f32,
    ) {
        self.tracks
            .entry(uuid)
            .and_modify(|track| track.add_observation(class_id, time, score))
            .or_insert_with(|| TrackReport::new(display_id, class_id, time, score));
    }

    pub fn remove_bad_tracks(&mut self) {
        self.tracks.retain(|_uuid, track| {
            track.last_seen_secs
                .and_then(|last| track.first_seen_secs.map(|first| last - first >= 1.0))
                .unwrap_or(false) && track.class_id.is_some()
        });
    }

    pub fn save_report(&self) -> Result<(), Box<dyn error::Error>> {
        let file = File::create("./src/report/report.json")?;
        serde_json::to_writer_pretty(file, &self.tracks)?;
        Ok(())
    }
}

impl Default for Report {
    fn default() -> Self {
        Self::new()
    }
}
