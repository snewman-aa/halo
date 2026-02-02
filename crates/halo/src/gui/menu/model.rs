use crate::config::{Config, SlotConfig};
use crate::gui::menu::{
    ANGLE_STEP, ICON_SIZE, INNER_RADIUS, MENU_RADIUS, OUTER_RADIUS, REFERENCE_HEIGHT, SLOT_COUNT,
    SLOT_RADIUS, START_OFFSET, SUB_KEYS, SUBSLOT_RING_RADIUS_FACTOR, SUBSLOT_SCALE_FACTOR,
    SUBSLOT_SIZE_FACTOR,
};
use derive_more::{From, Into};
use gdk_pixbuf::Pixbuf;
use hypraise::desktop::{self, AppInfo, AppQuery};
use hypraise::wm::{ActiveClient, Point, WindowClass, get_active_clients};
use std::f64::consts::PI;
use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, From, Into)]
pub struct Radians(pub f64);

impl Add for Radians {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Radians {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl Mul<f64> for Radians {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self(self.0 * rhs)
    }
}

impl Div<f64> for Radians {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Self(self.0 / rhs)
    }
}

impl Radians {
    pub fn new(val: f64) -> Self {
        Self(val)
    }

    pub fn sin(self) -> f64 {
        self.0.sin()
    }

    pub fn cos(self) -> f64 {
        self.0.cos()
    }

    pub fn atan(val: f64) -> Self {
        Self(val.atan())
    }

