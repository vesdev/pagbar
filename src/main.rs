use std::time::Duration;
use systemstat::Platform;

use bar::{Bar, BarConfig, Position};

use clap::Parser;
use egui::{RichText, Ui};

mod bar;
mod config;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<std::path::PathBuf>,
}

fn main() {
    env_logger::init();
    let args = Cli::parse();

    if args.config.is_none() {
        //LOOK FOR CONFIG IN XDG_CONFIG_HOME
        let base_dirs = xdg::BaseDirectories::new().unwrap();
        let options =
            config::get_options(Some(base_dirs.get_config_home().join("pagbar/config.toml")));
        bar::run(bar::Protocol::X11, options, || Box::<PagBar>::default())
    } else {
        let options = config::get_options(args.config);
        bar::run(bar::Protocol::X11, options, || Box::<PagBar>::default());
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
    fn last(&mut self, options: &BarConfig, _ctx: &egui::Context, ui: &mut Ui) {
        let stats = |ui: &mut Ui| {
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
        };

        if matches!(options.position, Position::Bottom | Position::Top) {
            ui.horizontal_centered(stats);
        } else {
            ui.vertical_centered(stats);
        }
    }

    fn first(&mut self, _cfg: &BarConfig, _ctx: &egui::Context, _ui: &mut Ui) {}

    fn middle(&mut self, cfg: &BarConfig, ctx: &egui::Context, ui: &mut Ui) {
        ui.centered_and_justified(|ui| {
            let date = chrono::Local::now();
            ctx.request_repaint_after(Duration::from_secs(1));

            let date = if let Position::Bottom | Position::Top = cfg.position {
                date.format("%H:%M:%S").to_string()
            } else {
                date.format("%H\n:%M:\n%S").to_string()
            };

            ui.heading(RichText::new(date).size(25.).color(cfg.text));
        });
    }
}
