use edgefirst_tracker::DetectionBox;

#[derive(Debug, Clone)]
pub struct Detection {
    pub bbox: [f32; 4],
    pub confidence: f32,
    pub class_id: usize,
    pub track_id: usize,
}
    
impl DetectionBox for Detection {
    fn bbox(&self) -> [f32; 4] {
        self.bbox
    }

    fn score(&self) -> f32 {
        self.confidence
    }

    fn label(&self) -> usize {
        self.class_id
    }
}
