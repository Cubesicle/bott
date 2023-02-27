use egui::Ui;
use egui_opengl_internal::OpenGLApp;
use std::collections::HashMap;
use windows::Win32::UI::Input::KeyboardAndMouse::{VIRTUAL_KEY, VK_RSHIFT, GetAsyncKeyState};

#[derive(PartialEq)]
enum Page {
    Record,
    Replay,
    Debug,
}

#[derive(Eq, Hash, PartialEq)]
enum Keybind {
    ToggleGUI
}

pub struct RBotGUI {
    open: bool,
    page: Page,
    keybinds: Option<HashMap<Keybind, VIRTUAL_KEY>>
}

impl RBotGUI {
    pub const fn new() -> Self {
        RBotGUI {
            open: true,
            page: Page::Record,
            keybinds: None,
        }
    }

    pub fn init(&mut self) {
        self.keybinds = Some(HashMap::from([
            (Keybind::ToggleGUI, VK_RSHIFT)
        ]));
    }

    pub fn name(&self) -> &'static str { "rBot" }

    pub fn show(&mut self, ctx: &egui::Context, _: &mut i32) {
        egui::containers::Window::new(self.name()).open(&mut self.open).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.page,
                    Page::Record,
                    "Record"
                );
                ui.selectable_value(
                    &mut self.page,
                    Page::Replay,
                    "Replay"
                );
                ui.selectable_value(
                    &mut self.page,
                    Page::Debug,
                    "Debug"
                );
                ui.menu_button("Eject", |ui| {
                    ui.label("Do you want to eject the DLL?");
                    if ui.button("No").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Yes").clicked() {                
                        unsafe { crate::EXITING = true; }
                    }
                });
            });
            ui.separator();
            match self.page {
                Page::Record => Self::record_page(ui),
                Page::Replay => Self::replay_page(ui),
                Page::Debug => Self::debug_page(ui),
            }
            ui.separator();
            ui.vertical_centered(|ui| {
                ui.label("Made with ❤ by Cubesicle.");
            });
        });
    }

    pub fn detect_keybinds(&mut self) {
        unsafe {
            let key = self.keybinds.as_ref().unwrap().get(&Keybind::ToggleGUI).unwrap().0 as i32;
            if GetAsyncKeyState(key) & 0x01 == 1 {
                self.open = !self.open;
            }
        }
    }

    fn record_page(ui: &mut Ui) {
        ui.heading("Record");
    }

    fn replay_page(ui: &mut Ui) {
        ui.heading("Replay");
    }

    fn debug_page(ui: &mut Ui) {
        ui.heading("Debug");
    }
}

pub static mut APP: OpenGLApp<i32> = OpenGLApp::new();
pub static mut GUI: RBotGUI = RBotGUI::new();