use super::model::{Slot, SlotGeometry, State, SubSlot};
use super::{CENTER_CIRCLE_RADIUS, ICON_INACTIVE_ALPHA, ICON_SIZE};
use crate::gui::theme::ThemeColors;
use cairo::Context;
use gdk_pixbuf::Pixbuf;
use gdk4::prelude::*;
use hypraise::wm::WindowClass;
use palette::Srgba;
use std::f64::consts::PI;
use std::iter::zip;

fn draw_slot_circle(
    cr: &Context,
    center: hypraise::wm::Point,
    radius: f64,
    color: Srgba<f64>,
) -> Result<(), cairo::Error> {
    let (r, g, b, a) = color.into_components();
    cr.set_source_rgba(r, g, b, a);
    cr.arc(center.x, center.y, radius, 0.0, 2.0 * PI);
    cr.fill()
}

fn draw_slot_icon(
    cr: &Context,
    pixbuf: &Pixbuf,
    center: hypraise::wm::Point,
    slot_radius: f64,
    dimmed: bool,
) -> Result<(), cairo::Error> {
    // fit icon into slot
    let icon_scale = (slot_radius * 2.0 * 0.75) / ICON_SIZE as f64;
    let (iw, ih) = (
        pixbuf.width() as f64 * icon_scale,
        pixbuf.height() as f64 * icon_scale,
    );
    // center icon in slot
    let (ix, iy) = (center.x - iw / 2.0, center.y - ih / 2.0);

    cr.save()?;
    cr.translate(ix, iy);
    cr.scale(icon_scale, icon_scale);

    // dim icon if app not running and not hovered
    if dimmed {
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
        draw_slot_circle(
            cr,
            self.geometry.center,
            self.geometry.radius,
            state.color(colors),
        )
    }

    fn draw_content(&self, cr: &Context) -> Result<(), cairo::Error> {
        if let Some(pixbuf) = &self.slot.pixbuf {
            let running = self.slot.is_running(self.active_classes);
            draw_slot_icon(
                cr,
                pixbuf,
                self.geometry.center,
                self.geometry.radius,
                !running && !self.hovered,
            )
        } else if let Some(app) = &self.slot.app {
            self.draw_text(cr, &app.name)
        } else {
            Ok(())
        }
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

struct SubSlotRenderer<'a> {
    subslot: &'a SubSlot,
}

impl<'a> SubSlotRenderer<'a> {
    fn new(subslot: &'a SubSlot) -> Self {
        Self { subslot }
    }

    fn draw(&self, cr: &Context, colors: &ThemeColors) -> Result<(), cairo::Error> {
        draw_slot_circle(
            cr,
            self.subslot.geometry.center,
            self.subslot.geometry.radius,
            colors.running,
        )?;

        self.draw_content(cr)?;
        self.draw_badge(cr)?;
        Ok(())
    }

    fn draw_content(&self, cr: &Context) -> Result<(), cairo::Error> {
        if let Some(pixbuf) = &self.subslot.pixbuf {
            draw_slot_icon(
                cr,
                pixbuf,
                self.subslot.geometry.center,
                self.subslot.geometry.radius,
                false,
            )
        } else {
            self.draw_text(cr, &self.subslot.client.class)
        }
    }

    fn draw_text(&self, cr: &Context, text: &str) -> Result<(), cairo::Error> {
        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
        cr.set_font_size(10.0 * self.subslot.geometry.scale);
        if let Ok(ext) = cr.text_extents(text) {
            cr.move_to(
                self.subslot.geometry.center.x - ext.width() / 2.0,
                self.subslot.geometry.center.y + ext.height() / 2.0,
            );
            cr.show_text(text)?;
        }
        Ok(())
    }

    fn draw_badge(&self, cr: &Context) -> Result<(), cairo::Error> {
        let text = self.subslot.key.to_string().to_uppercase();
        let center = self.subslot.geometry.center;
        let scale = self.subslot.geometry.scale * 3.0;

        cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
        cr.set_font_size(12.0 * scale);

        if let Ok(ext) = cr.text_extents(&text) {
            let padding = 4.0 * scale;
            let width = ext.width() + padding * 2.0;
            let height = ext.height() + padding * 2.0;

            // position badge at the bottom center of the slot
            let bx = center.x - width / 2.0;
            let by = center.y + self.subslot.geometry.radius - height;

            // "Keycap" background
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.6);
            draw_rounded_rect(cr, bx, by, width, height, 4.0 * scale)?;
            cr.fill()?;

            // text
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.move_to(bx + padding, by + height - padding);
            cr.show_text(&text)?;
        }
        Ok(())
    }
}

fn draw_rounded_rect(
    cr: &Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
) -> Result<(), cairo::Error> {
    cr.new_sub_path();
    cr.arc(x + radius, y + radius, radius, PI, 1.5 * PI);
    cr.arc(x + width - radius, y + radius, radius, 1.5 * PI, 2.0 * PI);
    cr.arc(
        x + width - radius,
        y + height - radius,
        radius,
        0.0,
        0.5 * PI,
    );
    cr.arc(x + radius, y + height - radius, radius, 0.5 * PI, PI);
    cr.close_path();
    Ok(())
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

    for subslot in &state.subslots {
        SubSlotRenderer::new(subslot).draw(cr, colors)?;
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

