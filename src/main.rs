mod backend;
mod bar;
mod error;

fn main() {
    bar::run(
        backend::Xcb::new().unwrap(),
        0,
        bar::Config {
            position: bar::Position::Bottom,
            thickness: 50,
        },
    );
}
