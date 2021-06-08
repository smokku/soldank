use crate::{
    constants, debug::DebugState, mapfile::PolyType, mq, vec2, GameGraphics, MainState, MapFile,
    Vec2,
};
use gfx2d::{
    macroquad::prelude::{color_u8, Color, DrawMode, QuadGl, Vertex},
    math::PI,
    Texture2D, Transform,
};

pub fn debug_render(
    gl: &mut QuadGl,
    state: &DebugState,
    game: &MainState,
    map: &MapFile,
    graphics: &GameGraphics,
) {
    let state = &state.render;

    if state.render_wireframe || state.highlight_polygons {
        // TODO: merge to single gl.geometry() calls for all vertices

        for poly in map.polygons.iter() {
            if match poly.polytype {
                PolyType::Normal => state.hlt_poly_normal,
                PolyType::OnlyBulletsCollide => state.hlt_poly_only_bullets_coll,
                PolyType::OnlyPlayersCollide => state.hlt_poly_only_players_coll,
                PolyType::NoCollide => state.hlt_poly_no_coll,
                PolyType::Ice => state.hlt_poly_ice,
                PolyType::Deadly => state.hlt_poly_deadly,
                PolyType::BloodyDeadly => state.hlt_poly_bloody_deadly,
                PolyType::Hurts => state.hlt_poly_hurts,
                PolyType::Regenerates => state.hlt_poly_regenerates,
                PolyType::Lava => state.hlt_poly_lava,
                PolyType::AlphaBullets => state.hlt_poly_alpha_bullets,
                PolyType::AlphaPlayers => state.hlt_poly_alpha_players,
                PolyType::BravoBullets => state.hlt_poly_bravo_bullets,
                PolyType::BravoPlayers => state.hlt_poly_bravo_players,
                PolyType::CharlieBullets => state.hlt_poly_charlie_bullets,
                PolyType::CharliePlayers => state.hlt_poly_charlie_players,
                PolyType::DeltaBullets => state.hlt_poly_delta_bullets,
                PolyType::DeltaPlayers => state.hlt_poly_delta_players,
                PolyType::Bouncy => state.hlt_poly_bouncy,
                PolyType::Explosive => state.hlt_poly_explosive,
                PolyType::HurtsFlaggers => state.hlt_poly_hurt_flaggers,
                PolyType::OnlyFlaggers => state.hlt_poly_flagger_coll,
                PolyType::NotFlaggers => state.hlt_poly_non_flagger_coll,
                PolyType::FlagCollide => state.hlt_poly_flag_coll,
                PolyType::Background => false,
                PolyType::BackgroundTransition => false,
            } {
                let vertices = poly
                    .vertices
                    .iter()
                    .map(|v| Vertex::new(v.x, v.y, 0., 0., 0., color_u8!(255, 255, 0, 128)))
                    .collect::<Vec<Vertex>>();
                assert!(vertices.len() == 3);
                let indices = [0, 1, 2];
                gl.texture(None);
                gl.draw_mode(DrawMode::Triangles);
                gl.geometry(&vertices, &indices);
            }

            if state.render_wireframe {
                let vertices = poly
                    .vertices
                    .iter()
                    .map(|v| {
                        let color = if v.color.a < 8 {
                            color_u8!(255, 0, 0, 255 - v.color.a)
                        } else {
                            color_u8!(v.color.r, v.color.g, v.color.b, v.color.a)
                        };
                        Vertex::new(v.x, v.y, 0., 0., 0., color)
                    })
                    .collect::<Vec<Vertex>>();
                assert!(vertices.len() == 3);
                let indices = [0, 1, 1, 2, 2, 0];
                gl.texture(None);
                gl.draw_mode(DrawMode::Lines);
                gl.geometry(&vertices, &indices);
            }
        }
    }

    if state.render_spawns {
        for spawn in map.spawnpoints.iter() {
            if state.render_spawns_team[spawn.team as usize] {
                let x = spawn.x as f32;
                let y = spawn.y as f32;
                let scale =
                    f32::exp(game.zoom) * constants::GAME_WIDTH / constants::WINDOW_WIDTH as f32;
                let size = 8. * scale;
                let sprite = match spawn.team {
                    0 => Some(graphics.get_dynamic_sprite("Marker", "SpawnGeneral")),
                    1 => Some(graphics.get_dynamic_sprite("Marker", "SpawnAlpha")),
                    2 => Some(graphics.get_dynamic_sprite("Marker", "SpawnBravo")),
                    3 => Some(graphics.get_dynamic_sprite("Marker", "SpawnCharlie")),
                    4 => Some(graphics.get_dynamic_sprite("Marker", "SpawnDelta")),
                    5 => Some(graphics.get_dynamic_sprite("Marker", "FlagAlpha")),
                    6 => Some(graphics.get_dynamic_sprite("Marker", "FlagBravo")),
                    7 => Some(graphics.get_dynamic_sprite("Marker", "Grenades")),
                    8 => Some(graphics.get_dynamic_sprite("Marker", "Medkits")),
                    9 => Some(graphics.get_dynamic_sprite("Marker", "Clusters")),
                    10 => Some(graphics.get_dynamic_sprite("Marker", "Vest")),
                    11 => Some(graphics.get_dynamic_sprite("Marker", "Flamer")),
                    12 => Some(graphics.get_dynamic_sprite("Marker", "Berserker")),
                    13 => Some(graphics.get_dynamic_sprite("Marker", "Predator")),
                    14 => Some(graphics.get_dynamic_sprite("Marker", "FlagYellow")),
                    15 => Some(graphics.get_dynamic_sprite("Marker", "RamboBow")),
                    16 => Some(graphics.get_dynamic_sprite("Marker", "StatGun")),
                    _ => None,
                };

                let (texture, tx, ty) = if let Some(sprite) = sprite {
                    (
                        sprite.texture.clone(),
                        sprite.texcoords_x,
                        sprite.texcoords_y,
                    )
                } else {
                    (None, (0., 0.), (0., 0.))
                };

                let vertices = [
                    Vertex::new(x - size, y - size, 0., tx.0, ty.0, mq::WHITE),
                    Vertex::new(x + size, y - size, 0., tx.1, ty.0, mq::WHITE),
                    Vertex::new(x + size, y + size, 0., tx.1, ty.1, mq::WHITE),
                    Vertex::new(x - size, y + size, 0., tx.0, ty.1, mq::WHITE),
                ];
                let indices = [0, 1, 2, 0, 2, 3];

                gl.texture(texture.map(Texture2D::from_miniquad_texture));
                gl.draw_mode(DrawMode::Triangles);
                gl.geometry(&vertices, &indices);
            }
        }
    }

    if state.render_colliders {
        gl.texture(None);
        gl.draw_mode(DrawMode::Triangles);
        for collider in map.colliders.iter() {
            const STEPS: usize = 16;
            let pos = vec2(collider.x, collider.y);
            let mut vertices = Vec::with_capacity(STEPS);
            for step in 0..STEPS {
                let m = Transform::FromOrigin {
                    pos,
                    scale: vec2(1.0, 1.0),
                    rot: ((2. * PI / STEPS as f32) * step as f32, Vec2::zero()),
                }
                .matrix();

                vertices.push(m * vec2(collider.diameter / 2., 0.0));
            }

            for (i, &vert) in vertices.iter().enumerate() {
                let next = vertices[(i + 1) % STEPS];
                gl.geometry(
                    &[
                        Vertex::new(pos.x, pos.y, 0., 0., 0., color_u8!(255, 0, 0, 192)),
                        Vertex::new(vert.x, vert.y, 0., 0., 0., color_u8!(255, 0, 0, 128)),
                        Vertex::new(next.x, next.y, 0., 0., 0., color_u8!(255, 0, 0, 128)),
                    ],
                    &[0, 1, 2],
                );
            }
        }
    }
}
