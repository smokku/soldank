use super::*;
use image;
use std::convert::AsRef;
use std::path::Path;

pub fn load_image_rgba<P: AsRef<Path>>(filename: P) -> image::RgbaImage {
    let img = image::open(filename).unwrap();
    match img {
        image::DynamicImage::ImageRgba8(img) => img,
        _ => img.to_rgba8(),
    }
}

pub fn premultiply_image(img: &mut image::RgbaImage) {
    for pixel in img.pixels_mut() {
        let a = f32::from(pixel[3]) / 255.0;

        *pixel = image::Rgba([
            (f32::from(pixel[0]) * a) as u8,
            (f32::from(pixel[1]) * a) as u8,
            (f32::from(pixel[2]) * a) as u8,
            pixel[3],
        ]);
    }
}

pub fn remove_color_key(img: &mut image::RgbaImage, color_key: Color) {
    for pixel in img.pixels_mut() {
        if rgba(pixel[0], pixel[1], pixel[2], pixel[3]) == color_key {
            *pixel = image::Rgba([0, 0, 0, 0]);
        }
    }
}
