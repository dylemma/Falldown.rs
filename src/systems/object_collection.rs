use amethyst::{
    ecs::prelude::*,
    shrev::EventChannel,
};
use crate::falldown::{Affiliation, CollectionEvent, EntityContactEvent};
use std::convert::Into;

pub struct ObjectCollection {
    contact_reader: Option<ReaderId<EntityContactEvent>>,
}

impl ObjectCollection {
    pub fn new() -> ObjectCollection {
        ObjectCollection {
            contact_reader: None,
        }
    }
}

impl<'s> System<'s> for ObjectCollection {
    type SystemData = (
        Read<'s, EventChannel<EntityContactEvent>>,
        ReadStorage<'s, Affiliation>,
        Entities<'s>
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            contact_events,
            affiliations,
            entities,
        ) = data;

        for (entity1, entity2, _contact) in contact_events.read(self.contact_reader.as_mut().unwrap()) {
            let affiliation1 = affiliations.get(*entity1);
            let affiliation2 = affiliations.get(*entity2);

            match ((entity1, affiliation1, entity2, affiliation2)).into() {
                CollectionEvent::CaughtBlock { block, color, is_correct, .. } => {
                    if is_correct {
                        println!("Player caught {:?} block {}", color, block.id());
                        entities.delete(*block).unwrap();
                    } else {
                        println!("Player ran into {:?} block {}", color, block.id());
                    }
                },
                CollectionEvent::Unknown => {
                    println!("Some other collision happened between {:?} and {:?}", entity1, entity2);
                },
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        let mut contact_events = res.fetch_mut::<EventChannel<EntityContactEvent>>();
        self.contact_reader = Some(contact_events.register_reader());
    }
}
