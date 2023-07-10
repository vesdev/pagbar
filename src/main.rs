use bar::{layout::*, *};
use clap::Parser;

mod bar;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<std::path::PathBuf>,
}

fn main() {
    env_logger::init();
    let args = Cli::parse();

    let layout_factory = || -> Layout {
        Layout::default()
            .split(Split::new(1.).widget(Box::new(widget::Clock::default())))
            .split(Split::new(4.).widget(Box::new(widget::Clock::default())))
            .split(Split::new(1.).widget(Box::new(widget::Clock::default())))
    };

    if args.config.is_none() {
        //LOOK FOR CONFIG IN XDG_CONFIG_HOME
        let base_dirs = xdg::BaseDirectories::new().unwrap();
        let config = bar::from_path(
            base_dirs.get_config_home().join("pagbar/config.toml"),
            layout_factory,
        );
        bar::run(bar::Protocol::X11, config);
    } else {
        let config = bar::from_path(args.config.unwrap(), layout_factory);
        bar::run(bar::Protocol::X11, config);
    }
}
