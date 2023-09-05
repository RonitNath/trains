use std::f32::consts::PI;

pub use bevy::{ prelude::*, utils::{ HashMap, HashSet } };
use bevy::sprite::{ MaterialMesh2dBundle, Mesh2dHandle };
pub use bevy_rapier2d::prelude::*;
use rand::{ seq::SliceRandom, Rng };
pub use bevy::reflect::Reflect;

pub use crate::config::*;

pub type Render = MaterialMesh2dBundle<ColorMaterial>;

pub fn sig(num: f32, figs: usize) -> f32 {
    let mult = (10.0_f32).powi(figs as i32);
    (num * mult).round() / mult
}

#[test]
fn test_sig() {
    assert_eq!(sig(1.234_567_9, 2), 1.23);
    assert_eq!(sig(1.234_567_9, 3), 1.235);
    assert_eq!(sig(1.234_567_9, 4), 1.2346);
    assert_eq!(sig(1.234_567_9, 5), 1.23457);
    assert_eq!(sig(1.234_567_9, 6), 1.234568);
    assert_eq!(sig(1.234_567_9, 7), 1.2345679);
    assert_eq!(sig(1.234_567_9, 8), 1.234_567_9);
    assert_eq!(sig(1.234_567_9, 9), 1.234_567_9);
}

pub fn make_render(
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
    transform: Transform
) -> Render {
    MaterialMesh2dBundle {
        mesh: mesh.into(),
        material,
        transform,
        ..Default::default()
    }
}

pub fn i32_neighbors_pair_shuffled((x, y): (i32, i32)) -> Vec<(i32, i32)> {
    let mut rng = rand::thread_rng();
    let mut neighbors = i32_neighbors((x, y));
    neighbors.shuffle(&mut rng);
    neighbors
}

pub fn i32_neighbors((x, y): (i32, i32)) -> Vec<(i32, i32)> {
    vec![(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
}

#[derive(Bundle, Clone)]
pub struct PhysicsBody {
    pub body: RigidBody,
    pub collider: Collider,
    pub render: Render,
    pub x_f: ExternalForce,
    pub x_i: ExternalImpulse,
    pub vel: Velocity,
    pub damping: Damping,
}

impl PhysicsBody {
    pub fn new(collider: Collider, render: Render) -> Self {
        Self {
            collider,
            render,
            body: RigidBody::Dynamic,
            x_f: ExternalForce::default(),
            x_i: ExternalImpulse::default(),
            vel: Velocity::default(),
            damping: Damping {
                linear_damping: DEFAULT_LIN_DAMPING,
                angular_damping: DEFAULT_ANG_DAMPING,
            },
        }
    }

    pub fn from_assets(
        radius: f32,
        color: Color,
        pos: Vec2,
        dir: Vec2,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<ColorMaterial>>
    ) -> Self {
        let collider = Collider::ball(radius);
        let mesh = Mesh::from(shape::Circle::new(radius));
        let mut transform = Transform::from_xyz(pos.x, pos.y, VISIBLE_Z);
        transform.rotate_z(Vec2::Y.angle_between(dir));
        let render = Render {
            mesh: meshes.add(mesh).into(),
            material: materials.add(ColorMaterial::from(color)),
            transform,
            ..Default::default()
        };
        Self::new(collider, render)
    }

    pub fn color(mut self, color: Handle<ColorMaterial>) -> Self {
        self.render.material = color;
        self
    }

    pub fn pos(mut self, pos: Vec3) -> Self {
        self.render.transform = Transform::from_translation(pos);
        self
    }

    pub fn basic(
        mesh: Mesh2dHandle,
        material: Handle<ColorMaterial>,
        collider: Collider,
        pos: Vec3
    ) -> Self {
        Self::new(collider, Render {
            mesh: mesh.into(),
            material,
            transform: Transform::from_translation(pos),
            ..Default::default()
        })
    }
}

#[derive(Reflect, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

impl Side {
    pub fn color(&self) -> Color {
        match self {
            Side::Top => Color::BLUE,
            Side::Bottom => Color::RED,
            Side::Left => Color::GREEN,
            Side::Right => Color::YELLOW,
        }
    }

    pub fn to_vec2(&self) -> Vec2 {
        match self {
            Side::Top => Vec2::Y,
            Side::Bottom => Vec2::NEG_Y,
            Side::Left => Vec2::NEG_X,
            Side::Right => Vec2::X,
        }
    }

    pub fn opposite(&self) -> Side {
        match self {
            Side::Top => Side::Bottom,
            Side::Bottom => Side::Top,
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }

    pub fn scalar_to_pos(&self, scalar: f32, global_pos: &Transform) -> Transform {
        let side_dir = self.to_vec2();
        let facing_dir = global_pos.local_y().truncate();
        let angle = Vec2::Y.angle_between(facing_dir);
        let rotated_dir = Vec2::from_angle(angle).rotate(side_dir);
        let rotated_angle = Vec2::Y.angle_between(rotated_dir);

        let pos = global_pos.translation.truncate() + rotated_dir * scalar;
        let pos = pos.extend(global_pos.translation.z);
        let mut pos = Transform::from_translation(pos);
        pos.rotation = global_pos.rotation;
        // pos.rotate_z(rotated_angle);
        pos
    }

    pub fn iter() -> impl Iterator<Item = Side> {
        vec![Side::Top, Side::Bottom, Side::Left, Side::Right].into_iter()
    }

    pub fn shuffled() -> Vec<Side> {
        let mut rng = rand::thread_rng();
        let mut sides = vec![Side::Top, Side::Bottom, Side::Left, Side::Right];
        sides.shuffle(&mut rng);
        sides
    }

    /// Returned as attacher (child), attachee (parent). Returns as (child, parent)
    pub fn compute_sides(child_tf: &Transform, parent_tf: &Transform) -> (Side, Side) {
        // go through each side, and find the two with the smallest distance
        let mut smallest_dist = f32::MAX;

        // (child, parent)
        let mut pair = (Side::Top, Side::Top);
        for child_side in Side::iter() {
            for parent_side in Side::iter() {
                let child_tf = child_side.scalar_to_pos(1.0, child_tf);
                let parent_tf = parent_side.scalar_to_pos(1.0, parent_tf);
                let child_pos = child_tf.translation.truncate();
                let parent_pos = parent_tf.translation.truncate();

                let dist = child_pos.distance_squared(parent_pos);
                if dist < smallest_dist {
                    smallest_dist = dist;
                    pair = (child_side, parent_side);
                }
            }
        }

        pair
    }

    // pub fn from_dot(dot: f32) -> Self {
    //     if dot > 0.5 {
    //         Side::Bottom
    //     } else if dot < 0.5 && dot >= 0.0 {
    //         Side::Right
    //     } else if dot <= -0.0 && dot > -0.5 {
    //         Side::Top
    //     } else if dot < -0.5 {
    //         Side::Left
    //     } else {
    //         panic!("Invalid dot product: {}", dot);
    //         Side::Top
    //     }
    // }

    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..=3) {
            0 => Side::Top,
            1 => Side::Bottom,
            2 => Side::Left,
            _ => Side::Right,
        }
    }
}

/// Sides of a triangle
#[derive(Reflect, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Triside {
    Down,
    Left,
    Right,
}

