extern crate rgb;

use super::U8N;
use rgb::*;

pub type Color = RGBA<U8N>;

pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color {
        r: U8N(r),
        g: U8N(g),
        b: U8N(b),
        a: U8N(255),
    }
}

pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color {
        r: U8N(r),
        g: U8N(g),
        b: U8N(b),
        a: U8N(a),
    }
}
