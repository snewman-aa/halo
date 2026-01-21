use crate::sys::wm::Point;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Show,
    Hide,
    CursorMove(Point),
    ConfigReload,
}
