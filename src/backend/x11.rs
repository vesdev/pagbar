//! Example how to use pure `egui_glow`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(unsafe_code)]

use std::time::Duration;

use crate::bar::{Bar, BarOptions, Position};
use egui_winit::winit::{self, monitor::MonitorHandle, platform::x11::WindowBuilderExtX11};
use glow::Context;

pub fn run(options: BarOptions, bar: Box<dyn Bar>) {
    let event_loop = winit::event_loop::EventLoopBuilder::with_user_event().build();
    let monitors: Vec<MonitorHandle> = event_loop.available_monitors().collect();
    let monitor = monitors.get(options.monitor).unwrap_or_else(|| {
        panic!(
            "monitor {} out of range 0..{}",
            options.monitor,
            monitors.len() - 1
        )
    });
    let (x, y, width, height) = match options.position {
        Position::Left => (
            monitor.position().x,
            monitor.position().y,
            options.size as u32,
            monitor.size().height,
        ),
        Position::Right => (
            monitor.position().x + monitor.size().width as i32 - options.size as i32,
            monitor.position().y,
            options.size as u32,
            monitor.size().height,
        ),
        Position::Top => (
            monitor.position().x,
            monitor.position().y,
            monitor.size().width,
            options.size as u32,
        ),
        Position::Bottom => (
            monitor.position().x,
            monitor.position().y + monitor.size().height as i32 - options.size as i32,
            monitor.size().width,
            options.size as u32,
        ),
    };

    // ??? SOME WEIRD THING WHERE MONITOR 0 is correct with physical size
    // and other monitors work correctly with logical size
    let use_physical = options.monitor == 0;

    let (window, context) = create_display(
        &event_loop,
        x,
        y,
        width,
        height,
        options.title.clone(),
        use_physical,
    );
    events(window, context, event_loop, bar, options);
}

/// The majority of `GlutinWindowContext` is taken from `eframe`
struct GlutinWindowContext {
    pub window: winit::window::Window,
    gl_context: glutin::context::PossiblyCurrentContext,
    gl_display: glutin::display::Display,
    gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

impl GlutinWindowContext {
    // refactor this function to use `glutin-winit` crate eventually.
    // preferably add android support at the same time.
    #[allow(unsafe_code)]
    unsafe fn new(
        event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        title: String,
        phyiscal_size: bool,
    ) -> Self {
        use egui::NumExt;
        use glutin::context::NotCurrentGlContextSurfaceAccessor;
        use glutin::display::GetGlDisplay;
        use glutin::display::GlDisplay;
        use glutin::prelude::GlSurface;
        use raw_window_handle::HasRawWindowHandle;

        let winit_window_builder = winit::window::WindowBuilder::new()
            .with_resizable(false)
            .with_position(winit::dpi::PhysicalPosition::new(x, y))
            .with_x11_window_type(vec![winit::platform::x11::XWindowType::Dock])
            .with_maximized(true)
            .with_title(title) // Keep hidden until we've painted something. See https://github.com/emilk/egui/pull/2279
            .with_visible(false);

        let winit_window_builder = if phyiscal_size {
            winit_window_builder.with_inner_size(winit::dpi::PhysicalSize { width, height })
        } else {
            winit_window_builder.with_inner_size(winit::dpi::LogicalSize { width, height })
        };

        let config_template_builder = glutin::config::ConfigTemplateBuilder::new()
            .prefer_hardware_accelerated(Some(true))
            .with_depth_size(0)
            .with_stencil_size(0)
            .with_transparency(false);

        log::debug!("trying to get gl_config");
        let (mut window, gl_config) =
            glutin_winit::DisplayBuilder::new() // let glutin-winit helper crate handle the complex parts of opengl context creation
                .with_preference(glutin_winit::ApiPrefence::FallbackEgl) // https://github.com/emilk/egui/issues/2520#issuecomment-1367841150
                .with_window_builder(Some(winit_window_builder.clone()))
                .build(
                    event_loop,
                    config_template_builder,
                    |mut config_iterator| {
                        config_iterator.next().expect(
                            "failed to find a matching configuration for creating glutin config",
                        )
                    },
                )
                .expect("failed to create gl_config");
        let gl_display = gl_config.display();
        log::debug!("found gl_config: {:?}", &gl_config);

        let raw_window_handle = window.as_ref().map(|w| w.raw_window_handle());
        log::debug!("raw window handle: {:?}", raw_window_handle);
        let context_attributes =
            glutin::context::ContextAttributesBuilder::new().build(raw_window_handle);
        // by default, glutin will try to create a core opengl context. but, if it is not available, try to create a gl-es context using this fallback attributes
        let fallback_context_attributes = glutin::context::ContextAttributesBuilder::new()
            .with_context_api(glutin::context::ContextApi::Gles(None))
            .build(raw_window_handle);
        let not_current_gl_context = unsafe {
            gl_display
                    .create_context(&gl_config, &context_attributes)
                    .unwrap_or_else(|_| {
                        log::debug!("failed to create gl_context with attributes: {:?}. retrying with fallback context attributes: {:?}",
                            &context_attributes,
                            &fallback_context_attributes);
                        gl_config
                            .display()
                            .create_context(&gl_config, &fallback_context_attributes)
                            .expect("failed to create context even with fallback attributes")
                    })
        };

        // this is where the window is created, if it has not been created while searching for suitable gl_config
        let window = window.take().unwrap_or_else(|| {
            log::debug!("window doesn't exist yet. creating one now with finalize_window");
            glutin_winit::finalize_window(event_loop, winit_window_builder.clone(), &gl_config)
                .expect("failed to finalize glutin window")
        });

        let (width, height): (u32, u32) = window.inner_size().into();
        let width = std::num::NonZeroU32::new(width.at_least(1)).unwrap();
        let height = std::num::NonZeroU32::new(height.at_least(1)).unwrap();
        let surface_attributes =
            glutin::surface::SurfaceAttributesBuilder::<glutin::surface::WindowSurface>::new()
                .build(window.raw_window_handle(), width, height);
        log::debug!(
            "creating surface with attributes: {:?}",
            &surface_attributes
        );
        let gl_surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &surface_attributes)
                .unwrap()
        };
        log::debug!("surface created successfully: {gl_surface:?}.making context current");
        let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

