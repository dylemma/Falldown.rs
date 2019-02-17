use amethyst::{
    assets::{Asset, AssetStorage, Handle, Loader, ProcessingState, RonFormat},
    core::transform::Transform,
    Error,
    ecs::prelude::{Component, DenseVecStorage, VecStorage},
    prelude::*,
    renderer::{
        Camera, PngFormat, Projection, Rgba, SpriteRender, SpriteSheet, SpriteSheetFormat, SpriteSheetHandle, Texture, TextureMetadata,
    }
};
use rand::{seq::SliceRandom, thread_rng};
use serde::{Serialize, Deserialize};

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
        match self.colors.choose(&mut thread_rng() ) {
//        match thread_rng().choose(&self.colors) {
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
        init_spawner(world, spritesheet_handle);
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
        sprite_number: 0
    };
    world.create_entity()
        .with(pallatte)
        .with(Spawner::new(0.333, sprite))
        .build();
}

fn load_sprite_sheet(world: &mut World) -> SpriteSheetHandle {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            "assets/texture/falldown_spritesheet.png",
            PngFormat,
            TextureMetadata::srgb_scale(),
            (),
            &texture_storage,
        )
    };

    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        "assets/texture/falldown_spritesheet.ron",
        SpriteSheetFormat,
        texture_handle,
        (),
        &sprite_sheet_store,
    )
}

//fn load_color_pallatte(world: &mut World) -> Handle<ColorPallatte> {
//    let loader = world.read_resource::<Loader>();
//    let pallatte_storage = world.read_resource::<AssetStorage<ColorPallatte>>();
//    loader.load(
//        "assets/theme/color_pallatte.ron",
//        RonFormat,
//        (),
//        (),
//        &pallatte_storage
//    )
//}

fn init_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_z(1.0);
    world.create_entity()
        .with(Camera::from(Projection::orthographic(0.0, ARENA_WIDTH, 0.0, ARENA_HEIGHT)))
        .with(transform)
        .build();
}
