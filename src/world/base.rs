use crate::{ prelude::*, logic::body::BodyBundle };

use super::{ assets::GeneratedAssets, Tag };

pub struct BasePlugin;

impl Plugin for BasePlugin {
    fn build(&self, app: &mut App) {
        app;
    }
}

#[derive(Component)]
pub struct Base;

impl Base {
    pub fn spawn(
        pos: Vec2,
        dir: Vec2,
        color: String,
        commands: &mut Commands,
        assets: &Res<GeneratedAssets>
    ) -> Entity {
        let mesh = assets.meshes.get("BASE").unwrap();
        let (acolor, amaterial) = assets.colors.get(&format!("A{}", color)).unwrap();
        let (color, material) = assets.colors.get(&color).unwrap();
        let body = BodyBundle::spawn(
            RADIUS,
            *color,
            pos,
            dir,
            mesh.clone(),
            material.clone(),
            commands
        );

        commands
            .entity(body)
            .insert(Base)
            .insert(Tag::base())
            .with_children(|parent| {
                parent.spawn(Render {
                    material: amaterial.clone(),
                    mesh: assets.meshes.get("BASE_TAG").unwrap().clone(),
                    transform: Transform::from_translation(Vec2::ZERO.extend(CHILD_VISIBLE_Z)),
                    ..Default::default()
                });
            })
            .id()
    }
}
