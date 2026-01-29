use crate::config;
use crate::events::AppEvent;
use crate::gui::menu::{self, State};
use crate::gui::theme::{self, ThemeColors};
use crate::gui::window;
use gtk::prelude::*;
use gtk4 as gtk;
use hypraise::wm::{self, Point, ShellCommand};
use relm4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct AppModel {
    pub state: Rc<RefCell<State>>,
    pub visible: bool,
    pub config_tx: async_channel::Sender<AppEvent>,
    pub root: gtk::ApplicationWindow,
    pub drawing_area: gtk::DrawingArea,
}

#[derive(Debug)]
pub enum AppMsg {
    Show,
    Hide,
    Click(u32),
    CursorMove(Point),
    ConfigReload,
}

impl From<AppEvent> for AppMsg {
    fn from(event: AppEvent) -> Self {
        match event {
            AppEvent::Show => AppMsg::Show,
            AppEvent::Hide => AppMsg::Hide,
            AppEvent::Click(b) => AppMsg::Click(b),
            AppEvent::CursorMove(p) => AppMsg::CursorMove(p),
            AppEvent::ConfigReload => AppMsg::ConfigReload,
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Init = (
        State,
        async_channel::Sender<AppEvent>,
        async_channel::Receiver<AppEvent>,
    );
    type Input = AppMsg;
    type Output = ();

    view! {
        #[root]
        #[name = "window"]
        gtk::ApplicationWindow {
            set_title: Some("Halo"),
            #[watch]
            set_visible: model.visible,
            #[watch]
            set_opacity: if model.visible { 1.0 } else { 0.0 },
            add_css_class: "halo-window",
            set_decorated: false,

            add_controller = gtk::EventControllerKey {
                connect_key_pressed[sender] => move |_, key, _, _| {
                    if key == gtk::gdk::Key::Escape {
                        sender.input(AppMsg::Hide);
                        return glib::Propagation::Stop;
                    }
                    glib::Propagation::Proceed
                }
            },

            #[name = "overlay"]
            gtk::Overlay {
                #[name = "drawing_area"]
                gtk::DrawingArea {
                    set_hexpand: true,
                    set_vexpand: true,
                    add_css_class: "halo-drawing-area",

                    add_controller = gtk::EventControllerMotion {
                        connect_motion[sender] => move |_, x, y| {
                            sender.input(AppMsg::CursorMove(Point::new(x, y)));
                        }
                    },

                    add_controller = gtk::GestureClick {
                        set_button: 0, // Listen to all buttons
                        connect_released[sender] => move |gesture, _, _, _| {
                            sender.input(AppMsg::Click(gesture.current_button()));
                        }
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (state, config_tx, rx) = init;

        theme::load_css();
        window::init_layer_shell(&root);

        let state = Rc::new(RefCell::new(state));

        let model = AppModel {
            state: state.clone(),
            visible: false,
            config_tx,
            root: root.clone(),
            drawing_area: gtk::DrawingArea::default(),
        };

        let widgets = view_output!();

        let mut model = model;
        model.drawing_area = widgets.drawing_area.clone();

        let state_draw = model.state.clone();
        widgets
            .drawing_area
            .set_draw_func(move |drawing_area, cr, _, _| {
                let style_context = drawing_area.style_context();
                let colors = ThemeColors::from_context(&style_context);
                if let Err(e) = menu::draw(cr, &state_draw.borrow(), &colors) {
                    log::error!("Drawing error: {}", e);
                }
            });

        let sender_clone = sender.clone();
        relm4::spawn(async move {
            while let Ok(event) = rx.recv().await {
                sender_clone.input(AppMsg::from(event));
            }
        });

        root.set_visible(false);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::Show => {
                let monitor_name = wm::get_active_monitor();
                let mut monitor_height = 1440.0;
                if let Some(name) = &monitor_name {
                    window::set_window_monitor(&self.root, name);
                    if let Some(m) = window::get_monitor_by_name(name) {
                        monitor_height = m.geometry().height() as f64;
                    }
                }

                self.visible = true;

                let cursor_pos = window::get_cursor_position(&self.root)
                    .or_else(wm::get_cursor_pos_on_active_monitor)
                    .unwrap_or_default();

                let classes = wm::get_active_classes();
                self.state
                    .borrow_mut()
                    .refresh(cursor_pos, classes, monitor_height);
                self.drawing_area.queue_draw();
            }
            AppMsg::Hide => {
                self.visible = false;
            }
            AppMsg::Click(btn) => {
                if !self.visible {
                    return;
                }
                if btn == 3 {
                    let state = self.state.borrow();

                    let _ = state
                        .hover_index
                        .and_then(|i| state.slots.get(i))
                        .filter(|s| s.is_running(&state.active_classes))
                        .and_then(|s| s.app.as_ref())
                        .map(|app| {
                            if let Err(e) = wm::close_window(&app.class) {
                                log::error!("Failed to close window: {}", e);
                            }
                        });
                }
                self.visible = false;
            }
            AppMsg::CursorMove(point) => {
                if !self.visible {
                    return;
                }
                let action = self.state.borrow_mut().update_cursor(point);
                if action.should_activate
                    && let Some(app_info) = self.state.borrow().get_hovered_app()
                {
                    if app_info.exec.as_str() == "HALO_SETUP" {
                        if let Ok(path) = config::write_default_config() {
                            let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
                        }
                    } else if let Err(e) = wm::run_or_raise(
                        &app_info.class,
                        &ShellCommand::from(app_info.exec.to_string()),
                    ) {
                        log::error!("Failed to run or raise '{}': {}", app_info.name, e);
                    }
                    self.visible = false;
                }
                if action.should_redraw {
                    self.drawing_area.queue_draw();
                }
            }
            AppMsg::ConfigReload => match config::load_config() {
                Ok(new_config) => {
                    let new_slots = State::init_slots(&new_config);
                    self.state.borrow_mut().slots = new_slots;
                    self.drawing_area.queue_draw();
                    log::info!("Configuration reloaded");
                }
                Err(e) => log::error!("Failed to reload config: {}", e),
            },
        }
    }
}
