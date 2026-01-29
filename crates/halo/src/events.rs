use hypraise::wm::Point;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Show,
    Hide,
    Click(u32),
    CursorMove(Point),
    ConfigReload,
}
