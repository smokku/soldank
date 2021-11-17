use crate::{cvars::Config, MapFile, PolyType};
use ::resources::Resources;
use enumflags2::{bitflags, BitFlags};
use hecs::{Entity, World};
use multiqueue2::{BroadcastReceiver, BroadcastSender};
pub use rapier2d::prelude::*;
pub use soldank_shared::physics::*;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

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

pub struct PhysicsEventHandler {
    event_sender: Arc<Mutex<BroadcastSender<ContactEvent>>>,
}

impl PhysicsEventHandler {
    pub fn new(event_sender: BroadcastSender<ContactEvent>) -> Self {
        PhysicsEventHandler {
            event_sender: Arc::new(Mutex::new(event_sender)),
        }
    }
}

impl EventHandler for PhysicsEventHandler {
    fn handle_intersection_event(&self, _event: IntersectionEvent) {}

    fn handle_contact_event(&self, event: ContactEvent, _contact_pair: &ContactPair) {
        if let Err(err) = self.event_sender.lock().unwrap().try_send(event) {
            log::error!("Cannot send ContactEvent: {}", err);
        }
    }
}

#[derive(Default, Debug)]
pub struct Contact {
    pub timestamp: f64,
    pub entities: HashSet<Entity>,
}

pub fn process_contact_events(world: &mut World, resources: &Resources, now: f64) {
    let event_recv = resources
        .get_mut::<Arc<Mutex<BroadcastReceiver<ContactEvent>>>>()
        .unwrap();
    for event in event_recv.lock().unwrap().try_iter() {
        // log::debug!("Received contact event: {:?}", event);
        match event {
            ContactEvent::Started(handle1, handle2) => {
                let entity1: Entity = handle1.entity();
                let entity2: Entity = handle2.entity();
                if let Ok(mut contact) = world.get_mut::<Contact>(entity1) {
                    contact.timestamp = now;
                    contact.entities.insert(entity2);
                }
                if let Ok(mut contact) = world.get_mut::<Contact>(entity2) {
                    contact.timestamp = now;
                    contact.entities.insert(entity1);
                }
            }
            ContactEvent::Stopped(handle1, handle2) => {
                let entity1: Entity = handle1.entity();
                let entity2: Entity = handle2.entity();
                if let Ok(mut contact) = world.get_mut::<Contact>(entity1) {
                    contact.entities.remove(&entity2);
                }
                if let Ok(mut contact) = world.get_mut::<Contact>(entity2) {
                    contact.entities.remove(&entity1);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct PreviousPhysics {
    pub position: Isometry<Real>,
    pub linvel: Vector<Real>,
    pub angvel: AngVector<Real>,
    pub force: Vector<Real>,
    pub torque: AngVector<Real>,
    pub last_contact: f64,
}

impl Default for PreviousPhysics {
    fn default() -> Self {
        Self {
            position: Isometry::identity(),
            linvel: Default::default(),
            angvel: Default::default(),
            force: Default::default(),
            torque: Default::default(),
            last_contact: 0.,
        }
    }
}

pub fn update_previous_physics(world: &mut World) {
    for (_entity, (mut prev, pos, vel, force, cont)) in world
        .query::<(
            &mut PreviousPhysics,
            Option<&RigidBodyPosition>,
            Option<&RigidBodyVelocity>,
            Option<&RigidBodyForces>,
            Option<&Contact>,
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
        if let Some(contact) = cont {
            prev.last_contact = contact.timestamp;
        }
    }
}

impl From<PolyType> for InteractionGroups {
    fn from(polytype: PolyType) -> Self {
        let memberships = BitFlags::<InteractionFlag>::all();
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
