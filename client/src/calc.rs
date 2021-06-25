pub use gfx2d::math::*;
use std::ops::{Add, Mul, Sub};

pub fn distance(p1: Vec2, p2: Vec2) -> f32 {
    (p2 - p1).length()
}

pub fn vec2length(v: Vec2) -> f32 {
    v.length()
}

pub fn vec2normalize(v: Vec2) -> Vec2 {
    let magnitude = v.length();
    iif!(magnitude < 0.001, Vec2::ZERO, v / magnitude)
}

pub fn vec2angle(v: Vec2) -> Rad {
    Vec2::X.angle_between(v)
}

pub fn point_line_distance(p1: Vec2, p2: Vec2, p3: Vec2) -> f32 {
    let u = ((p3.x - p1.x) * (p2.x - p1.x) + (p3.y - p1.y) * (p2.y - p1.y))
        / ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2));

    let x = p1.x + u * (p2.x - p1.x);
    let y = p1.y + u * (p2.y - p1.y);

    ((x - p3.x).powi(2) + (y - p3.y).powi(2)).sqrt()
}

pub fn lerp<T>(a: T, b: T, t: f32) -> T
where
    T: Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T> + Copy + Clone,
{
    a + (b - a) * t
}
