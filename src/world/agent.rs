use std::f32::consts::PI;

use crate::{
    prelude::*,
    logic::{ body::{ BodyBundle, Body }, spawning::spawn, sk::SpatialKnowledge },
};

use super::{ assets::GeneratedAssets, Tag, EntityKind };

pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Goal>()
            .register_type::<Stamina>()
            .register_type::<Agent>()
            .add_event::<S1>()
            .add_event::<S2>()
            .add_event::<S3>()
            .add_systems(Update, (
                handle_s3.after(handle_s2),
                handle_s2.after(handle_s1),
                handle_s1.after(decide),
                decide.after(vision),
                vision,
                gain_stamina,
            ));
    }
}

#[derive(Event)]
pub struct S3 {
    id: Entity,
    event: A3,
}

pub enum A3 {
    GainInfo {
        about: Entity,
        info: EntityKind,
        pos: Vec2,
    },
}

pub fn handle_s3(mut rdr: EventReader<S3>, mut q: Query<&mut SpatialKnowledge>) {
    for e in rdr.iter() {
        if let Ok(mut sk) = q.get_mut(e.id) {
            match e.event {
                A3::GainInfo { about, info, pos } => {
                    sk.obtains_info_about(about, info, pos);
                }
            }
        } else {
            warn!("S3 query failed");
        }
    }
}

/// gain 1 stamina per second
pub fn gain_stamina(mut q: Query<&mut Stamina>, time: Res<Time>) {
    for mut stamina in q.iter_mut() {
        if stamina.energy < 1.0 {
            stamina.energy += time.delta_seconds();
        }
    }
}

pub enum Movement {
    Rotate(Vec2),
    Move(Vec2),
}

#[derive(Event)]
pub struct S1 {
    pub id: Entity,
    pub action: Action,
}

pub enum Action {
    M(Movement),
    G(Goal),
    None,
    SpawnChild,
    ProcessVision,

    PruneVision,
}

#[derive(Component)]
pub struct Vision {
    pub toi: f32,
    pub fov: f32,
    pub half_cone_count: isize,
    pub seeing: HashMap<Entity, Vec<Vec2>>,
}

impl Vision {
    pub fn new(toi: f32, fov: f32, half_cone_count: isize) -> Self {
        Self {
            toi,
            fov,
            half_cone_count,
            seeing: HashMap::new(),
        }
    }
}

/// Casts rays in a cone in front of the agent
/// and reports what it sees to the SpatialKnowledge
///
/// Next: report about tiles which you see and which no longer are occupied
pub fn vision(
    mut agents: Query<(Entity, &Transform, &mut Vision, &Body, &Agent, &SpatialKnowledge)>,
    rc: Res<RapierContext>,
    mut gizmos: Gizmos,
    mut stage_1: EventWriter<S1>,
    mut stage_2: EventWriter<S2>
) {
    for (entity, tf, mut vision, body, agent, sk) in agents.iter_mut() {
        let my_pos = tf.translation.truncate();
        let my_facing = tf.local_y().truncate();

        let mut seeing: HashMap<Entity, Vec<Vec2>> = HashMap::new();

        let right_extent = vision.fov;
        let step_size = right_extent / (vision.half_cone_count as f32);
        let angles = (-vision.half_cone_count..=vision.half_cone_count)
            .map(|i| (i as f32) * step_size)
            .collect::<Vec<_>>();

        angles.iter().for_each(|angle| {
            let dir = Vec2::from_angle(*angle).rotate(my_facing);
            let mut dist = vision.toi * facing_debuff(my_facing, dir);
            if
                let Some((ix, f_dist)) = rc.cast_ray(
                    my_pos,
                    dir,
                    dist,
                    false,
                    QueryFilter::default().exclude_collider(entity)
                )
            {
                dist = f_dist;
                let hit_pos = my_pos + dir * dist;
                seeing.entry(ix).or_default().push(hit_pos);
            }

            gizmos.ray_2d(my_pos, dir * dist, Color::BLACK);
        });

        if !seeing.is_empty() {
            stage_1.send(S1 {
                id: entity,
                action: Action::ProcessVision,
            });
        } else {
            if sk.has_seen() {
                stage_1.send(S1 {
                    id: entity,
                    action: Action::PruneVision,
                });
            }
        }
        vision.seeing = seeing;
    }
}

pub fn decide(
    agents: Query<(Entity, &Transform, &Body, &Agent, &Stamina, &Goal, &SpatialKnowledge)>,
    mut stage_1: EventWriter<S1>,
    mut gizmos: Gizmos
) {
    for (e, tf, body, agent, stamina, goal, sk) in agents.iter() {
        let my_pos = tf.translation.truncate();
        let my_facing = tf.local_y().truncate();

        match goal.mission {
            Missions::None => {
                // None
            }
            Missions::MoveTo(mut pos) => {
                let (mut path, obstructed) = sk.path(my_pos, pos);
                let abs_dist = pos.distance(my_pos);
                if obstructed {
                    path.pop();
                    path.pop();
                    if let Some(path_pos) = path.pop() {
                        pos = path_pos;
                    }
                }
                let dir = (pos - my_pos).normalize_or_zero();
                let angle_amt = my_facing.angle_between(dir);

                gizmos.circle_2d(pos, 10.0, Color::RED);

                use Action::M;
                if angle_amt.abs() > body.ang_margin() {
                    stage_1.send(S1 {
                        id: e,
                        action: M(Movement::Rotate(dir)),
                    });
                }
                if abs_dist > body.lin_margin() {
                    stage_1.send(S1 {
                        id: e,
                        action: M(Movement::Move(pos)),
                    });
                } else {
                    stage_1.send(S1 {
                        id: e,
                        action: Action::None,
                    });
                }
            }
        }
    }
}

