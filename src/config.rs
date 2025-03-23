use anyhow::{Context, Result};
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct CliSettings {
    pub preferred_editor: Option<String>,
    pub default_user: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Vm {
    pub name: String,
    pub ip: String,
    pub user: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub cli: Option<CliSettings>,
    pub vms: Vec<Vm>,
}

pub fn get_default_path() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
        })
        .join("pve-ssh/config.toml")
}

pub fn ensure_config_exists(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        let dir = path.parent().unwrap();
        fs::create_dir_all(dir)?;

        let username = whoami::username();

        fs::write(
            path,
            format!(
                r#"# pve-ssh config

[cli]
# preferred_editor = "nvim"
default_user = "{username}"

[[vms]]
name = "example"
ip = "192.168.1.100"
user = "{username}"
"#,
            ),
        )?;

        println!("✅ Created default config at {}", path.display());
    }
    Ok(())
}

pub fn load_config(path: &PathBuf) -> Result<AppConfig> {
    ensure_config_exists(&path)?;

    let contents = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config at {}", path.display()))?;

    let config: AppConfig = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse TOML from {}", path.display()))?;

    Ok(config)
}
