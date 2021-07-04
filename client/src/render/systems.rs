use gfx2d::{DrawBatch, Transform};

use crate::render::Sprites;
use soldank_shared::components::*;

pub fn render_sprites(world: &hecs::World, sprites: &Sprites, batch: &mut DrawBatch) {
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
            pos,
        ),
    ) in world.query::<(&mut Sprite, &Position)>().iter()
    {
        let transform = match transform {
            Transform::Pos(p) => Transform::Pos(*p + **pos),
            Transform::FromOrigin { pos: p, scale, rot } => Transform::FromOrigin {
                pos: *p + **pos,
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
                pos: *p + **pos,
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
