use crate::falldown::FallingObject;

use amethyst::{
    core::{
        nalgebra::Vector3,
        timing::Time,
        transform::Transform,
    },

    ecs::prelude::{Entities, Join, Read, ReadStorage, WriteStorage, System},
};

pub struct FallingObjectSystem;

impl<'s> System<'s> for FallingObjectSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, FallingObject>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
    );

    fn run(&mut self, (entities, falling_objects, mut transforms, time): Self::SystemData) {
        for (e, o, t) in (&*entities, &falling_objects, &mut transforms).join() {
            // alias to help intelliJ out
            let obj: &FallingObject = o;
            let transform: &mut Transform = t;

            // move the object downward
            transform.translate_y(-obj.fall_rate * time.delta_seconds());
            if transform.translation().y < -obj.radius {
                // delete objects that reach the bottom of the screen
                entities.delete(e).unwrap();
            }

            // spin the object
            transform.rotate_global(Vector3::z_axis(), obj.spin_rate * time.delta_seconds());
        }
    }
}