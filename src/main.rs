mod backend;
mod bar;

fn main() {
    bar::run(bar::Config {
        protocol: bar::Protocol::X11,
        title: "pagbar".to_string(),
        monitor: 0,
        position: bar::Position::Top,
        thickness: 100,
    })
}
