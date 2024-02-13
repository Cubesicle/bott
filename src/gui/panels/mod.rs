mod debug;
mod load;
mod save;
mod settings;

pub use debug::Debug;
use egui::Ui;
pub use load::Load;
pub use save::Save;
pub use settings::Settings;

pub trait Panel: Default {
    fn ui(&mut self, ui: &mut Ui);
}
