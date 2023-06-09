use bar::{Bar, Color};
use sysinfo::{DiskExt, SystemExt};

mod backend;
mod bar;

fn main() {
    let config = bar::Config {
        protocol: bar::Protocol::X11,
        title: "pagbar".to_string(),
        monitor: 0,
        position: bar::Position::Bottom,
        thickness: 100,
        bg_color: Color {
            r: 28,
            g: 30,
            b: 38,
        },
        text_color: Color {
            r: 228,
            g: 168,
            b: 138,
        },
    };
    bar::run(config, Box::<PagBar>::default())
}

pub struct PagBar {
    sys: sysinfo::System,
}

impl Default for PagBar {
    fn default() -> Self {
        Self {
            sys: sysinfo::System::new_all(),
        }
    }
}

impl Bar for PagBar {
    fn update(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("").min_width(300.).show(ctx, |ui| {
            ctx.set_pixels_per_point(1.5);
            ui.centered_and_justified(|ui| {
                self.sys.refresh_all();
                let date = chrono::Local::now();
                let date = date.format("%H:%M").to_string();
                let disk = &self.sys.disks()[0];
                let disk_use =
                    (1. - disk.available_space() as f64 / disk.total_space() as f64) * 100.;
                let memory_use =
                    self.sys.used_memory() as f64 / self.sys.total_memory() as f64 * 100.;
                ui.heading(format!("/ {disk_use:.0}% < ram {memory_use:.0}% < {date}"));
            });
        });
    }
}
