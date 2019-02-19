use amethyst::{
    controls::{MouseFocusUpdateSystem, CursorHideSystem, WindowFocus},
    core::{
        nalgebra::Vector3,
        shrev::{EventChannel, ReaderId},
        transform::Transform,
    },
    ecs::prelude::{
        Component, DenseVecStorage, Join, NullStorage, Read,
        ReadExpect, ReadStorage, Resources, System, WriteStorage,
    },
    input::{InputEvent, InputHandler},
    renderer::ScreenDimensions,
};
use crate::falldown::{ARENA_HEIGHT, ARENA_WIDTH, Player};

const PI_OVER_180: f32 = std::f32::consts::PI / 180.0;

pub struct PlayerMovementSystem;

impl<'s> System<'s> for PlayerMovementSystem {
    type SystemData = (
        ReadStorage<'s, Player>,
        WriteStorage<'s, Transform>,
        ReadExpect<'s, ScreenDimensions>,
        Read<'s, InputHandler>,
    );

    fn run(&mut self, (players, mut transforms, screen_dims, input): Self::SystemData) {
        if let Some((mouse_x, mouse_y)) = mouse_logical_position(&input, &screen_dims) {
            for (_, t) in (&players, &mut transforms).join() {
                let transform: &mut Transform = t;
                transform.set_x(mouse_x * ARENA_WIDTH);
                transform.set_y(mouse_y * ARENA_HEIGHT);
            }
        }
    }
}

pub struct FollowMouse {
    pub x_ratio: f32,
    pub y_ratio: f32,
}

impl Component for FollowMouse {
    type Storage = DenseVecStorage<Self>;
}

pub struct FollowMouseSystem;

impl <'s> System<'s> for FollowMouseSystem {
    type SystemData = (
        ReadStorage<'s, FollowMouse>,
        WriteStorage<'s, Transform>,
        Read<'s, InputHandler>,
        ReadExpect<'s, ScreenDimensions>,
    );

