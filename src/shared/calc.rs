use na::Vector2;

pub fn distance(p1: Vector2<f32>, p2: Vector2<f32>) -> f32 {
  ((p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2)).sqrt()
}

pub fn vec2length(v: Vector2<f32>) -> f32 {
  ((v.x).powi(2) + (v.y).powi(2)).sqrt()
}

pub fn vec2normalize(v_out: Vector2<f32>, v: Vector2<f32>) -> Vector2<f32> {
  let len = vec2length(v);
  if (len < 0.001) && (len > -0.001) {
    Vector2::new(0.0, 0.0)
  } else {
    Vector2::new(v_out.x / len, v_out.y / len)
  }
}

pub fn point_line_distance(p1: Vector2<f32>, p2: Vector2<f32>, p3: Vector2<f32>) -> f32 {
  let u = ((p3.x - p1.x) * (p2.x - p1.x) + (p3.y - p1.y) * (p2.y - p1.y))
    / ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2));

  let x = p1.x + u * (p2.x - p1.x);
  let y = p1.y + u * (p2.y - p1.y);

  ((x - p3.x).powi(2) + (y - p3.y).powi(2)).sqrt()
}
