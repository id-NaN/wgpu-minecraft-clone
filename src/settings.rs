use std::io::ErrorKind;

use color_eyre::Result;
use log::info;
use once_cell::sync::Lazy;
use serde_derive::{Deserialize, Serialize};

const CONFIG_PATH: &str = "./settings.toml";

#[derive(Debug, Serialize, Deserialize)]
pub enum WindowMode {
    Default { width: u32, height: u32 },
    Maximized,
    Fullscreen,
}

impl Default for WindowMode {
    fn default() -> Self {
        Self::Default {
            width: 800,
            height: 600,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "log::LevelFilter")]
enum LevelFilterDef {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct LogSettings {
    #[serde(with = "LevelFilterDef")]
    pub max_level: log::LevelFilter,
}

impl Default for LogSettings {
    fn default() -> Self {
        Self {
            max_level: log::LevelFilter::Info,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct GraphicsSettings {
    pub fov: f32,
    pub render_distance: u32,
    pub window: WindowMode,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            fov: 70.0,
            render_distance: 128,
            window: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Settings {
    pub log: LogSettings,
    pub graphics: GraphicsSettings,
}

impl Settings {
    pub fn load_or_create() -> Self {
        info!("Loading Settings...");
        let settings = match std::fs::read_to_string(CONFIG_PATH) {
            Ok(content) => {
                info!("Found Settings!");
                toml::from_str(&content).unwrap()
            }
            Err(error) => {
                if error.kind() == ErrorKind::NotFound {
                    info!("No settings file yet, creating new!");
                    Settings::default()
                } else {
                    panic!("Unable to read the settings file: {error:?}")
                }
            }
        };

        settings.save().expect("Error saving new Settings");
        settings
    }

    fn save(&self) -> Result<()> {
        let new_content = toml::to_string(self)?;
        std::fs::write(CONFIG_PATH, new_content)?;
        Ok(())
    }
}

pub static SETTINGS: Lazy<Settings> = Lazy::new(Settings::load_or_create);