    pub fn normalize(self) -> Self {
        // normalize to [-PI, PI]
        Self((self.0 + PI).rem_euclid(2.0 * PI) - PI)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AngularSegment {
    pub start: Radians,
    pub end: Radians,
}

impl AngularSegment {
    pub fn new(start: f64, end: f64) -> Self {
        Self {
            start: Radians(start),
            end: Radians(end),
        }
    }

    pub fn len(&self) -> f64 {
        self.end.0 - self.start.0
    }
}

#[derive(Debug, Clone)]
pub struct SlotGeometry {
    pub center: Point,
    pub radius: f64,
    pub scale: f64,
}

impl SlotGeometry {
    pub fn angle(index: usize) -> Radians {
        Radians(START_OFFSET + (index as f64 * ANGLE_STEP))
    }

    pub fn angle_difference(a: f64, b: f64) -> f64 {
        // Normalize the difference to [-PI, PI]
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

    pub fn calculate_ring(index: usize, total: usize, center: Point, scale_factor: f64) -> Self {
        let angle = Radians(-PI / 2.0 + (index as f64 / total as f64) * 2.0 * PI);
        Self::from_angle(angle, center, scale_factor)
    }

    pub fn from_angle(angle: Radians, center: Point, scale_factor: f64) -> Self {
        let radius_dist = OUTER_RADIUS * SUBSLOT_RING_RADIUS_FACTOR * scale_factor;

        let (x, y) = (
            center.x + radius_dist * angle.cos(),
            center.y + radius_dist * angle.sin(),
        );

        Self {
            center: Point::new(x, y),
            radius: SLOT_RADIUS * SUBSLOT_SIZE_FACTOR * scale_factor,
            scale: SUBSLOT_SCALE_FACTOR,
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

#[derive(Clone)]
pub struct SubSlot {
    pub client: ActiveClient,
    pub key: char,
    pub geometry: SlotGeometry,
    pub pixbuf: Option<Pixbuf>,
}

pub struct State {
    pub center: Point,
    pub slots: Vec<Slot>,
    pub subslots: Vec<SubSlot>,
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
            subslots: Vec::new(),
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
                SlotGeometry::angle_difference(cursor_angle, SlotGeometry::angle(a).0).total_cmp(
                    &SlotGeometry::angle_difference(cursor_angle, SlotGeometry::angle(b).0),
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

        // subslots
        self.subslots.clear();
        let sub_clients = get_active_clients().into_iter().filter(|c| {
            let slot_classes = self
                .slots
                .iter()
                .filter_map(|s| s.app.as_ref())
                .map(|app| app.class.to_lowercase())
                .collect::<Vec<_>>();
            !slot_classes.contains(&c.class.to_lowercase())
        });

        for (sc, shortcut) in sub_clients.zip(SUB_KEYS) {
            let query = AppQuery::new(sc.class.to_string());
            let app_info = desktop::find_desktop_entry(&query);
            let pixbuf = app_info.as_ref().and_then(Slot::load_icon);

            // placeholder geometry
            let geometry = SlotGeometry {
                center,
                radius: 0.0,
                scale: 0.0,
            };

            self.subslots.push(SubSlot {
                client: sc,
                key: *shortcut,
                geometry,
                pixbuf,
            });
        }

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
        self.slot_geometries = self.calculate_main_slots(&filled_indices);

        let segments = self.find_free_segments();
        self.distribute_subslots(&segments);
    }

    fn calculate_main_slots(&self, filled_indices: &[usize]) -> Vec<Option<SlotGeometry>> {
        self.slots
            .iter()
            .enumerate()
            .map(|(i, slot)| {
                slot.app.as_ref().map(|_| {
                    SlotGeometry::calculate(i, filled_indices, self.center, self.scale_factor)
                })
            })
            .collect()
    }

    fn find_free_segments(&self) -> Vec<AngularSegment> {
        let mut free_segments = vec![AngularSegment::new(-PI, PI)];

        for (i, geom) in self.slot_geometries.iter().enumerate() {
            if let Some(g) = geom {
                let center_angle = SlotGeometry::angle(i).normalize();

                let distance = MENU_RADIUS * self.scale_factor;
                // padding
                let half_angle = Radians::atan(g.radius * 1.3 / distance);

                let start = center_angle - half_angle;
                let end = center_angle + half_angle;

                let mut block_intervals = Vec::new();

                if start.0 < -PI {
                    // wraps past -PI
                    block_intervals.push(AngularSegment::new(-PI, end.0));
                    block_intervals.push(AngularSegment::new(start.0 + 2.0 * PI, PI));
                } else if end.0 > PI {
                    // wraps past PI
                    block_intervals.push(AngularSegment::new(start.0, PI));
                    block_intervals.push(AngularSegment::new(-PI, end.0 - 2.0 * PI));
                } else {
                    block_intervals.push(AngularSegment { start, end });
                }

                for block in block_intervals {
                    let mut new_segments = Vec::new();
                    for seg in free_segments {
                        if seg.end <= block.start || seg.start >= block.end {
                            new_segments.push(seg);
                        } else {
                            if seg.start < block.start {
                                new_segments.push(AngularSegment {
                                    start: seg.start,
                                    end: block.start,
                                });
                            }
                            if seg.end > block.end {
                                new_segments.push(AngularSegment {
                                    start: block.end,
                                    end: seg.end,
                                });
                            }
                        }
                    }
                    free_segments = new_segments;
                }
            }
        }
        free_segments
    }

    fn distribute_subslots(&mut self, free_segments: &[AngularSegment]) {
        let subslot_count = self.subslots.len();
        if subslot_count == 0 {
            return;
        }

        let total_free_length: f64 = free_segments.iter().map(|s| s.len()).sum();

        let segments = if total_free_length < 0.1 {
            vec![AngularSegment::new(-PI, PI)]
        } else {
            free_segments.to_vec()
        };
        let valid_total_length: f64 = segments.iter().map(|s| s.len()).sum();

        // calculate ideal fractional counts
        let allocations: Vec<f64> = segments
            .iter()
            .map(|s| (s.len() / valid_total_length) * subslot_count as f64)
            .collect();

        // initial floor allocation
        let mut final_counts: Vec<usize> = allocations.iter().map(|f| f.floor() as usize).collect();
        let current_total: usize = final_counts.iter().sum();

        // distribute remainder to segments with largest fractional parts
        let remainder = subslot_count - current_total;
        if remainder > 0 {
            let mut indices: Vec<usize> = (0..segments.len()).collect();
            indices.sort_by(|&a, &b| {
                let frac_a = allocations[a].fract();
                let frac_b = allocations[b].fract();
                frac_b
                    .partial_cmp(&frac_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            for i in 0..remainder {
                final_counts[indices[i]] += 1;
            }
        }

        let mut subslot_iter = self.subslots.iter_mut();

        for (i, seg) in segments.iter().enumerate() {
            let count = final_counts[i];
            if count == 0 {
                continue;
            }

            let step = seg.len() / count as f64;

            for k in 0..count {
                if let Some(subslot) = subslot_iter.next() {
                    // center within the allocated step
                    let angle = seg.start + Radians(step * (k as f64 + 0.5));
                    subslot.geometry =
                        SlotGeometry::from_angle(angle, self.center, self.scale_factor);
                }
            }
        }

        for subslot in subslot_iter {
            subslot.geometry =
                SlotGeometry::from_angle(Radians(0.0), self.center, self.scale_factor);
        }
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
