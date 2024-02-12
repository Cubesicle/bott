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

use self::pages::Page;

mod pages;

#[derive(Eq, Hash, PartialEq)]
enum Keybind {
    ToggleGUI,
}

#[derive(Eq, PartialEq)]
enum SelectedPage {
    Record,
    Replay,
    Debug,
}

pub struct RBotGUI {
    open: bool,
    record_page: Option<pages::Record>,
    replay_page: Option<pages::Replay>,
    debug_page: Option<pages::Debug>,
    selected_page: SelectedPage,
    keybinds: Option<HashMap<Keybind, VIRTUAL_KEY>>,
    popup_open: bool,
    popup_anchored: bool,
    popup_text: Option<String>,
}

impl RBotGUI {
    const fn new() -> Self {
        Self {
            open: true,
            record_page: None,
            replay_page: None,
            debug_page: None,
            selected_page: SelectedPage::Record,
            keybinds: None,
            popup_open: false,
            popup_anchored: true,
            popup_text: None,
        }
    }
}

impl RBotGUI {
    pub fn init(&mut self) {
        self.record_page = Some(pages::Record::default());
        self.replay_page = Some(pages::Replay::default());
        self.debug_page = Some(pages::Debug::default());
        self.keybinds = Some(HashMap::from([(Keybind::ToggleGUI, VK_RSHIFT)]));
    }

    pub fn name(&self) -> &'static str {
        "Bott"
    }

    pub fn show(&mut self, ctx: &egui::Context, _: &mut i32) {
        egui::Window::new(self.name())
            .open(&mut self.open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.selected_page, SelectedPage::Record, "Record");
                    ui.selectable_value(&mut self.selected_page, SelectedPage::Replay, "Replay");
                    ui.selectable_value(&mut self.selected_page, SelectedPage::Debug, "Debug");
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
                    SelectedPage::Record => self.record_page.as_mut().map(|p| p.ui(ui)),
                    SelectedPage::Replay => self.replay_page.as_mut().map(|p| p.ui(ui)),
                    SelectedPage::Debug => self.debug_page.as_mut().map(|p| p.ui(ui)),
                };
                ui.separator();
                ui.vertical_centered(|ui| {
                    ui.label("Made with ❤ by Cubesicle.");
                });
            });

        let mut popup_window = egui::Window::new("Info")
            .open(&mut self.popup_open)
            .collapsible(false)
            .resizable(false);
        if self.popup_anchored {
            popup_window = popup_window.anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0));
        }
        popup_window.show(ctx, |ui| {
            ui.add(
                egui::Label::new(self.popup_text.as_ref().unwrap_or(&String::new()))
                    .selectable(false),
            );
            self.popup_anchored = false;
        });

        if !self.popup_open {
            self.popup_text = None;
        }
    }

    pub fn open_popup(&mut self, text: &str) {
        if self.popup_open {
            let new_text = self
                .popup_text
                .as_ref()
                .unwrap_or(&String::new())
                .to_owned()
                + "\n"
                + text;
            self.popup_text = Some(new_text);
        } else {
            self.popup_open = true;
            self.popup_text = Some(text.to_string());
        }
        self.popup_anchored = true;
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
}

pub static mut APP: OpenGLApp<i32> = OpenGLApp::new();
pub static mut GUI: RBotGUI = RBotGUI::new();
