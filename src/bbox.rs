use crate::util::raw_bytes2img;
use image::{DynamicImage, ImageBuffer, ImageError, Rgb};
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;
use nalgebra::SVector;
use ultralytics_inference::{Results, SourceMeta};

pub struct Bbox {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

pub struct CBbox {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

pub struct Detection {
    pub bbox: [f32; 4],
    pub score: f32,
    pub track_id: u32,
}

pub fn draw_boxes(
    results: &Vec<(SourceMeta, Results)>,
    weight: u32,
    height: u32,
    raw_img: &[u8],
) -> Result<DynamicImage, ImageError> {
    let mut img_buf = ImageBuffer::<Rgb<u8>, _>::from_raw(weight, height, raw_img.to_vec())
        .expect("failed to convert raw_slice");

    for result in results {
        if let Some(boxes) = &result.1.boxes {
            for i in 0..boxes.len() {
                let class_id = boxes.cls()[i] as usize;
                let confidence = boxes.conf()[i];
                let name = result.1.names[&class_id].as_str();
                println!(
                    "Обнаружен: {} с уверенностью {:.2} bbox: {}",
                    name,
                    confidence,
                    boxes.xywh()
                );

                if name == "car" {
                    let coords = boxes.xywh();
                    let x_center = coords[[i, 0]] as i32;
                    let y_center = coords[[i, 1]] as i32;

                    let w = coords[[i, 2]] as u32;
                    let h = coords[[i, 3]] as u32;

                    let x = x_center - (w as i32 / 2);
                    let y = y_center - (h as i32 / 2);

                    let rect: Rect = Rect::at(x, y).of_size(w, h);

                    let green_color = Rgb([0, 255, 0]);
                    draw_hollow_rect_mut(&mut img_buf, rect, green_color);
                }
            }
        }
    }
    println!("________________");
    raw_bytes2img(weight, height, &img_buf)
}

pub fn bbox_to_measurement(bbox: CBbox) -> SVector<f32, 4> {
    SVector::from([bbox.x as f32, bbox.y as f32, bbox.w as f32, bbox.h as f32])
}

pub fn bbox_cx_cy(bbox: Bbox) -> CBbox {
    let cx = bbox.x + bbox.w / 2;
    let cy = bbox.y + bbox.h / 2;

    CBbox {
        x: cx,
        y: cy,
        w: bbox.w,
        h: bbox.w,
    }
}
