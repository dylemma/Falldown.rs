extern crate amethyst;
extern crate rand;
extern crate serde;

mod falldown;
mod systems;

use crate::falldown::Running;

use amethyst::{
    core:: {
        transform::TransformBundle,
    },
    prelude::*,
    renderer::{DisplayConfig, DrawFlat2D, Pipeline, RenderBundle, Stage},
    utils::application_root_dir,
};

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let path = format!(
        "{}/resources/display_config.ron",
        application_root_dir()
    );
    let config = DisplayConfig::load(&path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([1., 1., 1., 1.], 1.0)
            .with_pass(DrawFlat2D::new())
    );

    let game_data = GameDataBuilder::default()
        .with_bundle(RenderBundle::new(pipe, Some(config))
            .with_sprite_sheet_processor()
        )?
        .with_bundle(TransformBundle::new())?
        .with(systems::FallingObjectSystem, "falling_objects", &[])
    ;
    let mut game = Application::new("./", Running, game_data)?;

    game.run();

    Ok(())
}
