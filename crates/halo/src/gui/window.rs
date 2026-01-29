use gtk::gdk;
use gtk::prelude::*;
use gtk4 as gtk;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use hypraise::wm::{MonitorName, Point};

pub fn get_cursor_position(window: &gtk::ApplicationWindow) -> Option<Point> {
    gdk::Display::default()
        .and_then(|d| d.default_seat())
        .and_then(|s| s.pointer())
        .zip(window.surface())
        .and_then(|(p, s)| s.device_position(&p))
        .map(|(x, y, _)| Point::new(x, y))
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
