use std::hash::Hash;
use std::marker::PhantomData;

use amethyst::{
    controls::{CursorHideSystem, MouseFocusUpdateSystem},
    core::{
        SystemBundle,
        transform::Transform,
    },
    ecs::prelude::{
        DispatcherBuilder, Join, Read, ReadExpect, ReadStorage, System, WriteStorage,
    },
    Error,
    input::InputHandler,
    renderer::ScreenDimensions,
};

use crate::falldown::{ARENA_HEIGHT, ARENA_WIDTH, FollowMouse, Player};

const PI_OVER_180: f32 = std::f32::consts::PI / 180.0;

// -------------------------------------------------------------------

/// System that updates the Transform of any entity with the `FollowMouse` component.
/// The `A` and `B` are the type parameters of the `InputHandler`
pub struct FollowMouseSystem<A, B> {
    _marker: PhantomData<(A, B)>
}

impl<A, B> FollowMouseSystem<A, B> {
    pub fn new() -> FollowMouseSystem<A, B> {
        FollowMouseSystem {
            _marker: PhantomData,
        }
    }
}

impl<'s, A, B> System<'s> for FollowMouseSystem<A, B>
    where
        A: Hash + Eq + Clone + Send + Sync + 'static, // type constraints from InputHandler
        B: Hash + Eq + Clone + Send + Sync + 'static, // type constraints from InputHandler
{
    type SystemData = (
        ReadStorage<'s, FollowMouse>,
        WriteStorage<'s, Transform>,
        Read<'s, InputHandler<A, B>>,
        ReadExpect<'s, ScreenDimensions>,
    );

    fn run(&mut self, (followers, mut transforms, input, screen_dims): Self::SystemData) {

        // get an (x, y) tuple of values in the range [0.0, 1.0) representing the mouse's position on screen
        let mouse_logical_position = input.mouse_position().map(|(pixel_x, pixel_y)| {
            (
                (pixel_x as f32) / screen_dims.width(),
                (screen_dims.height() - (pixel_y as f32)) / screen_dims.height()
            )
        });

        if let Some((mouse_x, mouse_y)) = mouse_logical_position {
            let target_x = mouse_x * ARENA_WIDTH;
            let target_y = mouse_y * ARENA_HEIGHT;

            // move all `FollowMouse` entities towards the mouse's position within the Arena
            for (follower, transform) in (&followers, &mut transforms).join() {
                transform.translate_x((target_x - transform.translation().x) * follower.x_ratio);
                transform.translate_y((target_y - transform.translation().y) * follower.y_ratio);
            }
        }
    }
}

// -------------------------------------------------------------------

pub struct PlayerRotateSystem;

impl<'s> System<'s> for PlayerRotateSystem {
    type SystemData = (
        WriteStorage<'s, Player>,
        WriteStorage<'s, Transform>,
    );

    fn run(&mut self, (mut players, mut transforms): Self::SystemData) {
        for (p, t) in (&mut players, &mut transforms).join() {
            let player: &mut Player = p;
            let transform: &mut Transform = t;

            // update the "trail" with the current position
            player.trail.push(*transform.translation());

            // see how far the player has moved in the last few frames,
            // and decide an angle for it to face (more travel = more angle)
            let x_movement = {
                match player.trail.oldest() {
                    Some(v) => transform.translation().x - v.x,
                    None => 0f32
                }
            };
            let angle = {
                if x_movement.abs() > 0.1 {
                    x_movement * -2.0
                } else {
                    0f32
                }
            }.min(45.0).max(-45.0);
            transform.set_rotation_euler(0.0, 0.0, angle * PI_OVER_180);
        }
    }
}

// -------------------------------------------------------------------

pub struct PlayerControlBundle<A, B> {
    _marker: PhantomData<(A, B)>,
}

impl<A, B> PlayerControlBundle<A, B> {
    pub fn new() -> PlayerControlBundle<A, B> {
        PlayerControlBundle {
            _marker: PhantomData,
        }
    }
}

impl<'a, 'b, A, B> SystemBundle<'a, 'b> for PlayerControlBundle<A, B>
    where
        A: Hash + Eq + Clone + Send + Sync + 'static, // type constraints from InputHandler
        B: Hash + Eq + Clone + Send + Sync + 'static, // type constraints from InputHandler
{
    fn build(self, builder: &mut DispatcherBuilder) -> Result<(), Error> {
        builder.add(FollowMouseSystem::<A, B>::new(), "follow_mouse", &[]);
        builder.add(PlayerRotateSystem, "player_rotate", &["follow_mouse"]);
        builder.add(MouseFocusUpdateSystem::new(), "mouse_focus", &[]);
        builder.add(CursorHideSystem::new(), "cursor_hide", &["mouse_focus"]);
        Ok(())
    }
}