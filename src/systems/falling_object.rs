use crate::falldown::FallingObject;

use amethyst::{
    core::{
        nalgebra::Vector3,
        timing::Time,
        transform::Transform,
    },

    ecs::prelude::{Join, Read, ReadStorage, WriteStorage, System},
};

pub struct FallingObjectSystem;

impl<'s> System<'s> for FallingObjectSystem {
    type SystemData = (
        ReadStorage<'s, FallingObject>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
    );

    fn run(&mut self, (falling_objects, mut transforms, time): Self::SystemData) {
        use crate::falldown::ARENA_HEIGHT;

        for (o, t) in (&falling_objects, &mut transforms).join() {
            // alias to help intelliJ out
            let obj: &FallingObject = o;
            let transform: &mut Transform = t;

            // move the object downward
            let y_pos = transform.translation().y;
            let mut next_y = y_pos - obj.fall_rate * time.delta_seconds();
            if next_y < -obj.radius {
                // reset the object to the top of the arena
                // TODO: despawn the object instead
                next_y = ARENA_HEIGHT + obj.radius;
            }
            transform.set_y(next_y);

            // spin the object
            transform.rotate_global(Vector3::z_axis(), obj.spin_rate * time.delta_seconds());
        }
    }
}