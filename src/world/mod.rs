use bevy::transform::commands;

use crate::{ prelude::*, logic::body::BodyBundle };

use self::{ agent::Agent, assets::{ GeneratedAssets, init_assets } };

pub mod agent;
pub mod base;
pub mod assets;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((agent::AgentPlugin, base::BasePlugin, assets::AssetPlugin)).add_systems(
            Startup,
            initialize.after(init_assets)
        );
    }
}

pub fn initialize(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    assets: Res<GeneratedAssets>
) {
    // Add 1 agent to the center of the world

    let pos = Vec2::ZERO;
    let dir = Vec2::Y;
    let color = String::from("WHITE");
    Agent::spawn(pos, dir, color, &mut commands, &assets);
}

#[derive(Component)]
pub struct Tag {
    pub kind: EntityKind,
}

impl Tag {
    pub fn new(kind: EntityKind) -> Self {
        Self { kind }
    }

    pub fn agent() -> Self {
        Self::new(EntityKind::Agent)
    }

    pub fn base() -> Self {
        Self::new(EntityKind::Base)
    }
}

pub enum EntityKind {
    Agent,
    Base,
}
