use std::collections::BTreeSet;

use bevy::{ time::Stopwatch };
use rand::seq::IteratorRandom;

use crate::{
    prelude::*,
    setup::MoveCamera,
    world::agent::Agent,
    logic::hud::{ ActiveControl, CameraFollows },
};

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            mouse_selection,
            // random_active_control,
            camera_commands,
        )).init_resource::<GUISelect>();
    }
}

// fn random_active_control(
//     mut commands: Commands,
//     agents: Query<(Entity, With<Agent>, Without<ActiveControl>)>,
//     keys: Res<Input<KeyCode>>
// ) {
//     if keys.just_pressed(KeyCode::R) {
//         if let Some((entity, (), ())) = agents.iter().choose(&mut rand::thread_rng()) {
//             if let Some(mut ec) = commands.get_entity(entity) {
//                 ec.insert(ActiveControl);
//             }
//         }
//     }
// }

/// allows wasd movement of the camera
fn camera_commands(
    keys: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection, With<Camera>)>,
    // mut active_control: Query<
    // (Entity, &mut ExternalImpulse, &Transform, &Agent, With<ActiveControl>, Without<Camera>)
    // >,
    mut commands: Commands,
    // mut movement_wtr: EventWriter<Movement>,
    time: Res<Time>,
    // mut body_controller: EventWriter<BodyControlEvent>,
    mut camera_moved: EventReader<MoveCamera>,
    camera_follow: Res<CameraFollows>,
    agents: Query<&Transform, (With<Agent>, Without<Camera>)>
) {
    for (mut camera_tf, mut proj, ()) in query.iter_mut() {
        let mut x = 0.0;
        let mut y = 0.0;
        let mut r = 0.0;
        let mut pressed = false;

        for e in camera_moved.iter() {
            camera_tf.translation = e.to.extend(camera_tf.translation.z);
        }

        // c to center
        if keys.just_pressed(KeyCode::C) {
            camera_tf.translation = Vec2::ZERO.extend(camera_tf.translation.z);
            proj.scale = 1.5;
        }

        if keys.pressed(KeyCode::W) {
            y += 1.0;
            pressed = true;
        }
        if keys.pressed(KeyCode::A) {
            x -= 1.0;
            pressed = true;
        }
        if keys.pressed(KeyCode::S) {
            y -= 1.0;
            pressed = true;
        }
        if keys.pressed(KeyCode::D) {
            x += 1.0;
            pressed = true;
        }

        // shift is orthographic projection in
        if keys.pressed(KeyCode::ShiftLeft) {
            // proj must be > 0
            if proj.scale > 0.01 {
                proj.scale *= 0.9;
            }
        }
        // space is out
        if keys.pressed(KeyCode::Space) {
            proj.scale *= 1.1;
        }

        match camera_follow.0 {
            Some(entity) => {
                if let Ok(tf) = agents.get(entity) {
                    camera_tf.translation = tf.translation
                        .truncate()
                        .extend(camera_tf.translation.z);
                }
            }
            None => {
                let camera_scalar = CAMERA_SPEED_SCALAR;
                camera_tf.translation.x += x * proj.scale * camera_scalar;
                camera_tf.translation.y += y * proj.scale * camera_scalar;
            }
        }

        // match active_control.iter_mut().next() {
        //     Some((ac_entity, _xi, ac_tf, _agent, (), ())) => {
        //         let facing = ac_tf.local_y().truncate();

        //         action_cooldown.tick(time.delta());

        //         if keys.pressed(KeyCode::Q) {
        //             r += 1.0;
        //             pressed = true;
        //         }
        //         if keys.pressed(KeyCode::E) {
        //             r -= 1.0;
        //             pressed = true;
        //         }
        //         if keys.pressed(KeyCode::Z) {
        //             x = 0.0;
        //             y = 0.0;
        //             r = 0.0;
        //             pressed = true;
        //         }
        //         if keys.pressed(KeyCode::X) {
        //             if let Some(mut ec) = commands.get_entity(ac_entity) {
        //                 ec.remove::<ActiveControl>();
        //             }
        //         }

        //         use BodyControlEvent as BCE;
        //         let mut mvmt = None;

        //         if keys.just_pressed(KeyCode::J) {
        //             mvmt = Some(BCE::left_punch(ac_entity));
        //             pressed = true;
        //         }

        //         if keys.just_pressed(KeyCode::K) {
        //             mvmt = Some(BCE::right_punch(ac_entity));
        //             pressed = true;
        //         }

        //         if keys.just_pressed(KeyCode::H) {
        //             mvmt = Some(BCE::toggle_hands(ac_entity));
        //             pressed = true;
        //         }

        //         if pressed {
        //             if x != 0.0 || y != 0.0 {
        //                 // let angle = Vec2::Y.angle_between(facing);
        //                 // let impulse_dir = Vec2::from_angle(angle).rotate(Vec2::new(x, y));
        //                 // xi.impulse += impulse_dir * agent.one_step_impulse();
        //                 mvmt = Some(BCE::r#move(ac_entity, facing.rotate(Vec2::new(y, -x)), 1.0));
        //             }
        //             if r != 0.0 {
        //                 // each press is turning 12.5 degrees in a direction
        //                 // xi.torque_impulse += (agent.full_turn_torque() / 2.0) * r;
        //                 mvmt = Some(
        //                     BCE::rotate(ac_entity, facing.rotate(Vec2::from_angle(r)), 0.125)
        //                 );
        //             }
        //             // if action_cooldown.elapsed_secs() > 0.5 {
        //             if let Some(mvmt) = mvmt {
        //                 body_controller.send(mvmt);
        //             }
        //             // }
        //             // action_cooldown.reset();
        //         }

        //         let cam_z = camera_tf.translation.z;
        //         camera_tf.translation = ac_tf.translation;
        //         camera_tf.translation = ac_tf.translation;
        //         camera_tf.translation.z = cam_z;
        //     }
        //     None => {
        //         if pressed {
        //
        //         }
        //     }
        // }
    }
}

