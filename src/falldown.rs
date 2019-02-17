pub struct Block;

use amethyst::{
    assets::{Asset, AssetStorage, Handle, Loader, ProcessingState, RonFormat},
    core::transform::Transform,
    Error,
    ecs::prelude::{Component, VecStorage},
    prelude::*,
    renderer::{
        Camera, PngFormat, Projection, Rgba, SpriteRender, SpriteSheet, SpriteSheetFormat, SpriteSheetHandle, Texture, TextureMetadata,
    }
};
use rand::{Rng, thread_rng};
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColorPallatte {
    colors: Vec<Rgba>,
}
impl ColorPallatte {
    fn new(colors: Vec<Rgba>) -> ColorPallatte {
        ColorPallatte { colors }
    }
    fn next_random(&self) -> Rgba {
        match thread_rng().choose(&self.colors) {
            Some(color) => *color,
            None => Default::default()
        }
    }
}
//impl Default for ColorPallatte {
//    fn default() -> Self {
//        ColorPallatte {
//            colors: vec![Rgba::red(), Rgba::green(), Rgba::blue()],
//        }
//    }
//}
//impl Asset for ColorPallatte {
//    const NAME: &'static str = "falldown::ColorPallatte";
//    type Data = Self;
//    type HandleStorage = VecStorage<Handle<Self>>;
//}
//impl From<ColorPallatte> for Result<ProcessingState<ColorPallatte>, Error> {
//    fn from(p: ColorPallatte) -> Self {
//        Ok(ProcessingState::Loaded(p))
//    }
//}

// ------------------------------------

pub struct Running;

impl SimpleState for Running {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { world, .. } = data;

        let color_pallatte = init_color_pallatte();
        let spritesheet_handle = load_sprite_sheet(world);
        // let color_pallatte_handle = load_color_pallatte(world);

        init_camera(world);
        init_dummy_sprite(world, spritesheet_handle, &color_pallatte);
    }
}

fn init_color_pallatte() -> ColorPallatte {
    ColorPallatte {
        colors: vec![
            Rgba(0.196, 0.804, 0.196, 1.0), // lime green
            Rgba(0.000, 0.749, 1.000, 1.0), // deep sky blue
            Rgba(0.953, 0.640, 0.375, 1.0), // sandybrown
            Rgba(0.598, 0.195, 0.797, 1.0), // darkorchid
            Rgba(0.926, 0.078, 0.238, 1.0), // crimson
        ]
    }
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

fn init_dummy_sprite(world: &mut World, sprite_sheet: SpriteSheetHandle, pallatte: &ColorPallatte) {

    let sprite_render = SpriteRender {
        sprite_sheet,
        sprite_number: 0,
    };
    let mut rng = thread_rng();

    for _ in 0..10 {
        let mut transform = Transform::default();
        let y_offset: f32 = rng.gen();
        let x_offset: f32 = rng.gen();
        transform
            .set_rotation_euler(0., 0., 3.14 / 4.0)
            .set_xyz(ARENA_WIDTH * x_offset, ARENA_HEIGHT * y_offset, 0.);

        let falling = FallingObject {
            fall_rate: 40.0,
            radius: 5.0, // half of the sprite size
            spin_rate: 3.1415 * 1.5, // 3/4 turn per second
        };

        let color = pallatte.next_random();

        world
            .create_entity()
            .with(transform)
            .with(sprite_render.clone())
            .with(color)
            .with(falling)
            .build();
    }
}