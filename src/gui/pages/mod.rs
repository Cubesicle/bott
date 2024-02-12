mod debug;
mod record;
mod replay;

pub use debug::Debug;
use egui::Ui;
pub use record::Record;
pub use replay::Replay;

pub trait Page: Default {
    fn ui(&mut self, ui: &mut Ui);
}
