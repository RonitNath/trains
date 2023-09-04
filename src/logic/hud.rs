use bevy_egui::{ egui, EguiContexts };

use crate::{ prelude::*, world::agent::Agent };

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraFollows>().add_systems(Update, display);
    }
}

#[derive(Component)]
pub struct ActiveControl;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct CameraFollows(pub Option<Entity>);

pub fn display(
    ac: Query<(Entity, &Agent), With<ActiveControl>>,
    mut contexts: EguiContexts,
    mut selected_agent: Local<Option<Entity>>,
    mut display_map: Local<bool>,
    mut camera_follows: ResMut<CameraFollows>
) {
    // let mut agent = None;

    let mut agents = vec![];
    for (e, agent) in ac.iter() {
        agents.push((e, agent));
    }

    egui::Window
        ::new("Active Control")
        .default_open(false)
        .show(contexts.ctx_mut(), |ui| {
            match *selected_agent {
                None => {
                    match agents.len() {
                        0 => {
                            ui.label("Select an agent");
                        }
                        1 => {
                            ui.label("1 agent selected");
                            *selected_agent = Some(agents[0].0);
                        }
                        n => {
                            for agent in agents.iter() {
                                if ui.button(format!("Select agent {:?}", agent.0)).clicked() {
                                    *selected_agent = Some(agent.0);
                                }
                            }
                        }
                    }
                }
                Some(e) => {
                    ui.label(format!("Selected agent {:?}", e));
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
                    match *display_map {
                        true => {
                            if ui.button("Hide map").clicked() {
                                *display_map = false;
                            }
                        }
                        false => {
                            if ui.button("Display map").clicked() {
                                *display_map = true;
                            }
                        }
                    }
                }
            }
        });
}
