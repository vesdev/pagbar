use std::ptr::read_unaligned;

use egui::{pos2, vec2};

use crate::bar::Bar;

use super::{backend::BackendData, widget::Widget, Config, Orientation};

#[derive(Default)]
pub struct Layout {
    splits: Vec<Split>,
}

impl Layout {
    pub fn split(&mut self, split: Split) {
        self.splits.push(split);
    }

    fn weighted_sum(&mut self) -> f32 {
        self.splits.iter().map(|s| s.weight).sum()
    }

    pub fn draw(&mut self, ctx: &egui::Context, cfg: &Config, backend: &BackendData) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let sum = self.weighted_sum();
            let mut pos = 0.;
            for split in self.splits.iter_mut() {
                let (start, end);

                if cfg.position.orientation() == Orientation::Vertical {
                    start = pos2(0., pos / sum * backend.height as f32);
                    end = pos2(
                        backend.width as f32,
                        (pos + split.weight) / sum * backend.height as f32,
                    );

                    ui.allocate_ui_at_rect(egui::Rect::from_two_pos(start, end), |ui| {
                        ui.vertical_centered(|ui| {
                            split.draw(ctx, ui, cfg);
                        });
                    });
                } else {
                    start = pos2(pos / sum * backend.width as f32, 0.);
                    end = pos2(
                        (pos + split.weight) / sum * backend.width as f32,
                        backend.height as f32,
                    );

                    ui.allocate_ui_at_rect(egui::Rect::from_two_pos(start, end), |ui| {
                        ui.horizontal_centered(|ui| {
                            split.draw(ctx, ui, cfg);
                        });
                    });
                }

                pos += split.weight;
            }
        });
    }
}

pub struct Split {
    weight: f32,
    widgets: Vec<Box<dyn Widget>>,
}

impl Split {
    pub fn new(weight: f32) -> Self {
        Self {
            weight,
            widgets: Vec::default(),
        }
    }

    pub fn widget(&mut self, widget: Box<dyn Widget>) {
        self.widgets.push(widget);
    }

    fn draw(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, cfg: &Config) {
        for widget in self.widgets.iter_mut() {
            widget.draw(ctx, ui, cfg);
        }
    }
}
