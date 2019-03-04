use amethyst::{
    ecs::prelude::*,
    shrev::EventChannel,
};
use crate::falldown::{Affiliation, EntityContactEvent};

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
            if let Some((player, other, other_aff)) = extract_player_contact(&affiliations, entity1, entity2) {
                match *other_aff {
                    Affiliation::Enemy => {
                        println!("Player collided with enemy {:?}", other);
                        entities.delete(*other).unwrap();
                    },
                    Affiliation::Player => {
                        println!("Players collided? {:?} with {:?}", player, other);
                    },
                }
            }
        }
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        let mut contact_events = res.fetch_mut::<EventChannel<EntityContactEvent>>();
        self.contact_reader = Some(contact_events.register_reader());
    }
}

// extracts a (player_entity, other_entity, other_affiliation) tuple from a pair of entities that "collided"
fn extract_player_contact<'s>(affiliations: &'s ReadStorage<Affiliation>, entity1: &'s Entity, entity2: &'s Entity) -> Option<(&'s Entity, &'s Entity, &'s Affiliation)> {
    let affiliation1 = affiliations.get(*entity1);
    let affiliation2 = affiliations.get(*entity2);

    match (affiliation1, affiliation2) {
        // player on the left
        (Some(Affiliation::Player), Some(other_aff)) => Some((entity1, entity2, other_aff)),

        // player on the right
        (Some(other_aff), Some(Affiliation::Player)) => Some((entity2, entity1, other_aff)),

        // no player found
        _ => {
            println!("Non-player-related collision between {:?} and {:?}", entity1, entity2);
            None
        },
    }
}