impl Triside {
    pub fn color(&self) -> Color {
        match self {
            Triside::Down => Color::RED,
            Triside::Left => Color::GREEN,
            Triside::Right => Color::YELLOW,
        }
    }

    pub fn to_vec2(&self) -> Vec2 {
        match self {
            Triside::Down => Vec2::new(0.0, -1.0),
            Triside::Left => Vec2::new(-0.866, 0.5).normalize_or_zero(),
            Triside::Right => Vec2::new(0.866, 0.5).normalize_or_zero(),
        }
    }

    pub fn iter() -> impl Iterator<Item = Triside> {
        vec![Triside::Down, Triside::Left, Triside::Right].into_iter()
    }

    /// Returns as (child, parent). Computes the closest side.
    pub fn compute_sides(child_tf: &Transform, parent_tf: &Transform) -> (Triside, Triside) {
        // go through each side, and find the two with the smallest distance
        let mut smallest_dist = f32::MAX;

        // (child, parent)
        let mut pair = (Triside::Down, Triside::Down);
        for child_side in Triside::iter() {
            for parent_side in Triside::iter() {
                let child_tf = child_side.scalar_to_pos(1.0, child_tf);
                let parent_tf = parent_side.scalar_to_pos(1.0, parent_tf);
                let child_pos = child_tf.translation.truncate();
                let parent_pos = parent_tf.translation.truncate();

                let dist = child_pos.distance_squared(parent_pos);
                if dist < smallest_dist {
                    smallest_dist = dist;
                    pair = (child_side, parent_side);
                }
            }
        }

        pair
    }

