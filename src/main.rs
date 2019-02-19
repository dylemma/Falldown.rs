extern crate amethyst;
extern crate rand;
extern crate serde;

mod falldown;
mod systems;
mod util;

use crate::falldown::Running;

use amethyst::{
    controls::{MouseFocusUpdateSystem, CursorHideSystem},
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
        .with_bundle(
            InputBundle::<String, String>::new()
        )?
        .with_bundle(TransformBundle::new())?
        .with_bundle(RenderBundle::new(pipe, Some(config))
            .with_sprite_sheet_processor()
            .with_sprite_visibility_sorting(&["transform_system"])
        )?
        .with(systems::SpawnerSystem, "spawner", &[])
        .with(systems::FallingObjectSystem, "falling_objects", &["spawner"])
        //.with(systems::PlayerMovementSystem, "player_movement", &[])
        .with(systems::FollowMouseSystem, "follow_mouse", &[])
        .with(systems::PlayerRotateSystem, "player_rotate", &["follow_mouse"])
       // .with(systems::MoveTargetSystem, "move_target_system", &[])
//        .with(systems::MouseMovementSystem::new(), "mouse_movement", &[])
        .with(MouseFocusUpdateSystem::new(), "mouse_focus", &[])
        .with(CursorHideSystem::new(), "cursor_hide", &["mouse_focus"])
    ;

    let assets_directory = app_root.join("assets");
    let mut game = Application::new(assets_directory, Running, game_data)?;

    game.run();

    Ok(())
}
