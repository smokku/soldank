use super::*;
use crate::debug::DebugState;
use gfx2d::{
    // macroquad::prelude::{color_u8, Color, DrawMode, QuadGl, Vertex},
    math::PI,
    Texture,
    Transform,
};
use nona::{Align, BlendFactor, Canvas, Color, CompositeOperation, Gradient, LineCap, Point};

fn draw_line(
    canvas: &mut Canvas<nonaquad::nvgimpl::RendererCtx>,
    x1: f32,
    y1: f32,
    color1: MapColor,
    x2: f32,
    y2: f32,
    color2: MapColor,
) {
    let point1 = Point::new(x1, y1);
    let point2 = Point::new(x2, y2);
    canvas.begin_path();
    canvas.stroke_paint(Gradient::Linear {
        start: point1,
        end: point2,
        start_color: Color::rgba_i(color1.r, color1.g, color1.b, color1.a),
        end_color: Color::rgba_i(color2.r, color2.g, color2.b, color2.a),
    });
    canvas.move_to(point1);
    canvas.line_to(point2);
    canvas.stroke().unwrap();
}

pub fn debug_render(
    canvas: &mut Canvas<nonaquad::nvgimpl::RendererCtx>,
    state: &DebugState,
    world: &World,
    resources: &Resources,
) {
    let state = &state.render;
    let game = resources.get::<MainState>().unwrap();
    let map = resources.get::<MapFile>().unwrap();

    let zoom = f32::exp(game.zoom);

    if state.highlight_polygons {
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
                canvas.begin_path();
                canvas.fill_paint(Color::rgba_i(255, 255, 0, 128));
                for poly in map.polygons.iter() {
                    canvas.move_to((poly.vertices[0].x, poly.vertices[0].y));
                    canvas.line_to((poly.vertices[1].x, poly.vertices[1].y));
                    canvas.line_to((poly.vertices[2].x, poly.vertices[2].y));
                    canvas.line_to((poly.vertices[0].x, poly.vertices[0].y));
                }
                canvas.fill().unwrap();
            }
        }
    }

    if state.render_wireframe {
        canvas.stroke_width(0.7 * zoom);
        canvas.line_cap(LineCap::Round);
        for poly in map.polygons.iter() {
            let [v1, v2, v3] = &poly.vertices;
            draw_line(canvas, v1.x, v1.y, v1.color, v2.x, v2.y, v2.color);
            draw_line(canvas, v2.x, v2.y, v2.color, v3.x, v3.y, v3.color);
            draw_line(canvas, v3.x, v3.y, v3.color, v1.x, v1.y, v1.color);
        }
    }

    // if state.render_spawns {
    //     for spawn in map.spawnpoints.iter() {
    //         if state.render_spawns_team[spawn.team as usize] {
    //             let x = spawn.x as f32;
    //             let y = spawn.y as f32;
    //             let scale =
    //                 f32::exp(game.zoom) * constants::GAME_WIDTH / constants::WINDOW_WIDTH as f32;
    //             let size = 8. * scale;
    //             let sprite = match spawn.team {
    //                 0 => Some(graphics.sprites.get("Marker", "SpawnGeneral")),
    //                 1 => Some(graphics.sprites.get("Marker", "SpawnAlpha")),
    //                 2 => Some(graphics.sprites.get("Marker", "SpawnBravo")),
    //                 3 => Some(graphics.sprites.get("Marker", "SpawnCharlie")),
    //                 4 => Some(graphics.sprites.get("Marker", "SpawnDelta")),
    //                 5 => Some(graphics.sprites.get("Marker", "FlagAlpha")),
    //                 6 => Some(graphics.sprites.get("Marker", "FlagBravo")),
    //                 7 => Some(graphics.sprites.get("Marker", "Grenades")),
    //                 8 => Some(graphics.sprites.get("Marker", "Medkits")),
    //                 9 => Some(graphics.sprites.get("Marker", "Clusters")),
    //                 10 => Some(graphics.sprites.get("Marker", "Vest")),
    //                 11 => Some(graphics.sprites.get("Marker", "Flamer")),
    //                 12 => Some(graphics.sprites.get("Marker", "Berserker")),
    //                 13 => Some(graphics.sprites.get("Marker", "Predator")),
    //                 14 => Some(graphics.sprites.get("Marker", "FlagYellow")),
    //                 15 => Some(graphics.sprites.get("Marker", "RamboBow")),
    //                 16 => Some(graphics.sprites.get("Marker", "StatGun")),
    //                 _ => None,
    //             };

    //             let (texture, tx, ty) = if let Some(sprite) = sprite {
    //                 (sprite.texture, sprite.texcoords_x, sprite.texcoords_y)
    //             } else {
    //                 (None, (0., 0.), (0., 0.))
    //             };

    //             let vertices = [
    //                 Vertex::new(x - size, y - size, 0., tx.0, ty.0, mq::WHITE),
    //                 Vertex::new(x + size, y - size, 0., tx.1, ty.0, mq::WHITE),
    //                 Vertex::new(x + size, y + size, 0., tx.1, ty.1, mq::WHITE),
    //                 Vertex::new(x - size, y + size, 0., tx.0, ty.1, mq::WHITE),
    //             ];
    //             let indices = [0, 1, 2, 0, 2, 3];

    //             gl.texture(texture.map(Texture2D::from_miniquad_texture));
    //             gl.draw_mode(DrawMode::Triangles);
    //             gl.geometry(&vertices, &indices);
    //         }
    //     }
    // }

    if state.render_colliders {
        for collider in map.colliders.iter() {
            //         const STEPS: usize = 16;
            let pos = Point::new(collider.x, collider.y);
            let radius = collider.diameter / 2.;
            canvas.begin_path();
            canvas.circle(pos, radius);
            canvas.fill_paint(Gradient::Radial {
                center: pos,
                in_radius: 0.,
                out_radius: radius,
                inner_color: Color::rgba_i(255, 0, 0, 192),
                outer_color: Color::rgba_i(255, 0, 0, 128),
            });
            canvas.fill().unwrap();
        }
    }

    if state.render_physics {
        physics(canvas, world, resources, zoom);
    }
}