    pub fn filter_abc(&self, [a, b, c]: [Vec2; 3]) -> [Vec2; 2] {
        match self {
            Triside::Right => { [a, b] }
            Triside::Down => { [b, c] }
            Triside::Left => { [c, a] }
        }
    }

    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..=2) {
            0 => Triside::Down,
            1 => Triside::Left,
            _ => Triside::Right,
        }
    }

    pub fn shuffled() -> Vec<Triside> {
        let mut rng = rand::thread_rng();
        let mut sides = vec![Triside::Down, Triside::Left, Triside::Right];
        sides.shuffle(&mut rng);
        sides
    }

    /// Each side's midpoint is 0.5 away from the center
    pub fn scalar_to_pos(&self, scalar: f32, global_pos: &Transform) -> Transform {
        let side_dir = self.to_vec2();
        let global_facing_dir = global_pos.local_y().truncate();
        let global_angle = Vec2::Y.angle_between(global_facing_dir);
        let side_facing_dir = -side_dir;
        let side_angle = Vec2::Y.angle_between(side_facing_dir);
        let rotated_dir = side_dir.rotate(Vec2::from_angle(global_angle));

        let pos = global_pos.translation.truncate() + rotated_dir * scalar;
        let pos = pos.extend(global_pos.translation.z);
        let mut pos = Transform::from_translation(pos);
        pos.rotation = global_pos.rotation;
        pos.rotate_z(side_angle);
        pos
    }
}

pub fn random_pos(der: f32) -> Vec2 {
    let mut rng = rand::thread_rng();
    Vec2::new(rng.gen_range(-der..der), rng.gen_range(-der..der))
}

pub fn random_dir() -> Vec2 {
    let mut rng = rand::thread_rng();
    Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize()
}

pub fn random_color() -> Color {
    let mut rng = rand::thread_rng();
    Color::rgb(rng.gen(), rng.gen(), rng.gen())
}

pub fn grid(tile_size: f32, pos: Vec2) -> Grid {
    let x = (pos.x / tile_size).round() as i32;
    let y = (pos.y / tile_size).round() as i32;
    Grid { x, y }
}

/// Returns the position of the center of the tile
pub fn pos(tile_size: f32, grid: Grid) -> Vec2 {
    let x = (grid.x as f32) * tile_size;
    let y = (grid.y as f32) * tile_size;
    Vec2::new(x, y)
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Reflect, PartialOrd, Ord)]
pub struct Grid {
    pub x: i32,
    pub y: i32,
}

impl Grid {
    // diagonal is 14, straight is 10
    pub fn distance(&self, other: Grid) -> i32 {
        let x = (self.x - other.x).abs();
        let y = (self.y - other.y).abs();
        let diag = x.min(y);
        let straight = x.max(y) - diag;
        diag * 14 + straight * 10
    }

    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn left(&self) -> Self {
        Self::new(self.x - 1, self.y)
    }

    pub fn right(&self) -> Self {
        Self::new(self.x + 1, self.y)
    }

    pub fn up(&self) -> Self {
        Self::new(self.x, self.y + 1)
    }

    pub fn down(&self) -> Self {
        Self::new(self.x, self.y - 1)
    }

    pub fn q1(&self) -> Self {
        Self::new(self.x + 1, self.y + 1)
    }

    pub fn q2(&self) -> Self {
        Self::new(self.x - 1, self.y + 1)
    }

    pub fn q3(&self) -> Self {
        Self::new(self.x - 1, self.y - 1)
    }

    pub fn q4(&self) -> Self {
        Self::new(self.x + 1, self.y - 1)
    }

    pub fn neighbors_by_cost(&self) -> (Vec<Self>, Vec<Self>) {
        let ten_cost = vec![self.left(), self.right(), self.up(), self.down()];

        let fourteen_cost = vec![self.q1(), self.q2(), self.q3(), self.q4()];

        (ten_cost, fourteen_cost)
    }

    pub fn neighbors(&self) -> Vec<Self> {
        vec![
            self.left(),
            self.right(),
            self.up(),
            self.down(),
            self.q1(),
            self.q2(),
            self.q3(),
            self.q4()
        ]
    }

    pub fn random_neighbors(&self) -> Vec<Self> {
        let mut rng = rand::thread_rng();
        let mut neighbors = self.neighbors();
        neighbors.shuffle(&mut rng);
        neighbors
    }
}

pub fn facing_debuff(facing: Vec2, desired_dir: Vec2) -> f32 {
    let angle_diff = facing.angle_between(desired_dir);
    let x = angle_diff.abs();
    (2.0 * x).cos() / 4.0 + (0.5 * x).cos() / 4.0 + 0.5
}

#[test]
fn test_distance() {
    let origin = Grid::new(0, 0);
    let up = Grid::new(0, 1);
    let right = Grid::new(1, 0);
    let diag = Grid::new(1, 1);
    let diag2 = Grid::new(2, 2);

    let custom1 = Grid::new(1, 2);

    assert_eq!(origin.distance(up), 10);
    assert_eq!(origin.distance(right), 10);
    assert_eq!(origin.distance(diag), 14);
    assert_eq!(origin.distance(diag2), 28);

    assert_eq!(up.distance(right), 14);
    assert_eq!(up.distance(diag), 10);

    assert_eq!(right.distance(diag), 10);

    assert_eq!(origin.distance(custom1), 24);
}
