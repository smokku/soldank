use gfx2d::{
    math::{vec2, Vec2},
    DrawBatch, Transform,
};
use hecs::World;

use super::components::*;
use crate::{constants::*, physics::RigidBodyPosition, render::Sprites};

fn draw_sprite_in_batch(
    batch: &mut DrawBatch,
    sprites: &Sprites,
    sprite: &mut Sprite,
    pos: Position,
    rot: f32,
) {
    let Sprite {
        group,
        name,
        sprite,
        color,
        transform,
    } = sprite;

    let transform = match transform {
        Transform::Pos(p) => Transform::Pos(*p + *pos),
        Transform::FromOrigin {
            pos: p,
            scale,
            rot: r,
        } => Transform::FromOrigin {
            pos: *p + *pos,
            scale: *scale,
            rot: (r.0 + rot, r.1),
        },
        Transform::WithPivot {
            pivot,
            pos: p,
            scale,
            rot: r,
        } => Transform::WithPivot {
            pivot: *pivot,
            pos: *p + *pos,
            scale: *scale,
            rot: *r + rot,
        },
        t => *t,
    };

    if sprite.is_none() {
        sprite.replace(sprites.get(group.as_str(), name.as_str()).clone());
    }

    batch.add_sprite(sprite.as_ref().unwrap(), *color, transform);
}

pub fn render_sprites(world: &World, sprites: &Sprites, batch: &mut DrawBatch, phys_scale: f32) {
    for (_entity, (mut sprite, position, rb_position)) in world
        .query::<(&mut Sprite, Option<&Position>, Option<&RigidBodyPosition>)>()
        .iter()
    {
        let params = if let Some(rbp) = rb_position {
            Some((
                Position::new(
                    rbp.position.translation.vector.x * phys_scale,
                    rbp.position.translation.vector.y * phys_scale,
                ),
                rbp.position.rotation.angle(),
            ))
        } else {
            position.map(|pos| (pos.clone(), 0.0))
        };

        if let Some((pos, rot)) = params {
            draw_sprite_in_batch(batch, sprites, &mut *sprite, pos, rot);
        }
    }
}

pub fn update_cursor(world: &mut World, x: f32, y: f32) {
    for (_entity, mut cursor) in world.query::<&mut Cursor>().iter() {
        cursor.x = x;
        cursor.y = y;
    }

    for (_entity, (mut camera, pos)) in world.query::<(&mut Camera, &Position)>().iter() {
        if camera.is_active && camera.centered {
            let zoom = f32::exp(camera.zoom);
            let mut m = Vec2::ZERO;

            m.x = zoom * (x - GAME_WIDTH / 2.0) / 7.0
                * ((2.0 * 640.0 / GAME_WIDTH - 1.0)
                    + (GAME_WIDTH - 640.0) / GAME_WIDTH * 0.0 / 6.8);
            m.y = zoom * (y - GAME_HEIGHT / 2.0) / 7.0;

            let norm = **pos - camera.offset;
            let s = norm * 0.14;
            camera.offset += s;
            camera.offset += m;
        }
    }
}

pub fn render_cursor(world: &World, sprites: &Sprites, batch: &mut DrawBatch) {
    for (_entity, (cursor, mut sprite)) in world.query::<(&Cursor, &mut Sprite)>().iter() {
        let offset = if let Some(sprite) = sprite.sprite.as_ref() {
            vec2(sprite.width, sprite.height) / -2.
        } else {
            Vec2::ZERO
        };
        draw_sprite_in_batch(
            batch,
            sprites,
            &mut *sprite,
            Position(**cursor + offset),
            0.0,
        );
    }
}
