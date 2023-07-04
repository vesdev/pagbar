use std::time::Duration;
use systemstat::Platform;

use bar::{Bar, Position};

use clap::Parser;

mod bar;
mod layout;

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
        let config = bar::from_path(
            base_dirs.get_config_home().join("pagbar/config.toml"),
            || Box::new(layout::preset::ThreeSplit::default()),
        );
        bar::run(bar::Protocol::X11, config);
    } else {
        let options = bar::from_path(args.config.unwrap(), || {
            Box::new(layout::preset::ThreeSplit::default())
        });
        bar::run(bar::Protocol::X11, options);
    }
}
