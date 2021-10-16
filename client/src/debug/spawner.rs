use super::*;
use crate::{components, physics::*, rand, MapSpawnpoint, Soldier};

#[derive(Default)]
pub struct SpawnerState {
    pub(crate) visible: bool,

    spawn_entity: SpawnEntity,
}

impl IVisit for SpawnerState {
    fn visit(&mut self, f: &mut dyn FnMut(&mut dyn INode)) {
        f(&mut cvar::Property("visible", &mut self.visible, false));
    }
}

#[derive(Debug, PartialEq)]
enum SpawnEntity {
    Nothing,
    Gostek,
    AK47,
    ParticleEmitter,
    Ball,
}

impl Default for SpawnEntity {
    fn default() -> Self {
        SpawnEntity::Nothing
    }
}

impl SpawnerState {
    pub fn build_ui(
        &mut self,
        egui_ctx: &egui::CtxRef,
        world: &mut World,
        x: f32,
        y: f32,
        scale: f32,
        gravity: f32,
    ) {
        if self.visible {
            let mut visible = self.visible;
            egui::Window::new("Spawn Entity")
                .open(&mut visible)
                .resizable(false)
                .show(egui_ctx, |ui| {
                    ui.selectable_value(&mut self.spawn_entity, SpawnEntity::Nothing, " Nothing ");
                    ui.selectable_value(&mut self.spawn_entity, SpawnEntity::Gostek, "Gostek");
                    ui.selectable_value(&mut self.spawn_entity, SpawnEntity::AK47, "AK74");
                    ui.selectable_value(
                        &mut self.spawn_entity,
                        SpawnEntity::ParticleEmitter,
                        "Particle Emitter",
                    );
                    ui.selectable_value(&mut self.spawn_entity, SpawnEntity::Ball, "Ball");
                });
            self.visible = visible;
        }

        if egui_ctx.input().pointer.any_pressed() && !egui_ctx.wants_pointer_input() {
            let pos = vec2(x.round() as f32, y.round() as f32);

            match self.spawn_entity {
                SpawnEntity::Nothing => {}
                SpawnEntity::Gostek => {
                    log::debug!("Spawning Gostek");
                    world.spawn((Soldier::new(
                        &MapSpawnpoint {
                            active: false,
                            x: pos.x as i32,
                            y: pos.y as i32,
                            team: 0,
                        },
                        gravity,
                    ),));
                }
                SpawnEntity::AK47 => {
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
                SpawnEntity::ParticleEmitter => {
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
                SpawnEntity::Ball => {
                    log::debug!("Spawning Ball @{}", pos / scale);
                    /* Create the bouncing ball. */
                    let rigid_body = RigidBodyBundle {
                        position: (pos / scale).into(),
                        ..Default::default()
                    };
                    let collider = ColliderBundle {
                        shape: ColliderShape::ball(0.75), // TODO: compute this value from Sprite size
                        material: ColliderMaterial {
                            restitution: 0.7,
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    let ball = world.spawn(rigid_body);
                    world.insert(ball, collider).unwrap();

                    /* Ball that will be drawn */
                    let sprite_scale = 0.5; // make the sprite half size than the actual PNG image
                    world
                        .insert_one(
                            ball,
                            components::Sprite {
                                group: "Ball".into(),
                                name: format!("Ball{}", (rand::rand() % 8) + 1),
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
}
