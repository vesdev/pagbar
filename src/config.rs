use serde::{Deserialize, Serialize};

use crate::bar::{Color, Position};

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
