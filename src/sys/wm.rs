use derive_more::{AsRef, Deref, Display, From, Into};
use hyprland::data::{Clients, CursorPosition, Monitors};
use hyprland::dispatch::{Dispatch, DispatchType, WindowIdentifier};
use hyprland::error::HyprError;
use hyprland::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, Deref, From, Into, AsRef,
)]
#[serde(transparent)]
pub struct WindowClass(String);

crate::impl_string_newtype!(WindowClass);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, Deref, From, Into, AsRef)]
pub struct MonitorName(String);

crate::impl_string_newtype!(MonitorName);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, Deref, From, Into, AsRef)]
pub struct ShellCommand(String);

crate::impl_string_newtype!(ShellCommand);

#[derive(Debug, Error)]
pub enum RunOrRaiseError {
    #[error(transparent)]
    Hypr(#[from] HyprError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn get_active_classes() -> Vec<WindowClass> {
    Clients::get()
        .map(|clients| clients.into_iter().map(|c| WindowClass(c.class)).collect())
        .unwrap_or_default()
}

pub fn focus_window(class: &WindowClass) -> Result<(), HyprError> {
    Dispatch::call(DispatchType::FocusWindow(
        WindowIdentifier::ClassRegularExpression(&class.0),
    ))
}

pub fn close_window(class: &WindowClass) -> Result<(), HyprError> {
    Dispatch::call(DispatchType::CloseWindow(
        WindowIdentifier::ClassRegularExpression(&class.0),
    ))
}

pub fn get_active_monitor() -> Option<MonitorName> {
    Monitors::get()
        .ok()?
        .into_iter()
        .find(|m| m.focused)
        .map(|m| MonitorName(m.name))
}

pub fn get_cursor_pos_on_active_monitor() -> Option<Point> {
    let cursor = CursorPosition::get().ok()?;
    let monitors = Monitors::get().ok()?;
    let focused = monitors.into_iter().find(|m| m.focused)?;

    let x = cursor.x as f64 - focused.x as f64;
    let y = cursor.y as f64 - focused.y as f64;

    Some(Point::new(x, y))
}

pub fn run_or_raise(class: &WindowClass, exec: &ShellCommand) -> Result<(), RunOrRaiseError> {
    if Clients::get()?
        .iter()
        .any(|c| c.class.eq_ignore_ascii_case(&class.0))
    {
        focus_window(class)?;
    } else {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(&exec.0)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()?;
    }
    Ok(())
}
