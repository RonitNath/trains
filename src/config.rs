use std::f32::consts::PI;

pub const AGENT_INV_SIZE: usize = 4;
pub const BASE_INV_SIZE: usize = 100;

pub const RADIUS: f32 = 50.0;
pub const EYEBALL_SCALAR: f32 = 0.15;

pub const VISION_SCALAR: f32 = 10.0;
pub const FOV: f32 = PI / 2.0;
pub const HALF_CONES: isize = 7;

pub const VISIBLE_Z: f32 = -1.0;
pub const CHILD_VISIBLE_Z: f32 = 0.5;

pub const AGENTS_PER_BASE_PER_SECOND: usize = 2;
pub const NUM_BASES: usize = 1;

pub const AGENT_MVMT_COOLDOWN: f32 = 0.5;

pub const CAMERA_SPEED_SCALAR: f32 = 10.0;

pub const SPEED_SCALAR: f32 = 5.0;
pub const DEFAULT_LIN_DAMPING: f32 = 10.0;
pub const DEFAULT_ANG_DAMPING: f32 = 10.0;

pub const PIXELS_PER_METER: f32 = 100.0;
pub const DEFAULT_CAM_SCALE: f32 = 1.5;
pub const GRAVITY: bool = false;
