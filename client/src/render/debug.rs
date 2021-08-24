use super::*;
use crate::debug::DebugState;
use femtovg::{
    Align, Baseline, BlendFactor, Canvas, Color, CompositeOperation, FontId, ImageFlags, LineCap,
    Paint, Path, Renderer,
};
use gfx2d::{
    // macroquad::prelude::{color_u8, Color, DrawMode, QuadGl, Vertex},
    math::PI,
    Texture,
    Transform,
};
use std::collections::HashMap;

#[allow(clippy::too_many_arguments)]
fn draw_line<R: Renderer>(
    canvas: &mut Canvas<R>,
    x1: f32,
    y1: f32,
    color1: MapColor,
    x2: f32,
    y2: f32,
    color2: MapColor,
    line_width: f32,
) {
    let mut paint = Paint::linear_gradient(
        x1,
        y1,
        x2,
        y2,
        Color::rgba(color1.r, color1.g, color1.b, color1.a),
        Color::rgba(color2.r, color2.g, color2.b, color2.a),
    );
    paint.set_line_width(line_width);
    paint.set_line_cap(LineCap::Round);

    let mut path = Path::new();
    path.move_to(x1, y1);
    path.line_to(x2, y2);

    canvas.stroke_path(&mut path, paint);
}

pub fn debug_render<R: Renderer>(
    canvas: &mut Canvas<R>,
    graphics: &GameGraphics,
    state: &DebugState,
    world: &World,
    resources: &Resources,
    screen_scale: f32,
) {
    let state = &state.render;
    let game = resources.get::<MainState>().unwrap();
    let map = resources.get::<MapFile>().unwrap();
    let fonts = resources.get::<HashMap<&str, FontId>>().unwrap();

    let zoom = f32::exp(game.zoom);

    let mut path = femtovg::Path::new();
    path.move_to(0., 0.);
    path.line_to(-100., 100.);
    canvas.stroke_path(
        &mut path,
        femtovg::Paint::color(femtovg::Color::rgb(0, 0, 255)),
    );

    let mut paint = Paint::color(Color::hex("B7410E"));
    paint.set_font(&[*fonts.get("roboto").unwrap()]);
    paint.set_font_size(40.0);
    paint.set_text_baseline(Baseline::Top);
    paint.set_text_align(Align::Right);
    canvas
        .fill_text(800.0, 10.0, "alpha texture font - working!!!", paint)
        .unwrap();

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
                let mut path = Path::new();
                for poly in map.polygons.iter() {
                    path.move_to(poly.vertices[0].x, poly.vertices[0].y);
                    path.line_to(poly.vertices[1].x, poly.vertices[1].y);
                    path.line_to(poly.vertices[2].x, poly.vertices[2].y);
                    path.line_to(poly.vertices[0].x, poly.vertices[0].y);
                }
                canvas.fill_path(&mut path, Paint::color(Color::rgba(255, 255, 0, 128)));
            }
        }
    }

    if state.render_wireframe {
        for poly in map.polygons.iter() {
            let [v1, v2, v3] = &poly.vertices;
            draw_line(
                canvas,
                v1.x,
                v1.y,
                v1.color,
                v2.x,
                v2.y,
                v2.color,
                0.7 * zoom,
            );
            draw_line(
                canvas,
                v2.x,
                v2.y,
                v2.color,
                v3.x,
                v3.y,
                v3.color,
                0.7 * zoom,
            );
            draw_line(
                canvas,
                v3.x,
                v3.y,
                v3.color,
                v1.x,
                v1.y,
                v1.color,
                0.7 * zoom,
            );
        }
    }

    if state.render_spawns {
        for spawn in map.spawnpoints.iter() {
            if state.render_spawns_team[spawn.team as usize] {
                let x = spawn.x as f32;
                let y = spawn.y as f32;
                let scale = zoom * screen_scale;
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

                // let image = state.spawns_images.entry(0).or_insert_with(|| {
                //     canvas
                //         .create_image_from_file(
                //             ImageFlags::empty(),
                //             "./client/resources/markers/1.png",
                //         )
                //         .unwrap()
                // });

                // let vertices = [
                //     Vertex::new(x - size, y - size, 0., tx.0, ty.0, mq::WHITE),
                //     Vertex::new(x + size, y - size, 0., tx.1, ty.0, mq::WHITE),
                //     Vertex::new(x + size, y + size, 0., tx.1, ty.1, mq::WHITE),
                //     Vertex::new(x - size, y + size, 0., tx.0, ty.1, mq::WHITE),
                // ];
                // let indices = [0, 1, 2, 0, 2, 3];

                // let paint = Paint {
                //     xform: (),
                //     extent: (),
                //     radius: (),
                //     feather: (),
                //     inner_color: (),
                //     outer_color: (),
                //     image: (),
                // };

                // let mut path = Path::new();
                // canvas.rect((x - size, y - size, size * 2., size * 2.));
                // canvas.stroke_paint(Color::rgb_i(255, 255, 255));
                // canvas.stroke().unwrap();

                // let mut paint: Paint = Color::rgb_i(255, 255, 255).into();
                // paint.image = Some(*image);
                // paint.inner_color.a *= 0.5;
                // paint.outer_color.a *= 0.5;

                // let mut path = Path::new();
                // canvas.rect((x - size, y - size, size * 2., size * 2.));
                // canvas.fill_paint(paint);
                // canvas.fill().unwrap();

                // gl.texture(texture.map(Texture2D::from_miniquad_texture));
                // gl.draw_mode(DrawMode::Triangles);
                // gl.geometry(&vertices, &indices);
            }
        }
    }

    if state.render_colliders {
        for collider in map.colliders.iter() {
            //         const STEPS: usize = 16;
            let radius = collider.diameter / 2.;
            let mut path = Path::new();
            path.circle(collider.x, collider.y, radius);
            canvas.fill_path(
                &mut path,
                Paint::radial_gradient(
                    collider.x,
                    collider.y,
                    0.,
                    radius,
                    Color::rgba(255, 0, 0, 192),
                    Color::rgba(255, 0, 0, 128),
                ),
            );
        }
    }

    if state.render_physics {
        physics(canvas, world, resources, zoom);
    }
}

