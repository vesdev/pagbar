use std::{
    ffi::{c_int, c_void, CStr, CString},
    ptr,
};

use crate::{
    bar::{Config, Monitor, Position},
    error::Error,
    error::*,
};

use x11::*;

pub trait Protocol {
    fn get_monitors(&mut self) -> Result<Vec<Monitor>>;

    fn create_window(&mut self, monitor: &Monitor, title: String, config: Config) -> Result<()>;
}

xcb::atoms_struct! {
    #[derive(Debug)]
    struct Atoms {
        wm_protocols        => b"WM_PROTOCOLS",
        wm_del_window       => b"WM_DELETE_WINDOW",
        wm_state            => b"_NET_WM_STATE",
        wm_state_maxv       => b"_NET_WM_STATE_MAXIMIZED_VERT",
        wm_state_maxh       => b"_NET_WM_STATE_MAXIMIZED_HORZ",
        wm_type             => b"_NET_WM_WINDOW_TYPE",
        wm_type_dock        => b"_NET_WM_WINDOW_TYPE_DOCK",
        wm_strut_partial    => b"_NET_WM_STRUT_PARTIAL",
    }
}

type GlXCreateContextAttribsARBProc = unsafe extern "C" fn(
    dpy: *mut xlib::Display,
    fbc: glx::GLXFBConfig,
    share_context: glx::GLXContext,
    direct: xlib::Bool,
    attribs: *const c_int,
) -> glx::GLXContext;

static mut CTX_ERROR_OCCURED: bool = false;
unsafe extern "C" fn ctx_error_handler(
    _dpy: *mut xlib::Display,
    _ev: *mut xlib::XErrorEvent,
) -> i32 {
    CTX_ERROR_OCCURED = true;
    0
}

pub struct Xcb {
    connection: xcb::Connection,
    screen_number: i32,
}

impl Xcb {
    pub fn new() -> Result<Box<Self>> {
        let (connection, screen_number) = xcb::Connection::connect_with_xlib_display()?;
        connection.set_event_queue_owner(xcb::EventQueueOwner::Xcb);
        let glx_ver =
            connection.wait_for_reply(connection.send_request(&xcb::glx::QueryVersion {
                major_version: 1,
                minor_version: 3,
            }))?;
        assert!(glx_ver.major_version() >= 1 && glx_ver.minor_version() >= 3);

        Ok(Box::new(Self {
            connection,
            screen_number,
        }))
    }

    fn window_title(&mut self, window: xcb::x::Window, title: String) -> Result<()> {
        self.connection
            .send_and_check_request(&xcb::x::ChangeProperty {
                mode: xcb::x::PropMode::Replace,
                window,
                property: xcb::x::ATOM_WM_NAME,
                r#type: xcb::x::ATOM_STRING,
                data: title.as_bytes(),
            })?;
        Ok(())
    }

    fn window_dock(
        &mut self,
        window: xcb::x::Window,
        monitor: &Monitor,
        config: Config,
    ) -> Result<()> {
        // retrieve atoms
        let atoms = Atoms::intern_all(&self.connection)?;
        // SET WINDOW TYPE AS A DOCK
        self.connection
            .send_and_check_request(&xcb::x::ChangeProperty {
                mode: xcb::x::PropMode::Replace,
                window,
                property: atoms.wm_type,
                r#type: xcb::x::ATOM_ATOM,
                data: &[atoms.wm_type_dock],
            })?;

        self.connection
            .send_and_check_request(&xcb::x::ChangeProperty {
                mode: xcb::x::PropMode::Replace,
                window,
                property: atoms.wm_strut_partial,
                r#type: xcb::x::ATOM_CARDINAL,
                data: &[
                    //reserved space
                    0_u32,                                   // left
                    0,                                       // right
                    0,                                       // top
                    config.thickness as u32,                 // bottom
                    0,                                       // lefy start y
                    0,                                       // lefy end y
                    0,                                       // right start y
                    0,                                       // right end y
                    0,                                       //top start x
                    0,                                       //top end x
                    monitor.x as u32,                        //bottom start x
                    monitor.x as u32 + monitor.width as u32, //bottom end x
                ],
            })?;
        Ok(())
    }

    fn screen(connection: &xcb::Connection, screen_number: i32) -> Result<&xcb::x::Screen> {
        connection
            .get_setup()
            .roots()
            .nth(screen_number as usize)
            .ok_or(Error::Unknown)
    }

    fn get_glxfbconfig(
        dpy: *mut xlib::Display,
        screen_num: i32,
        visual_attribs: &[i32],
    ) -> x11::glx::GLXFBConfig {
        unsafe {
            let mut fbcount: c_int = 0;
            let fbcs = x11::glx::glXChooseFBConfig(
                dpy,
                screen_num,
                visual_attribs.as_ptr(),
                &mut fbcount as *mut c_int,
            );

            if fbcount == 0 {
                panic!("could not find compatible fb config");
            }
            // we pick the first from the list
            let fbc = *fbcs;
            xlib::XFree(fbcs as *mut c_void);
            fbc
        }
    }

    fn check_glx_extension(glx_exts: &str, ext_name: &str) -> bool {
        for glx_ext in glx_exts.split(" ") {
            if glx_ext == ext_name {
                return true;
            }
        }
        false
    }

    unsafe fn load_gl_func(name: &str) -> *mut c_void {
        let cname = CString::new(name).unwrap();
        let ptr: *mut c_void =
            std::mem::transmute(glx::glXGetProcAddress(cname.as_ptr() as *const u8));
        if ptr.is_null() {
            panic!("could not load {}", name);
        }
        ptr
    }
}

