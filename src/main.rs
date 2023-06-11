use std::time::Duration;

use bar::Bar;

use chrono::Timelike;
use clap::Parser;
use egui::Ui;
use sysinfo::{DiskExt, SystemExt};

mod bar;
mod config;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<std::path::PathBuf>,
}

fn main() {
    let args = Cli::parse();

    if args.config.is_none() {
        //LOOK FOR CONFIG IN XDG_CONFIG_HOME
        let base_dirs = xdg::BaseDirectories::new().unwrap();
        let options =
            config::get_options(Some(base_dirs.get_config_home().join("pagbar/config.toml")));
        bar::run(options, Box::<PagBar>::default())
    } else {
        let options = config::get_options(args.config);
        bar::run(options, Box::<PagBar>::default())
    }
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
    fn last(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        ui.centered_and_justified(|ui| {
            let disk = &self.sys.disks()[0];
            let disk_use = (1. - disk.available_space() as f64 / disk.total_space() as f64) * 100.;
            let memory_use = self.sys.used_memory() as f64 / self.sys.total_memory() as f64 * 100.;

            ui.heading(format!("/ disk {disk_use:.0}% / ram {memory_use:.0}%"));
        });
    }

    fn first(&mut self, ctx: &egui::Context, ui: &mut Ui) {}

    fn middle(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        ctx.set_pixels_per_point(1.2);
        self.sys.refresh_all();
        let date = chrono::Local::now();
        ctx.request_repaint_after(Duration::from_secs(60 - date.second() as u64));
        let date = date.format("%H:%M").to_string();
        ui.centered_and_justified(|ui| {
            ui.heading(date);
        });
    }
}
