use egui_opengl_internal::OpenGLApp;

pub fn update(ctx: &egui::Context, _: &mut i32) {
    egui::containers::Window::new("RBot").show(ctx, |ui| {
        ui.heading("Hello World!");
        ui.separator();
        if ui.button("exit").clicked() {                
            unsafe { crate::EXITING = true; }
        }
    });
}

pub static mut APP: OpenGLApp<i32> = OpenGLApp::new();