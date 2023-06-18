use std::{collections::HashMap, path::PathBuf};

use crate::bar::{self, *};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub title: Option<String>,
    pub colors: ConfigColors,
    pub bar: HashMap<String, ConfigBar>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigColors {
    pub background: Option<Color>,
    pub text: Option<Color>,
    pub text_secondary: Option<Color>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigBar {
    pub monitor: usize,
    pub position: Position,
    pub size: u16,
}

pub fn get_options(path: Option<PathBuf>) -> Vec<BarOption> {
    let mut result = Vec::new();
    if let Some(config_path) = path {
        let config = toml::from_str::<Config>(
            &std::fs::read_to_string(config_path).expect("Config file not found!"),
        )
        .unwrap();

        for (_, bar) in config.bar {
            result.push(bar::BarOption {
                monitor: bar.monitor,
                title: config.title.clone().unwrap_or("pagbar".into()),
                position: bar.position,
                size: bar.size,
                background: config
                    .colors
                    .background
                    .unwrap_or(Color { r: 0, g: 0, b: 0 }),
                text: config.colors.text.unwrap_or(Color {
                    r: 255,
                    g: 255,
                    b: 255,
                }),
                text_secondary: config.colors.text_secondary.unwrap_or(Color {
                    r: 150,
                    g: 150,
                    b: 150,
                }),
            });
        }
    } else {
        result.push(bar::BarOption::default());
    }

    result
}
