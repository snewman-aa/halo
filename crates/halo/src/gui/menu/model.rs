use crate::config::{Config, SlotConfig};
use crate::gui::menu::{
    ANGLE_STEP, ICON_SIZE, INNER_RADIUS, MENU_RADIUS, OUTER_RADIUS, REFERENCE_HEIGHT, SLOT_COUNT,
    SLOT_RADIUS, START_OFFSET,
};
use gdk_pixbuf::Pixbuf;
use hypraise::desktop::AppInfo;
use hypraise::wm::{Point, WindowClass};
use std::f64::consts::PI;

#[derive(Debug, Clone)]
pub struct SlotGeometry {
    pub center: Point,
    pub radius: f64,
    pub scale: f64,
}

impl SlotGeometry {
    pub fn angle(index: usize) -> f64 {
        START_OFFSET + (index as f64 * ANGLE_STEP)
    }

    pub fn angle_difference(a: f64, b: f64) -> f64 {
        // Normalize the difference to [-PI, PI] to find the shortest path around the circle
        ((a - b + PI).rem_euclid(2.0 * PI) - PI).abs()
    }

    /// Squishes slots when many are filled. It looks at the previous and next filled slots to
    /// determine available angular space (`width`).
    pub fn calculate(
        index: usize,
        filled_indices: &[usize],
        center: Point,
        scale_factor: f64,
    ) -> Self {
        let current_pos = filled_indices.iter().position(|&x| x == index).unwrap();
        let prev_idx =
            filled_indices[(current_pos + filled_indices.len() - 1) % filled_indices.len()];
        let next_idx = filled_indices[(current_pos + 1) % filled_indices.len()];

        // if this is the only slot, it gets the full circle (2 PI)
        let d_l = if prev_idx == index {
            2.0 * PI
        } else {
            // distance counter-clockwise
            ((index + SLOT_COUNT - prev_idx) % SLOT_COUNT) as f64 * ANGLE_STEP
        };
        let d_r = if next_idx == index {
            2.0 * PI
        } else {
            // distance clockwise
            ((next_idx + SLOT_COUNT - index) % SLOT_COUNT) as f64 * ANGLE_STEP
        };

        // average available space to scale the icon size
        // basically, room to breathe relative to slot density
        let width = (d_l + d_r) / 2.0;
        let scale = (width / ANGLE_STEP).sqrt().min(2.5);
        let current_slot_radius = SLOT_RADIUS * scale * scale_factor;

        let angle = Self::angle(index);
        let (x, y) = (
            center.x + (MENU_RADIUS * scale_factor) * angle.cos(),
            center.y + (MENU_RADIUS * scale_factor) * angle.sin(),
        );

        Self {
            center: Point::new(x, y),
            radius: current_slot_radius,
            scale,
        }
    }
}

#[derive(Clone)]
pub struct Slot {
    pub app: Option<AppInfo>,
    pub pixbuf: Option<Pixbuf>,
}

impl Slot {
    pub fn new(app: Option<AppInfo>) -> Self {
        let pixbuf = app.as_ref().and_then(Self::load_icon);
        Self { app, pixbuf }
    }

    fn load_icon(app: &AppInfo) -> Option<Pixbuf> {
        (!app.icon.as_os_str().is_empty())
            .then(|| Pixbuf::from_file_at_scale(&app.icon, ICON_SIZE, ICON_SIZE, true).ok())?
    }

    pub fn empty() -> Self {
        Self {
            app: None,
            pixbuf: None,
        }
    }

    pub fn from_config(cfg: &SlotConfig) -> Self {
        let app = cfg
            .app
            .as_ref()
            .map(|query| AppInfo::new(query, cfg.class.clone(), cfg.exec.clone()));
        Self::new(app)
    }

    pub fn is_running(&self, active_classes: &[WindowClass]) -> bool {
        self.app.as_ref().is_some_and(|app| {
            active_classes
                .iter()
                .any(|c| c.to_lowercase() == app.class.to_lowercase())
        })
    }

