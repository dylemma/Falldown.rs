use amethyst::ecs::prelude::*;
use amethyst::ecs::storage::{TryDefault, UnprotectedStorage};
use amethyst::shrev::{ Event, EventChannel};
use hibitset::BitSetLike;
use std::ops::DerefMut;
use amethyst::ecs::storage::MaskedStorage;
use amethyst::ecs::world::Index;
use std::marker::PhantomData;

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

pub trait ToEvent<E> {
    fn to_event(&self) -> E;
}
impl<T> ToEvent<T> for T
where
    T: Event + Clone
{
    fn to_event(&self) -> T { self.clone() }
}

// -----------------

pub struct RemovalFlaggedStorage<C, E = C, T = DenseVecStorage<C>> {
    phantom: PhantomData<C>,
    channel: EventChannel<DetailedComponentEvent<E>>,
    storage: T,
}

impl<C, E, T> Default for RemovalFlaggedStorage<C, E, T>
where
    T: TryDefault,
    C: ToEvent<E>,
    E: Event,
{
    fn default() -> Self {
        RemovalFlaggedStorage {
            phantom: PhantomData,
            channel: EventChannel::<DetailedComponentEvent<E>>::new(),
            storage: T::unwrap_default(),
        }
    }
}

impl<C, E, T> UnprotectedStorage<C> for RemovalFlaggedStorage<C, E, T>
where
    C: Component + ToEvent<E>,
    E: Event + Clone,
    T: UnprotectedStorage<C>
{
//impl<C: Component + ToEvent<E>, E, T: UnprotectedStorage<C>> UnprotectedStorage<C> for RemovalFlaggedStorage<C, E, T> {
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
        self.channel.single_write(DetailedComponentEvent::Removed(id, removed.to_event()));
        removed
    }
}

impl<C, E, T> RemovalTracked<E> for RemovalFlaggedStorage<C, E, T> {
    fn removal_channel(&self) -> &EventChannel<DetailedComponentEvent<E>> { &self.channel }

    fn removal_channel_mut(&mut self) -> &mut EventChannel<DetailedComponentEvent<E>> { &mut self.channel }
}


// -----------------------

pub trait RemovalBroadcaster<T> {
    fn detailed_channel(&self) -> &EventChannel<DetailedComponentEvent<T>>;

    fn detailed_channel_mut(&mut self) -> &mut EventChannel<DetailedComponentEvent<T>>;

    fn register_detailed_reader(&mut self) -> ReaderId<DetailedComponentEvent<T>>;

    fn flag_detailed(&mut self, event: DetailedComponentEvent<T>);
}


impl<'e, C, E, D> RemovalBroadcaster<E> for Storage<'e, C, D>
where
    C: Component + ToEvent<E>,
    C::Storage: RemovalTracked<E>,
    E: Event,
    D: DerefMut<Target = MaskedStorage<C>>,
{
    fn detailed_channel(&self) -> &EventChannel<DetailedComponentEvent<E>> {
        unsafe { self.open() }.1.removal_channel()
    }

    fn detailed_channel_mut(&mut self) -> &mut EventChannel<DetailedComponentEvent<E>> {
        unsafe { self.open() }.1.removal_channel_mut()
    }

    fn register_detailed_reader(&mut self) -> ReaderId<DetailedComponentEvent<E>> {
        self.detailed_channel_mut().register_reader()
    }

    fn flag_detailed(&mut self, event: DetailedComponentEvent<E>) {
        self.detailed_channel_mut().single_write(event);
    }
}

