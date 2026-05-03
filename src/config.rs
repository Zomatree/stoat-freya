use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use etcetera::{AppStrategy, AppStrategyArgs, app_strategy::choose_native_strategy};
use freya::prelude::{State, use_consume};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Config {
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub last_channels: HashMap<String, String>,
    #[serde(default)]
    pub collapsed_categories: HashSet<String>,
    #[serde(default)]
    pub hide_channel_list: bool,
    #[serde(default)]
    pub hide_members_list: bool,
}

pub fn get_config_path() -> PathBuf {
    let strategy = choose_native_strategy(AppStrategyArgs {
        top_level_domain: "chat".to_string(),
        author: "stoat".to_string(),
        app_name: "Stoat Chat".to_string(),
    })
    .unwrap();

    let mut dir = strategy.config_dir();
    let _ = std::fs::create_dir_all(&dir);

    dir.push("config.json");

    println!("{dir:?}");

    dir
}

pub fn read_config() -> Config {
    let path = get_config_path();

    let Ok(value) = std::fs::read_to_string(path) else {
        return Config::default();
    };

    serde_json::from_str(&value).unwrap_or_default()
}

pub fn write_config(config: &Config) {
    println!("writing {config:?}");
    let path = get_config_path();

    std::fs::write(path, serde_json::to_string(config).unwrap()).unwrap();
}

pub fn use_config() -> State<Config> {
    use_consume()
}
