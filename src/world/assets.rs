use bevy::sprite::Mesh2dHandle;

use crate::{ prelude::* };

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GeneratedAssets>().add_systems(Startup, init_assets);
    }
}

#[derive(Resource, Default)]
pub struct GeneratedAssets {
    pub meshes: HashMap<String, Mesh2dHandle>,
    pub colors: HashMap<String, (Color, Handle<ColorMaterial>)>,
}

pub fn init_assets(
    mut ga: ResMut<GeneratedAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    let sizes = vec![
        ("AGENT".to_string(), 1.0),
        ("BASE".to_string(), 1.0),
        ("EYEBALL".to_string(), EYEBALL_SCALAR),
        ("BASE_TAG".to_string(), 0.9)
    ];

    for (title, size) in sizes {
        let mesh = meshes.add(
            Mesh::from(shape::Circle { radius: size * RADIUS, ..Default::default() })
        );
        ga.meshes.insert(title, mesh.into());
    }

    let colors = vec![
        ("WHITE".to_string(), Color::WHITE),
        ("BLACK".to_string(), Color::BLACK),
        ("RED".to_string(), Color::RED),
        ("BLUE".to_string(), Color::BLUE),
        ("GREEN".to_string(), Color::GREEN),
        ("YELLOW".to_string(), Color::YELLOW),
        ("ORANGE".to_string(), Color::ORANGE),
        ("PURPLE".to_string(), Color::PURPLE)
    ];

    for (title, color) in colors {
        let material = materials.add(color.into());
        ga.colors.insert(title.clone(), (color, material));
        // for each color, generate an "A{color}" color which is the color with each channel as 1-channel value
        // for example, ARED is (1-RED.r, etc)

        let a_color = Color::rgb(1.0 - color.r(), 1.0 - color.g(), 1.0 - color.b());
        let a_material = materials.add(a_color.into());
        ga.colors.insert(format!("A{}", title), (a_color, a_material));
    }
}
