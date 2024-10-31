use parking_lot::Mutex;
use strum_macros::Display;

#[derive(Display, Eq, PartialEq)]
pub enum Mode {
    Standby,
    Record,
    Replay,
}

pub static MODE: Mutex<Mode> = Mutex::new(Mode::Standby);