use std::f32::consts::PI;

use bevy::sprite::Mesh2dHandle;

use crate::prelude::*;

#[derive(Component, Reflect)]
pub struct Body {
    pub color: Color,
    pub radius: f32,
}

impl Body {
    pub fn new(radius: f32, color: Color) -> Self {
        Self { radius, color }
    }

    pub fn lin_margin(&self) -> f32 {
        self.radius
    }

    pub fn ang_margin(&self) -> f32 {
        PI / self.radius
    }

    pub fn linvel(&self) -> f32 {
        self.radius.powi(2)
    }

    pub fn angvel(&self) -> f32 {
        self.radius
    }
}

#[derive(Bundle)]
pub struct BodyBundle {
    pub body: Body,
    pub pb: PhysicsBody,
}

impl BodyBundle {
    pub fn new(body: Body, pb: PhysicsBody) -> Self {
        Self { body, pb }
    }

    pub fn spawn(
        radius: f32,
        color: Color,
        pos: Vec2,
        dir: Vec2,
        mesh: Mesh2dHandle,
        material: Handle<ColorMaterial>,
        commands: &mut Commands
    ) -> Entity {
        let mut tf = Transform::from_translation(pos.extend(VISIBLE_Z));
        tf.rotate_z(Vec2::Y.angle_between(dir));

        let pb = PhysicsBody::new(Collider::ball(radius), Render {
            mesh,
            material,
            transform: tf,
            ..Default::default()
        });
        let body = Body::new(radius, color);

        commands.spawn(BodyBundle::new(body, pb)).id()
    }
}
