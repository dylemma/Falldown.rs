use std::collections::hash_map::RandomState;
use std::collections::HashMap;

use amethyst::{
    ecs::{
        prelude::*,
        ReaderId,
        world::Index,
    },
    prelude::*,
};
use hibitset::{BitSetLike, DrainableBitSet};
use ncollide3d::world::CollisionObjectHandle;

use crate::falldown::{Collider, EntityCollisionWorld};
use crate::storage::DetailedComponentEvent;
use crate::storage::RemovalBroadcaster;

#[derive(Default)]
pub struct SyncCollisionWorld {
    collision_reader_id: Option<ReaderId<DetailedComponentEvent<Collider>>>,
    removed_colliders_vec: Vec<CollisionObjectHandle>,
}
impl SyncCollisionWorld {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'s> System<'s> for SyncCollisionWorld {
    type SystemData = (
        Entities<'s>,
        Write<'s, EntityCollisionWorld>,
        WriteStorage<'s, Collider>,
        ReadStorage<'s, crate::falldown::FallingObject>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut collision_world,
            colliders,
            falling_storage,
        ) = data;

        self.removed_colliders_vec.clear();
        let events = colliders.detailed_channel().read(self.collision_reader_id.as_mut().unwrap());
        for event in events {
            match event {
                DetailedComponentEvent::Modified(_) => {},
                DetailedComponentEvent::Inserted(_) => {},
                DetailedComponentEvent::Removed(_, collider) => {
                    self.removed_colliders_vec.push(collider.0);
                },
            }
        }

        if !self.removed_colliders_vec.is_empty() {
            println!("Removing {:?}", self.removed_colliders_vec);
            collision_world.0.remove(&self.removed_colliders_vec);
        }

    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        let mut collision_storage: WriteStorage<Collider> = SystemData::fetch(res);
        self.collision_reader_id = Some(collision_storage.register_detailed_reader());
    }
}