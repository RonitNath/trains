mod setup;
pub mod prelude;
pub mod controls;
mod config;
pub mod world;
pub mod logic;

use rand::Rng;

use crate::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            setup::SetupPlugin,
            controls::ControlsPlugin,
            world::WorldPlugin,
            logic::LogicPlugin,
        ))
        .run();
}

#[test]
pub fn dice() {
    // roll a 20-sided dice

    let mut rng = rand::thread_rng();
    let roll = rng.gen_range(1..=20);

    println!("You rolled a {}", roll);
}
