use crate::prelude::*;

pub mod body;
pub mod spawning;
pub mod hud;

pub struct LogicPlugin;

impl Plugin for LogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(hud::HudPlugin);
    }
}
