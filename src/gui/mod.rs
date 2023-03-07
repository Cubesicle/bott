use egui_opengl_internal::OpenGLApp;
use std::{collections::HashMap, hash::Hash};
use windows::Win32::{UI::{Input::KeyboardAndMouse::{VIRTUAL_KEY, VK_RSHIFT, VK_RCONTROL, VK_SHIFT, MapVirtualKeyW, MAPVK_VSC_TO_VK_EX, VK_CONTROL, VK_MENU, VK_LCONTROL, VK_RMENU, VK_LMENU}, WindowsAndMessaging::{WM_KEYDOWN, KF_EXTENDED}}, Foundation::{WPARAM, LPARAM}};

mod pages;

#[derive(Eq, Hash, PartialEq)]
enum Keybinds {
    ToggleGUI
}

pub struct RBotGUI {
    open: bool,
    current_page: pages::Pages,
    pages: Option<HashMap<pages::Pages, Box<dyn pages::Page>>>,
    keybinds: Option<HashMap<Keybinds, VIRTUAL_KEY>>,
}

impl RBotGUI {
    pub const fn new() -> Self {
        Self {
            open: true,
            current_page: pages::Pages::Record,
            pages: None,
            keybinds: None,
        }
    }

    pub fn init(&mut self) {
        self.pages = Some(HashMap::new());
        self.pages.as_mut().unwrap().insert(pages::Pages::Record, Box::new(pages::Record::default()));
        self.pages.as_mut().unwrap().insert(pages::Pages::Replay, Box::new(pages::Replay::default()));
        self.pages.as_mut().unwrap().insert(pages::Pages::Debug, Box::new(pages::Debug::default()));

        self.keybinds = Some(HashMap::from([
            (Keybinds::ToggleGUI, VK_RSHIFT)
        ]));
    }

    pub fn name(&self) -> &'static str { "rBot" }

    pub fn show(&mut self, ctx: &egui::Context, _: &mut i32) {
        egui::Window::new(self.name())
            .open(&mut self.open)
            .show(ctx, |ui| {
                egui::TopBottomPanel::top("header").show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut self.current_page,
                            pages::Pages::Record,
                            "Record"
                        );
                        ui.selectable_value(
                            &mut self.current_page,
                            pages::Pages::Replay,
                            "Replay"
                        );
                        ui.selectable_value(
                            &mut self.current_page,
                            pages::Pages::Debug,
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
                    ui.add_space(2.0);
                });
                egui::TopBottomPanel::bottom("footer").min_height(0.0).show_inside(ui, |ui| {
                    ui.add_space(8.0);
                    ui.vertical_centered(|ui| {
                        ui.label("Made with ❤ by Cubesicle.");
                    });
                });
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    egui::ScrollArea::both().show(ui, |ui| {
                        match &self.current_page {
                            page => {
                                self.pages.as_mut().unwrap().get_mut(page).unwrap().ui(ui);
                            }
                        }
                    });
                });
            });
    }

    pub fn handle_keydown(&mut self, msg: u32, wparam: WPARAM, lparam: LPARAM) {
        if msg != WM_KEYDOWN { return; }

        let vk_code: u16 = {
            let vk_code = (wparam.0 & 0xffff) as u16;

            let key_flags = ((lparam.0 >> 16) & 0xffff) as u16;

            let scan_code = (key_flags & 0xff) as u8;
            let is_extended_key = (key_flags as u32 & KF_EXTENDED) == KF_EXTENDED;

            if vk_code == VK_SHIFT.0 {
                unsafe { (MapVirtualKeyW(scan_code as u32, MAPVK_VSC_TO_VK_EX) & 0xffff) as u16 }
            } else if vk_code == VK_CONTROL.0 {
                if is_extended_key { VK_RCONTROL.0 } else { VK_LCONTROL.0 }
            } else if vk_code == VK_MENU.0 {
                if is_extended_key { VK_RMENU.0 } else { VK_LMENU.0 }
            } else {
                vk_code
            }
        };

        let key = self.keybinds.as_ref().unwrap().get(&Keybinds::ToggleGUI).unwrap().0;
        if vk_code == key {
            self.open = !self.open;
        }
    }
}

pub static mut APP: OpenGLApp<i32> = OpenGLApp::new();
pub static mut GUI: RBotGUI = RBotGUI::new();
