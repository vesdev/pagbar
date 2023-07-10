use super::*;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserConfig {
    pub title: Option<String>,
    pub colors: UserConfigColors,
    pub bar: HashMap<String, UserConfigBar>,
    pub panels: HashMap<String, Vec<String>>,
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
    pub layout: IndexMap<String, f32>,
}
