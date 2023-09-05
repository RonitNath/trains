use bevy_egui::{ egui, EguiContexts };

use crate::{ prelude::*, world::agent::{ Agent, S1, Movement, Action, Goal, Missions } };

use super::sk::SpatialKnowledge;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraFollows>()
            .init_resource::<SelectedPos>()
            .init_resource::<ActiveControl>()
            .add_systems(Update, display);
    }
}

#[derive(Resource, Default)]
pub struct ActiveControl(pub Option<Entity>);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct CameraFollows(pub Option<Entity>);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct SelectedPos(pub Option<Vec2>);

pub fn display(
    mut contexts: EguiContexts,
    mut display: Local<HashSet<String>>,
    mut camera_follows: ResMut<CameraFollows>,
    keys: Res<Input<KeyCode>>,
    sp: Res<SelectedPos>,
    mut s1_wtr: EventWriter<S1>,
    mut ac: ResMut<ActiveControl>,
    info: Query<(&SpatialKnowledge, &Goal, &Transform)>,
    mut gizmos: Gizmos
) {
    // let mut agent = None;

    egui::Window::new("Active Control").show(contexts.ctx_mut(), |ui| {
        match ac.0 {
            None => {
                ui.label("Select an agent");
            }
            Some(e) => {
                ui.label(format!("Selected agent {:?}", e));
                if ui.button("Deselect").clicked() {
                    ac.0 = None;
                    return;
                }
                match camera_follows.0 {
                    None => {
                        if ui.button("Follow agent").clicked() {
                            camera_follows.0 = Some(e);
                        }
                    }
                    Some(a) => {
                        match a == e {
                            true => {
                                // the agent we are following is the agent we selected
                                if ui.button("Stop following agent").clicked() {
                                    camera_follows.0 = None;
                                }
                            }
                            false => {
                                // we are following an agent other than the one we selected
                                if ui.button("Follow agent").clicked() {
                                    camera_follows.0 = Some(e);
                                }
                            }
                        }
                    }
                }

                if ui.button("Spawn Child").clicked() {
                    s1_wtr.send(S1 {
                        id: e,
                        action: Action::SpawnChild,
                    });
                }

                match sp.0 {
                    Some(pos) => {
                        if let Ok((sk, goal, tf)) = info.get(e) {
                            let my_pos = tf.translation.truncate();
                            if
                                ui.button(format!("Move to {:?}", pos)).clicked() ||
                                keys.pressed(KeyCode::M)
                            {
                                s1_wtr.send(S1 {
                                    id: e,
                                    action: Action::G(Goal::move_to(pos)),
                                });
                            }

                            if let Missions::MoveTo(pos) = &goal.mission {
                                let (path, b) = sk.path(my_pos, *pos);

                                for pos in path.iter() {
                                    ui.label(format!("Pathpoint: {:?}", pos));

                                    gizmos.circle_2d(*pos, sk.tile_size / 2.0, Color::GREEN);
                                }
                            }
                        }
                    }
                    None => {
                        ui.label("Select a position to be able to move to");
                    }
                }

                match display.contains(&"map".to_string()) {
                    true => {
                        if ui.button("Hide map").clicked() {
                            display.remove(&"map".to_string());
                        }
                        if let Ok((sk, goal, tf)) = info.get(e) {
                            let my_pos = tf.translation.truncate();
                            let occupied = sk.get_occupied();
                            let mut c = 0;
                            let display_tiles = match display.contains(&"tiles".to_string()) {
                                true => {
                                    if ui.button("Hide tiles").clicked() {
                                        display.remove(&"tiles".to_string());
                                    }
                                    true
                                }
                                false => {
                                    if ui.button("Display tiles").clicked() {
                                        display.insert("tiles".to_string());
                                    }
                                    false
                                }
                            };
                            let mut num_details = 0;
                            for info in sk.details() {
                                num_details += 1;
                                ui.label(format!("Info: {:?}", info));
                            }
                            ui.label(format!("Num details: {}", num_details));
                            let mut num_temp = 0;
                            for temp in sk.temp.iter() {
                                num_temp += 1;
                                ui.label(format!("Temp: {:?}", temp));
                            }
                            ui.label(format!("Num temp: {num_temp}"));

                            for grid in occupied.iter() {
                                c += 1;
                                let pos = sk.pos(*grid);

                                if display_tiles {
                                    for entity in sk.tile(grid).iter() {
                                        ui.label(format!("{:?} contains {entity:?}", grid));
                                    }
                                }

                                gizmos.circle_2d(pos, sk.tile_size / 2.0, Color::BLUE);
                            }
                            ui.label(format!("{} tiles occupied", c));
                        } else {
                            ui.label("No spatial knowledge");
                        }
                    }
                    false => {
                        if ui.button("Display map").clicked() {
                            display.insert("map".to_string());
                        }
                    }
                }
            }
        }
    });
}
