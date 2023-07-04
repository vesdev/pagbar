use super::Bar;

pub trait Layout {
    fn display(&mut self, ctx: &egui::Context, bar: &Bar);
}

pub mod preset {

    use systemstat::Platform;

    use super::Layout;
    use crate::bar::Bar;
    use crate::Position;

    pub struct ThreeSplit {
        sys: systemstat::System,
    }

    impl Layout for ThreeSplit {
        fn display(&mut self, ctx: &egui::Context, bar: &Bar) {
            use egui::*;
            let visuals: Visuals = bar.into();
            ctx.set_visuals(visuals);
            // NOTE:
            // usually central panel would be added after
            // side panels, but since we want it to be centered
            // regardless of side panel size its added before
            egui::CentralPanel::default().show(ctx, |ui| self.center(bar, ctx, ui));

            if matches!(&bar.position, Position::Bottom | Position::Top) {
                SidePanel::left("first")
                    .resizable(false)
                    .min_width(0.)
                    .show_separator_line(false)
                    .show(ctx, |ui| self.first(bar, ctx, ui));

                SidePanel::right("last")
                    .resizable(false)
                    .min_width(0.)
                    .show_separator_line(false)
                    .show(ctx, |ui| self.last(bar, ctx, ui));
            } else {
                TopBottomPanel::top("first")
                    .resizable(false)
                    .min_height(0.)
                    .show_separator_line(false)
                    .show(ctx, |ui| self.first(bar, ctx, ui));

                TopBottomPanel::bottom("last")
                    .resizable(false)
                    .min_height(0.)
                    .show_separator_line(false)
                    .show(ctx, |ui| self.last(bar, ctx, ui));
            }
        }
    }

    impl Default for ThreeSplit {
        fn default() -> Self {
            Self {
                sys: systemstat::System::new(),
            }
        }
    }

    impl ThreeSplit {
        fn last(&mut self, options: &Bar, _ctx: &egui::Context, ui: &mut egui::Ui) {
            use egui::*;
            let stats = |ui: &mut Ui| {
                let memory = match self.sys.memory() {
                    Ok(mem) => (1. - mem.free.as_u64() as f64 / mem.total.as_u64() as f64) * 100.,
                    Err(_) => 0.,
                };

                let disk = match self.sys.mount_at("/") {
                    Ok(mount) => {
                        (1. - mount.free.as_u64() as f64 / mount.total.as_u64() as f64) * 100.
                    }
                    Err(_) => 0.,
                };

                let temp = self.sys.cpu_temp().unwrap_or(0.);

                ui.heading(RichText::new("/ ".to_string()).color(options.text));
                ui.heading(RichText::new(format!("{disk:.0}%")).color(options.text_secondary));
                ui.heading(RichText::new("ram ".to_string()).color(options.text));
                ui.heading(RichText::new(format!("{memory:.0}%")).color(options.text_secondary));
                ui.heading(RichText::new("cpu".to_string()).color(options.text));
                ui.heading(RichText::new(format!("{temp}°C")).color(options.text_secondary));
                ui.add_space(10.);
            };

            if matches!(options.position, Position::Bottom | Position::Top) {
                ui.horizontal_centered(stats);
            } else {
                ui.vertical_centered(stats);
            }
        }

        fn first(&mut self, _cfg: &Bar, _ctx: &egui::Context, _ui: &mut egui::Ui) {}

        fn center(&mut self, cfg: &Bar, ctx: &egui::Context, ui: &mut egui::Ui) {
            use egui::*;
            ui.centered_and_justified(|ui| {
                let date = chrono::Local::now();
                ctx.request_repaint_after(std::time::Duration::from_secs(1));

                let date = if let Position::Bottom | Position::Top = cfg.position {
                    date.format("%H:%M:%S").to_string()
                } else {
                    date.format("%H\n:%M:\n%S").to_string()
                };

                ui.heading(RichText::new(date).size(25.).color(cfg.text));
            });
        }
    }
}
