use std::collections::VecDeque;

use amethyst::{
    assets::{Asset, AssetStorage, Completion, Handle, Loader, ProcessingState, Progress, ProgressCounter},
    core::{
        nalgebra::Vector3,
        transform::{
            components::Parent,
            Transform,
        },
    },
    ecs::prelude::{Component, DenseVecStorage, Entity, HashMapStorage, VecStorage},
    Error,
    prelude::*,
    renderer::{
        Camera, PngFormat, Projection, Rgba, SpriteRender, SpriteSheet, SpriteSheetFormat,
        SpriteSheetHandle, Texture, TextureMetadata, Transparent,
    },
};
use ncollide3d::{
    shape::{Ball, ShapeHandle},
    world::{CollisionGroups, CollisionObjectHandle, CollisionWorld, GeometricQueryType},
};
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};

use crate::storage::{RemovalFlaggedStorage, ToEvent};

// ------------------------------------

pub const ARENA_HEIGHT: f32 = 300.;
pub const ARENA_WIDTH: f32 = 300.;

// ------------------------------------

pub fn player_collision_group() -> CollisionGroups {
    CollisionGroups::new()
        .with_membership(&[0])
        .with_whitelist(&[1])
}
pub fn enemy_collision_group() -> CollisionGroups {
    CollisionGroups::new()
        .with_membership(&[1])
        .with_whitelist(&[0])
}

// ------------------------------------

#[derive(Default, Debug)]
pub struct FallingObject {
    pub fall_rate: f32,
    pub spin_rate: f32,
    pub radius: f32,
}

impl Component for FallingObject {
    type Storage = VecStorage<Self>;
}

// ------------------------------------

pub struct FollowMouse {
    pub x_ratio: f32,
    pub y_ratio: f32,
}

impl Component for FollowMouse {
    type Storage = DenseVecStorage<Self>;
}

// ------------------------------------

//#[derive(Default)]
pub struct Player {
    pub trail: MovementTrail,
}

impl Player {
    fn new() -> Player {
        Player {
            trail: MovementTrail::new(3),
        }
    }
}

impl Component for Player {
    type Storage = HashMapStorage<Self>;
}

// ------------------------------------

pub enum Affiliation {
    Player,
    Enemy
}

impl Component for Affiliation {
    type Storage = VecStorage<Self>;
}

// ------------------------------------

pub struct MovementTrail {
    capacity: usize,
    trail: VecDeque<Vector3<f32>>,
}

impl MovementTrail {
    pub fn new(capacity: usize) -> MovementTrail {
        MovementTrail {
            capacity,
            trail: VecDeque::with_capacity(capacity),
        }
    }

    pub fn push(&mut self, pos: Vector3<f32>) {
        if self.trail.len() >= self.capacity {
            self.trail.pop_front();
        }
        self.trail.push_back(pos);
    }

    pub fn oldest(&self) -> Option<&Vector3<f32>> {
        self.trail.front()
    }

}

// ------------------------------------

pub struct Spawner {
    spawn_rate: f32,
    spawn_countdown: f32,
    sprite: SpriteRender,
    pub remaining: u32,
}

impl Spawner {
    pub fn new(spawn_rate: f32, sprite: SpriteRender) -> Spawner {
        Spawner {
            spawn_rate,
            spawn_countdown: spawn_rate,
            sprite,
            remaining: 100,
        }
    }

    /// Advance the spawn countdown by the given `time`.
    /// Returns `true` if the countdown reached 0 and
    /// was reset.
    pub fn advance_and_reset(&mut self, time: f32) -> bool {
        self.spawn_countdown -= time;
        if self.spawn_countdown <= 0.0 {
            self.spawn_countdown = self.spawn_rate;
            true
        } else {
            false
        }
    }

    pub fn sprite(&self) -> SpriteRender {
        self.sprite.clone()
    }

}

impl Component for Spawner {
    type Storage = DenseVecStorage<Self>;
}

