use halo::config;
use halo::gui::app::AppModel;
use halo::gui::menu::State;
use halo::sys::runtime;
use hypraise::wm::Point;
use relm4::prelude::*;

fn main() {
    env_logger::init();

    let config = config::load_or_setup();
    let slots = State::init_slots(&config);
    let state = State::new(slots, Point::default(), Vec::new(), 1.0);

    let (tx, rx) = async_channel::bounded(32);

    // Start Background Services
    runtime::start_background_services(tx.clone());

    let app = RelmApp::new("org.troia.halo");

    app.run::<AppModel>((state, tx.clone(), rx));
}
