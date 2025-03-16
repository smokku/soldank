use super::*;
use crate::engine::world::WorldCameraExt;

pub fn debug_render(
    graphics: &mut GameGraphics,
    world: &World,
    resources: &Resources,
    config: &Config,
) {
    let state = &config.debug.render;
    let map = resources.get::<MapFile>().unwrap();

    let screen_size = window::screen_size();
    let screen_scale = GAME_WIDTH / screen_size.0;

    let (camera, _pos) = world.get_camera_and_camera_position();
    let zoom = f32::exp(camera.zoom);

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
            let a = |mc: MapColor| {
                let mut c: Color = mc.into();
                if c.a < u8::MAX / 2 {
                    c.a = u8::MAX - c.a
                }
                c
            };

            graphics.draw_debug_line((v1.x, v1.y), a(v1.color), (v2.x, v2.y), a(v2.color), w);
            graphics.draw_debug_line((v2.x, v2.y), a(v2.color), (v3.x, v3.y), a(v3.color), w);
            graphics.draw_debug_line((v3.x, v3.y), a(v3.color), (v1.x, v1.y), a(v1.color), w);
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
                (collider.x, collider.y),
                collider.diameter / 2.,
                0.0,
                rgba(255, 0, 0, 192),
                rgba(255, 0, 0, 128),
            );
        }
    }

    if state.render_position {
        for (_entity, pos) in world.query::<&components::Position>().iter() {
            graphics.draw_debug_disk(**pos, 2.0 * zoom, 0.0, rgb(255, 128, 32), rgb(255, 128, 32));
            graphics.draw_debug_disk(**pos, 0.75 * zoom, 0.0, rgb(0, 0, 0), rgb(0, 0, 0));
        }
        let (camera, camera_position) = world.get_camera_and_camera_position();
        for (_entity, pos) in world.query::<&components::Cursor>().iter() {
            let cam = camera.mouse_to_world(*camera_position, pos.x, pos.y);
            graphics.draw_debug_disk(cam, 2.0 * zoom, 0.0, rgb(255, 128, 32), rgb(128, 128, 32));
            graphics.draw_debug_disk(cam, 0.75 * zoom, 0.0, rgb(0, 0, 0), rgb(0, 0, 0));
        }
    }

    if state.render_physics {
        physics(graphics, world, resources, zoom, config.phys.scale);
    }
}

pub fn physics(
    graphics: &mut GameGraphics,
    world: &World,
    _resources: &Resources,
    zoom: f32,
    scale: f32,
) {
    use rapier2d::prelude::*;
    let cl = rgb(0, 255, 0);
    let th = 0.5 * zoom;

    for (_entity, rb) in world
        .query::<crate::physics::RigidBodyComponentsQuery>()
        .iter()
    {
        let cl = rgb(255, 255, 0);
        let Isometry {
            translation: tr,
            rotation: rot,
        } = rb.position.position;
        let center = Vec2::new(tr.x, tr.y) * scale;
        let line = center + Vec2::new(rot.re, rot.im).normalize() * (10. * zoom).clamp(2., 10.);
        graphics.draw_debug_line(center, cl, line, cl, th);
        graphics.draw_debug_disk(center, 2.0 * zoom, rot.angle(), cl, cl);
        graphics.draw_debug_disk(center, 0.75 * zoom, rot.angle(), rgb(0, 0, 0), rgb(0, 0, 0));
    }

    for (_entity, coll) in world
        .query::<crate::physics::ColliderComponentsQuery>()
        .iter()
    {
        let Isometry {
            translation: tr,
            rotation: rot,
        } = coll.position.0;
        let center = Vec2::new(tr.x, tr.y) * scale;
        let line = center + Vec2::new(rot.re, rot.im).normalize() * (8. * zoom).clamp(2., 10.);
        graphics.draw_debug_line(center, cl, line, cl, th);
        graphics.draw_debug_circle(center, 2.0 * zoom, rot.angle(), cl, th);
        graphics.draw_debug_disk(center, 0.75 * zoom, rot.angle(), rgb(0, 0, 0), rgb(0, 0, 0));

        match coll.shape.as_typed_shape() {
            TypedShape::Ball(ball) => {
                let r = ball.radius * scale;
                graphics.draw_debug_circle(center, r, rot.angle(), cl, th);
            }
            TypedShape::Cuboid(cuboid) => {
                let hw = cuboid.half_extents.x * scale;
                let hh = cuboid.half_extents.y * scale;
                graphics.draw_debug_polyline(
                    &[
                        (center.x - hw, center.y - hh, cl),
                        (center.x + hw, center.y - hh, cl),
                        (center.x + hw, center.y + hh, cl),
                        (center.x - hw, center.y + hh, cl),
                    ],
                    th,
                );
            }
            TypedShape::Capsule(capsule) => {
                let r = capsule.radius * scale;
                let mut a: Vec2 = (coll.position.0 * capsule.segment.a).into();
                let mut b: Vec2 = (coll.position.0 * capsule.segment.b).into();
                a *= scale;
                b *= scale;
                let off = Vec2::new(rot.re, rot.im).normalize() * r;
                let a1 = a + off;
                let b1 = b + off;
                let a2 = a - off;
                let b2 = b - off;
                graphics.draw_debug_line(a1, cl, b1, cl, th);
                graphics.draw_debug_line(a2, cl, b2, cl, th);
                graphics.draw_debug_half_circle(a, r, rot.angle() + gfx2d::math::PI, cl, th);
                graphics.draw_debug_half_circle(b, r, rot.angle(), cl, th);
            }
            TypedShape::Segment(_) => todo!(),
            TypedShape::Triangle(triangle) => {
                graphics.draw_debug_polyline(
                    &[
                        (triangle.a.x * scale, triangle.a.y * scale, cl),
                        (triangle.b.x * scale, triangle.b.y * scale, cl),
                        (triangle.c.x * scale, triangle.c.y * scale, cl),
                    ],
                    th,
                );
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
