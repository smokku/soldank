use crate::{components, constants::*, cvars::Config};
use gfx2d::{macroquad::prelude as mq, math::*};
use hecs::World;
use rapier2d::prelude::*;
use resources::Resources;

pub fn init(world: &mut World, resources: &mut Resources) {
    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();

    /* Create the ground. */
    let collider = ColliderBuilder::cuboid(100.0, 0.1)
        .translation(vector![0.0, -20.0])
        .build();
    collider_set.insert(collider);

    /* Create the bouncing ball. */
    let rigid_body = RigidBodyBuilder::new_dynamic()
        .translation(vector![0.0, -30.0])
        .build();
    let collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
    let ball_body_handle = rigid_body_set.insert(rigid_body);
    collider_set.insert_with_parent(collider, ball_body_handle, &mut rigid_body_set);

    /* Ball entity that will be drawn */
    let sprite_scale = 0.5;
    world.spawn((
        components::Position::new(0.0, 0.0),
        components::Sprite {
            group: "Ball".into(),
            name: "Ball1".into(),
            transform: gfx2d::Transform::origin(
                vec2(50., 50.) * (sprite_scale / -2.),
                vec2(1.0, 1.0) * sprite_scale,
                (0.0, vec2(50., 50.) * (sprite_scale / 2.)),
            ),
            ..Default::default()
        },
        ball_body_handle,
    ));

    /* Create other structures necessary for the simulation. */
    resources.insert(PhysicsPipeline::new());
    resources.insert(IslandManager::new());
    resources.insert(BroadPhase::new());
    resources.insert(NarrowPhase::new());
    resources.insert(rigid_body_set);
    resources.insert(collider_set);
    resources.insert(JointSet::new());
    resources.insert(CCDSolver::new());
}

pub fn step(resources: &Resources) {
    let gravity = vector![0.0, 9.81];
    let integration_parameters = IntegrationParameters {
        dt: TIMESTEP_RATE as f32,
        ..Default::default()
    };
    let physics_hooks = ();
    let event_handler = ();

    let mut physics_pipeline = resources.get_mut::<PhysicsPipeline>().unwrap();
    let mut island_manager = resources.get_mut::<IslandManager>().unwrap();
    let mut broad_phase = resources.get_mut::<BroadPhase>().unwrap();
    let mut narrow_phase = resources.get_mut::<NarrowPhase>().unwrap();
    let mut rigid_body_set = resources.get_mut::<RigidBodySet>().unwrap();
    let mut collider_set = resources.get_mut::<ColliderSet>().unwrap();
    let mut joint_set = resources.get_mut::<JointSet>().unwrap();
    let mut ccd_solver = resources.get_mut::<CCDSolver>().unwrap();

    physics_pipeline.step(
        &gravity,
        &integration_parameters,
        &mut island_manager,
        &mut broad_phase,
        &mut narrow_phase,
        &mut rigid_body_set,
        &mut collider_set,
        &mut joint_set,
        &mut ccd_solver,
        &physics_hooks,
        &event_handler,
    );
}

pub fn sync_to_world(world: &mut World, resources: &Resources, timecur: f64) {
    let rigid_body_set = resources.get_mut::<RigidBodySet>().unwrap();
    let scale = resources.get::<Config>().unwrap().phys.scale;

    for (_entity, (body_handle, position, sprite)) in world
        .query::<(
            &RigidBodyHandle,
            &mut components::Position,
            &mut components::Sprite,
        )>()
        .iter()
    {
        let ball_body = &rigid_body_set[*body_handle];
        position.x = ball_body.translation().x * scale;
        position.y = ball_body.translation().y * scale;
        if let gfx2d::Transform::FromOrigin { rot, .. } = &mut sprite.transform {
            rot.0 = timecur as f32 % (2. * PI);
        }
    }
}

pub fn render(world: &World, resources: &Resources) {
    let rigid_body_set = resources.get_mut::<RigidBodySet>().unwrap();
    let scale = resources.get::<Config>().unwrap().phys.scale;

    for (_entity, body_handle) in world.query::<&RigidBodyHandle>().iter() {
        let body = &rigid_body_set[*body_handle];
        let tr = body.translation();
        let center = vec2(tr.x, tr.y) * scale;
        mq::draw_circle(center.x, center.y, 1.5, mq::YELLOW);
        mq::draw_circle(center.x, center.y, 0.75, mq::BLACK);
    }
}
