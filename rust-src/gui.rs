use std::{collections::LinkedList, thread, time::SystemTime};
use anyhow::Result;
use parking_lot::Mutex;
use crate::{bot::{self, Mode::{Record, Replay, Standby}}, gd};

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
static REPLAY_FILE_LIST: Mutex<LinkedList<String>> = Mutex::new(LinkedList::new());

pub fn run(ctx: &egui::Context) {
    #[cfg(target_os = "android")]
    if !*OPEN.lock() { return; }
    
    let bot_mode = &mut *bot::MODE.lock();
    window!("Bott").frame(egui::Frame::window(&ctx.style()).stroke(egui::Stroke::new(
        ctx.style().visuals.window_stroke.width,
        match bot_mode {
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
            ui.selectable_value(bot_mode, Standby, Standby.to_string());
            ui.selectable_value(bot_mode, Record, Record.to_string());
            ui.selectable_value(bot_mode, Replay, Replay.to_string());
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
                    let replay_file_name = &mut *REPLAY_FILE_NAME.lock();
                    let replay_file_input = ui.add(egui::TextEdit::singleline(replay_file_name).hint_text("File name"));
                    let file_chooser_id = ui.make_persistent_id("replay_file_chooser");
                    if replay_file_input.gained_focus() {
                        populate_replay_file_list();
                        ui.memory_mut(|mem| mem.toggle_popup(file_chooser_id));
                    }
                    if replay_file_input.changed() {
                        clear_replay_file_list();
                        populate_replay_file_list();
                    }
                    if let Some(replay_file_list) = REPLAY_FILE_LIST.try_lock() {
                        if !replay_file_list.is_empty() {
                            if !ui.memory(|mem| mem.is_popup_open(file_chooser_id)) { clear_replay_file_list(); }
                        }
                    }
                    egui::popup::popup_below_widget(ui, file_chooser_id, &replay_file_input, egui::PopupCloseBehavior::CloseOnClick, |ui| {
                        ui.style_mut().interaction.selectable_labels = false;
                        ui.set_width(replay_file_input.rect.width() - egui::Frame::popup(ui.style()).total_margin().sum().x);
                        egui::containers::ScrollArea::vertical().show(ui, |ui| {
                            if let Some(replay_file_list) = REPLAY_FILE_LIST.try_lock() {
                                if !replay_file_list.is_empty() {
                                    for name in &*replay_file_list {
                                        if ui.button(name).clicked() {
                                            *replay_file_name = name.to_string();
                                        }
                                    }
                                } else {
                                    ui.label("Nothing to see here...");
                                }
                            } else {
                                ui.label("Loading...");
                            }
                        });
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            let _ = bot::save_replay(&bot::REPLAY_DIR.join(&replay_file_name)).map_err(|e|
                                gd::geode::log::error(e.to_string())
                            );
                        }
                        if ui.button("Load").clicked() {
                            let _ = bot::load_replay(&bot::REPLAY_DIR.join(&replay_file_name)).map_err(|e|
                                gd::geode::log::error(e.to_string())
                            );
                        }
                        if ui.button("Clear").clicked() {
                            bot::clear_inputs();
                        }
                        if ui.button("Open folder").clicked() {
                            let _ = opener::open(&*bot::REPLAY_DIR);
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

fn populate_replay_file_list() {
    thread::spawn(move || -> Result<()> {
        let replay_file_list = &mut *REPLAY_FILE_LIST.lock();
        for entry in std::fs::read_dir(&*bot::REPLAY_DIR)? {
            if let Ok(entry) = entry {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.contains(&*REPLAY_FILE_NAME.lock()) { replay_file_list.push_back(name); }
                }
            }
        }
        Ok(())
    });
}

fn clear_replay_file_list() {
    thread::spawn(move || REPLAY_FILE_LIST.lock().clear());
}