// ------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColorPallatte {
    colors: Vec<Rgba>,
}

impl ColorPallatte {
    pub fn new(colors: Vec<Rgba>) -> ColorPallatte {
        ColorPallatte { colors }
    }
    pub fn next_random(&self) -> Rgba {
        match self.colors.choose(&mut thread_rng()) {
            Some(color) => *color,
            None => Default::default()
        }
    }
}

impl Default for ColorPallatte {
    fn default() -> Self {
        ColorPallatte {
            colors: vec![Rgba::red(), Rgba::green(), Rgba::blue()],
        }
    }
}

impl Component for ColorPallatte {
    type Storage = DenseVecStorage<Self>;
}

impl Asset for ColorPallatte {
    const NAME: &'static str = "falldown::ColorPallatte";
    type Data = Self;
    type HandleStorage = VecStorage<Handle<Self>>;
}

impl From<ColorPallatte> for Result<ProcessingState<ColorPallatte>, Error> {
    fn from(p: ColorPallatte) -> Self {
        Ok(ProcessingState::Loaded(p))
    }
}

// ------------------------------------

#[derive(Default)]
pub struct Loading {
    progress: ProgressCounter,
    sprite_sheet: Option<SpriteSheetHandle>,
}
impl Loading {
    pub fn new() -> Loading {
        Default::default()
    }
}

impl SimpleState for Loading {
    fn on_start(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;
        let sprite_sheet = load_sprite_sheet(world, &mut self.progress);
        self.sprite_sheet = Some(sprite_sheet);
    }

    fn update(&mut self, _data: &mut StateData<GameData>) -> SimpleTrans {
        match self.progress.complete() {
            Completion::Loading => {
                println!("Loading...");
                Trans::None
            },
            Completion::Failed => {
                println!("Loading of assets failed. I am slain :(");
                Trans::Quit
            },
            Completion::Complete => {
                if let Some(sprite_sheet) = &self.sprite_sheet {
                    Trans::Switch(Box::new(Running {
                        sprite_sheet: sprite_sheet.clone(),
                    }))
                } else {
                    println!("false start :(");
                    Trans::None
                }
            },
        }
    }


}

// ------------------------------------

pub struct Running {
    sprite_sheet: SpriteSheetHandle,
}

impl SimpleState for Running {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        init_camera(world);
        init_spawner(world, self.sprite_sheet.clone());
        init_player(world, self.sprite_sheet.clone());
        init_cursor(world, self.sprite_sheet.clone());
    }

    fn on_stop(&mut self, data: StateData<GameData>) {
        let StateData { world, .. } = data;
        println!("finishing SimpleState");

        for co in world.read_resource::<EntityCollisionWorld>().collision_objects() {
            println!("Leftover collision object - {}", co.handle().0)
        }
    }
}

// ------------------------------------

pub struct Collider {
    shape: ShapeHandle<f32>,
    groups: CollisionGroups,
    query_type: GeometricQueryType<f32>,
    pub(crate) handle: Option<CollisionObjectHandle>,
}
impl ToEvent<Option<CollisionObjectHandle>> for Collider {
    fn to_event(&self) -> Option<CollisionObjectHandle> {
        self.handle
    }
}
impl Component for Collider {
    type Storage = RemovalFlaggedStorage<Self, Option<CollisionObjectHandle>>;
}
impl Collider {
    pub fn new(
        shape: ShapeHandle<f32>,
        groups: CollisionGroups,
        query_type: GeometricQueryType<f32>
    ) -> Collider {
        Collider {
            shape,
            groups,
            query_type,
            handle: None,
        }
    }
    pub fn shape(&self) -> &ShapeHandle<f32> { &self.shape }
    pub fn groups(&self) -> &CollisionGroups { &self.groups }
    pub fn query_type(&self) -> &GeometricQueryType<f32> { &self.query_type }
}

// ------------------------------------