pub fn physics<R: Renderer>(
    canvas: &mut Canvas<R>,
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
        let center = (tr.x * scale, tr.y * scale);

        let mut path = Path::new();
        path.circle(center.0, center.1, 1.5);
        canvas.fill_path(&mut path, Paint::color(Color::rgbf(1., 1., 0.)));

        let mut path = Path::new();
        path.circle(center.0, center.1, 0.75);
        canvas.fill_path(&mut path, Paint::color(Color::black()));
    }

    let cl: Color = Color::rgbf(0., 1., 0.);
    let th: f32 = 0.5 * zoom;

    for (_entity, coll) in world
        .query::<crate::physics::ColliderComponentsQuery>()
        .iter()
    {
        let Isometry {
            translation: tr,
            rotation: rot,
        } = coll.position.0;
        let center = (tr.x * scale, tr.y * scale);
        let mut path = Path::new();
        path.move_to(center.0, center.1);
        path.line_to(center.0 + rot.re * 10., center.1 + rot.im * 10.);
        path.circle(center.0, center.1, 1.5);
        let mut paint = Paint::color(cl);
        paint.set_line_width(th);
        canvas.stroke_path(&mut path, paint);

        let mut path = Path::new();
        path.circle(center.0, center.1, 0.75);
        canvas.fill_path(&mut path, Paint::color(Color::black()));

        let mut path = Path::new();
        match coll.shape.as_typed_shape() {
            TypedShape::Ball(ball) => {
                let r = ball.radius * scale;
                path.circle(center.0, center.1, r);
            }
            TypedShape::Cuboid(cuboid) => {
                let hw = cuboid.half_extents.x * scale;
                let hh = cuboid.half_extents.y * scale;
                path.rect(center.0 - hw, center.1 - hh, hw * 2., hh * 2.);
            }
            TypedShape::Capsule(_) => todo!(),
            TypedShape::Segment(_) => todo!(),
            TypedShape::Triangle(triangle) => {
                path.move_to(triangle.a.x * scale, triangle.a.y * scale);
                path.line_to(triangle.b.x * scale, triangle.b.y * scale);
                path.line_to(triangle.c.x * scale, triangle.c.y * scale);
                path.line_to(triangle.a.x * scale, triangle.a.y * scale);
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
        let mut paint = Paint::color(cl);
        paint.set_line_width(th);
        canvas.stroke_path(&mut path, paint);
    }
}