    pub fn is_broken(&self) -> bool {
        self.app
            .as_ref()
            .map(|a| a.exec.as_str().is_empty())
            .unwrap_or(false)
    }
}

pub struct State {
    pub center: Point,
    pub slots: Vec<Slot>,
    pub hover_index: Option<usize>,
    pub active_classes: Vec<WindowClass>,
    pub scale_factor: f64,
    pub slot_geometries: Vec<Option<SlotGeometry>>,
}

impl State {
    pub fn new(
        slots: Vec<Slot>,
        center: Point,
        active_classes: Vec<WindowClass>,
        scale_factor: f64,
    ) -> Self {
        let mut state = Self {
            center,
            slots,
            hover_index: None,
            active_classes,
            scale_factor,
            slot_geometries: Vec::new(),
        };
        state.recalculate_geometries();
        state
    }

    pub fn init_slots(config: &Config) -> Vec<Slot> {
        let mut slots = vec![Slot::empty(); SLOT_COUNT];

        config
            .slots
            .iter()
            .filter_map(|cfg| cfg.direction.map(|dir| (dir, cfg)))
            .for_each(|(dir, cfg)| {
                slots[dir.as_index()] = Slot::from_config(cfg);
            });

        slots
    }

    pub fn update_cursor(&mut self, cursor: Point) -> CursorAction {
        let dist = self.distance_from_center(cursor);

        // dead zone
        if dist <= INNER_RADIUS * self.scale_factor {
            return self.clear_hover();
        }

        let new_idx = self.find_nearest_slot(cursor);
        let changed = self.hover_index != new_idx;
        let activate = dist > OUTER_RADIUS * self.scale_factor && new_idx.is_some();

        self.hover_index = new_idx;

        CursorAction::new(changed || activate, activate)
    }

    fn distance_from_center(&self, cursor: Point) -> f64 {
        let (dx, dy) = (cursor.x - self.center.x, cursor.y - self.center.y);
        dx.hypot(dy)
    }

    fn clear_hover(&mut self) -> CursorAction {
        let changed = self.hover_index.is_some();
        self.hover_index = None;
        CursorAction::new(changed, false)
    }

    fn cursor_angle(&self, cursor: Point) -> f64 {
        let (dx, dy) = (cursor.x - self.center.x, cursor.y - self.center.y);
        dy.atan2(dx)
    }

    fn find_nearest_slot(&self, cursor: Point) -> Option<usize> {
        let cursor_angle = self.cursor_angle(cursor);

        (0..SLOT_COUNT)
            .filter(|&i| self.slots[i].app.is_some())
            .min_by(|&a, &b| {
                SlotGeometry::angle_difference(cursor_angle, SlotGeometry::angle(a)).total_cmp(
                    &SlotGeometry::angle_difference(cursor_angle, SlotGeometry::angle(b)),
                )
            })
    }

    pub fn get_hovered_app(&self) -> Option<&AppInfo> {
        self.hover_index
            .and_then(|idx| self.slots[idx].app.as_ref())
    }

    pub fn refresh(
        &mut self,
        center: Point,
        active_classes: Vec<WindowClass>,
        monitor_height: f64,
    ) {
        self.active_classes = active_classes;
        self.center = center;
        self.hover_index = None;
        self.scale_factor = monitor_height / REFERENCE_HEIGHT;
        self.recalculate_geometries();
    }

    fn filled_slot_indices(&self) -> Vec<usize> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.app.as_ref().map(|_| i))
            .collect()
    }

    fn recalculate_geometries(&mut self) {
        let filled_indices = self.filled_slot_indices();

        self.slot_geometries = self
            .slots
            .iter()
            .enumerate()
            .map(|(i, slot)| {
                slot.app.as_ref().map(|_| {
                    SlotGeometry::calculate(i, &filled_indices, self.center, self.scale_factor)
                })
            })
            .collect();
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CursorAction {
    pub should_redraw: bool,
    pub should_activate: bool,
}

impl CursorAction {
    pub fn new(should_redraw: bool, should_activate: bool) -> Self {
        Self {
            should_redraw,
            should_activate,
        }
    }
}
