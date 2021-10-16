use gfx2d::{
    math::{vec2, Vec2},
    DrawBatch, Transform,
};
use hecs::World;

use super::{components::*, render_skeleton, render_soldier, SoldierGraphics};
use crate::{
    calc::lerp, constants::*, physics::RigidBodyPosition, render::Sprites, soldier::Soldier,
};

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
            position.map(|pos| (*pos, 0.0))
        };

        if let Some((pos, rot)) = params {
            draw_sprite_in_batch(batch, sprites, &mut *sprite, pos, rot);
        }
    }
}

pub fn update_cursor(world: &mut World, mouse_x: f32, mouse_y: f32) {
    for (_entity, mut cursor) in world.query::<&mut Cursor>().iter() {
        cursor.x = mouse_x;
        cursor.y = mouse_y;
    }

    for (_entity, mut camera) in world.query::<&mut Camera>().iter() {
        if camera.is_active && camera.centered {
            let zoom = f32::exp(camera.zoom);
            let mut offset = Vec2::ZERO;

            offset.x = zoom
                * (mouse_x - GAME_WIDTH / 2.0)
                * ((2.0 * 640.0 / GAME_WIDTH - 1.0)
                    + (GAME_WIDTH - 640.0) / GAME_WIDTH * 0.0 / 6.8);
            offset.y = zoom * (mouse_y - GAME_HEIGHT / 2.0);

            camera.offset = lerp(camera.offset, offset, 0.14);
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

#[allow(clippy::too_many_arguments)]
pub fn render_soldiers(
    world: &World,
    soldier_graphics: &SoldierGraphics,
    sprites: &[Vec<gfx2d::Sprite>],
    batch: &mut DrawBatch,
    debug_batch: &mut DrawBatch,
    frame_percent: f32,
    scale: f32,
    skeleton: bool,
) {
    for (_entity, soldier) in world.query::<&Soldier>().iter() {
        let frame_percent = iif!(soldier.active, frame_percent, 1.0);
        render_soldier(soldier, soldier_graphics, sprites, batch, frame_percent);
        if skeleton {
            render_skeleton(soldier, debug_batch, scale, frame_percent);
        }
    }
}
