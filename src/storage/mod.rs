use amethyst::ecs::prelude::*;
use amethyst::ecs::storage::{TryDefault, UnprotectedStorage};
use amethyst::shrev::{ Event, EventChannel};
use hibitset::BitSetLike;
use std::ops::DerefMut;
use amethyst::ecs::storage::MaskedStorage;
use amethyst::ecs::world::Index;

pub trait RemovalTracked<C> {
    fn removal_channel(&self) -> &EventChannel<DetailedComponentEvent<C>>;

    fn removal_channel_mut(&mut self) -> &mut EventChannel<DetailedComponentEvent<C>>;
}

// -----------------

pub enum DetailedComponentEvent<T> {
    Inserted(Index),
    Modified(Index),
    Removed(Index, T),
}

// -----------------

pub struct RemovalFlaggedStorage<C, T = DenseVecStorage<C>> {
    channel: EventChannel<DetailedComponentEvent<C>>,
    storage: T,
}

impl<C, T> Default for RemovalFlaggedStorage<C, T>
where
    T: TryDefault,
    C: Event,
{
    fn default() -> Self {
        RemovalFlaggedStorage {
            channel: EventChannel::<DetailedComponentEvent<C>>::new(),
            storage: T::unwrap_default(),
        }
    }
}

impl<C: Component + Event + Clone, T: UnprotectedStorage<C>> UnprotectedStorage<C> for RemovalFlaggedStorage<C, T> {
    unsafe fn clean<B: BitSetLike>(&mut self, has: B) { self.storage.clean(has); }

    unsafe fn get(&self, id: u32) -> &C { self.storage.get(id) }

    unsafe fn get_mut(&mut self, id: u32) -> &mut C {
        self.channel.single_write(DetailedComponentEvent::Modified(id));
        self.storage.get_mut(id)
    }

    unsafe fn insert(&mut self, id: u32, value: C) {
        self.channel.single_write(DetailedComponentEvent::Inserted(id));
        self.storage.insert(id, value)
    }

    unsafe fn remove(&mut self, id: u32) -> C {
        let removed = self.storage.remove(id);
        self.channel.single_write(DetailedComponentEvent::Removed(id, removed.clone()));
        removed
    }
}

impl<C, T> RemovalTracked<C> for RemovalFlaggedStorage<C, T> {
    fn removal_channel(&self) -> &EventChannel<DetailedComponentEvent<C>> { &self.channel }

    fn removal_channel_mut(&mut self) -> &mut EventChannel<DetailedComponentEvent<C>> { &mut self.channel }
}


// -----------------------

pub trait RemovalBroadcaster<T> {
    fn detailed_channel(&self) -> &EventChannel<DetailedComponentEvent<T>>;

    fn detailed_channel_mut(&mut self) -> &mut EventChannel<DetailedComponentEvent<T>>;

    fn register_detailed_reader(&mut self) -> ReaderId<DetailedComponentEvent<T>>;

    fn flag_detailed(&mut self, event: DetailedComponentEvent<T>);
}


impl<'e, T, D> RemovalBroadcaster<T> for Storage<'e, T, D>
where
    T: Component,
    T::Storage: RemovalTracked<T>,
    T: Event,
    D: DerefMut<Target = MaskedStorage<T>>,
{
    fn detailed_channel(&self) -> &EventChannel<DetailedComponentEvent<T>> {
        unsafe { self.open() }.1.removal_channel()
    }

    fn detailed_channel_mut(&mut self) -> &mut EventChannel<DetailedComponentEvent<T>> {
        unsafe { self.open() }.1.removal_channel_mut()
    }

    fn register_detailed_reader(&mut self) -> ReaderId<DetailedComponentEvent<T>> {
        self.detailed_channel_mut().register_reader()
    }

    fn flag_detailed(&mut self, event: DetailedComponentEvent<T>) {
        self.detailed_channel_mut().single_write(event);
    }
}

