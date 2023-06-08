use crate::{
    bar::{Config, Position},
    error::*,
};

mod glow_backend;
use egui_winit::winit::{self};
pub trait Protocol {
    fn run(&self, monitor: usize, title: String, config: Config) -> Result<()>;
}

pub struct Xcb {}

impl Xcb {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

impl Protocol for Xcb {
    fn run(&self, monitor: usize, title: String, config: Config) -> Result<()> {
        let event_loop = winit::event_loop::EventLoopBuilder::with_user_event().build();
        let mut monitors = event_loop.available_monitors();
        let monitor = &monitors.nth(monitor).unwrap();
        let (x, y, width, height) = match config.position {
            Position::Left => (
                monitor.position().x,
                monitor.position().y,
                config.thickness as u32,
                monitor.size().height,
            ),
            Position::Right => (
                monitor.position().x + monitor.size().width as i32 - config.thickness as i32,
                monitor.position().y,
                config.thickness as u32,
                monitor.size().height,
            ),
            Position::Top => (
                monitor.position().x,
                monitor.position().y,
                monitor.size().width,
                config.thickness as u32,
            ),
            Position::Bottom => (
                monitor.position().x,
                monitor.position().y + monitor.size().height as i32 - config.thickness as i32,
                monitor.size().width,
                config.thickness as u32,
            ),
        };

        let (window, context) = glow_backend::create_display(&event_loop, x, y, width, height);

        glow_backend::events(window, context, event_loop);

        Ok(())
    }
}