    fn run(&mut self, (followers, mut transforms, input, screen_dims): Self::SystemData) {
        // get an (x, y) tuple of values in the range [0.0, 1.0) representing the mouse's
        // current position within the "Arena"
        if let Some((mouse_x, mouse_y)) = mouse_logical_position(&input, &screen_dims) {
            let target_x = mouse_x * ARENA_WIDTH;
            let target_y = mouse_y * ARENA_HEIGHT;
            for (follower, mut transform) in (&followers, &mut transforms).join() {
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
        for (mut p, t) in (&mut players, &mut transforms).join() {
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

//pub struct MoveTargetSystem;
//
//impl<'s> System<'s> for MoveTargetSystem {
//    type SystemData = (
//        WriteStorage<'s, Transform>,
//        ReadStorage<'s, MoveTarget>
//    );
//
//    fn run(&mut self, (mut transforms, targets): Self::SystemData) {
//        for (mut t, target) in (&mut transforms, &targets).join() {
//            let transform: &mut Transform = t;
//            target.apply_movement_to(transform.translation_mut());
//        }
//    }
//}

// -------------------------------------------------------------------

// TODO: MouseMovementSystem operates on this
//pub struct MoveTarget {
//    target: Vector3<f32>,
//    spring_factor: Vector3<f32>,
//}
//
//impl Component for MoveTarget {
//    type Storage = DenseVecStorage<Self>;
//}
//
//impl MoveTarget {
//    pub fn new(initial_pos: Vector3<f32>) -> MoveTarget {
//        MoveTarget {
//            target: initial_pos,
//            spring_factor: Vector3::new(1.0, 1.0, 1.0),
//        }
//    }
//
//    pub fn with_spring_factor(mut self, xyz: f32) -> Self {
//        self.spring_factor = Vector3::new(xyz, xyz, xyz);
//        self
//    }
//
//    pub fn with_spring_factors(mut self, x: f32, y: f32, z: f32) -> Self {
//        self.spring_factor = Vector3::new(x, y, z);
//        self
//    }
//
//    /*pub fn apply_movement_to(&self, (x, y, z): (f32, f32, f32)) -> (f32, f32, f32) {
//        (
//            x + (self.target.x - x) * self.spring_factor.x,
//            y + (self.target.y - y) * self.spring_factor.y,
//            z + (self.target.z - z) * self.spring_factor.z,
//        )
//    }*/
//
//    pub fn apply_movement_to(&self, v: &mut Vector3<f32>) {
//        v.x += (self.target.x - v.x) * self.spring_factor.x;
//        v.y += (self.target.y - v.y) * self.spring_factor.y;
//        v.z += (self.target.z - v.z) * self.spring_factor.z;
//    }
//
//    pub fn adjust_target(&mut self, delta_x: f32, delta_y: f32, delta_z: f32) {
//        self.target.x += delta_x;
//        self.target.y += delta_y;
//        self.target.z += delta_z;
//    }
//
//    pub fn set_target<T: Into<Vector3<f32>>>(&mut self, target: T) {
//        self.target = target.into();
//    }
//}

// -------------------------------------------------------------------

/// Tag type for entities that should have their MoveTarget
/// updated by the MouseMovementSystem.
//#[derive(Default)]
//pub struct MouseMoveTargetTag;
//
//impl Component for MouseMoveTargetTag {
//    type Storage = NullStorage<Self>;
//}

// -------------------------------------------------------------------

//pub struct MouseMovementSystem {
//    reader: Option<ReaderId<InputEvent<()>>>,
//}
//
//impl MouseMovementSystem {
//    pub fn new() -> MouseMovementSystem {
//        MouseMovementSystem { reader: None }
//    }
//}
//
//impl<'s> System<'s> for MouseMovementSystem {
//    type SystemData = (
//        Read<'s, EventChannel<InputEvent<()>>>,
//        Read<'s, WindowFocus>,
//        ReadStorage<'s, MouseMoveTargetTag>,
//        WriteStorage<'s, MoveTarget>,
//    );
//
//    fn run(&mut self, (events, focus, tags, mut move_targets): Self::SystemData) {
//        // Collect all of the mouse movement since last frame and SUM it.
//        // Note there actually DOES tend to be several events at once!
//        let (dx, dy) = events.read(self.reader.as_mut().unwrap())
//            .flat_map(|event| {
//                match *event {
//                    InputEvent::MouseMoved { delta_x, delta_y } => Some((delta_x as f32, delta_y as f32)),
//                    _ => None
//                }
//            })
//            .fold((0.0f32, 0.0f32), |(sum_x, sum_y), (dx, dy)| {
//                (sum_x + dx, sum_y - dy) // NOTE: y movement is inverted
//            });
//
////        if focus.is_focused {
//            if dx != 0.0 || dy != 0.0 {
//                println!("Mouse moved by {}, {}", dx, dy);
//                for (_, mut move_target) in (&tags, &mut move_targets).join() {
//                    move_target.adjust_target(dx, dy, 0.0);
//                }
//            }
////        }
////        for event in events.read(self.reader.as_mut().unwrap()) {
////            match *event {
////                InputEvent::MouseMoved { delta_x, delta_y } => {
////                    println!("Mouse moved by {}, {}", delta_x, delta_y);
////                }
////                _ => {}
////            }
////        }
//    }
//
//    fn setup(&mut self, res: &mut Resources) {
//        use amethyst::core::specs::prelude::SystemData;
//
//        Self::SystemData::setup(res);
//        self.reader = Some(res.fetch_mut::<EventChannel<InputEvent<()>>>().register_reader());
//    }
//}

fn mouse_logical_position(
    input: &InputHandler,
    screen_dims: &ScreenDimensions,
) -> Option<(f32, f32)> {
    input.mouse_position().map(|(pixel_x, pixel_y)| {
        (
            (pixel_x as f32) / screen_dims.width(),
            (screen_dims.height() - (pixel_y as f32)) / screen_dims.height()
        )
    })
}