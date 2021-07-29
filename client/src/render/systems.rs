use gfx2d::{DrawBatch, Transform};

use crate::{components::*, physics::RigidBodyPosition, render::Sprites};

pub fn render_sprites(
    world: &hecs::World,
    sprites: &Sprites,
    batch: &mut DrawBatch,
    phys_scale: f32,
) {
    for (
        _entity,
        (
            Sprite {
                group,
                name,
                sprite,
                color,
                transform,
            },
            position,
            rb_position,
        ),
    ) in world
        .query::<(&mut Sprite, Option<&Position>, Option<&RigidBodyPosition>)>()
        .iter()
    {
        let pos = if let Some(rbp) = rb_position {
            Some(Position::new(
                rbp.position.translation.vector.x * phys_scale,
                rbp.position.translation.vector.y * phys_scale,
            ))
        } else if let Some(pos) = position {
            Some(pos.clone())
        } else {
            None
        };

        if let Some(pos) = pos {
            let transform = match transform {
                Transform::Pos(p) => Transform::Pos(*p + *pos),
                Transform::FromOrigin { pos: p, scale, rot } => Transform::FromOrigin {
                    pos: *p + *pos,
                    scale: *scale,
                    rot: *rot,
                },
                Transform::WithPivot {
                    pivot,
                    pos: p,
                    scale,
                    rot,
                } => Transform::WithPivot {
                    pivot: *pivot,
                    pos: *p + *pos,
                    scale: *scale,
                    rot: *rot,
                },
                t => *t,
            };

            if sprite.is_none() {
                sprite.replace(sprites.get(group.as_str(), name.as_str()).clone());
            }

            batch.add_sprite(sprite.as_ref().unwrap(), *color, transform);
        }
    }
}
