use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn nova_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join(".nova")
}

pub fn config_path() -> PathBuf { nova_dir().join("config.yml") }

pub fn ensure_dir() {
    let _ = std::fs::create_dir_all(nova_dir());
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub location: String,
    pub lat: f64,
    pub lon: f64,
    pub tz: f64,
    #[serde(default = "default_cloud")]
    pub cloud_limit: i64,
    #[serde(default = "default_humidity")]
    pub humidity_limit: f64,
    #[serde(default = "default_temp")]
    pub temp_limit: f64,
    #[serde(default = "default_wind")]
    pub wind_limit: f64,
    #[serde(default = "default_true")]
    pub show_planets: bool,
    #[serde(default = "default_true")]
    pub show_events: bool,
}

fn default_cloud() -> i64 { 40 }
fn default_humidity() -> f64 { 80.0 }
fn default_temp() -> f64 { -10.0 }
fn default_wind() -> f64 { 8.0 }
fn default_true() -> bool { true }

impl Default for Config {
    fn default() -> Self {
        Self {
            location: "Oslo".into(),
            lat: 59.91,
            lon: 10.75,
            tz: 1.0,
            cloud_limit: default_cloud(),
            humidity_limit: default_humidity(),
            temp_limit: default_temp(),
            wind_limit: default_wind(),
            show_planets: true,
            show_events: true,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = config_path();
        if let Ok(data) = std::fs::read_to_string(&path) {
            serde_yaml::from_str(&data).unwrap_or_default()
        } else {
            let cfg = Config::default();
            cfg.save();
            cfg
        }
    }

    pub fn save(&self) {
        ensure_dir();
        if let Ok(yaml) = serde_yaml::to_string(self) {
            let _ = std::fs::write(config_path(), yaml);
        }
    }
}
