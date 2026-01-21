use crate::sys::wm::{MonitorName, Point};
use gtk::gdk;
use gtk::prelude::*;
use gtk4 as gtk;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use palette::Srgba;

pub fn get_cursor_position(window: &gtk::ApplicationWindow) -> Option<Point> {
    gdk::Display::default()
        .and_then(|d| d.default_seat())
        .and_then(|s| s.pointer())
        .zip(window.surface())
        .and_then(|(p, s)| s.device_position(&p))
        .map(|(x, y, _)| Point::new(x, y))
}

pub struct ThemeColors {
    pub hovered: Srgba<f64>,
    pub running: Srgba<f64>,
    pub default: Srgba<f64>,
    pub center_circle: Srgba<f64>,
    pub broken: Srgba<f64>,
}

impl ThemeColors {
    pub fn from_context(context: &gtk::StyleContext) -> Self {
        Self {
            hovered: Self::lookup_color(
                context,
                "theme_selected_bg_color",
                Srgba::new(0.4, 0.4, 0.8, 0.9),
                Some(0.9),
            ),
            running: Self::lookup_color(
                context,
                "theme_fg_color",
                Srgba::new(0.25, 0.25, 0.25, 0.85),
                Some(0.3),
            ),
            broken: Self::lookup_color(
                context,
                "error_bg_color",
                Srgba::new(0.8, 0.2, 0.2, 0.5),
                Some(0.5),
            ),
            default: Self::lookup_color(
                context,
                "theme_bg_color",
                Srgba::new(0.15, 0.15, 0.15, 0.5),
                Some(0.5),
            ),
            center_circle: Self::lookup_color(
                context,
                "theme_fg_color",
                Srgba::new(0.2, 0.2, 0.2, 0.15),
                Some(0.1),
            ),
        }
    }

    fn lookup_color(
        context: &gtk::StyleContext,
        name: &str,
        fallback: Srgba<f64>,
        alpha_override: Option<f64>,
    ) -> Srgba<f64> {
        context
            .lookup_color(name)
            .map(|c| {
                let (r, g, b, a) = (
                    c.red() as f64,
                    c.green() as f64,
                    c.blue() as f64,
                    c.alpha() as f64,
                );
                Srgba::new(r, g, b, alpha_override.unwrap_or(a))
            })
            .unwrap_or(fallback)
    }
}

pub fn init_layer_shell(window: &gtk::ApplicationWindow) {
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_namespace(Some("halo"));
    window.set_exclusive_zone(-1);
    for edge in [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom] {
        window.set_anchor(edge, true);
    }
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
}

pub fn load_css() {
    let provider = gtk::CssProvider::new();
    let css_data = "
.halo-window, .halo-drawing-area {
    background: none;
    background-color: transparent;
}
";
    provider.load_from_data(css_data);

    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
pub fn get_monitor_by_name(name: &MonitorName) -> Option<gdk::Monitor> {
    let display = gdk::Display::default()?;
    let monitors = display.monitors();
    (0..monitors.n_items()).find_map(|i| {
        monitors
            .item(i)
            .and_then(|item| item.downcast::<gdk::Monitor>().ok())
            .filter(|m| m.connector().is_some_and(|n| n.as_str() == **name))
    })
}

pub fn set_window_monitor(window: &gtk::ApplicationWindow, monitor_name: &MonitorName) {
    if let Some(monitor) = get_monitor_by_name(monitor_name) {
        window.set_monitor(Some(&monitor));
    }
}
