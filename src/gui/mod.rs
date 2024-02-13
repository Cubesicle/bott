use std::collections::HashMap;
use std::hash::Hash;
use std::sync::atomic::Ordering;

use egui_opengl_internal::OpenGLApp;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    MapVirtualKeyW, MAPVK_VSC_TO_VK_EX, VIRTUAL_KEY, VK_CONTROL, VK_LCONTROL, VK_LMENU, VK_MENU,
    VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{KF_EXTENDED, WM_KEYDOWN};

use self::panels::Panel;

mod panels;

#[derive(Eq, Hash, PartialEq)]
enum Keybind {
    ToggleGUI,
}

#[derive(Eq, PartialEq)]
enum Page {
    //Record,
    //Replay,
    Main,
    Debug,
}

pub struct RBotGUI {
    open: bool,
    settings_panel: Option<panels::Settings>,
    record_panel: Option<panels::Save>,
    replay_panel: Option<panels::Load>,
    debug_panel: Option<panels::Debug>,
    selected_page: Page,
    keybinds: Option<HashMap<Keybind, VIRTUAL_KEY>>,
}

impl RBotGUI {
    const fn new() -> Self {
        Self {
            open: true,
            settings_panel: None,
            record_panel: None,
            replay_panel: None,
            debug_panel: None,
            selected_page: Page::Main,
            keybinds: None,
        }
    }
}

impl RBotGUI {
    pub fn init(&mut self) {
        self.settings_panel = Some(panels::Settings::default());
        self.record_panel = Some(panels::Save::default());
        self.replay_panel = Some(panels::Load::default());
        self.debug_panel = Some(panels::Debug::default());
        self.keybinds = Some(HashMap::from([(Keybind::ToggleGUI, VK_RSHIFT)]));
    }

    pub fn name(&self) -> &'static str {
        "Bott"
    }

    pub fn show(&mut self, ctx: &egui::Context, _: &mut i32) {
        egui::Window::new(self.name())
            .default_size(egui::vec2(0.0, 0.0))
            .open(&mut self.open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    //ui.selectable_value(&mut self.selected_page, SelectedPage::Record, "Record");
                    //ui.selectable_value(&mut self.selected_page, SelectedPage::Replay, "Replay");
                    ui.selectable_value(&mut self.selected_page, Page::Main, "Main");
                    ui.selectable_value(&mut self.selected_page, Page::Debug, "Debug");
                    ui.menu_button("Eject", |ui| {
                        ui.label("Do you want to eject the DLL?");
                        if ui.button("No").clicked() {
                            ui.close_menu();
                        }
                        if ui.button("Yes").clicked() {
                            crate::EXITING.store(true, Ordering::Relaxed);
                        }
                    });
                });
                ui.separator();
                match self.selected_page {
                    Page::Main => {
                        self.settings_panel.as_mut().map(|p| p.ui(ui));
                        ui.separator();
                        self.record_panel.as_mut().map(|p| p.ui(ui));
                        ui.separator();
                        self.replay_panel.as_mut().map(|p| p.ui(ui));
                    }
                    Page::Debug => {
                        self.debug_panel.as_mut().map(|p| p.ui(ui));
                    }
                }
                ui.separator();
                ui.vertical_centered(|ui| {
                    ui.label("Made with ❤ by Cubesicle.");
                });
            });
    }

    pub fn handle_keydown(&mut self, msg: u32, wparam: WPARAM, lparam: LPARAM) {
        if msg != WM_KEYDOWN {
            return;
        }

        let vk_code: u16 = {
            let vk_code = (wparam.0 & 0xffff) as u16;

            let key_flags = ((lparam.0 >> 16) & 0xffff) as u16;

            let scan_code = (key_flags & 0xff) as u8;
            let is_extended_key = (key_flags as u32 & KF_EXTENDED) == KF_EXTENDED;

            if vk_code == VK_SHIFT.0 {
                unsafe { (MapVirtualKeyW(scan_code as u32, MAPVK_VSC_TO_VK_EX) & 0xffff) as u16 }
            } else if vk_code == VK_CONTROL.0 {
                if is_extended_key {
                    VK_RCONTROL.0
                } else {
                    VK_LCONTROL.0
                }
            } else if vk_code == VK_MENU.0 {
                if is_extended_key {
                    VK_RMENU.0
                } else {
                    VK_LMENU.0
                }
            } else {
                vk_code
            }
        };

        let key = self
            .keybinds
            .as_ref()
            .unwrap()
            .get(&Keybind::ToggleGUI)
            .unwrap()
            .0;
        if vk_code == key {
            self.open = !self.open;
        }
    }

    pub fn show_message_tooltip(
        ui: &mut egui::Ui,
        message: &mut Option<String>,
        widget_response: &egui::Response,
    ) {
        match message {
            Some(s) if widget_response.hovered() => {
                egui::popup::show_tooltip(ui.ctx(), widget_response.id, |ui| {
                    ui.label(s.as_str());
                });
            }
            Some(_) if !widget_response.hovered() => {
                *message = None;
            }
            _ => {}
        }
    }
}

pub static mut APP: OpenGLApp<i32> = OpenGLApp::new();
pub static mut GUI: RBotGUI = RBotGUI::new();
