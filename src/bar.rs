use std::{collections::HashMap, path::PathBuf};

use egui::Color32;
use indexmap::IndexMap;
use serde::{de::Visitor, Deserialize, Serialize};

use self::{
    backend::BackendData,
    layout::{Layout, Split},
    widget::{Widget, WidgetSet},
};

mod backend;
pub mod layout;
mod user_config;
pub mod widget;

pub fn run(protocol: Protocol, bars: Vec<Bar>) {
    match protocol {
        Protocol::X11 => backend::x11::run(bars),
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl Position {
    pub fn orientation(&self) -> Orientation {
        if matches!(self, Position::Left | Position::Right) {
            Orientation::Vertical
        } else {
            Orientation::Horizontal
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<Color> for Color32 {
    fn from(value: Color) -> Self {
        Color32::from_rgb(value.r, value.g, value.b)
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b))
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(ColorVisitor)
    }
}

struct ColorVisitor;

impl<'de> Visitor<'de> for ColorVisitor {
    type Value = Color;

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.starts_with('#') && v.len() >= 7 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&v[1..3], 16),
                u8::from_str_radix(&v[3..5], 16),
                u8::from_str_radix(&v[5..7], 16),
            ) {
                return Ok(Color { r, g, b });
            }
        }

        Err(E::custom(format!("invalid hex {}", v)))
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("rgb hex string")
    }
}

pub struct Bar {
    cfg: Config,
    layout: Layout,
}

pub struct Config {
    pub monitor: usize,
    pub title: String,
    pub position: Position,
    pub size: u16,
    pub background: Color,
    pub text: Color,
    pub text_secondary: Color,
}

pub fn from_path(path: PathBuf, widget_set: &mut WidgetSet) -> Vec<Bar> {
    let mut result = Vec::new();
    let config = toml::from_str::<user_config::UserConfig>(
        &std::fs::read_to_string(path.clone())
            .expect(format!("Config file not found! {:?}", path).as_str()),
    )
    .unwrap();

    let mut layout_factory =
        |layout: IndexMap<String, f32>, panels: &HashMap<String, Vec<String>>| -> Layout {
            let mut result = Layout::default();
            for (panel, weight) in layout {
                let mut split = Split::new(weight);

                if let Some(panel) = panels.get(&panel) {
                    for widget in panel {
                        if let Some(widget) = widget_set.create_widget(widget) {
                            split.widget(widget);
                        }
                    }
                }

                result.split(split);
            }
            result
        };

    for (_, bar) in config.bar {
        result.push(Bar {
            cfg: Config {
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
            },
            layout: layout_factory(bar.layout, &config.panels),
        })
    }
    result
}
impl From<&Config> for egui::Visuals {
    fn from(value: &Config) -> Self {
        egui::Visuals {
            dark_mode: false,
            extreme_bg_color: egui::Color32::from_rgb(
                value.background.r,
                value.background.g,
                value.background.b,
            ),
            faint_bg_color: egui::Color32::from_rgb(
                value.background.r,
                value.background.g,
                value.background.b,
            ),
            window_fill: egui::Color32::from_rgb(
                value.background.r,
                value.background.g,
                value.background.b,
            ),
            panel_fill: egui::Color32::from_rgb(
                value.background.r,
                value.background.g,
                value.background.b,
            ),
            override_text_color: Some(egui::Color32::from_rgb(
                value.background.r,
                value.background.g,
                value.background.b,
            )),

            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Protocol {
    X11,
}