impl Protocol for Xcb {
    fn get_monitors(&mut self) -> Result<Vec<Monitor>> {
        let screen = Self::screen(&self.connection, self.screen_number)?;
        let reply = self
            .connection
            .wait_for_reply(self.connection.send_request(&xcb::randr::GetMonitors {
                window: screen.root(),
                get_active: false,
            }))?;

        let mut monitors = Vec::new();
        for monitor in reply.monitors() {
            monitors.push(Monitor {
                x: monitor.x(),
                y: monitor.y(),
                width: monitor.width(),
                height: monitor.height(),
            });
        }
        Ok(monitors)
    }

    fn create_window(&mut self, monitor: &Monitor, title: String, config: Config) -> Result<()> {
        let screen = Self::screen(&self.connection, self.screen_number)?;

        let fbc = Self::get_glxfbconfig(
            self.connection.get_raw_dpy(),
            self.screen_number,
            &[
                glx::GLX_X_RENDERABLE,
                1,
                glx::GLX_DRAWABLE_TYPE,
                glx::GLX_WINDOW_BIT,
                glx::GLX_RENDER_TYPE,
                glx::GLX_RGBA_BIT,
                glx::GLX_X_VISUAL_TYPE,
                glx::GLX_TRUE_COLOR,
                glx::GLX_RED_SIZE,
                8,
                glx::GLX_GREEN_SIZE,
                8,
                glx::GLX_BLUE_SIZE,
                8,
                glx::GLX_ALPHA_SIZE,
                8,
                glx::GLX_DEPTH_SIZE,
                24,
                glx::GLX_STENCIL_SIZE,
                8,
                glx::GLX_DOUBLEBUFFER,
                1,
                0,
            ],
        );

        let vi_ptr: *mut xlib::XVisualInfo =
            unsafe { glx::glXGetVisualFromFBConfig(self.connection.get_raw_dpy(), fbc) };
        let vi = unsafe { *vi_ptr };

        let cmap: xcb::x::Colormap = self.connection.generate_id();
        self.connection.send_request(&xcb::x::CreateColormap {
            alloc: xcb::x::ColormapAlloc::None,
            mid: cmap,
            window: screen.root(),
            visual: vi.visualid as u32,
        });

        let (x, y, width, height) = match config.position {
            Position::Left => (monitor.x, monitor.y, config.thickness, monitor.height),
            Position::Right => (
                monitor.x + monitor.width as i16 - config.thickness as i16,
                monitor.y,
                config.thickness,
                monitor.height,
            ),
            Position::Top => (monitor.x, monitor.y, monitor.width, config.thickness),
            Position::Bottom => (
                monitor.x,
                monitor.y + monitor.height as i16 - config.thickness as i16,
                monitor.width,
                config.thickness,
            ),
        };

        let window: xcb::x::Window = self.connection.generate_id();
        self.connection.send_request(&xcb::x::CreateWindow {
            depth: xcb::x::COPY_FROM_PARENT as u8,
            wid: window,
            parent: screen.root(),
            x,
            y,
            width,
            height,
            border_width: 0,
            class: xcb::x::WindowClass::InputOutput,
            visual: vi.visualid as u32,
            value_list: &[
                xcb::x::Cw::BackPixel(screen.white_pixel()),
                xcb::x::Cw::EventMask(xcb::x::EventMask::EXPOSURE | xcb::x::EventMask::KEY_PRESS),
                xcb::x::Cw::Colormap(cmap),
            ],
        });

        self.connection.check_request(
            self.connection
                .send_request_checked(&xcb::x::MapWindow { window }),
        )?;

        self.window_title(window, title)?;
        self.window_dock(window, monitor, config)?;

        unsafe {
            xlib::XFree(vi_ptr as *mut c_void);
        }

        let glx_exts = unsafe {
            CStr::from_ptr(glx::glXQueryExtensionsString(
                self.connection.get_raw_dpy(),
                self.screen_number,
            ))
        }
        .to_str()
        .unwrap();

        if !Self::check_glx_extension(glx_exts, "GLX_ARB_create_context") {
            panic!("could not find GLX extension GLX_ARB_create_context");
        }

        let glx_create_context_attribs: GlXCreateContextAttribsARBProc =
            unsafe { std::mem::transmute(Self::load_gl_func("glXCreateContextAttribsARB")) };

        // loading all other symbols
        unsafe {
            gl::load_with(|n| Self::load_gl_func(n));
        }

        // installing an event handler to check if error is generated
        unsafe {
            CTX_ERROR_OCCURED = false;
        }

        let _old_handler = unsafe { xlib::XSetErrorHandler(Some(ctx_error_handler)) };

        let context_attribs: [c_int; 5] = [
            glx::arb::GLX_CONTEXT_MAJOR_VERSION_ARB as c_int,
            3,
            glx::arb::GLX_CONTEXT_MINOR_VERSION_ARB as c_int,
            0,
            0,
        ];
        let ctx = unsafe {
            glx_create_context_attribs(
                self.connection.get_raw_dpy(),
                fbc,
                ptr::null_mut(),
                xlib::True,
                &context_attribs[0] as *const c_int,
            )
        };

        self.connection.flush()?;

        //event loop
        loop {} //TODO: graceful exit

        unsafe {
            glx::glXDestroyContext(self.connection.get_raw_dpy(), ctx);
        }

        self.connection
            .send_request(&xcb::x::UnmapWindow { window });
        self.connection
            .send_request(&xcb::x::DestroyWindow { window });
        self.connection.send_request(&xcb::x::FreeColormap { cmap });
        self.connection.flush()?;

        Ok(())
    }
}
