use crate::icon::{self, IconName};
use crate::wm::WindowClass;
use derive_more::{AsRef, Deref, Display, From, Into};
use freedesktop_entry_parser::parse_entry;
use fs_err as fs;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, Deref, From, Into, AsRef)]
pub struct AppName(String);

crate::impl_string_newtype!(AppName);

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, Deref, From, Into, AsRef,
)]
#[serde(transparent)]
pub struct ExecCommand(String);

crate::impl_string_newtype!(ExecCommand);

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, Deref, From, Into, AsRef,
)]
#[serde(transparent)]
pub struct AppQuery(String);

crate::impl_string_newtype!(AppQuery);

#[derive(Debug, Clone)]
pub struct AppInfo {
    pub name: AppName,
    pub icon: PathBuf,
    pub class: WindowClass,
    pub exec: ExecCommand,
}

impl AppInfo {
    pub fn new(query: &AppQuery, class: Option<WindowClass>, exec: Option<ExecCommand>) -> Self {
        let base = find_desktop_entry(query);

        Self {
            name: base
                .as_ref()
                .map(|b| b.name.clone())
                .unwrap_or_else(|| AppName::new(query.to_string())),
            icon: base.as_ref().map(|b| b.icon.clone()).unwrap_or_else(|| {
                icon::find_icon_path(&IconName::from(query.to_string())).unwrap_or_default()
            }),
            class: class
                .or_else(|| base.as_ref().map(|b| b.class.clone()))
                .unwrap_or_else(|| WindowClass::new(query.to_string())),
            exec: exec
                .or_else(|| base.as_ref().map(|b| b.exec.clone()))
                .unwrap_or_else(|| ExecCommand::new("".to_string())),
        }
    }
}

static ENTRIES: OnceLock<RwLock<Vec<AppInfo>>> = OnceLock::new();

pub fn refresh_cache() {
    let apps = scan_entries();
    let lock = ENTRIES.get_or_init(|| RwLock::new(Vec::new()));
    *lock.write() = apps;
}

fn get_all_entries() -> Vec<AppInfo> {
    let lock = ENTRIES.get_or_init(|| RwLock::new(scan_entries()));
    lock.read().clone()
}

fn get_desktop_directories() -> Vec<PathBuf> {
    let xdg = xdg::BaseDirectories::new();
    let mut dirs = Vec::new();

    if let Some(home) = xdg.get_data_home() {
        dirs.push(home.join("applications"));
    }

    dirs.extend(
        xdg.get_data_dirs()
            .into_iter()
            .map(|p| p.join("applications")),
    );
    dirs
}

fn collect_desktop_files() -> Vec<PathBuf> {
    let mut entries = HashMap::new();
    let dirs = get_desktop_directories();

    for dir in dirs.iter().rev() {
        if let Ok(read_dir) = fs::read_dir(dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("desktop")
                    && let Some(id) = path.file_name().and_then(|s| s.to_str())
                {
                    entries.insert(id.to_string(), path);
                }
            }
        }
    }
    entries.into_values().collect()
}

pub fn scan_entries() -> Vec<AppInfo> {
    collect_desktop_files()
        .into_iter()
        .filter_map(|path| parse_desktop_file(&path))
        .collect()
}

pub fn parse_desktop_file(path: &Path) -> Option<AppInfo> {
    let entry = parse_entry(path).ok()?;
    let section = entry.section("Desktop Entry")?;

    let entry_type = section.attr("Type").first()?;
    if entry_type != "Application" {
        return None;
    }

    if let Some(no_display) = section.attr("NoDisplay").first()
        && no_display == "true"
    {
        return None;
    }

    let name = section.attr("Name").first()?.to_string();

    let icon_str = section.attr("Icon").first();
    let icon_path = if let Some(icon) = icon_str {
        icon::find_icon_path(&IconName::from(icon.to_string()))
            .unwrap_or_else(|| PathBuf::from(icon))
    } else {
        PathBuf::new()
    };

    let exec_raw = section.attr("Exec").first()?;
    let exec = strip_field_codes(exec_raw);

    let id = path.file_name()?.to_str()?;
    let class = section
        .attr("StartupWMClass")
        .first()
        .cloned()
        .unwrap_or_else(|| id.trim_end_matches(".desktop").to_string());

    Some(AppInfo {
        name: AppName::new(name),
        icon: icon_path,
        class: WindowClass::new(class),
        exec: ExecCommand::new(exec),
    })
}

fn strip_field_codes(exec: &str) -> String {
    shell_words::split(exec)
        .map(|args| {
            let clean_args: Vec<_> = args
                .into_iter()
                .filter(|arg| !arg.starts_with('%'))
                .collect();
            shell_words::join(clean_args)
        })
        .unwrap_or_else(|_| exec.to_string())
}

pub fn find_desktop_entry(query: &AppQuery) -> Option<AppInfo> {
    find_desktop_entry_in_list(query, &get_all_entries())
}

pub fn find_desktop_entry_in_list(query: &AppQuery, entries: &[AppInfo]) -> Option<AppInfo> {
    let lower_query = query.to_lowercase();
    entries
        .iter()
        .find(|app| {
            app.name.to_lowercase() == lower_query || app.class.to_lowercase() == lower_query
        })
        .cloned()
}

pub fn resolve_apps(queries: &[AppQuery]) -> Vec<Option<AppInfo>> {
    queries.iter().map(find_desktop_entry).collect()
}
