//! Example how to use pure `egui_glow`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(unsafe_code)]

use std::{collections::HashMap, sync::Arc};

use crate::bar::{self, Bar, BarOption, Position};
use egui_winit::winit::{
    self, monitor::MonitorHandle, platform::x11::WindowBuilderExtX11, window::WindowId,
};

pub fn run(options: Vec<BarOption>, bar_factory: fn() -> Box<dyn Bar>) {
    // workaround for winit scaling bug
    std::env::set_var("WINIT_X11_SCALE_FACTOR", "1");

    let event_loop = winit::event_loop::EventLoopBuilder::with_user_event().build();
    let mut windows = HashMap::new();
    for option in options {
        let monitor = event_loop
            .available_monitors()
            .nth(option.monitor)
            .unwrap_or_else(|| panic!("No monitors found"));

        let (x, y, width, height) = match option.position {
            Position::Left => (
                monitor.position().x,
                monitor.position().y,
                option.size as u32,
                monitor.size().height,
            ),
            Position::Right => (
                monitor.position().x + monitor.size().width as i32 - option.size as i32,
                monitor.position().y,
                option.size as u32,
                monitor.size().height,
            ),
            Position::Top => (
                monitor.position().x,
                monitor.position().y,
                monitor.size().width,
                option.size as u32,
            ),
            Position::Bottom => (
                monitor.position().x,
                monitor.position().y + monitor.size().height as i32 - option.size as i32,
                monitor.size().width,
                option.size as u32,
            ),
        };
        let window = create_display(
            &event_loop,
            monitor.clone(),
            x,
            y,
            width,
            height,
            option.title.clone(),
        );

        let glow = Arc::new(window.1);

        windows.insert(
            window.0.window.id(),
            Context {
                glutin: window.0,
                egui_glow: egui_glow::EguiGlow::new(&event_loop, glow.clone(), None),
                glow,
                option,
                bar: bar_factory(),
            },
        );
    }
    events(windows, event_loop);
}

struct Context {
    pub glutin: GlutinWindowContext,
    pub glow: Arc<glow::Context>,
    pub egui_glow: egui_glow::EguiGlow,
    pub option: BarOption,
    pub bar: Box<dyn Bar>,
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
    #[allow(unsafe_code)]
    unsafe fn new(
        event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
        monitor: MonitorHandle,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        title: String,
    ) -> Self {
        use egui::NumExt;
        use glutin::context::NotCurrentGlContextSurfaceAccessor;
        use glutin::display::GetGlDisplay;
        use glutin::display::GlDisplay;
        use glutin::prelude::GlSurface;
        use raw_window_handle::HasRawWindowHandle;

        let winit_window_builder = Self::window_builder(x, y, width, height, title);
        let config_template_builder = Self::config_template_builder();

        log::debug!("trying to get gl_config");
        let (mut window, gl_config) =
            Self::display_builder(event_loop, &winit_window_builder, config_template_builder);

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

    fn window_builder(
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        title: String,
    ) -> winit::window::WindowBuilder {
        winit::window::WindowBuilder::new()
            .with_resizable(false)
            .with_override_redirect(false)
            .with_position(winit::dpi::PhysicalPosition::new(x, y))
            .with_x11_window_type(vec![winit::platform::x11::XWindowType::Dock])
            .with_inner_size(winit::dpi::PhysicalSize { width, height })
            .with_title(title)
            // Keep hidden until we've painted something. See https://github.com/emilk/egui/pull/2279
            .with_visible(false)
    }

    fn config_template_builder() -> glutin::config::ConfigTemplateBuilder {
        glutin::config::ConfigTemplateBuilder::new()
            .prefer_hardware_accelerated(Some(true))
            .with_depth_size(0)
            .with_stencil_size(0)
            .with_transparency(false)
    }

    fn display_builder(
        event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
        winit_window_builder: &winit::window::WindowBuilder,
        config_template_builder: glutin::config::ConfigTemplateBuilder,
    ) -> (Option<winit::window::Window>, glutin::config::Config) {
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
            .expect("failed to create gl_config")
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
    mut window_context: HashMap<WindowId, Context>,
    event_loop: winit::event_loop::EventLoop<()>,
) {
    event_loop.run(move |event, _, control_flow| {
        let mut redraw = |context: &mut Context| {
            let quit = false;

            let repaint_after = context.egui_glow.run(context.glutin.window(), |ctx| {
                bar::display_bar(&mut context.bar, ctx, &context.option)
            });

            *control_flow = if quit {
                winit::event_loop::ControlFlow::Exit
            } else if repaint_after.is_zero() {
                context.glutin.window().request_redraw();
                winit::event_loop::ControlFlow::Poll
            } else if let Some(repaint_after_instant) =
                std::time::Instant::now().checked_add(repaint_after)
            {
                winit::event_loop::ControlFlow::WaitUntil(repaint_after_instant)
            } else {
                winit::event_loop::ControlFlow::Wait
            };

            {
                let clear_color = (
                    context.option.background.r as f32 / 255.,
                    context.option.background.g as f32 / 255.,
                    context.option.background.b as f32 / 255.,
                );
                unsafe {
                    use glow::HasContext as _;
                    context
                        .glow
                        .clear_color(clear_color.0, clear_color.1, clear_color.2, 1.0);
                    context.glow.clear(glow::COLOR_BUFFER_BIT);
                }

                // draw things behind egui here
                context.egui_glow.paint(context.glutin.window());

                // draw things on top of egui here
                context.glutin.swap_buffers().unwrap();
                context.glutin.window().set_visible(true);
            }
        };

        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            winit::event::Event::RedrawEventsCleared if cfg!(windows) => {
                for (_, gl_window) in window_context.iter_mut() {
                    redraw(gl_window)
                }
            }
            winit::event::Event::RedrawRequested(window_id) if !cfg!(windows) => {
                redraw(window_context.get_mut(&window_id).unwrap())
            }
            //TODO: handle monitor resize
            winit::event::Event::WindowEvent {
                event, window_id, ..
            } => {
                use winit::event::WindowEvent;
                let context = window_context.get_mut(&window_id).unwrap();
                if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }

                if let winit::event::WindowEvent::Resized(physical_size) = &event {
                    context.glutin.resize(*physical_size);
                } else if let winit::event::WindowEvent::ScaleFactorChanged {
                    new_inner_size, ..
                } = &event
                {
                    context.glutin.resize(**new_inner_size);
                }

                let event_response = context.egui_glow.on_event(&event);

                if event_response.repaint {
                    context.glutin.window().request_redraw();
                }
            }
            winit::event::Event::LoopDestroyed => {
                for (_, context) in window_context.iter_mut() {
                    context.egui_glow.destroy();
                }
            }
            winit::event::Event::NewEvents(winit::event::StartCause::ResumeTimeReached {
                ..
            }) => {
                for (_, context) in window_context.iter_mut() {
                    context.glutin.window().request_redraw();
                }
            }

            _ => (),
        }
    });
}

fn create_display(
    event_loop: &winit::event_loop::EventLoopWindowTarget<()>,
    monitor: MonitorHandle,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    title: String,
) -> (GlutinWindowContext, glow::Context) {
    let glutin_window_context =
        unsafe { GlutinWindowContext::new(event_loop, monitor, x, y, width, height, title) };
    let gl = unsafe {
        glow::Context::from_loader_function(|s| {
            let s = std::ffi::CString::new(s)
                .expect("failed to construct C string from string for gl proc address");

            glutin_window_context.get_proc_address(&s)
        })
    };

    (glutin_window_context, gl)
}
