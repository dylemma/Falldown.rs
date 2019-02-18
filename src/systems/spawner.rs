use amethyst::{
    core::{
        timing::Time,
        transform::Transform,
    },
    ecs::prelude::{Entities, Join, Read, ReadStorage, System, WriteStorage},
    renderer::{Rgba, SpriteRender},
};
use rand::{Rng, thread_rng};
use std::f32::consts::PI;

use crate::falldown::{ARENA_WIDTH, ARENA_HEIGHT, ColorPallatte, FallingObject, Spawner};
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
        WriteStorage<'s, Transform>,
        WriteStorage<'s, FallingObject>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, Rgba>,
    );

    fn run(&mut self, (mut spawners, pallettes, time, entities, mut transforms, mut falling_objects, mut sprites, mut colors): Self::SystemData) {
        for (s, p) in (&mut spawners, &pallettes).join() {
            let spawner: &mut Spawner = s;
            let pallatte: &ColorPallatte = p;

            let should_spawn = spawner.advance_and_reset(time.delta_seconds());
            if should_spawn {
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

                entities.build_entity()
                    .with(transform, &mut transforms)
                    .with(object, &mut falling_objects)
                    .with(spawner.sprite(), &mut sprites)
                    .with(pallatte.next_random(), &mut colors)
                    .build();
            }
        }
    }
}



