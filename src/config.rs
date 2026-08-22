use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    net::IpAddr,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub interface: Option<String>,
    pub refresh_seconds: u64,
    pub show_network: bool,
    pub show_mijia: bool,
    #[serde(deserialize_with = "deserialize_mijia_devices")]
    pub mijia: Vec<MiJiaConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiJiaConfig {
    #[serde(default)]
    pub id: String,
    pub ip: IpAddr,
    pub token: String,
    #[serde(default = "default_device_name")]
    pub name: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_power_siid")]
    pub power_siid: u32,
    #[serde(default = "default_power_piid")]
    pub power_piid: u32,
}

fn default_model() -> String {
    "local.mijia.device".into()
}
fn default_device_name() -> String {
    "米家插座".into()
}
fn default_power_siid() -> u32 {
    11
}
fn default_power_piid() -> u32 {
    2
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            interface: None,
            refresh_seconds: 2,
            show_network: true,
            show_mijia: true,
            mijia: Vec::new(),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum MiJiaDevices {
    One(MiJiaConfig),
    Many(Vec<MiJiaConfig>),
}

fn deserialize_mijia_devices<'de, D>(deserializer: D) -> Result<Vec<MiJiaConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(match Option::<MiJiaDevices>::deserialize(deserializer)? {
        None => Vec::new(),
        Some(MiJiaDevices::One(device)) => vec![device],
        Some(MiJiaDevices::Many(devices)) => devices,
    })
}

impl MiJiaConfig {
    pub fn stable_id(&self) -> String {
        if self.id.trim().is_empty() {
            self.ip.to_string()
        } else {
            self.id.clone()
        }
    }
}

pub fn path() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cosmic-applet-monitor/config.json")
}

pub fn load() -> (AppConfig, Option<String>) {
    let path = path();
    match fs::read_to_string(path) {
        Ok(contents) => match serde_json::from_str(&contents) {
            Ok(config) => (config, None),
            Err(err) => (AppConfig::default(), Some(format!("invalid config: {err}"))),
        },
        Err(err) if err.kind() == io::ErrorKind::NotFound => (AppConfig::default(), None),
        Err(err) => (
            AppConfig::default(),
            Some(format!("cannot read config: {err}")),
        ),
    }
}

pub fn save(config: &AppConfig) -> Result<(), String> {
    save_to(&path(), config).map_err(|err| format!("cannot save config: {err}"))
}

fn save_to(path: &Path, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "config path has no parent"))?;
    fs::create_dir_all(parent)?;
    fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;

    let temporary = parent.join(format!(".config.json.tmp.{}", std::process::id()));
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)?;
        serde_json::to_writer_pretty(&mut file, config)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        fs::rename(&temporary, path)?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_are_safe_without_credentials() {
        let config = AppConfig::default();
        assert!(config.mijia.is_empty());
        assert_eq!(config.refresh_seconds, 2);
    }

    #[test]
    fn accepts_legacy_single_device_config() {
        let config: AppConfig =
            serde_json::from_str(r#"{"mijia":{"ip":"192.168.1.2","token":"00","name":"插座"}}"#)
                .unwrap();
        assert_eq!(config.mijia.len(), 1);
        assert_eq!(config.mijia[0].stable_id(), "192.168.1.2");
    }
}