#[derive(Event)]
pub struct S2 {
    pub id: Entity,
    pub action: A2,
}

pub enum A2 {
    Tag(Entity),
}

pub fn handle_s2(
    mut rdr: EventReader<S2>,
    tags: Query<(&Tag, &Transform)>,
    mut stage_3: EventWriter<S3>
) {
    for e in rdr.iter() {
        match e.action {
            A2::Tag(target) => {
                if let Ok((tag, tf)) = tags.get(target) {
                    stage_3.send(S3 {
                        id: e.id,
                        event: A3::GainInfo {
                            about: target,
                            info: tag.kind,
                            pos: tf.translation.truncate(),
                        },
                    });
                } else {
                    warn!("Entity without a tag");
                }
            }
        }
    }
}

pub fn handle_s1(
    mut rdr: EventReader<S1>,
    mut q: Query<
        (&Transform, &mut Velocity, &Body, &mut Stamina, &mut Goal, &mut SpatialKnowledge, &Vision)
    >,
    time: Res<Time>,
    rc: Res<RapierContext>,
    mut commands: Commands,
    ga: Res<GeneratedAssets>,
    mut stage_2: EventWriter<S2>
) {
    for e in rdr.iter() {
        if let Ok((tf, mut vel, body, mut stamina, mut goal, mut sk, vision)) = q.get_mut(e.id) {
            let my_pos = tf.translation.truncate();
            let my_facing = tf.local_y().truncate();

            use Action::*;
            match e.action {
                None => {
                    goal.mission = Missions::None;
                }
                PruneVision => {
                    sk.clear_seen();
                }
                M(Movement::Rotate(dir)) => {
                    let amt = my_facing.angle_between(dir) / PI;
                    if stamina.energy > 0.1 {
                        vel.angvel += amt * body.angvel() * time.delta_seconds();

                        stamina.energy -= 0.1 * time.delta_seconds();
                    }
                }
                M(Movement::Move(pos)) => {
                    if stamina.energy > 0.5 {
                        let dir = (pos - my_pos).normalize_or_zero();
                        // let str = my_pos.distance(pos) / body.radius;
                        vel.linvel +=
                            dir *
                            body.linvel() *
                            time.delta_seconds() *
                            facing_debuff(tf.local_y().truncate(), dir);

                        stamina.energy -= time.delta_seconds() / 2.0;
                    }
                }
                G(g) => {
                    *goal = g;
                }
                SpawnChild => {
                    let spawn_pos = my_pos + my_facing * body.radius * 2.01;

                    if spawn(RADIUS, spawn_pos, std::option::Option::None, &rc) {
                        Agent::spawn(
                            spawn_pos,
                            my_facing.rotate(Vec2::NEG_Y),
                            "WHITE".to_string(),
                            &mut commands,
                            &ga
                        );
                    } else {
                        warn!("Spot not open for spawning");
                    }
                }
                ProcessVision => {
                    sk.report(&vision.seeing);
                    for (entity, _pos) in vision.seeing.iter() {
                        if !sk.queried(entity) {
                            stage_2.send(S2 {
                                id: e.id,
                                action: A2::Tag(*entity),
                            });
                            sk.query_in_progress(*entity);
                        }
                    }
                }
            };
        } else {
            error!("Attempted to move entity which couldn't be found");
        }
    }
}

#[derive(Component, Reflect)]
pub struct Stamina {
    pub energy: f32,
}

impl Stamina {
    pub fn new(energy: f32) -> Self {
        Self { energy }
    }
}

#[derive(Component, Reflect)]
pub struct Agent;

#[derive(Bundle)]
pub struct AgentBundle {
    agent: Agent,
    stamina: Stamina,
    goal: Goal,
    vision: Vision,
    sk: SpatialKnowledge,
}

#[derive(Reflect, Component, Clone, Copy)]
pub struct Goal {
    pub mission: Missions,
}

impl Goal {
    pub fn new() -> Self {
        Self {
            mission: Missions::None,
        }
    }

    pub fn move_to(pos: Vec2) -> Self {
        Self {
            mission: Missions::MoveTo(pos),
        }
    }
}

#[derive(Reflect, Clone, Copy)]
pub enum Missions {
    MoveTo(Vec2),
    None,
}

impl AgentBundle {
    pub fn new() -> Self {
        Self {
            agent: Agent,
            stamina: Stamina::new(1.0),
            goal: Goal::new(),
            vision: Vision::new(RADIUS * VISION_SCALAR, FOV, HALF_CONES),
            sk: SpatialKnowledge::new(RADIUS / 2.0),
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
