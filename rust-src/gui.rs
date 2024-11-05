use std::time::SystemTime;
use parking_lot::Mutex;
use crate::{bot::{self, Mode::{Record, Replay, Standby}, MODE, RECORDED_INPUTS}, gd};

#[cfg(not(target_os = "android"))]
macro_rules! window {
    ($name:expr) => { egui::Window::new($name).open(&mut *OPEN.lock()) };
}

#[cfg(target_os = "android")]
macro_rules! window {
    ($name:expr) => { egui::Window::new($name) };
}

static OPEN: Mutex<bool> = Mutex::new(true);
static REPLAY_FILE_NAME: Mutex<String> = Mutex::new(String::new());

pub fn run(ctx: &egui::Context) {
    #[cfg(target_os = "android")]
    if !*OPEN.lock() { return; }
    
    let mode = &mut *MODE.lock();
    window!("Bott").frame(egui::Frame::window(&ctx.style()).stroke(egui::Stroke::new(
        ctx.style().visuals.window_stroke.width,
        match mode {
            Standby => ctx.style().visuals.window_stroke.color,
            Record => egui::Color32::from_rgb(220, 38, 38).lerp_to_gamma(
                ctx.style().visuals.window_stroke.color,
                (0.5 * f32::sin(
                    (SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis() % 1000) as f32 /
                    (500.0 / core::f32::consts::PI)
                ) + 0.5).clamp(0.0, 1.0)
            ),
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
                    ui.text_edit_singleline(&mut *REPLAY_FILE_NAME.lock());
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            let _ = bot::save_replay(bot::REPLAY_DIR.join(REPLAY_FILE_NAME.lock().clone())).map_err(|e|
                                gd::geode::log::error(e.to_string())
                            );
                        }
                        if ui.button("Load").clicked() {
                            let _ = bot::load_replay(bot::REPLAY_DIR.join(REPLAY_FILE_NAME.lock().clone())).map_err(|e|
                                gd::geode::log::error(e.to_string())
                            );
                        }
                        if ui.button("Clear").clicked() {
                            std::thread::spawn(move || RECORDED_INPUTS.lock().clear());
                        }
                    });
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