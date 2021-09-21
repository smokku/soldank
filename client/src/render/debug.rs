use super::*;

pub fn debug_render(
    ctx: &mut Context,
    graphics: &mut GameGraphics,
    world: &World,
    resources: &Resources,
) {
    let state = &resources.get::<Config>().unwrap().debug.render;
    let game = resources.get::<MainState>().unwrap();
    let map = resources.get::<MapFile>().unwrap();

    let screen_size = ctx.screen_size();
    let screen_scale = game.game_width / screen_size.0;

    let zoom = f32::exp(game.zoom);

    // let fonts = resources.get::<HashMap<&str, FontId>>().unwrap();
    // let mut paint = Paint::color(Color::hex("B7410E"));
    // paint.set_font(&[*fonts.get("roboto").unwrap()]);
    // paint.set_font_size(40.0);
    // paint.set_text_baseline(Baseline::Top);
    // paint.set_text_align(Align::Right);
    // canvas
    //     .fill_text(800.0, 10.0, "femtovg ❤ miniquad", paint)
    //     .unwrap();

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
                let vertices = poly
                    .vertices
                    .iter()
                    .map(|v| vertex(vec2(v.x, v.y), Vec2::ZERO, rgba(255, 255, 0, 128)))
                    .collect::<Vec<Vertex>>();
                graphics.add_debug_geometry(None, vertices.as_slice());
            }
        }
    }

    if state.render_wireframe {
        for poly in map.polygons.iter() {
            let [v1, v2, v3] = &poly.vertices;
            let w = 0.7 * zoom;
            graphics.draw_debug_line(v1.x, v1.y, v1.color, v2.x, v2.y, v2.color, w);
            graphics.draw_debug_line(v2.x, v2.y, v2.color, v3.x, v3.y, v3.color, w);
            graphics.draw_debug_line(v3.x, v3.y, v3.color, v1.x, v1.y, v1.color, w);
        }
    }

    if state.render_spawns {
        let scale = zoom * screen_scale;
        let size = 8. * scale;

        for spawn in map.spawnpoints.iter() {
            if state.render_spawns_team[spawn.team as usize] {
                let (group, sprite) = match spawn.team {
                    0 => ("Marker", "SpawnGeneral"),
                    1 => ("Marker", "SpawnAlpha"),
                    2 => ("Marker", "SpawnBravo"),
                    3 => ("Marker", "SpawnCharlie"),
                    4 => ("Marker", "SpawnDelta"),
                    5 => ("Marker", "FlagAlpha"),
                    6 => ("Marker", "FlagBravo"),
                    7 => ("Marker", "Grenades"),
                    8 => ("Marker", "Medkits"),
                    9 => ("Marker", "Clusters"),
                    10 => ("Marker", "Vest"),
                    11 => ("Marker", "Flamer"),
                    12 => ("Marker", "Berserker"),
                    13 => ("Marker", "Predator"),
                    14 => ("Marker", "FlagYellow"),
                    15 => ("Marker", "RamboBow"),
                    16 => ("Marker", "StatGun"),
                    _ => ("", ""),
                };
                graphics.draw_debug_sprite(
                    group,
                    sprite,
                    spawn.x as f32,
                    spawn.y as f32,
                    size,
                    size,
                );
            }
        }
    }

    if state.render_colliders {
        for collider in map.colliders.iter() {
            graphics.draw_debug_disk(
                collider.x,
                collider.y,
                collider.diameter / 2.,
                rgba(255, 0, 0, 192),
                rgba(255, 0, 0, 128),
            );
        }
    }

    if state.render_physics {
        // physics(canvas, world, resources, zoom);
    }
}

// pub fn physics<R: Renderer>(
//     canvas: &mut Canvas<R>,
//     world: &World,
//     resources: &Resources,
//     zoom: f32,
// ) {
//     use rapier2d::prelude::*;

//     let scale = resources.get::<Config>().unwrap().phys.scale;

//     for (_entity, rb) in world
//         .query::<crate::physics::RigidBodyComponentsQuery>()
//         .iter()
//     {
//         let tr = rb.position.position.translation;
//         let center = (tr.x * scale, tr.y * scale);

//         let mut path = Path::new();
//         path.circle(center.0, center.1, 1.5);
//         canvas.fill_path(&mut path, Paint::color(Color::rgbf(1., 1., 0.)));

//         let mut path = Path::new();
//         path.circle(center.0, center.1, 0.75);
//         canvas.fill_path(&mut path, Paint::color(Color::black()));
//     }

//     let cl: Color = Color::rgbf(0., 1., 0.);
//     let th: f32 = 0.5 * zoom;

//     for (_entity, coll) in world
//         .query::<crate::physics::ColliderComponentsQuery>()
//         .iter()
//     {
//         let Isometry {
//             translation: tr,
//             rotation: rot,
//         } = coll.position.0;
//         let center = (tr.x * scale, tr.y * scale);
//         let mut path = Path::new();
//         path.move_to(center.0, center.1);
//         path.line_to(center.0 + rot.re * 10., center.1 + rot.im * 10.);
//         path.circle(center.0, center.1, 1.5);
//         let mut paint = Paint::color(cl);
//         paint.set_line_width(th);
//         canvas.stroke_path(&mut path, paint);

//         let mut path = Path::new();
//         path.circle(center.0, center.1, 0.75);
//         canvas.fill_path(&mut path, Paint::color(Color::black()));

//         let mut path = Path::new();
//         match coll.shape.as_typed_shape() {
//             TypedShape::Ball(ball) => {
//                 let r = ball.radius * scale;
//                 path.circle(center.0, center.1, r);
//             }
//             TypedShape::Cuboid(cuboid) => {
//                 let hw = cuboid.half_extents.x * scale;
//                 let hh = cuboid.half_extents.y * scale;
//                 path.rect(center.0 - hw, center.1 - hh, hw * 2., hh * 2.);
//             }
//             TypedShape::Capsule(_) => todo!(),
//             TypedShape::Segment(_) => todo!(),
//             TypedShape::Triangle(triangle) => {
//                 path.move_to(triangle.a.x * scale, triangle.a.y * scale);
//                 path.line_to(triangle.b.x * scale, triangle.b.y * scale);
//                 path.line_to(triangle.c.x * scale, triangle.c.y * scale);
//                 path.line_to(triangle.a.x * scale, triangle.a.y * scale);
//             }
//             TypedShape::TriMesh(_) => todo!(),
//             TypedShape::Polyline(_) => todo!(),
//             TypedShape::HalfSpace(_) => todo!(),
//             TypedShape::HeightField(_) => todo!(),
//             TypedShape::Compound(_) => todo!(),
//             TypedShape::ConvexPolygon(_) => todo!(),
//             TypedShape::RoundCuboid(_) => todo!(),
//             TypedShape::RoundTriangle(_) => todo!(),
//             TypedShape::RoundConvexPolygon(_) => todo!(),
//             TypedShape::Custom(_) => todo!(),
//         }
//         let mut paint = Paint::color(cl);
//         paint.set_line_width(th);
//         canvas.stroke_path(&mut path, paint);
//     }
// }
