use egui::{style::WidgetVisuals, Color32};

use crate::backend::{self, *};

pub fn run(config: Config, bar: Box<dyn Bar>) {
    match config.protocol {
        Protocol::X11 => backend::x11::run(config, bar),
    }
}

pub trait Bar {
    fn update(&mut self, ctx: &egui::Context);
}

#[derive(Debug, Clone)]
pub enum Position {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub protocol: Protocol,
    pub title: String,
    pub monitor: usize,
    pub position: Position,
    pub thickness: u16,
    pub bg_color: Color,
    pub text_color: Color,
}

impl From<Config> for egui::Visuals {
    fn from(value: Config) -> Self {
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

#[derive(Debug, Clone)]
pub enum Protocol {
    X11,
}
