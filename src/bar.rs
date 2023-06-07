use crate::backend::*;
use crate::error::*;

pub fn run(mut proto: Box<dyn Protocol>, monitor: usize, config: Config) -> Result<()> {
    let monitors = proto.get_monitors().unwrap();
    let monitor = &monitors[monitor];
    proto.create_window(monitor, "pagbar".to_string(), config)?;

    Ok(())
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
    pub position: Position,
    pub thickness: u16,
}

pub struct Monitor {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}
