mod debug;
mod record;
mod replay;

pub use debug::Debug;
pub use record::Record;
pub use replay::Replay;

pub trait Page {
    fn ui(&mut self, ui: &mut egui::Ui);
}

#[derive(Eq, Hash, PartialEq)]
pub enum Pages {
    Record,
    Replay,
    Debug,
}