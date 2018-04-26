use super::*;
use gfx::SpriteData;

pub fn render_bullet(
    bullet: &Bullet,
    sprites: &[Vec<Sprite>],
    batch: &mut DrawBatch,
    _elapsed: f64,
    frame_percent: f32,
) {
    let frame_percent = iif!(bullet.active, frame_percent, 1.0);
    let pos = lerp(bullet.particle.old_pos, bullet.particle.pos, frame_percent);

    match bullet.style {
        BulletStyle::ThrownKnife => {
            let t = lerp(
                bullet.timeout_prev as f32,
                bullet.timeout as f32,
                frame_percent,
            );

            let (rot, sprite) = {
                if bullet.particle.velocity.x >= 0.0 {
                    (rad(t / PI), gfx::Weapon::Knife)
                } else {
                    (-rad(t / PI), gfx::Weapon::Knife2)
                }
            };

            batch.add_sprite(
                &sprites[sprite.group().id()][sprite.id()],
                rgb(255, 255, 255),
                Transform::WithPivot {
                    pivot: vec2(4.0, 1.0),
                    pos,
                    scale: vec2(1.0, 1.0),
                    rot,
                },
            );
        }
        _ => {
            if let Some(sprite) = bullet.sprite {
                let sprite = &sprites[sprite.group().id()][sprite.id()];
                let hit = lerp(bullet.hit_multiply_prev, bullet.hit_multiply, frame_percent);

                let scale = {
                    let scale = bullet.particle.velocity.magnitude() / 13.0;
                    let dist = (pos - bullet.initial_pos).magnitude();

                    if dist < scale * sprite.width {
                        dist / (scale * sprite.width)
                    } else {
                        scale
                    }
                };

                let alpha = f32::max(50.0, f32::min(230.0, 255.0 * hit * scale.powi(2) / 4.63));

                batch.add_sprite(
                    sprite,
                    rgba(255, 255, 255, alpha.round() as u8),
                    Transform::WithPivot {
                        pivot: vec2((10.0 / 95.0) * sprite.width, 0.5 * sprite.height),
                        pos,
                        scale: vec2(scale, 1.0),
                        rot: vec2angle(-bullet.particle.velocity),
                    },
                );
            }
        }
    };
}