#[derive(Resource, Default)]
pub struct GUISelect {
    entities: BTreeSet<Entity>,
}

impl GUISelect {
    pub fn new() -> Self {
        Self { entities: BTreeSet::new() }
    }

    pub fn add(&mut self, entity: Entity) {
        self.entities.insert(entity);
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }

    pub fn entities(&self) -> Vec<&Entity> {
        self.entities.iter().collect::<Vec<_>>()
    }
}

// return the entity that the mouse is hovering over on left click
pub fn mouse_selection(
    mouse_button_input: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    rapier_context: Res<RapierContext>,
    mut gui_select: ResMut<GUISelect>,
    camera: Query<(&Transform, &OrthographicProjection), With<Camera>>,
    mut gizmos: Gizmos,
    agents: Query<Entity, With<Agent>>,
    mut commands: Commands,
    keys: Res<Input<KeyCode>>
) {
    if mouse_button_input.pressed(MouseButton::Left) {
        let window = windows.get_single().unwrap();
        let mouse_pos = window.cursor_position().unwrap_or(Vec2::ZERO);
        let mouse_pos = Vec2::new(mouse_pos.x, mouse_pos.y);

        let window_size = Vec2::new(window.width(), window.height());
        let camera = camera.single();
        let camera_pos = camera.0.translation.truncate();
        let camera_scale = camera.1.scale;

        let mouse_pos = Vec2::new(
            mouse_pos.x - window_size.x / 2.0,
            window_size.y / 2.0 - mouse_pos.y
        );
        let pos = camera_pos + mouse_pos * camera_scale;

        // println!("scale: {:?}", camera_scale);
        // println!("camera: {:?}", camera_pos);
        // println!("window: {:?}", window_size);
        // println!("mouse: {:?}", mouse_pos);
        // println!("pos: {:?}", pos);

        gizmos.circle_2d(pos, 100.0, Color::BLACK);

        rapier_context.intersections_with_point(pos, QueryFilter::new(), |entity| {
            gui_select.add(entity);
            if keys.pressed(KeyCode::ControlLeft) {
                if let Ok(entity) = agents.get(entity) {
                    if let Some(mut ec) = commands.get_entity(entity) {
                        ec.insert(ActiveControl);
                    }
                }
            }
            true
        });
    }
}
