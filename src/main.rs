use bar::{layout::*, widget::WidgetSet, *};
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

    let mut widget_set = WidgetSet::default();

    if args.config.is_none() {
        //LOOK FOR CONFIG IN XDG_CONFIG_HOME
        let base_dirs = xdg::BaseDirectories::new().unwrap();
        let config = bar::from_path(
            base_dirs.get_config_home().join("pagbar/config.toml"),
            &mut widget_set,
        );
        bar::run(bar::Protocol::X11, config);
    } else {
        let config = bar::from_path(args.config.unwrap(), &mut widget_set);
        bar::run(bar::Protocol::X11, config);
    }
}
