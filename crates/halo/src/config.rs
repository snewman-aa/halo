use directories::ProjectDirs;
use hypraise::desktop::{AppQuery, ExecCommand};
use hypraise::wm::WindowClass;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use serde_with::DeserializeFromStr;
use strum::{Display as StrumDisplay, EnumIter, EnumString, IntoEnumIterator};
use thiserror::Error;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    DeserializeFromStr,
    EnumString,
    EnumIter,
    StrumDisplay,
)]
#[strum(ascii_case_insensitive)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    #[strum(serialize = "North", serialize = "n", serialize = "0")]
    North,
    #[strum(serialize = "NorthEast", serialize = "ne", serialize = "1")]
    NorthEast,
    #[strum(serialize = "East", serialize = "e", serialize = "2")]
    East,
    #[strum(serialize = "SouthEast", serialize = "se", serialize = "3")]
    SouthEast,
    #[strum(serialize = "South", serialize = "s", serialize = "4")]
    South,
    #[strum(serialize = "SouthWest", serialize = "sw", serialize = "5")]
    SouthWest,
    #[strum(serialize = "West", serialize = "w", serialize = "6")]
    West,
    #[strum(serialize = "NorthWest", serialize = "nw", serialize = "7")]
    NorthWest,
}

impl Direction {
    pub fn as_index(&self) -> usize {
        *self as usize
    }

    pub fn from_index(idx: usize) -> Option<Self> {
        Self::iter().nth(idx % 8)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SlotConfig {
    pub direction: Option<Direction>,
    pub app: Option<AppQuery>,
    pub class: Option<WindowClass>,
    pub exec: Option<ExecCommand>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub slots: Vec<SlotConfig>,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to determine config directory")]
    ConfigDirNotFound,
    #[error("Config error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("Notify error: {0}")]
    Notify(#[from] notify::Error),
}

pub fn get_config_path() -> Result<std::path::PathBuf, ConfigError> {
    let proj_dirs =
        ProjectDirs::from("org", "troia", "halo").ok_or(ConfigError::ConfigDirNotFound)?;
    Ok(proj_dirs.config_dir().join("config.toml"))
}

pub fn load_config() -> Result<Config, ConfigError> {
    let config_path = get_config_path()?;

    let s = config::Config::builder()
        .add_source(config::File::from(config_path).required(false))
        .add_source(config::Environment::with_prefix("HALO"))
        .build()?;

    Ok(s.try_deserialize()?)
}

pub fn load_or_setup() -> Config {
    if let Ok(path) = get_config_path()
        && !path.exists()
    {
        return Config {
            slots: vec![SlotConfig {
                direction: Some(Direction::North),
                app: Some(AppQuery::from("Setup".to_string())),
                class: Some(WindowClass::from("halo-setup".to_string())),
                exec: Some(ExecCommand::from("HALO_SETUP".to_string())),
            }],
        };
    }

    match load_config() {
        Ok(c) => c,
        Err(_) => Config {
            slots: vec![SlotConfig {
                direction: Some(Direction::North),
                app: Some(AppQuery::from("Setup".to_string())),
                class: Some(WindowClass::from("halo-setup".to_string())),
                exec: Some(ExecCommand::from("HALO_SETUP".to_string())),
            }],
        },
    }
}

pub fn write_default_config() -> std::io::Result<std::path::PathBuf> {
    let path =
        get_config_path().map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))?;
    if let Some(parent) = path.parent() {
        fs_err::create_dir_all(parent)?;
    }
    if !path.exists() {
        fs_err::write(&path, DEFAULT_CONFIG)?;
    }
    Ok(path)
}

const DEFAULT_CONFIG: &str = include_str!("default_config.toml");

use crate::events::AppEvent;
use async_channel::Sender;

pub async fn run_async_watcher(tx: Sender<AppEvent>) {
    let config_path = match get_config_path() {
        Ok(p) => p,
        Err(e) => {
            log::error!("Config watcher error: {}", e);
            return;
        }
    };
    let config_dir = match config_path.parent() {
        Some(p) => p.to_path_buf(),
        None => return,
    };

    if let Err(e) = fs_err::create_dir_all(&config_dir) {
        log::error!("Failed to create config directory for watching: {}", e);
        return;
    }

    let (bridge_tx, bridge_rx) = async_channel::unbounded();

    let mut watcher = match RecommendedWatcher::new(
        move |res| {
            let _ = bridge_tx.send_blocking(res);
        },
        notify::Config::default(),
    ) {
        Ok(w) => w,
        Err(e) => {
            log::error!("Failed to create watcher: {}", e);
            return;
        }
    };

    if let Err(e) = watcher.watch(&config_dir, RecursiveMode::NonRecursive) {
        log::error!("Failed to watch config directory: {}", e);
        return;
    }

    while let Ok(res) = bridge_rx.recv().await {
        match res {
            Ok(event) => {
                let meaningful_event = matches!(
                    event.kind,
                    EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                );

                if meaningful_event
                    && event.paths.iter().any(|p| p == &config_path)
                    && tx.send(AppEvent::ConfigReload).await.is_err()
                {
                    break;
                }
            }
            Err(e) => log::error!("Watch error: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_deserialization() {
        let cases = vec![
            ("\"north\"", Direction::North),
            ("\"North\"", Direction::North),
            ("\"NORTH\"", Direction::North),
            ("\"n\"", Direction::North),
            ("\"N\"", Direction::North),
            ("\"0\"", Direction::North),
            ("\"nw\"", Direction::NorthWest),
            ("\"NorthWest\"", Direction::NorthWest),
        ];

        for (json, expected) in cases {
            let deserialized: Direction = serde_json::from_str(json).unwrap();
            assert_eq!(deserialized, expected);
        }
    }
}
