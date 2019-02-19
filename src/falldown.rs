use amethyst::{
    assets::{Asset, AssetStorage, Handle, Loader, ProcessingState},
    core::{
        nalgebra::Vector3,
        transform::{
            components::Parent,
            Transform,
        },
    },
    Error,
    ecs::prelude::{Component, DenseVecStorage, HashMapStorage, VecStorage},
    prelude::*,
    renderer::{
        Camera, PngFormat, Projection, Rgba, SpriteRender, SpriteSheet, SpriteSheetFormat,
        SpriteSheetHandle, Texture, TextureMetadata, Transparent,
    },
};
use rand::{seq::SliceRandom, thread_rng};
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;

pub const ARENA_HEIGHT: f32 = 300.;
pub const ARENA_WIDTH: f32 = 300.;

// ------------------------------------

#[derive(Default)]
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
}

impl Spawner {
    pub fn new(spawn_rate: f32, sprite: SpriteRender) -> Spawner {
        Spawner {
            spawn_rate,
            spawn_countdown: spawn_rate,
            sprite,
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

pub struct Running;

impl SimpleState for Running {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        let spritesheet_handle = load_sprite_sheet(world);
        // let color_pallatte_handle = load_color_pallatte(world);

        init_camera(world);
        init_spawner(world, spritesheet_handle.clone());
        init_player(world, spritesheet_handle.clone());
        init_cursor(world, spritesheet_handle)
    }
}

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

fn load_sprite_sheet(world: &mut World) -> SpriteSheetHandle {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "texture/falldown_spritesheet.png",
            PngFormat,
            TextureMetadata::srgb_scale(),
            (),
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

fn init_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_z(1.0);
    world.create_entity()
        .with(Camera::from(Projection::orthographic(0.0, ARENA_WIDTH, 0.0, ARENA_HEIGHT)))
        .with(transform)
        .build();
}

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
        .build();

    let mut inner_transform = Transform::default();
    inner_transform.translate_y(-9.5);
    // Player Visuals
    world.create_entity()
        .with(inner_transform)
        .with(Transparent)
        .with(sprite)
        .with(Parent { entity: player })
        .build();
}

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