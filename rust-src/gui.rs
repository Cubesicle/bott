use std::time::SystemTime;
use parking_lot::Mutex;
use crate::bot::{self, Mode::{Record, Replay, Standby}, MODE};

#[cfg(not(target_os = "android"))]
macro_rules! window {
    ($name:expr) => { egui::Window::new($name).open(&mut *OPEN.lock()) };
}

#[cfg(target_os = "android")]
macro_rules! window {
    ($name:expr) => { egui::Window::new($name) };
}

static OPEN: Mutex<bool> = Mutex::new(true);

pub fn run(ctx: &egui::Context) {
    #[cfg(target_os = "android")]
    if !*OPEN.lock() { return; }
    
    let mode = &mut *MODE.lock();
    window!("Bott").frame(egui::Frame::window(&ctx.style()).stroke(egui::Stroke::new(
        ctx.style().visuals.window_stroke.width,
        match mode {
            Standby => ctx.style().visuals.window_stroke.color,
            Record => match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs() {
                s if s % 2 == 0 => egui::Color32::from_rgb(220, 38, 38),
                _ => ctx.style().visuals.window_stroke.color,
            },
            Replay => egui::Color32::from_rgb(37, 99, 235),
        }
    ))).default_size(egui::vec2(0.0, 0.0)).show(ctx, |ui| {
        ui.style_mut().interaction.selectable_labels = false;

        ui.horizontal(|ui| {
            ui.label("Mode:");
            ui.selectable_value(mode, Standby, Standby.to_string());
            ui.selectable_value(mode, Record, Record.to_string());
            ui.selectable_value(mode, Replay, Replay.to_string());
        });
        ui.separator();
        
        let footer_height = ui.text_style_height(&egui::TextStyle::Body) + 9.0;
        egui_extras::StripBuilder::new(ui)
            .size(egui_extras::Size::remainder())
            .size(egui_extras::Size::exact(footer_height))
            .vertical(|mut strip| {
                strip.cell(|ui| {
                    ui.label(format!(
                        "Inputs: {}",
                        bot::RECORDED_INPUTS.try_lock()
                            .and_then(|inputs| Some(inputs.len().to_string()))
                            .unwrap_or("loading...".to_string())
                    ));
                });
                strip.cell(|ui| {
                    ui.separator();
                    ui.vertical_centered(|ui| ui.label("Made by Cubesicle ❤"));
                });
            });
    });
}

pub fn is_showing() -> bool {
    *OPEN.lock()
}

pub fn show(show: bool) {
    *OPEN.lock() = show;
}