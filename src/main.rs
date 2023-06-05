#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::{egui, epaint::Pos2};
use thiserror::Error;
use xcb::*;

pub type Result<T> = std::result::Result<T, Error>;

fn match_window(setup: &x::Setup, conn: &Connection, title: &str) -> Result<x::Window> {
    let wm_client_list = conn.send_request(&x::InternAtom {
        only_if_exists: true,
        name: "_NET_CLIENT_LIST".as_bytes(),
    });
    let wm_client_list = conn.wait_for_reply(wm_client_list)?.atom();
    assert!(wm_client_list != x::ATOM_NONE, "EWMH not supported");

    for screen in setup.roots() {
        let window = screen.root();

        let pointer = conn.send_request(&x::QueryPointer { window });
        let pointer = conn.wait_for_reply(pointer)?;

        if pointer.same_screen() {
            let list = conn.send_request(&x::GetProperty {
                delete: false,
                window,
                property: wm_client_list,
                r#type: x::ATOM_NONE,
                long_offset: 0,
                long_length: 100,
            });
            let list = conn.wait_for_reply(list)?;

            for client in list.value::<x::Window>() {
                let cookie = conn.send_request(&x::GetProperty {
                    delete: false,
                    window: *client,
                    property: x::ATOM_WM_NAME,
                    r#type: x::ATOM_STRING,
                    long_offset: 0,
                    long_length: 1024,
                });
                let reply = conn.wait_for_reply(cookie)?;
                let reply_title = reply.value();
                let reply_title = std::str::from_utf8(reply_title).expect("invalid UTF-8");

                if reply_title == title {
                    return Ok(*client);
                }
            }
        }
    }

    Err(Error::WindowNotFound)
}

fn main() -> Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        initial_window_pos: Some(Pos2::new(0., 0.)),
        decorated: false,
        ..Default::default()
    };
    let (conn, _) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();

    println!(
        "{:?}",
        match_window(setup, &conn, "ves@nixos: ~/dev/pagbar")?
    );
    //let window = ewmh::proto::Window

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )?;
    Ok(())
}

struct MyApp {
    name: String,
    age: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "pagbar".to_owned(),
            age: 42,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("my_panel").show(ctx, |ui| {
            ui.heading("pagbar");
        });
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("xcb")]
    Xcb(#[from] xcb::Error),
    #[error("eframe")]
    Eframe(#[from] eframe::Error),
    #[error("Window not found")]
    WindowNotFound,
}
