use super::*;
use crate::debug::DebugState;
use gfx2d::{
    macroquad::prelude::{color_u8, Color, DrawMode, QuadGl, Vertex},
    math::PI,
    Texture2D, Transform,
};

pub fn debug_render(
    gl: &mut QuadGl,
    state: &DebugState,
    graphics: &GameGraphics,
    world: &World,
    resources: &Resources,
) {
    let state = &state.render;
    let game = resources.get::<MainState>().unwrap();
    let map = resources.get::<MapFile>().unwrap();

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
                PolyType::Background => state.hlt_poly_background,
                PolyType::BackgroundTransition => state.hlt_poly_background_transition,
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
                    0 => Some(graphics.sprites.get("Marker", "SpawnGeneral")),
                    1 => Some(graphics.sprites.get("Marker", "SpawnAlpha")),
                    2 => Some(graphics.sprites.get("Marker", "SpawnBravo")),
                    3 => Some(graphics.sprites.get("Marker", "SpawnCharlie")),
                    4 => Some(graphics.sprites.get("Marker", "SpawnDelta")),
                    5 => Some(graphics.sprites.get("Marker", "FlagAlpha")),
                    6 => Some(graphics.sprites.get("Marker", "FlagBravo")),
                    7 => Some(graphics.sprites.get("Marker", "Grenades")),
                    8 => Some(graphics.sprites.get("Marker", "Medkits")),
                    9 => Some(graphics.sprites.get("Marker", "Clusters")),
                    10 => Some(graphics.sprites.get("Marker", "Vest")),
                    11 => Some(graphics.sprites.get("Marker", "Flamer")),
                    12 => Some(graphics.sprites.get("Marker", "Berserker")),
                    13 => Some(graphics.sprites.get("Marker", "Predator")),
                    14 => Some(graphics.sprites.get("Marker", "FlagYellow")),
                    15 => Some(graphics.sprites.get("Marker", "RamboBow")),
                    16 => Some(graphics.sprites.get("Marker", "StatGun")),
                    _ => None,
                };

                let (texture, tx, ty) = if let Some(sprite) = sprite {
                    (sprite.texture, sprite.texcoords_x, sprite.texcoords_y)
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
                    rot: ((2. * PI / STEPS as f32) * step as f32, Vec2::ZERO),
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

    if state.render_physics {
        physics(world, resources);
    }
}

pub fn physics(world: &World, resources: &Resources) {
    use rapier2d::prelude::*;

    let scale = resources.get::<Config>().unwrap().phys.scale;

    for (_entity, rb) in world
        .query::<crate::physics::RigidBodyComponentsQuery>()
        .iter()
    {
        let tr = rb.position.position.translation;
        let center = vec2(tr.x, tr.y) * scale;
        mq::draw_circle(center.x, center.y, 1.5, mq::YELLOW);
        mq::draw_circle(center.x, center.y, 0.75, mq::BLACK);
    }

    for (_entity, coll) in world
        .query::<crate::physics::ColliderComponentsQuery>()
        .iter()
    {
        const CL: Color = mq::GREEN;
        const TH: f32 = 0.5;

        let Isometry {
            translation: tr,
            rotation: rot,
        } = coll.position.0;
        let center = vec2(tr.x, tr.y) * scale;
        mq::draw_line(
            center.x,
            center.y,
            center.x + rot.re * 10.,
            center.y + rot.im * 10.,
            0.5,
            CL,
        );
        mq::draw_circle(center.x, center.y, 1.5, CL);
        mq::draw_circle(center.x, center.y, 0.75, mq::BLACK);

        match coll.shape.as_typed_shape() {
            TypedShape::Ball(ball) => {
                let r = ball.radius * scale;
                mq::draw_circle_lines(center.x, center.y, r, TH, CL);
            }
            TypedShape::Cuboid(cuboid) => {
                let hw = cuboid.half_extents.x * scale;
                let hh = cuboid.half_extents.y * scale;
                mq::draw_rectangle_lines(
                    center.x - hw,
                    center.y - hh,
                    hw * 2.,
                    hh * 2.,
                    TH * 2.,
                    CL,
                );
            }
            TypedShape::Capsule(_) => todo!(),
            TypedShape::Segment(_) => todo!(),
            TypedShape::Triangle(triangle) => {
                let a: Vec2 = triangle.a.into();
                let b: Vec2 = triangle.b.into();
                let c: Vec2 = triangle.c.into();
                mq::draw_triangle_lines(a * scale, b * scale, c * scale, TH, CL);
            }
            TypedShape::TriMesh(_) => todo!(),
            TypedShape::Polyline(_) => todo!(),
            TypedShape::HalfSpace(_) => todo!(),
            TypedShape::HeightField(_) => todo!(),
            TypedShape::Compound(_) => todo!(),
            TypedShape::ConvexPolygon(_) => todo!(),
            TypedShape::RoundCuboid(_) => todo!(),
            TypedShape::RoundTriangle(_) => todo!(),
            TypedShape::RoundConvexPolygon(_) => todo!(),
            TypedShape::Custom(_) => todo!(),
        }
    }
}
