use std::{ffi::c_void, mem::transmute};

#[no_mangle]
pub extern "C" fn run_fn(ctx: *const c_void) {
    let ctx = unsafe { transmute::<_, &egui::Context>(ctx) };
    egui::Window::new("Bott").show(ctx, |ui| {
        ui.style_mut().interaction.selectable_labels = false;

        ui.label("it works ig");
    });
}