use bevy_egui::{ egui, EguiContexts };

use crate::{ prelude::*, world::agent::{ Agent, S1, Movement, Action, Goal } };

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraFollows>()
            .init_resource::<SelectedPos>()
            .add_systems(Update, display);
    }
}

#[derive(Component)]
pub struct ActiveControl;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct CameraFollows(pub Option<Entity>);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct SelectedPos(pub Option<Vec2>);

pub fn display(
    ac: Query<(Entity, &Agent), With<ActiveControl>>,
    mut contexts: EguiContexts,
    mut selected_agent: Local<Option<Entity>>,
    mut display_map: Local<bool>,
    mut camera_follows: ResMut<CameraFollows>,
    keys: Res<Input<KeyCode>>,
    mut s1_wtr: EventWriter<S1>,
    sp: Res<SelectedPos>
) {
    // let mut agent = None;

    let mut agents = vec![];
    for (e, agent) in ac.iter() {
        agents.push((e, agent));
    }

    egui::Window::new("Active Control").show(contexts.ctx_mut(), |ui| {
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

                match sp.0 {
                    Some(pos) => {
                        if
                            ui.button(format!("Move to {:?}", pos)).clicked() ||
                            keys.pressed(KeyCode::M)
                        {
                            s1_wtr.send(S1 {
                                id: e,
                                action: Action::G(Goal::move_to(pos)),
                            });
                        }
                    }
                    None => {
                        ui.label("Select a position to be able to move to");
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