pub type EntityCollisionWorld = CollisionWorld<f32, Entity>;

// ------------------------------------

fn init_spawner(world: &mut World, sprite_sheet: SpriteSheetHandle) {
    let pallatte = ColorPallatte {
        colors: vec![
            Rgba(0.196, 0.804, 0.196, 1.0), // lime green
            Rgba(0.000, 0.749, 1.000, 1.0), // deep sky blue
            Rgba(0.953, 0.640, 0.375, 1.0), // sandybrown
            Rgba(0.598, 0.195, 0.797, 1.0), // darkorchid
            Rgba(0.926, 0.078, 0.238, 1.0), // crimson
        ]
    };
    let sprite = SpriteRender {
        sprite_sheet,
        sprite_number: 0,
    };
    world.create_entity()
        .with(pallatte)
        .with(Spawner::new(0.03, sprite))
        .build();
}

// ------------------------------------

fn load_sprite_sheet<P: Progress>(world: &mut World, progress: P) -> SpriteSheetHandle {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "texture/falldown_spritesheet.png",
            PngFormat,
            TextureMetadata::srgb_scale(),
            progress,
            &texture_storage,
        )
    };

    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        "texture/falldown_spritesheet.ron",
        SpriteSheetFormat,
        texture_handle,
        (),
        &sprite_sheet_store,
    )
}

// ------------------------------------

fn init_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_z(1.0);
    world.create_entity()
        .with(Camera::from(Projection::orthographic(0.0, ARENA_WIDTH, 0.0, ARENA_HEIGHT)))
        .with(transform)
        .build();
}

// ------------------------------------

fn init_player(world: &mut World, sprite_sheet: SpriteSheetHandle) {
    let mut transform = Transform::default();
    transform.set_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.15, 0.1);

    let sprite = SpriteRender {
        sprite_sheet,
        sprite_number: 1, // player sprite
    };


    // Player
    let player = world.create_entity()
        .with(Player::new())
        .with(FollowMouse {
            x_ratio: 0.9,
            y_ratio: 0.0,
        })
        .with(transform)
        // .with(move_target)
        // .with(crate::systems::MouseMoveTargetTag)
        .with(Collider::new(
            ShapeHandle::new(Ball::new(15f32)),
            player_collision_group(),
            GeometricQueryType::Contacts(0f32, 0f32),
        ))
        .build();

    // Player Visuals
    let mut inner_transform = Transform::default();
    inner_transform.translate_y(-9.5);
    world.create_entity()
        .with(Affiliation::Player)
        .with(inner_transform)
        .with(Transparent)
        .with(sprite)
        .with(Parent { entity: player })
        .build();
}

// ------------------------------------

fn init_cursor(world: &mut World, sprite_sheet: SpriteSheetHandle) {
    let sprite = SpriteRender {
        sprite_sheet,
        sprite_number: 2, // 1 pixel thing
    };

    // Vertical Line that only follows the X position of the mouse
    {
        let mut transform = Transform::default();
        transform.set_scale(1.0, ARENA_HEIGHT, 1.0);
        transform.set_x(ARENA_WIDTH * 0.5);
        transform.set_y(ARENA_HEIGHT * 0.5);

        let follow_mouse = FollowMouse {
            x_ratio: 1.0,
            y_ratio: 0.0,
        };

        world.create_entity()
            .with(transform)
            .with(Transparent)
            .with(sprite.clone())
            .with(follow_mouse)
            .build();
    }

    // Horizontal Line that only follows the Y position of the mouse
    {
        let mut transform = Transform::default();
        transform.set_scale(ARENA_WIDTH, 1.0, 1.0);
        transform.set_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 0.0);

        let follow_mouse = FollowMouse {
            x_ratio: 0.0,
            y_ratio: 1.0,
        };

        world.create_entity()
            .with(transform)
            .with(Transparent)
            .with(sprite.clone())
            .with(follow_mouse)
            .build();
    }
}