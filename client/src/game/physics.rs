use crate::{cvars::Config, MapFile, PolyType};
use ::resources::Resources;
use enumflags2::{bitflags, BitFlags};
use hecs::World;
pub use rapier2d::prelude::*;
pub use soldank_shared::physics::*;

#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InteractionFlag {
    Player,
    Bullet,
    Alpha,
    Bravo,
    Charlie,
    Delta,
    Flag,
    Flagger,
}

pub struct SameParentFilter;
impl PhysicsHooksWithWorld for SameParentFilter {
    fn filter_contact_pair(
        &self,
        context: &PairFilterContext<RigidBodyComponentsSet, ColliderComponentsSet>,
        world: &World,
    ) -> Option<SolverFlags> {
        // if collider1 and collider2 have the same Parent() component
        // or collider1 is Parent() of collider2
        // or collider2 is Parent() of collider1
        // then there is no contact

        let collider1 = context.collider1.entity();
        let collider2 = context.collider2.entity();
        let parent1 = world.get::<Parent>(collider1).ok();
        let parent2 = world.get::<Parent>(collider2).ok();

        if let Some(parent1) = parent1 {
            if let Some(parent2) = parent2 {
                if **parent1 == **parent2 || **parent1 == collider2 || **parent2 == collider1 {
                    return None;
                }
            } else if **parent1 == collider2 {
                return None;
            }
        } else if let Some(parent2) = parent2 {
            if **parent2 == collider1 {
                return None;
            }
        }

        Some(SolverFlags::all())
    }
}

#[derive(Debug)]
pub struct PreviousPhysics {
    pub position: Isometry<Real>,
    pub linvel: Vector<Real>,
    pub angvel: AngVector<Real>,
    pub force: Vector<Real>,
    pub torque: AngVector<Real>,
}

impl Default for PreviousPhysics {
    fn default() -> Self {
        Self {
            position: Isometry::identity(),
            linvel: Default::default(),
            angvel: Default::default(),
            force: Default::default(),
            torque: Default::default(),
        }
    }
}

pub fn update_previous_physics(world: &mut World) {
    for (_entity, (mut prev, pos, vel, force)) in world
        .query::<(
            &mut PreviousPhysics,
            Option<&RigidBodyPosition>,
            Option<&RigidBodyVelocity>,
            Option<&RigidBodyForces>,
        )>()
        .iter()
    {
        if let Some(pos) = pos {
            prev.position = pos.position;
        }
        if let Some(vel) = vel {
            prev.linvel = vel.linvel;
            prev.angvel = vel.angvel;
        }
        if let Some(force) = force {
            prev.force = force.force;
            prev.torque = force.torque;
        }
    }
}

impl From<PolyType> for InteractionGroups {
    fn from(polytype: PolyType) -> Self {
        let mut memberships = BitFlags::<InteractionFlag>::all();
        let mut filter = BitFlags::<InteractionFlag>::all();

        match polytype {
            PolyType::Normal
            | PolyType::Ice
            | PolyType::Deadly
            | PolyType::BloodyDeadly
            | PolyType::Hurts
            | PolyType::Regenerates
            | PolyType::Lava
            | PolyType::Bouncy
            | PolyType::Explosive
            | PolyType::HurtsFlaggers
            | PolyType::Background
            | PolyType::BackgroundTransition => {}
            PolyType::OnlyBulletsCollide => {
                filter.remove(InteractionFlag::Player);
            }
            PolyType::OnlyPlayersCollide => {
                filter.remove(InteractionFlag::Bullet);
            }
            PolyType::NoCollide => {
                filter = BitFlags::<InteractionFlag>::empty();
            }
            PolyType::AlphaBullets => {
                filter = InteractionFlag::Bullet | InteractionFlag::Alpha;
            }
            PolyType::AlphaPlayers => {
                filter = InteractionFlag::Player | InteractionFlag::Alpha;
            }
            PolyType::BravoBullets => {
                filter = InteractionFlag::Bullet | InteractionFlag::Bravo;
            }
            PolyType::BravoPlayers => {
                filter = InteractionFlag::Player | InteractionFlag::Bravo;
            }
            PolyType::CharlieBullets => {
                filter = InteractionFlag::Bullet | InteractionFlag::Charlie;
            }
            PolyType::CharliePlayers => {
                filter = InteractionFlag::Player | InteractionFlag::Charlie;
            }
            PolyType::DeltaBullets => {
                filter = InteractionFlag::Bullet | InteractionFlag::Delta;
            }
            PolyType::DeltaPlayers => {
                filter = InteractionFlag::Player | InteractionFlag::Delta;
            }
            PolyType::OnlyFlaggers => {
                filter = InteractionFlag::Flagger.into();
            }
            PolyType::NotFlaggers => {
                filter.remove(InteractionFlag::Flagger);
            }
            PolyType::FlagCollide => {
                filter = InteractionFlag::Flag.into();
            }
        }

        InteractionGroups {
            memberships: memberships.bits(),
            filter: filter.bits(),
        }
    }
}

pub fn create_map_colliders(world: &mut World, resources: &Resources, config: &Config) {
    let map = resources.get::<MapFile>().unwrap();
    let scale = config.phys.scale;

    for polygon in map.polygons.iter() {
        match polygon.polytype {
            PolyType::NoCollide | PolyType::Background | PolyType::BackgroundTransition => continue,
            _ => {}
        }

        let vertices: Vec<Point<Real>> = polygon
            .vertices
            .iter()
            .map(|v| point![v.x / scale, v.y / scale])
            .collect();
        let mut collider = ColliderBundle {
            shape: ColliderShape::triangle(vertices[0], vertices[1], vertices[2]),
            flags: ColliderFlags {
                collision_groups: polygon.polytype.into(),
                ..Default::default()
            },
            ..Default::default()
        };
        if polygon.polytype == PolyType::Bouncy {
            collider.material.restitution = polygon.bounciness;
        }
        world.spawn(collider);
    }

    for coll in map.colliders.iter() {
        if !coll.active {
            continue;
        }

        let collider = ColliderBundle {
            shape: ColliderShape::ball(coll.diameter / scale / 2.),
            position: vector![coll.x / scale, coll.y / scale].into(),
            flags: ColliderFlags {
                collision_groups: InteractionGroups::all(),
                ..Default::default()
            },
            ..Default::default()
        };
        world.spawn(collider);
    }
}
