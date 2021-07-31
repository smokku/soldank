use super::*;
use crate::{components::*, physics::*};
use rand::thread_rng;
use rand::Rng;

#[derive(Default)]
pub struct SpawnerState {
    pub(crate) visible: bool,

    spawn_gostek: bool,
    spawn_ak74: bool,
    spawn_particle: bool,
    spawn_ball: bool,
}

impl IVisit for SpawnerState {
    fn visit(&mut self, f: &mut dyn FnMut(&mut dyn INode)) {
        f(&mut cvar::Property("visible", &mut self.visible, false));
    }
}

impl SpawnerState {
    pub fn build_ui(self: &mut Self, world: &mut World, x: f32, y: f32, scale: f32) {
        if self.visible {
            widgets::Window::new(hash!(), vec2(10., 104.), vec2(200., 106.))
                .label("Spawn Entity")
                .ui(&mut *root_ui(), |ui| {
                    if ui.button(
                        None,
                        toggle_button_label(self.spawn_gostek, "Gostek").as_str(),
                    ) {
                        self.spawn_gostek = !self.spawn_gostek;
                    }
                    if ui.button(None, toggle_button_label(self.spawn_ak74, "AK74").as_str()) {
                        self.spawn_ak74 = !self.spawn_ak74;
                    }
                    if ui.button(
                        None,
                        toggle_button_label(self.spawn_particle, "Particle Emitter").as_str(),
                    ) {
                        self.spawn_particle = !self.spawn_particle;
                    }
                    if ui.button(None, toggle_button_label(self.spawn_ball, "Ball").as_str()) {
                        self.spawn_ball = !self.spawn_ball;
                    }
                });
        }

        if (mq::is_mouse_button_pressed(mq::MouseButton::Left)
            || mq::is_mouse_button_pressed(mq::MouseButton::Right))
            && !macroquad::ui::root_ui().is_mouse_over(Vec2::from(mq::mouse_position()))
        {
            let pos = calc::vec2(x.round() as f32, y.round() as f32);

            if self.spawn_gostek {
                log::debug!("Spawning Gostek");
                // cmd.spawn(Soldier::new(&MapSpawnpoint {
                //     active: false,
                //     x: pos.x as i32,
                //     y: pos.y as i32,
                //     team: 0,
                // }));
            }
            if self.spawn_ak74 {
                log::debug!("Spawning AK74");
                // cmd.spawn((
                //     components::Position(pos),
                //     components::Sprite {
                //         sprite: Arc::new(gfx::Weapon::Ak74),
                //         ..Default::default()
                //     },
                //     components::KineticBody {
                //         pos,
                //         old_pos: pos,
                //         velocity: calc::vec2(-1.0, -4.0),
                //         one_over_mass: 1.0,
                //         timestep: 1.0,
                //         gravity: constants::GRAV * 2.25,
                //         e_damping: 0.99,
                //         active: true,
                //         ..Default::default()
                //     },
                //     components::Collider {
                //         with: components::CollisionKind::Bullet(components::Team::None),
                //     },
                // ));
            }
            if self.spawn_particle {
                log::debug!("Spawning Particle Emitter");
                // cmd.spawn((
                //     components::Position(pos),
                //     components::ParticleEmitter {
                //         active: true,
                //         amount: 40,
                //         initial_direction_spread: gfx2d::math::PI,
                //         initial_velocity_randomness: 0.0,
                //         explosiveness: 0.0,
                //         body: components::KineticBody {
                //             velocity: calc::vec2(0.0, -1.0),
                //             e_damping: 1.0,
                //             ..Default::default()
                //         },
                //         ..Default::default()
                //     },
                // ));
            }
            if self.spawn_ball {
                log::debug!("Spawning Ball @{}", pos / scale);
                /* Create the bouncing ball. */
                let rigid_body = RigidBodyBundle {
                    position: (pos / scale).into(),
                    changes: RigidBodyChanges::all(), // FIXME: remove after implementing change detection system
                    ..Default::default()
                };
                let collider = ColliderBundle {
                    shape: ColliderShape::ball(0.75), // TODO: compute this value from Sprite size
                    material: ColliderMaterial {
                        restitution: 0.7,
                        ..Default::default()
                    },
                    changes: ColliderChanges::all(), // FIXME: remove after implementing change detection system
                    ..Default::default()
                };
                let ball = world.spawn(rigid_body);
                world.insert(ball, collider).unwrap();

                /* Ball that will be drawn */
                let sprite_scale = 0.5; // make the sprite half size than the actual PNG image
                world
                    .insert_one(
                        ball,
                        Sprite {
                            group: "Ball".into(),
                            name: format!("Ball{}", thread_rng().gen_range(1..=8)).into(),
                            transform: gfx2d::Transform::origin(
                                vec2(50., 50.) * (sprite_scale / -2.),
                                vec2(1.0, 1.0) * sprite_scale,
                                (0.0, vec2(50., 50.) * (sprite_scale / 2.)),
                            ),
                            ..Default::default()
                        },
                    )
                    .unwrap();
            }
        }
    }
}