pub fn physics(
    canvas: &mut Canvas<nonaquad::nvgimpl::RendererCtx>,
    world: &World,
    resources: &Resources,
    zoom: f32,
) {
    use rapier2d::prelude::*;

    let scale = resources.get::<Config>().unwrap().phys.scale;

    for (_entity, rb) in world
        .query::<crate::physics::RigidBodyComponentsQuery>()
        .iter()
    {
        let tr = rb.position.position.translation;
        let center = nona::Point::new(tr.x * scale, tr.y * scale);

        canvas.begin_path();
        canvas.circle(center, 1.5);
        canvas.fill_paint(Color::rgb(1., 1., 0.));
        canvas.fill().unwrap();

        canvas.begin_path();
        canvas.circle(center, 0.75);
        canvas.fill_paint(Color::rgb(0., 0., 0.));
        canvas.fill().unwrap();
    }

    let cl: Color = Color::rgb(0., 1., 0.);
    let th: f32 = 0.5 * zoom;
    for (_entity, coll) in world
        .query::<crate::physics::ColliderComponentsQuery>()
        .iter()
    {
        let Isometry {
            translation: tr,
            rotation: rot,
        } = coll.position.0;
        let center = nona::Point::new(tr.x * scale, tr.y * scale);
        canvas.begin_path();
        canvas.move_to(center);
        canvas.line_to((center.x + rot.re * 10., center.y + rot.im * 10.));
        canvas.circle(center, 1.5);
        canvas.stroke_width(th);
        canvas.stroke_paint(cl);
        canvas.stroke().unwrap();

        canvas.begin_path();
        canvas.fill_paint(Color::rgb(0., 0., 0.));
        canvas.circle(center, 0.75);
        canvas.fill().unwrap();

        canvas.begin_path();
        match coll.shape.as_typed_shape() {
            TypedShape::Ball(ball) => {
                let r = ball.radius * scale;
                canvas.circle(center, r);
            }
            TypedShape::Cuboid(cuboid) => {
                let hw = cuboid.half_extents.x * scale;
                let hh = cuboid.half_extents.y * scale;
                canvas.rect((center.x - hw, center.y - hh, hw * 2., hh * 2.));
            }
            TypedShape::Capsule(_) => todo!(),
            TypedShape::Segment(_) => todo!(),
            TypedShape::Triangle(triangle) => {
                canvas.move_to((triangle.a.x * scale, triangle.a.y * scale));
                canvas.line_to((triangle.b.x * scale, triangle.b.y * scale));
                canvas.line_to((triangle.c.x * scale, triangle.c.y * scale));
                canvas.line_to((triangle.a.x * scale, triangle.a.y * scale));
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
        canvas.stroke_width(th);
        canvas.stroke_paint(Color::rgb(0., 1., 0.));
        canvas.stroke().unwrap();
    }
}
