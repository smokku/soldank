use std::ops::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

pub fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2{x, y}
}

impl Neg      for Vec2 {type Output = Vec2; fn neg(self)            -> Vec2 {vec2(-self.x       , -self.y       )}}
impl Add      for Vec2 {type Output = Vec2; fn add(self, rhs: Vec2) -> Vec2 {vec2(self.x + rhs.x, self.y + rhs.y)}}
impl Sub      for Vec2 {type Output = Vec2; fn sub(self, rhs: Vec2) -> Vec2 {vec2(self.x + rhs.x, self.y + rhs.y)}}
impl Mul<f32> for Vec2 {type Output = Vec2; fn mul(self, rhs: f32)  -> Vec2 {vec2(self.x * rhs  , self.y * rhs  )}}
impl Div<f32> for Vec2 {type Output = Vec2; fn div(self, rhs: f32)  -> Vec2 {vec2(self.x / rhs  , self.y / rhs  )}}

impl AddAssign      for Vec2 {fn add_assign(&mut self, rhs: Vec2) {self.x += rhs.x; self.y += rhs.y;}}
impl SubAssign      for Vec2 {fn sub_assign(&mut self, rhs: Vec2) {self.x += rhs.x; self.y += rhs.y;}}
impl MulAssign<f32> for Vec2 {fn mul_assign(&mut self, rhs: f32)  {self.x *= rhs;   self.y *= rhs;  }}
impl DivAssign<f32> for Vec2 {fn div_assign(&mut self, rhs: f32)  {self.x /= rhs;   self.y /= rhs;  }}
