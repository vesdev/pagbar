use std::path::PathBuf;

use crate::bar::{self, *};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub title: Option<String>,
    pub monitor: Option<usize>,
    pub position: Option<Position>,
    pub size: Option<u16>,
    pub colors: ConfigColors,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigColors {
    pub background: Option<Color>,
    pub text: Option<Color>,
}

pub fn get_options(path: Option<PathBuf>) -> BarOptions {
    if let Some(config_path) = path {
        let config = toml::from_str::<Config>(
            &std::fs::read_to_string(config_path).expect("Config file not found!"),
        )
        .unwrap();
        bar::BarOptions {
            protocol: bar::Protocol::X11,
            title: config.title.unwrap_or("pagbar".into()),
            position: config.position.unwrap_or(bar::Position::Bottom),
            size: config.size.unwrap_or(50),
            bg_color: config
                .colors
                .background
                .unwrap_or(Color { r: 0, g: 0, b: 0 }),
            text_color: config.colors.text.unwrap_or(Color {
                r: 255,
                g: 255,
                b: 255,
            }),
        }
    } else {
        bar::BarOptions {
            protocol: bar::Protocol::X11,
            title: "pagbar".to_string(),
            position: bar::Position::Bottom,
            size: 100,
            bg_color: Color { r: 0, g: 0, b: 0 },
            text_color: Color {
                r: 255,
                g: 255,
                b: 255,
            },
        }
    }
}
