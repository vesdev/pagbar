use super::{Bar, Config};
use crate::bar::Position;
use std::{collections::HashMap, rc::Rc};
use systemstat::Platform;

pub trait Widget {
    fn draw(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, cfg: &Config);
}

type WidgetFactory = Box<dyn Fn() -> Box<dyn Widget>>;

pub struct WidgetSet {
    widgets: HashMap<String, WidgetFactory>,
}

impl WidgetSet {
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, widget_factory: WidgetFactory) {
        self.widgets.insert(name, widget_factory.into());
    }

    pub fn create_widget(&mut self, name: &String) -> Option<Box<dyn Widget>> {
        Some(self.widgets.get_mut(name)?())
    }
}

impl Default for WidgetSet {
    fn default() -> Self {
        let mut result = Self {
            widgets: HashMap::new(),
        };

        let sys = Rc::new(systemstat::System::new());
        result.insert("clock".into(), Box::new(|| Box::new(Clock::default())));

        let sys_clone = sys.clone();
        result.insert(
            "disk".into(),
            Box::new(move || Box::new(Disk::new(sys_clone.clone()))),
        );
        let sys_clone = sys.clone();
        result.insert(
            "ram".into(),
            Box::new(move || Box::new(Ram::new(sys_clone.clone()))),
        );
        let sys_clone = sys.clone();
        result.insert(
            "cpu".into(),
            Box::new(move || Box::new(Cpu::new(sys_clone.clone()))),
        );
        result
    }
}

#[derive(Default)]
pub struct Clock;

impl Widget for Clock {
    fn draw(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, cfg: &Config) {
        let date = chrono::Local::now();
        ctx.request_repaint_after(std::time::Duration::from_secs(1));

        let date = if let Position::Bottom | Position::Top = cfg.position {
            date.format("%H:%M:%S").to_string()
        } else {
            date.format("%H\n:%M:\n%S").to_string()
        };

        ui.heading(egui::RichText::new(date).size(25.).color(cfg.text));
    }
}

pub struct Disk {
    sys: Rc<systemstat::System>,
}

impl Disk {
    pub fn new(sys: Rc<systemstat::System>) -> Self {
        Self { sys }
    }
}

impl Widget for Disk {
    fn draw(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, cfg: &Config) {
        let disk = match self.sys.mount_at("/") {
            Ok(mount) => (1. - mount.free.as_u64() as f64 / mount.total.as_u64() as f64) * 100.,
            Err(_) => 0.,
        };
        ui.heading(egui::RichText::new("/ ".to_string()).color(cfg.text));
        ui.heading(egui::RichText::new(format!("{disk:.0}%")).color(cfg.text_secondary));
    }
}

pub struct Ram {
    sys: Rc<systemstat::System>,
}

impl Ram {
    pub fn new(sys: Rc<systemstat::System>) -> Self {
        Self { sys }
    }
}

impl Widget for Ram {
    fn draw(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, cfg: &Config) {
        let memory = match self.sys.memory() {
            Ok(mem) => (1. - mem.free.as_u64() as f64 / mem.total.as_u64() as f64) * 100.,
            Err(_) => 0.,
        };
        ui.heading(egui::RichText::new("ram ".to_string()).color(cfg.text));
        ui.heading(egui::RichText::new(format!("{memory:.0}%")).color(cfg.text_secondary));
    }
}

pub struct Cpu {
    sys: Rc<systemstat::System>,
}

impl Cpu {
    pub fn new(sys: Rc<systemstat::System>) -> Self {
        Self { sys }
    }
}

impl Widget for Cpu {
    fn draw(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, cfg: &Config) {
        let temp = self.sys.cpu_temp().unwrap_or(0.);
        ui.heading(egui::RichText::new("cpu".to_string()).color(cfg.text));
        ui.heading(egui::RichText::new(format!("{temp}°C")).color(cfg.text_secondary));
    }
}
