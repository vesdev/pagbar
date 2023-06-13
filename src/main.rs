use std::time::Duration;
use systemstat::{saturating_sub_bytes, Platform, System};

use bar::{Bar, BarOptions};

use chrono::Timelike;
use clap::Parser;
use egui::{Color32, RichText, Ui};

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
    sys: systemstat::System,
}

impl Default for PagBar {
    fn default() -> Self {
        Self {
            sys: systemstat::System::new(),
        }
    }
}

impl Bar for PagBar {
    fn last(&mut self, options: &BarOptions, ctx: &egui::Context, ui: &mut Ui) {
        ui.horizontal_centered(|ui| {
            let memory = match self.sys.memory() {
                Ok(mem) => (1. - mem.free.as_u64() as f64 / mem.total.as_u64() as f64) * 100.,
                Err(_) => 0.,
            };

            let disk = match self.sys.mount_at("/") {
                Ok(mount) => (1. - mount.free.as_u64() as f64 / mount.total.as_u64() as f64) * 100.,
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
        });
    }

    fn first(&mut self, options: &BarOptions, ctx: &egui::Context, ui: &mut Ui) {}

    fn middle(&mut self, options: &BarOptions, ctx: &egui::Context, ui: &mut Ui) {
        let date = chrono::Local::now();
        ctx.request_repaint_after(Duration::from_secs(1));
        let date = date.format("%H:%M:%S").to_string();
        ui.centered_and_justified(|ui| {
            ui.heading(RichText::new(date).size(25.).color(options.text));
        });
    }
}
