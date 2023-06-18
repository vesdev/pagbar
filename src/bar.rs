use std::sync::Arc;

use egui::{Color32, Context, Ui};
use serde::{de::Visitor, Deserialize, Serialize};
mod backend;

pub fn run(protocol: Protocol, options: Vec<BarOption>, bar_factory: fn() -> Box<dyn Bar>) {
    match protocol {
        Protocol::X11 => backend::x11::run(options, bar_factory),
    }
}

pub trait Bar {
    fn first(&mut self, options: &BarOption, ctx: &egui::Context, ui: &mut Ui);
    fn middle(&mut self, options: &BarOption, ctx: &egui::Context, ui: &mut Ui);
    fn last(&mut self, options: &BarOption, ctx: &egui::Context, ui: &mut Ui);
}

fn display_bar(bar: &mut Box<dyn Bar>, ctx: &Context, options: &BarOption) {
    let visuals: egui::Visuals = options.clone().into();
    ctx.set_visuals(visuals);

    // NOTE:
    // usually central panel would be added after
    // side panels, but since we want it to be centered
    // regardless of side panel size its added before
    egui::CentralPanel::default().show(ctx, |ui| bar.middle(options, ctx, ui));

    if matches!(&options.position, Position::Bottom | Position::Top) {
        egui::SidePanel::left("first")
            .resizable(false)
            .min_width(0.)
            .show_separator_line(false)
            .show(ctx, |ui| bar.first(options, ctx, ui));

        egui::SidePanel::right("last")
            .resizable(false)
            .min_width(0.)
            .show_separator_line(false)
            .show(ctx, |ui| bar.last(options, ctx, ui));
    } else {
        egui::TopBottomPanel::top("first")
            .resizable(false)
            .min_height(0.)
            .show_separator_line(false)
            .show(ctx, |ui| bar.first(options, ctx, ui));

        egui::TopBottomPanel::bottom("last")
            .resizable(false)
            .min_height(0.)
            .show_separator_line(false)
            .show(ctx, |ui| bar.last(options, ctx, ui));
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BarOption {
    pub monitor: usize,
    pub title: String,
    pub position: Position,
    pub size: u16,
    pub background: Color,
    pub text: Color,
    pub text_secondary: Color,
}

impl Default for BarOption {
    fn default() -> Self {
        Self {
            monitor: 0,
            title: "pagbar".to_string(),
            position: Position::Bottom,
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

impl From<BarOption> for egui::Visuals {
    fn from(value: BarOption) -> Self {
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
