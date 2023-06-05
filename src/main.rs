use xcb::{x, Xid};

xcb::atoms_struct! {
    #[derive(Debug)]
    struct Atoms {
        wm_protocols    => b"WM_PROTOCOLS",
        wm_del_window   => b"WM_DELETE_WINDOW",
        wm_state        => b"_NET_WM_STATE",
        wm_state_maxv   => b"_NET_WM_STATE_MAXIMIZED_VERT",
        wm_state_maxh   => b"_NET_WM_STATE_MAXIMIZED_HORZ",
        wm_type         => b"_NET_WM_WINDOW_TYPE",
        wm_type_dock    => b"_NET_WM_WINDOW_TYPE_DOCK",
    }
}

/// The majority of `GlutinWindowContext` is taken from `eframe`

fn main() -> xcb::Result<()> {
    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();

    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();

    let window: x::Window = conn.generate_id();

    conn.send_request(&x::CreateWindow {
        depth: x::COPY_FROM_PARENT as u8,
        wid: window,
        parent: screen.root(),
        x: 0,
        y: 0,
        width: 1000,
        height: 50,
        border_width: 10,
        class: x::WindowClass::InputOutput,
        visual: screen.root_visual(),
        value_list: &[
            x::Cw::BackPixel(screen.white_pixel()),
            x::Cw::EventMask(x::EventMask::EXPOSURE | x::EventMask::KEY_PRESS),
        ],
    });

    conn.send_request(&x::MapWindow { window });

    let title = "pagbar";

    conn.send_request(&x::ChangeProperty {
        mode: x::PropMode::Replace,
        window,
        property: x::ATOM_WM_NAME,
        r#type: x::ATOM_STRING,
        data: title.as_bytes(),
    });

    conn.flush()?;

    // retrieve atoms
    let atoms = Atoms::intern_all(&conn)?;

    // SET WINDOW TYPE AS A DOCK
    conn.send_and_check_request(&x::ChangeProperty {
        mode: xcb::x::PropMode::Replace,
        window,
        property: atoms.wm_type,
        r#type: xcb::x::ATOM_ATOM,
        data: &[atoms.wm_type_dock],
    })?;

    #[allow(clippy::empty_loop)]
    loop {
        //application loop
    }
}
