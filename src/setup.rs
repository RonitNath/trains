use std::f32::consts::PI;

use egui::plot::{ Line, Plot, PlotPoints };
use crate::{ prelude::*, controls::GUISelect };
use bevy::{
    input::common_conditions::input_toggle_active,
    time::Stopwatch,
    render::{
        settings::{ WgpuFeatures, WgpuSettings },
        RenderPlugin,
        render_resource::PrimitiveTopology,
        mesh::Indices,
    },
    sprite::Mesh2dHandle,
};
use bevy_egui::{ egui, EguiContext, EguiPlugin, EguiContexts };
use bevy_inspector_egui::{
    self,
    bevy_inspector::hierarchy::SelectedEntities,
    DefaultInspectorConfigPlugin,
};

use bevy_prototype_debug_lines::*;
use bevy_rapier2d::rapier::prelude::{ Isometry, ColliderBuilder };
use bevy_window::PrimaryWindow;

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            DefaultPlugins.set(RenderPlugin {
                wgpu_settings: WgpuSettings {
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                },
            })
        )
            .add_plugins((
                RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER),
                RapierDebugRenderPlugin::default(),
            ))
            .add_plugins((EguiPlugin, DefaultInspectorConfigPlugin, DebugLinesPlugin::default()))
            .add_event::<MoveCamera>()
            .init_resource::<GameControl>()
            .add_systems(Startup, setup_config)
            .add_systems(Update, (inspector_ui.run_if(input_toggle_active(false, KeyCode::I)),))
            // .add_systems(Update, ui_example_system)
            .add_systems(Last, (bevy::window::close_on_esc, frame_time));
    }
}

#[derive(Default, Resource)]
pub struct GameControl {
    pub spawn_more: bool,
}

#[derive(Event)]
pub struct MoveCamera {
    pub to: Vec2,
}

fn frame_time(
    mut contexts: EguiContexts,
    time: Res<Time>,
    mut timer: Local<Stopwatch>,
    mut frame_datapoints: Local<Vec<f64>>,
    mut latest_average: Local<f64>,
    mut datapoints: Local<Vec<[f64; 2]>>
) {
    timer.tick(time.delta());
    frame_datapoints.push(time.delta_seconds_f64());
    // per second

    if timer.elapsed_secs() > 1.0 {
        let average = frame_datapoints.iter().sum::<f64>() / (frame_datapoints.len() as f64);
        datapoints.push([time.elapsed_seconds_f64(), 1.0 / average]);
        *latest_average = average;
        timer.reset();
        frame_datapoints.clear();
    }

    egui::Window
        ::new("Stats")
        .default_size([100.0, 200.0])
        .default_pos([0.0, -1000.0])
        .default_open(false)
        .show(contexts.ctx_mut(), |ui| {
            let time_since_start = time.elapsed_seconds_f64();
            let fps = 1.0 / *latest_average;
            ui.label(format!("Second {:.*}:\tFrame time: {:.*}", 0, time_since_start, 5, fps));
        });
}

fn setup_config(mut commands: Commands, mut rapier_config: ResMut<RapierConfiguration>) {
    if !GRAVITY {
        rapier_config.gravity = Vec2::ZERO;
    }
    // Add a camera so we can see the debug-render.
    commands.spawn(Camera2dBundle::default()).insert(OrthographicProjection {
        scale: DEFAULT_CAM_SCALE,
        ..Default::default()
    });
}

/// Sets up the inspector UI.
fn inspector_ui(world: &mut World, mut selected_entities: Local<SelectedEntities>) {
    let mut egui_context = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .single(world)
        .clone();
    egui::SidePanel
        ::left("hierarchy")
        .default_width(200.0)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Hierarchy");

                let gs = world.get_resource_mut::<GUISelect>();

                let mut empty = true;
                if let Some(gs) = gs.as_ref() {
                    for entity in gs.entities().iter() {
                        empty = false;
                        let name = format!("{:?}", entity);
                        if ui.button(name).clicked() {
                            selected_entities.select_replace(**entity);
                        }
                    }
                }
                if !empty {
                    // clear
                    if ui.button("Clear Selected").clicked() {
                        if let Some(mut gs) = gs {
                            gs.clear();
                        }
                    }
                }
                ui.separator();

                // button deselect all
                if ui.button("Deselect All").clicked() {
                    selected_entities.clear();
                }

                bevy_inspector_egui::bevy_inspector::hierarchy::hierarchy_ui(
                    world,
                    ui,
                    &mut selected_entities
                );

                ui.label("Press I to toggle UI");
                ui.allocate_space(ui.available_size());
            });
        });

    egui::SidePanel
        ::right("inspector")
        .default_width(250.0)
        .show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Inspector");

                match selected_entities.as_slice() {
                    &[entity] => {
                        bevy_inspector_egui::bevy_inspector::ui_for_entity(world, entity, ui);
                    }
                    entities => {
                        bevy_inspector_egui::bevy_inspector::ui_for_entities_shared_components(
                            world,
                            entities,
                            ui
                        );
                    }
                }

                ui.allocate_space(ui.available_size());
            });
        });
}