        gl_surface
            .set_swap_interval(
                &gl_context,
                glutin::surface::SwapInterval::Wait(std::num::NonZeroU32::new(1).unwrap()),
            )
            .unwrap();

        GlutinWindowContext {
            window,
            gl_context,
            gl_display,
            gl_surface,
        }
    }

    fn window(&self) -> &winit::window::Window {
        &self.window
    }

    fn resize(&self, physical_size: winit::dpi::PhysicalSize<u32>) {
        use glutin::surface::GlSurface;
        self.gl_surface.resize(
            &self.gl_context,
            physical_size.width.try_into().unwrap(),
            physical_size.height.try_into().unwrap(),
        );
    }

    fn swap_buffers(&self) -> glutin::error::Result<()> {
        use glutin::surface::GlSurface;
        self.gl_surface.swap_buffers(&self.gl_context)
    }

    fn get_proc_address(&self, addr: &std::ffi::CStr) -> *const std::ffi::c_void {
        use glutin::display::GlDisplay;
        self.gl_display.get_proc_address(addr)
    }
}

fn events(
    gl_window: GlutinWindowContext,
    gl: Context,
    event_loop: winit::event_loop::EventLoop<()>,
    mut bar: Box<dyn Bar>,
    options: BarOptions,
) {
    let gl = std::sync::Arc::new(gl);
    let mut egui_glow = egui_glow::EguiGlow::new(&event_loop, gl.clone(), None);

    let clear_color = (
        options.bg_color.r as f32 / 255.,
        options.bg_color.g as f32 / 255.,
        options.bg_color.b as f32 / 255.,
    );
    let visuals: egui::Visuals = options.into();
    event_loop.run(move |event, _, control_flow| {
        let mut redraw = || {
            let quit = false;

            let repaint_after = egui_glow.run(gl_window.window(), |ctx| {
                ctx.set_visuals(visuals.clone());
                bar.update(ctx);
            });

            *control_flow = if quit {
                winit::event_loop::ControlFlow::Exit
            } else if repaint_after.is_zero() {
                gl_window.window().request_redraw();
                winit::event_loop::ControlFlow::Poll
            } else if let Some(repaint_after_instant) =
                std::time::Instant::now().checked_add(Duration::from_secs(1))
            //tick rate
            {
                winit::event_loop::ControlFlow::WaitUntil(repaint_after_instant)
            } else {
                winit::event_loop::ControlFlow::Wait
            };

            {
                unsafe {
                    use glow::HasContext as _;
                    gl.clear_color(clear_color.0, clear_color.1, clear_color.2, 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT);
                }

                // draw things behind egui here

                egui_glow.paint(gl_window.window());

                // draw things on top of egui here

                gl_window.swap_buffers().unwrap();
                gl_window.window().set_visible(true);
            }
        };

        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            winit::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(),
            winit::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),

            winit::event::Event::WindowEvent { event, .. } => {
                use winit::event::WindowEvent;
                if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }

                if let winit::event::WindowEvent::Resized(physical_size) = &event {
                    gl_window.resize(*physical_size);
                } else if let winit::event::WindowEvent::ScaleFactorChanged {
                    new_inner_size, ..
                } = &event
                {
                    gl_window.resize(**new_inner_size);
                }

                let event_response = egui_glow.on_event(&event);

                if event_response.repaint {
                    gl_window.window().request_redraw();
                }
            }
            winit::event::Event::LoopDestroyed => {
                egui_glow.destroy();
            }
            winit::event::Event::NewEvents(winit::event::StartCause::ResumeTimeReached {
                ..
            }) => {
                gl_window.window().request_redraw();
            }

            _ => (),
        }
    });
}

fn create_display(
    event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    title: String,
    physical_size: bool,
) -> (GlutinWindowContext, glow::Context) {
    let glutin_window_context =
        unsafe { GlutinWindowContext::new(event_loop, x, y, width, height, title, physical_size) };
    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            let s = std::ffi::CString::new(s)
                .expect("failed to construct C string from string for gl proc address");

            glutin_window_context.get_proc_address(&s)
        })
    };

    (glutin_window_context, gl)
}
