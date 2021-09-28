extern crate rgb;

use rgb::*;

pub type Color = RGBA<u8>;

pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color { r, g, b, a: 255 }
}

pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color { r, g, b, a }
}

pub const BLACK: Color = Color {
    r: 0,
    g: 0,
    b: 0,
    a: 255,
};
pub const WHITE: Color = Color {
    r: 255,
    g: 255,
    b: 255,
    a: 255,
};
pub const CORNFLOWER_BLUE: Color = Color {
    r: 100,
    g: 149,
    b: 237,
    a: 255,
};
