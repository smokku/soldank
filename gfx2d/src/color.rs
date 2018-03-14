use super::*;

#[derive(Debug, Copy, Clone)]
pub struct Color([U8N; 4]);

pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color([U8N(r), U8N(g), U8N(b), U8N(255)])
}

pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color([U8N(r), U8N(g), U8N(b), U8N(a)])
}

impl Color {
    pub fn r(&self) -> u8 { self.0[0].0 }
    pub fn g(&self) -> u8 { self.0[1].0 }
    pub fn b(&self) -> u8 { self.0[2].0 }
    pub fn a(&self) -> u8 { self.0[3].0 }
    pub fn set_r(&mut self, value: u8) { self.0[0].0 = value; }
    pub fn set_g(&mut self, value: u8) { self.0[1].0 = value; }
    pub fn set_b(&mut self, value: u8) { self.0[2].0 = value; }
    pub fn set_a(&mut self, value: u8) { self.0[3].0 = value; }
}

impl ::std::convert::From<Color> for [U8N; 4] {
    fn from(color: Color) -> [U8N; 4] { color.0 }
}
