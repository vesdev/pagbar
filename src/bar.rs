use crate::backend::{self, *};

pub fn run(config: Config) {
    match config.protocol {
        Protocol::X11 => backend::x11::run(config),
    }
}

#[derive(Debug, Clone)]
pub enum Position {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub protocol: Protocol,
    pub title: String,
    pub monitor: usize,
    pub position: Position,
    pub thickness: u16,
}

#[derive(Debug, Clone)]
pub enum Protocol {
    X11,
}
