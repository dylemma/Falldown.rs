use amethyst::{
    core::{
        nalgebra::Isometry3,
        transform::Transform,
    },
    ecs::{
        prelude::*,
        ReaderId,
    },
};
use hibitset::DrainableBitSet;
use ncollide3d::events::ContactEvent;
use ncollide3d::world::CollisionObjectHandle;
use ncollide3d::world::CollisionWorld;

use crate::falldown::{Collider, EntityCollisionWorld};
use crate::storage::DetailedComponentEvent;
use crate::storage::RemovalBroadcaster;

#[derive(Default)]
pub struct SyncCollisionWorld {
    collision_reader_id: Option<ReaderId<DetailedComponentEvent<Option<CollisionObjectHandle>>>>,
    removed_colliders_vec: Vec<CollisionObjectHandle>,
    added_colliders: BitSet,
}
impl SyncCollisionWorld {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'s> System<'s> for SyncCollisionWorld {
    type SystemData = (
        Entities<'s>,
        WriteExpect<'s, EntityCollisionWorld>,
        WriteStorage<'s, Collider>,
        ReadStorage<'s, Transform>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut collision_world,
            mut colliders,
            transforms,
        ) = data;

        // collect the Added and Removed colliders from the event channel
        self.removed_colliders_vec.clear();
        self.added_colliders.clear();
        let events = colliders.detailed_channel().read(self.collision_reader_id.as_mut().unwrap());
        for event in events {
            match event {
                DetailedComponentEvent::Modified(_) => {},
                DetailedComponentEvent::Inserted(id) => {
                    self.added_colliders.add(*id);
                },
                DetailedComponentEvent::Removed(_, Some(collider_handle)) => {
                    self.removed_colliders_vec.push(*collider_handle);
                },
                DetailedComponentEvent::Removed(_, None) => {
                    println!("Removed a collider that didn't have a handle set");
                },
            }
        }

        // insert any new colliders into the world
        for id in self.added_colliders.drain() {
            let entity = entities.entity(id);
            let collider = colliders.get_mut(entity)
                .expect("Failed to get a newly-added collider from storage");

            let handle = collision_world
                .add(
                    Isometry3::identity(),
                    collider.shape().clone(),
                    collider.groups().clone(),
                    collider.query_type().clone(),
                    entity
                )
                .handle();
            collider.handle = Some(handle);
            // println!("Added {:?} to the world", handle);
        }

        // remove any deleted colliders from the world
        if !self.removed_colliders_vec.is_empty() {
            // println!("Removing {:?}", self.removed_colliders_vec);
            collision_world.remove(&self.removed_colliders_vec);
        }

        // copy the transforms from all collider entities into the collision world
        for (collider, transform) in (&colliders, &transforms).join() {
            collision_world.set_position(collider.handle.unwrap(), *transform.isometry());
        }

        // update the collision world
        collision_world.update();

        // process collision/contact events
        // TODO: do something interesting here instead of just println!
        for event in collision_world.contact_events() {
            if let &ContactEvent::Started(collider1, collider2) = event {
                println!("Collision started between {:?} and {:?}", collider1, collider2);
            }
            if let &ContactEvent::Stopped(collider1, collider2) = event {
                println!("Collision ended between {:?} and {:?}", collider1, collider2);
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        res.entry::<EntityCollisionWorld>()
            .or_insert_with(|| CollisionWorld::new(10.0));

        let mut collision_storage: WriteStorage<Collider> = SystemData::fetch(res);
        self.collision_reader_id = Some(collision_storage.register_detailed_reader());
    }
}