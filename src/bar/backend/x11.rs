#![allow(unsafe_code)]

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use crate::{
    bar::{self, Bar, Position},
    layout::Layout,
};
use egui_winit::winit::{
    self,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopWindowTarget},
    monitor::MonitorHandle,
    platform::x11::WindowBuilderExtX11,
    window::Window,
    window::WindowId,
};

enum UserEvent {
    RequestRedraw(WindowId),
}

pub fn run(bars: Vec<Bar>) {
    // workaround for winit scaling bug
    std::env::set_var("WINIT_X11_SCALE_FACTOR", "1");

    let event_loop = winit::event_loop::EventLoopBuilder::with_user_event().build();
    let repaint_signal = RepaintSignal(Arc::new(Mutex::new(event_loop.create_proxy())));
    let mut bar_map = create_bars(&event_loop, repaint_signal, bars);
    let mut window_map = HashMap::<WindowId, BarWindowId>::new();

    event_loop.run(move |event, event_loop, control_flow| match event {
        winit::event::Event::RedrawRequested(window_id) => {
            if let Some(bar) = window_map
                .get(&window_id)
                .copied()
                .and_then(|id| bar_map.get_mut(&id))
            {
                bar.on_redraw_request();
            }
        }
        winit::event::Event::WindowEvent { window_id, event } => {
            if let Some(bar) = window_map
                .get(&window_id)
                .copied()
                .and_then(|id| bar_map.get_mut(&id))
            {
                bar.on_window_event(event, control_flow, &mut window_map);
            }
        }
        winit::event::Event::UserEvent(UserEvent::RequestRedraw(window_id)) => {
            if let Some(bar) = window_map
                .get(&window_id)
                .copied()
                .and_then(|id| bar_map.get_mut(&id))
            {
                bar.on_user_event();
            }
        }
        winit::event::Event::Suspended => {
            for (_, bar) in bar_map.iter_mut() {
                bar.on_suspend(&mut window_map);
            }
        }
        winit::event::Event::Resumed => {
            for (_, window) in bar_map.iter_mut() {
                window.on_resume(event_loop, &mut window_map);
            }
        }
        winit::event::Event::MainEventsCleared => {
            for (_, window) in bar_map.iter_mut() {
                window.on_main_events_cleared();
            }
        }
        _ => (),
    });
}

fn create_bars<'a>(
    event_loop: &EventLoop<UserEvent>,
    repaint_signal: RepaintSignal,
    mut bars: Vec<Bar>,
) -> HashMap<BarWindowId, BarWindow> {
    let mut bar_map = HashMap::new();

    for bar in bars.drain(..) {
        let monitor = event_loop
            .available_monitors()
            .nth(bar.cfg.monitor)
            .unwrap_or_else(|| panic!("No monitors found"));
        let bar_window = BarWindow::new(&event_loop, repaint_signal.clone(), monitor, bar);

        bar_map.insert(bar_window.id, bar_window);
    }
    bar_map
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
        .with_position(winit::dpi::PhysicalPosition::new(x, y))
        .with_x11_window_type(vec![winit::platform::x11::XWindowType::Dock])
        .with_inner_size(winit::dpi::PhysicalSize { width, height })
        .with_title(title)
}

#[derive(Clone)]
struct RepaintSignal(Arc<Mutex<EventLoopProxy<UserEvent>>>);

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct BarWindowId(u64);

struct BarWindow {
    id: BarWindowId,
    ctx: egui::Context,
    painter: egui_wgpu::winit::Painter,
    state: egui_winit::State,
    window: Option<winit::window::Window>,
    repaint_signal: RepaintSignal,
    bar: Bar,
    monitor: MonitorHandle,
}

impl BarWindow {
    pub fn new(
        event_loop: &EventLoop<UserEvent>,
        repaint_signal: RepaintSignal,
        monitor: MonitorHandle,
        bar: Bar,
    ) -> Self {
        static ID: AtomicU64 = AtomicU64::new(0);
        let id = BarWindowId(ID.fetch_add(1, Ordering::SeqCst));

        let ctx = egui::Context::default();
        let state = egui_winit::State::new(&event_loop);

        let config = egui_wgpu::WgpuConfiguration {
            supported_backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        };

        let painter = egui_wgpu::winit::Painter::new(config, 1, None, false);

        Self {
            id,
            ctx,
            state,
            painter,
            window: None,
            repaint_signal,
            bar,
            monitor,
        }
    }

