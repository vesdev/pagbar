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
    pub text_secondary: Option<Color>,
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
        }
    } else {
        bar::BarOptions {
            protocol: bar::Protocol::X11,
            title: "pagbar".to_string(),
            position: bar::Position::Bottom,
            size: 100,
            background: Color { r: 0, g: 0, b: 0 },
            text: Color {
                r: 255,
                g: 255,
                b: 255,
            },
            text_secondary: Color {
                r: 150,
                g: 150,
                b: 150,
            },
        }
    }
}
