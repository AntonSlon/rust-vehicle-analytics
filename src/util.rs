use image::{DynamicImage, ImageBuffer, ImageError, Rgb, Rgba};
use ultralytics_inference::{Results, Source, SourceMeta};

use crate::{constants::SMOOTH, detection::Detection};

pub fn raw_bytes2source(w: u32, h: u32, raw_img: &[u8]) -> Result<Source, ImageError> {
    let img_buf = ImageBuffer::<Rgb<u8>, _>::from_raw(w, h, raw_img.to_vec())
        .expect("failed to convert raw_slice");
    let image = DynamicImage::ImageRgb8(img_buf);
    Ok(Source::ImageBuffer(image))
}

pub fn raw_bytes2img(w: u32, h: u32, raw_img: &[u8]) -> Result<DynamicImage, ImageError> {
    let img_buf = ImageBuffer::<Rgb<u8>, _>::from_raw(w, h, raw_img.to_vec())
        .expect("failed to convert raw_slice");
    Ok(DynamicImage::ImageRgb8(img_buf))
}

pub fn results2byte_track_boxes(results: &Vec<(SourceMeta, Results)>) -> Vec<Detection> {
    let mut byte_track_boxes: Vec<Detection> = Vec::new();
    let mut track_id = 0;
    for (_, result) in results {
        if let Some(boxes) = &result.boxes {
            let xyxy = boxes.xyxy();
            let confs = boxes.conf();
            let class_id = boxes.cls();

            for i in 0..boxes.len() {
                let row = xyxy.row(i);
                let x1 = row[0];
                let y1 = row[1];
                let x2 = row[2];
                let y2 = row[3];

                let score = confs[i];
                let class_id = class_id[i] as usize;

                if is_vehicle(class_id) {
                    let byte_track_box = Detection {
                        bbox: [x1, y1, x2, y2],
                        confidence: score,
                        class_id: class_id,
                        track_id: track_id,
                    };
                    track_id += 1;
                    byte_track_boxes.push(byte_track_box);
                }
            }
        }
    }
    byte_track_boxes
}

pub fn get_type_and_color(class_id: &usize) -> (String, Rgba<u8>){
    let (class_id_name, color): (String, Rgba<u8>) = match class_id {
        0 => {
            ("Person".to_string(),
            Rgba([255, 255, 255, 255])) // Пешеход — Белый
        }
        1 => {
            ("Bicycle".to_string(),
            Rgba([255, 165, 0, 255]))// Велосипед — Оранжевый
        }
        2 => {
            ("Car".to_string(), // Исправлено: добавлены скобки ()
            Rgba([0, 255, 0, 255])) // Легковой автомобиль — Зеленый
        }
        3 => {
            ("Motorcycle".to_string(), // Добавлено имя
            Rgba([255, 0, 0, 255])) // Мотоцикл — Красный
        }
        5 => {
            ("Bus".to_string(), // Добавлено имя
            Rgba([0, 0, 255, 255])) // Автобус — Синий
        }
        6 => {
            ("Train".to_string(), // Добавлено имя
            Rgba([0, 255, 255, 255])) // Поезд/Трамвай — Голубой
        }
        7 => {
            ("Truck".to_string(), // Добавлено имя
            Rgba([128, 0, 128, 255])) // Грузовик — Пурпурный
        }
        _ => {
            (format!("Unknown ({})", class_id), // Для остальных объектов пишем их ID
            Rgba([128, 128, 128, 255])) // Остальное — Серый
        }
    };
    return (class_id_name, color)
}

pub fn update_fps(smoothed_fps: f32, current_fps: f32) -> f32{
    return (smoothed_fps * SMOOTH) + (current_fps * (1.0 - SMOOTH))
}

fn is_vehicle(class_id: usize) -> bool {
    matches!(class_id, 1 | 2 | 3 | 5 | 7)
}
