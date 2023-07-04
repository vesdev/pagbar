use std::collections::HashMap;

use super::*;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserConfig {
    pub title: Option<String>,
    pub colors: UserConfigColors,
    pub bar: HashMap<String, UserConfigBar>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserConfigColors {
    pub background: Option<Color>,
    pub text: Option<Color>,
    pub text_secondary: Option<Color>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserConfigBar {
    pub monitor: usize,
    pub position: Position,
    pub size: u16,
}
