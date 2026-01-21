use crate::config::{Config, SlotConfig};
use crate::gui::geometry::SlotGeometry;
use crate::gui::ui::ThemeColors;
use crate::gui::{
    ANGLE_STEP, CENTER_CIRCLE_RADIUS, ICON_INACTIVE_ALPHA, ICON_SIZE, INNER_RADIUS, OUTER_RADIUS,
    REFERENCE_HEIGHT, SLOT_COUNT, START_OFFSET,
};
use crate::sys::desktop::AppInfo;
use crate::sys::wm::{Point, WindowClass};
use cairo::Context;
use gdk_pixbuf::Pixbuf;
use gdk4::prelude::*;
use palette::Srgba;
use std::f64::consts::PI;
use std::iter::zip;

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

    fn slot_angle(index: usize) -> f64 {
        START_OFFSET + (index as f64 * ANGLE_STEP)
    }

    fn angle_difference(a: f64, b: f64) -> f64 {
        ((a - b + PI).rem_euclid(2.0 * PI) - PI).abs()
    }

    fn find_nearest_slot(&self, cursor: Point) -> Option<usize> {
        let cursor_angle = self.cursor_angle(cursor);

        (0..SLOT_COUNT)
            .filter(|&i| self.slots[i].app.is_some())
            .min_by(|&a, &b| {
                Self::angle_difference(cursor_angle, Self::slot_angle(a))
                    .total_cmp(&Self::angle_difference(cursor_angle, Self::slot_angle(b)))
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

struct SlotRenderer<'a> {
    slot: &'a Slot,
    geometry: &'a SlotGeometry,
    hovered: bool,
    active_classes: &'a [WindowClass],
}

impl<'a> SlotRenderer<'a> {
    fn new(
        slot: &'a Slot,
        geometry: &'a SlotGeometry,
        hovered: bool,
        active_classes: &'a [WindowClass],
    ) -> Self {
        Self {
            slot,
            geometry,
            hovered,
            active_classes,
        }
    }

    fn draw(&self, cr: &Context, colors: &ThemeColors) -> Result<(), cairo::Error> {
        self.draw_circle(cr, colors)?;
        self.draw_content(cr)?;
        Ok(())
    }

    fn draw_circle(&self, cr: &Context, colors: &ThemeColors) -> Result<(), cairo::Error> {
        let state = SlotState::resolve(self.slot, self.hovered, self.active_classes);
        let color = state.color(colors);
        let (r, g, b, a) = color.into_components();
        cr.set_source_rgba(r, g, b, a);
        cr.arc(
            self.geometry.center.x,
            self.geometry.center.y,
            self.geometry.radius,
            0.0,
            2.0 * PI,
        );
        cr.fill()
    }

    fn draw_content(&self, cr: &Context) -> Result<(), cairo::Error> {
        if let Some(pixbuf) = &self.slot.pixbuf {
            self.draw_icon(cr, pixbuf)
        } else if let Some(app) = &self.slot.app {
            self.draw_text(cr, &app.name)
        } else {
            Ok(())
        }
    }

    fn draw_icon(&self, cr: &Context, pixbuf: &Pixbuf) -> Result<(), cairo::Error> {
        let icon_scale = (self.geometry.radius * 2.0 * 0.75) / ICON_SIZE as f64;
        let (iw, ih) = (
            pixbuf.width() as f64 * icon_scale,
            pixbuf.height() as f64 * icon_scale,
        );
        let (ix, iy) = (
            self.geometry.center.x - iw / 2.0,
            self.geometry.center.y - ih / 2.0,
        );

        cr.save()?;
        cr.translate(ix, iy);
        cr.scale(icon_scale, icon_scale);

        let running = self.slot.is_running(self.active_classes);
        if !running && !self.hovered {
            cr.push_group();
            cr.set_source_pixbuf(pixbuf, 0.0, 0.0);
            cr.paint()?;
            cr.pop_group_to_source()?;
            cr.paint_with_alpha(ICON_INACTIVE_ALPHA)?;
        } else {
            cr.set_source_pixbuf(pixbuf, 0.0, 0.0);
            cr.paint()?;
        }
        cr.restore()
    }

    fn draw_text(&self, cr: &Context, text: &str) -> Result<(), cairo::Error> {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
        cr.set_font_size(12.0 * self.geometry.scale);
        if let Ok(ext) = cr.text_extents(text) {
            cr.move_to(
                self.geometry.center.x - ext.width() / 2.0,
                self.geometry.center.y + ext.height() / 2.0,
            );
            cr.show_text(text)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SlotState {
    Broken,
    Hovered,
    Running,
    Idle,
}

impl SlotState {
    fn resolve(slot: &Slot, hovered: bool, active_classes: &[WindowClass]) -> Self {
        if slot.is_broken() {
            Self::Broken
        } else if hovered {
            Self::Hovered
        } else if slot.is_running(active_classes) {
            Self::Running
        } else {
            Self::Idle
        }
    }

    fn color(&self, colors: &ThemeColors) -> Srgba<f64> {
        match self {
            Self::Broken => colors.broken,
            Self::Hovered => colors.hovered,
            Self::Running => colors.running,
            Self::Idle => colors.default,
        }
    }
}

pub fn draw(cr: &Context, state: &State, colors: &ThemeColors) -> Result<(), cairo::Error> {
    draw_center_circle(cr, state, colors)?;

    for (i, (slot, geometry)) in zip(&state.slots, &state.slot_geometries).enumerate() {
        if let Some(geometry) = geometry {
            SlotRenderer::new(
                slot,
                geometry,
                state.hover_index == Some(i),
                &state.active_classes,
            )
            .draw(cr, colors)?;
        }
    }
    Ok(())
}

fn draw_center_circle(
    cr: &Context,
    state: &State,
    colors: &ThemeColors,
) -> Result<(), cairo::Error> {
    let (r, g, b, a) = colors.center_circle.into_components();
    cr.set_source_rgba(r, g, b, a);
    cr.arc(
        state.center.x,
        state.center.y,
        CENTER_CIRCLE_RADIUS * state.scale_factor,
        0.0,
        2.0 * PI,
    );
    cr.fill()
}
