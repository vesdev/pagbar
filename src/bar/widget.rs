use super::{Bar, Config};
use crate::bar::Position;

pub trait Widget {
    fn draw(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, cfg: &Config);
}

#[derive(Default)]
pub struct Clock;

impl Widget for Clock {
    fn draw(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, cfg: &Config) {
        let date = chrono::Local::now();
        ctx.request_repaint_after(std::time::Duration::from_secs(1));

        let date = if let Position::Bottom | Position::Top = cfg.position {
            date.format("%H:%M:%S").to_string()
        } else {
            date.format("%H\n:%M:\n%S").to_string()
        };

        ui.heading(egui::RichText::new(date).size(25.).color(cfg.text));
    }
}