    fn create_window(&mut self, event_loop: &EventLoopWindowTarget<UserEvent>) -> Window {
        let (x, y, w, h) = self.position();
        let window = window_builder(x, y, w, h, self.bar.cfg.title.clone())
            .build(&event_loop)
            .unwrap();

        pollster::block_on(self.painter.set_window(Some(&window))).expect("unable to set window");

        if let Some(max_size) = self.painter.max_texture_side() {
            self.state.set_max_texture_side(max_size);
        }

        let pixels_per_point = window.scale_factor() as f32;
        self.state.set_pixels_per_point(pixels_per_point);

        window.request_redraw();

        window
    }

    fn on_resume(
        &mut self,
        event_loop: &EventLoopWindowTarget<UserEvent>,
        window_map: &mut HashMap<WindowId, BarWindowId>,
    ) {
        let window = match self.window.as_mut() {
            None => {
                let w = self.create_window(event_loop);
                pollster::block_on(self.painter.set_window(Some(&w)))
                    .expect("unable to set window");
                let window_id = w.id();
                let repaint_signal = self.repaint_signal.clone();
                self.ctx.set_request_repaint_callback(move |_| {
                    let _ = repaint_signal
                        .0
                        .lock()
                        .unwrap()
                        .send_event(UserEvent::RequestRedraw(window_id));
                });
                window_map.insert(window_id, self.id);
                self.window = Some(w);
                self.window.as_mut().unwrap()
            }
            Some(window) => window,
        };
        window.request_redraw();
    }

    fn on_redraw_request(&mut self) {
        if let Some(window) = self.window.as_ref() {
            let raw_input = self.state.take_egui_input(window);
            let backend = &super::BackendData {
                width: self.position().2 as usize,
                height: self.position().3 as usize,
            };
            let output = self.ctx.run(raw_input, |ctx| {
                self.bar.layout.draw(ctx, &self.bar.cfg, backend);
            });

            self.state
                .handle_platform_output(window, &self.ctx, output.platform_output);

            self.painter.paint_and_update_textures(
                self.state.pixels_per_point(),
                egui::Rgba::default().to_array(),
                &self.ctx.tessellate(output.shapes),
                &output.textures_delta,
                false,
            );

            if output.repaint_after.is_zero() {
                window.request_redraw();
            }
        }
    }

    fn on_user_event(&mut self) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn on_main_events_cleared(&mut self) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn on_suspend(&mut self, window_map: &mut HashMap<WindowId, BarWindowId>) {
        if let Some(window) = self.window.as_ref() {
            window_map.remove(&window.id());
        }
        self.window = None;
    }

    fn on_window_event(
        &mut self,
        event: WindowEvent,
        control_flow: &mut ControlFlow,
        window_map: &mut HashMap<WindowId, BarWindowId>,
    ) {
        match event {
            WindowEvent::Resized(size) => {
                self.painter.on_window_resized(size.width, size.height);
            }
            WindowEvent::CloseRequested => {
                self.on_suspend(window_map);
                if window_map.is_empty() {
                    // no more open windows, close the app
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }

        let response = self.state.on_event(&self.ctx, &event);
        if response.repaint {
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
    }

    fn position(&self) -> (i32, i32, u32, u32) {
        let monitor = &self.monitor;
        match self.bar.cfg.position {
            Position::Left => (
                monitor.position().x,
                monitor.position().y,
                self.bar.cfg.size as u32,
                monitor.size().height,
            ),
            Position::Right => (
                monitor.position().x + monitor.size().width as i32 - self.bar.cfg.size as i32,
                monitor.position().y,
                self.bar.cfg.size as u32,
                monitor.size().height,
            ),
            Position::Top => (
                monitor.position().x,
                monitor.position().y,
                monitor.size().width,
                self.bar.cfg.size as u32,
            ),
            Position::Bottom => (
                monitor.position().x,
                monitor.position().y + monitor.size().height as i32 - self.bar.cfg.size as i32,
                monitor.size().width,
                self.bar.cfg.size as u32,
            ),
        }
    }
}

// let repaint_after = context.egui_glow.run(context.glutin.window(), |ctx| {
//     bar::display_bar(&mut context.bar, ctx, &context.option)
// });
