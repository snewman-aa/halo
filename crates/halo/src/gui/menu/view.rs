use super::model::{Slot, SlotGeometry, State};
use super::{CENTER_CIRCLE_RADIUS, ICON_INACTIVE_ALPHA, ICON_SIZE};
use crate::gui::theme::ThemeColors;
use cairo::Context;
use gdk_pixbuf::Pixbuf;
use gdk4::prelude::*;
use hypraise::wm::WindowClass;
use palette::Srgba;
use std::f64::consts::PI;
use std::iter::zip;

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
        // fit icon into slot
        let icon_scale = (self.geometry.radius * 2.0 * 0.75) / ICON_SIZE as f64;
        let (iw, ih) = (
            pixbuf.width() as f64 * icon_scale,
            pixbuf.height() as f64 * icon_scale,
        );
        // center icon in slot
        let (ix, iy) = (
            self.geometry.center.x - iw / 2.0,
            self.geometry.center.y - ih / 2.0,
        );

        cr.save()?;
        cr.translate(ix, iy);
        cr.scale(icon_scale, icon_scale);

        let running = self.slot.is_running(self.active_classes);

        // dim icon if app not running and not hovered
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
    /// Determines the visual state of a slot based on priority:
    /// 1. Broken (Config error)
    /// 2. Hovered
    /// 3. Running
    /// 4. Idle (Default)
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
