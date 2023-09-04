use crate::prelude::*;

pub mod body;
pub mod spawning;
pub mod hud;
pub mod sk;

pub struct LogicPlugin;

impl Plugin for LogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(hud::HudPlugin);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EP {
    pub id: Entity,
    pub pos: Vec2,
}

impl EP {
    pub fn new(id: Entity, pos: Vec2) -> Self {
        Self { id, pos }
    }
}
