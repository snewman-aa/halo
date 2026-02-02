use std::f64::consts::PI;

pub mod model;
pub mod view;

pub use model::{CursorAction, Slot, SlotGeometry, State};
pub use view::draw;

pub const SLOT_COUNT: usize = 8;
pub const REFERENCE_HEIGHT: f64 = 1440.0;
pub const ICON_SIZE: i32 = 256;
pub const INNER_RADIUS: f64 = 48.0; // hover distance (close)
pub const OUTER_RADIUS: f64 = 128.0; // activation distance (run-or-raise)
pub const MENU_RADIUS: f64 = 120.0; // slot orbital radius
pub const SLOT_RADIUS: f64 = 52.0; // slot bg circle size
pub const CENTER_CIRCLE_RADIUS: f64 = 32.0;
pub const ANGLE_STEP: f64 = 2.0 * PI / SLOT_COUNT as f64;
pub const START_OFFSET: f64 = -PI / 2.0;
pub const ICON_INACTIVE_ALPHA: f64 = 0.6;
pub const SUB_KEYS: &[char] = &['a', 's', 'd', 'f', 'q', 'w', 'e', 'r', 'z', 'x', 'c', 'v'];

// Subslot configuration
pub const SUBSLOT_RING_RADIUS_FACTOR: f64 = 1.6; // How far out the ring is (relative to OUTER_RADIUS)
pub const SUBSLOT_SIZE_FACTOR: f64 = 0.4; // Size of subslot circle relative to SLOT_RADIUS
pub const SUBSLOT_SCALE_FACTOR: f64 = 0.6; // Internal scale for text/icons
