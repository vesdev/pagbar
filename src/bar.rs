use serde::{de::Visitor, Deserialize, Serialize};

use crate::backend::{self};

pub fn run(options: BarOptions, bar: Box<dyn Bar>) {
    match options.protocol {
        Protocol::X11 => backend::x11::run(options, bar),
    }
}

pub trait Bar {
    fn update(&mut self, ctx: &egui::Context);
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Serialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BarOptions {
    pub protocol: Protocol,
    pub title: String,
    pub monitor: usize,
    pub position: Position,
    pub size: u16,
    pub bg_color: Color,
    pub text_color: Color,
}

impl From<BarOptions> for egui::Visuals {
    fn from(value: BarOptions) -> Self {
        egui::Visuals {
            dark_mode: false,
            extreme_bg_color: egui::Color32::from_rgb(
                value.bg_color.r,
                value.bg_color.g,
                value.bg_color.b,
            ),
            faint_bg_color: egui::Color32::from_rgb(
                value.bg_color.r,
                value.bg_color.g,
                value.bg_color.b,
            ),
            window_fill: egui::Color32::from_rgb(
                value.bg_color.r,
                value.bg_color.g,
                value.bg_color.b,
            ),
            panel_fill: egui::Color32::from_rgb(
                value.bg_color.r,
                value.bg_color.g,
                value.bg_color.b,
            ),
            override_text_color: Some(egui::Color32::from_rgb(
                value.text_color.r,
                value.text_color.g,
                value.text_color.b,
            )),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Protocol {
    X11,
}
