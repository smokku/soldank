pub use gfx2d::math::*;

pub fn distance(p1: Vec2, p2: Vec2) -> f32 {
    (p2 - p1).magnitude()
}

pub fn vec2length(v: Vec2) -> f32 {
    v.magnitude()
}

pub fn vec2normalize(v: Vec2) -> Vec2 {
    let magnitude = v.magnitude();
    iif!(magnitude < 0.001, Vec2::zero(), v / magnitude)
}

pub fn vec2angle(v: Vec2) -> Rad {
    Vec2::angle(Vec2::unit_x(), v)
}

pub fn point_line_distance(p1: Vec2, p2: Vec2, p3: Vec2) -> f32 {
    let u = ((p3.x - p1.x) * (p2.x - p1.x) + (p3.y - p1.y) * (p2.y - p1.y))
        / ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2));

    let x = p1.x + u * (p2.x - p1.x);
    let y = p1.y + u * (p2.y - p1.y);

    ((x - p3.x).powi(2) + (y - p3.y).powi(2)).sqrt()
}

pub fn lerp(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    a + (b - a) * t
}
