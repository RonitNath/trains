use crate::{ prelude::*, logic::body::{ BodyBundle, Body } };

use super::{ assets::GeneratedAssets, Tag };

pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<S1>().add_systems(Update, (handle_s1.after(decide), decide, gain_stamina));
    }
}

/// gain 1 stamina per second
pub fn gain_stamina(mut q: Query<&mut Stamina>, time: Res<Time>) {
    for mut stamina in q.iter_mut() {
        stamina.energy += time.delta_seconds();
    }
}

pub enum Movement {
    Rotate(f32),
    Move(f32),
}

#[derive(Event)]
pub struct S1 {
    pub id: Entity,
    pub action: Action,
}

pub enum Action {
    M(Movement),
}

pub fn decide(
    agents: Query<(Entity, &Transform, &Body, &Agent, &Stamina)>,
    mut stage_1: EventWriter<S1>
) {
    for (e, tf, body, agent, stamina) in agents.iter() {
        if stamina.energy > 0.5 {
            stage_1.send(S1 {
                id: e,
                action: Action::M(Movement::Move(1.0)),
            });
        }
    }
}

pub fn handle_s1(
    mut rdr: EventReader<S1>,
    mut q: Query<(&Transform, &mut Velocity, &Body, &mut Stamina)>
) {
    for e in rdr.iter() {
        if let Ok((tf, mut vel, body, mut stamina)) = q.get_mut(e.id) {
            use Action::*;
            match e.action {
                M(Movement::Rotate(amt)) => {
                    if stamina.energy > 0.1 {
                        vel.angvel += amt * body.angvel();

                        stamina.energy -= 0.1;
                    }
                }
                M(Movement::Move(amt)) => {
                    if stamina.energy > 0.5 {
                        vel.linvel += tf.local_y().truncate() * amt * body.linvel();

                        stamina.energy -= 0.5;
                    }
                }
            };
        } else {
            error!("Attempted to move entity which couldn't be found");
        }
    }
}

#[derive(Component)]
pub struct Stamina {
    pub energy: f32,
}

impl Stamina {
    pub fn new(energy: f32) -> Self {
        Self { energy }
    }
}

#[derive(Component)]
pub struct Agent;

#[derive(Bundle)]
pub struct AgentBundle {
    agent: Agent,
    stamina: Stamina,
}

impl AgentBundle {
    pub fn new() -> Self {
        Self {
            agent: Agent,
            stamina: Stamina::new(1.0),
        }
    }
}

impl Agent {
    pub fn new() -> Self {
        Self {}
    }

    pub fn spawn(
        pos: Vec2,
        dir: Vec2,
        color: String,
        commands: &mut Commands,
        assets: &Res<GeneratedAssets>
    ) -> Entity {
        let mesh = assets.meshes.get("AGENT").unwrap();
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
            .insert(AgentBundle::new())
            .insert(Tag::agent())
            .with_children(|parent| {
                parent.spawn(Render {
                    material: amaterial.clone(),
                    mesh: assets.meshes.get("EYEBALL").unwrap().clone(),
                    transform: Transform::from_translation(
                        Agent::right_eye_pos().extend(CHILD_VISIBLE_Z)
                    ),
                    ..Default::default()
                });

                parent.spawn(Render {
                    material: amaterial.clone(),
                    mesh: assets.meshes.get("EYEBALL").unwrap().clone(),
                    transform: Transform::from_translation(
                        Agent::left_eye_pos().extend(CHILD_VISIBLE_Z)
                    ),
                    ..Default::default()
                });
            })
            .id()
    }

    pub fn eye_dist() -> f32 {
        RADIUS * 0.5
    }

    pub fn eye_offset() -> f32 {
        RADIUS * 0.3
    }

    pub fn left_eye_pos() -> Vec2 {
        Vec2::new(-Agent::eye_offset(), Agent::eye_dist())
    }

    pub fn right_eye_pos() -> Vec2 {
        Vec2::new(Agent::eye_offset(), Agent::eye_dist())
    }
}
