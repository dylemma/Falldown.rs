use std::f32::consts::PI;

use amethyst::{
    core::{
        timing::Time,
        transform::Transform,
    },
    ecs::prelude::{Entities, Join, Read, ReadStorage, System, WriteStorage},
    renderer::{Rgba, SpriteRender},
};
use ncollide3d::shape::Ball;
use ncollide3d::shape::ShapeHandle;
use ncollide3d::world::GeometricQueryType;
use rand::{Rng, thread_rng};

use crate::falldown::{Affiliation, ARENA_HEIGHT, ARENA_WIDTH, Collider, ColorPallatte, FallingObject, Spawner};
use crate::falldown::enemy_collision_group;
use crate::util::RngExtras;

pub struct SpawnerSystem;

const SPAWNED_OBJECT_RADIUS: f32 = 5.0;

impl<'s> System<'s> for SpawnerSystem {
    type SystemData = (
        WriteStorage<'s, Spawner>,
        ReadStorage<'s, ColorPallatte>,
        Read<'s, Time>,
        // extra fields required in order to spawn entities with those fields
        Entities<'s>,
        WriteStorage<'s, Collider>,
        WriteStorage<'s, Affiliation>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, FallingObject>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, Rgba>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut spawners,
            pallettes,
            time,
            entities,
            mut collision_objects,
            mut affiliations,
            mut transforms,
            mut falling_objects,
            mut sprites,
            mut colors
        ) = data;

        for (s, p) in (&mut spawners, &pallettes).join() {
            let spawner: &mut Spawner = s;
            let pallatte: &ColorPallatte = p;

            let should_spawn = spawner.advance_and_reset(time.delta_seconds()) && spawner.remaining > 0;
            if should_spawn {
                spawner.remaining -= 1;
                let mut rng = thread_rng();

                // pick a random starting position along the top of the screen
                let mut transform = Transform::default();
                transform.set_xyz(
                    rng.gen_range(0.0, ARENA_WIDTH),
                    ARENA_HEIGHT + SPAWNED_OBJECT_RADIUS,
                    0.0,
                );

                // randomize the falling object's speed and spin
                let object = FallingObject {
                    fall_rate: rng.gen_range(60.0, 120.0),
                    spin_rate: rng.gen_range(0.25, 1.5) * PI * rng.plus_or_minus(1.0),
                    radius: SPAWNED_OBJECT_RADIUS,
                };

                let collider = Collider::new(
                    ShapeHandle::new(Ball::new(SPAWNED_OBJECT_RADIUS)),
                    enemy_collision_group(),
                    GeometricQueryType::Contacts(0f32, 0f32),
                );

                entities.build_entity()
                    .with(Affiliation::Enemy, &mut affiliations)
                    .with(collider, &mut collision_objects)
                    .with(transform, &mut transforms)
                    .with(object, &mut falling_objects)
                    .with(spawner.sprite(), &mut sprites)
                    .with(pallatte.next_random(), &mut colors)
                    .build();
            }
        }
    }
}



