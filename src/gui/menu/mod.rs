use std::f64::consts::PI;

pub mod model;
pub mod view;

pub use model::{CursorAction, Slot, SlotGeometry, State};
pub use view::draw;

pub const SLOT_COUNT: usize = 8;
pub const REFERENCE_HEIGHT: f64 = 1440.0;
pub const ICON_SIZE: i32 = 256;
pub const INNER_RADIUS: f64 = 50.0; // hover distance (close)
pub const OUTER_RADIUS: f64 = 160.0; // activation distance (run-or-raise)
pub const MENU_RADIUS: f64 = 150.0; // slot orbital radius
pub const SLOT_RADIUS: f64 = 55.0; // slot bg circle size
pub const CENTER_CIRCLE_RADIUS: f64 = 40.0;
pub const ANGLE_STEP: f64 = 2.0 * PI / SLOT_COUNT as f64;
pub const START_OFFSET: f64 = -PI / 2.0;
pub const ICON_INACTIVE_ALPHA: f64 = 0.6;
