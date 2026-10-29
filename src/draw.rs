
use ab_glyph::{FontArc, PxScale};
use image::{DynamicImage, Rgba};
use imageproc::{
    drawing::{draw_hollow_rect_mut, draw_text_mut, text_size, draw_line_segment_mut},
    rect::Rect,
};

use crate::util::get_type_and_color;

pub fn draw_deteciton(bbox: &[u32; 4], id: &u32, mut img: DynamicImage, class_id: &usize, font: &FontArc) -> DynamicImage{
    let x1 = bbox[0] as i32;
    let y1 = bbox[1] as i32;
    let w = bbox[2];
    let h = bbox[3];

    let rect: Rect = Rect::at(x1, y1).of_size(w, h);

    let (class_id_name, color) = get_type_and_color(class_id);

    draw_hollow_rect_mut(&mut img, rect, color);

    let text_y = (y1 - 22).max(0);
    draw_text_mut(
        &mut img,
        color,
        x1.max(0),
        text_y,
        PxScale::from(20.0),
        &font,
        format!("ID: {} class: {}", id, class_id_name).as_str(),
    );
    img
}

pub fn draw_fps(mut img: DynamicImage, fps: u64, font: &FontArc) -> DynamicImage{
    let scale = PxScale::from(24.0);
    let text = format!("TRACKER FPS: {:.1}", fps);
    let color = Rgba([0, 255, 0, 255]);

    let (text_width, _) = text_size(scale, &font, &text);

    let padding = 12;
    let x = img.width()
        .saturating_sub(text_width)
        .saturating_sub(padding) as i32;

    draw_text_mut(
        &mut img,
        color,
        x,
        padding as i32,
        scale,
        &font,
        &text,
    );
    img
}

pub fn draw_vector(
    img: &mut DynamicImage,
    start: (f32, f32),
    end: (f32, f32),
    color: Rgba<u8>
) {
    draw_line_segment_mut(
        img,
        start,
        end,
        color,
    );
}