extern crate amethyst;
extern crate hibitset;
extern crate nalgebra as na;
extern crate ncollide3d;
extern crate rand;

mod falldown;
mod storage;
mod systems;
mod util;

use crate::falldown::Loading;

use amethyst::{
    core:: {
        transform::TransformBundle,
    },
    input::InputBundle,
    prelude::*,
    renderer::{DisplayConfig, DrawFlat2D, Pipeline, RenderBundle, Stage},
    utils::application_root_dir,
};

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let config = DisplayConfig::load(app_root.join("resources/display_config.ron"));

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([1., 1., 1., 1.], 1.0)
            .with_pass(DrawFlat2D::new())
    );

    let game_data = GameDataBuilder::default()
        .with_bundle(InputBundle::<String, String>::new())?
        .with_bundle(systems::PlayerControlBundle::<String, String>::new())?
        .with_bundle(TransformBundle::new())?
        .with_bundle(RenderBundle::new(pipe, Some(config))
            .with_sprite_sheet_processor()
            .with_sprite_visibility_sorting(&["transform_system"])
        )?
        .with(systems::SpawnerSystem, "spawner", &[])
        .with(systems::FallingObjectSystem, "falling_objects", &["spawner"])
        .with(systems::SyncCollisionWorld::new(), "sync_collision", &[])
        .with(systems::ObjectCollection::new(), "object_collection", &["sync_collision"])
    ;

    let assets_directory = app_root.join("assets");
    let mut game = Application::new(assets_directory, Loading::new(), game_data)?;

    game.run();

    Ok(())
}
