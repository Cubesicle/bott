use serde::{Deserialize, Serialize};
use strum_macros::Display;

mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod geode {
    use std::{ffi::CStr, path::PathBuf, sync::LazyLock};
    use super::bindings::get_save_dir;

    pub static SAVE_DIR: LazyLock<PathBuf> = LazyLock::new(||
        PathBuf::from(unsafe { CStr::from_ptr(get_save_dir()) }.to_str().unwrap_or_default())
    );

    pub mod log {
        #![allow(dead_code)]
    
        use std::ffi::CString;
        use super::super::bindings::{log_debug, log_info, log_warn, log_error};
        
        pub fn debug(string: String) {
            let s = CString::new(string).unwrap_or_default().into_raw();
            unsafe { log_debug(s); }
        }
        
        pub fn info(string: String) {
            let s = CString::new(string).unwrap_or_default().into_raw();
            unsafe { log_info(s); }
        }
        
        pub fn warn(string: String) {
            let s = CString::new(string).unwrap_or_default().into_raw();
            unsafe { log_warn(s); }
        }
        
        pub fn error(string: String) {
            let s = CString::new(string).unwrap_or_default().into_raw();
            unsafe { log_error(s); }
        }
    }
}

#[derive(Clone, Copy, Display, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[repr(C)]
pub enum PlayerButton {
    Jump = 1,
    Left = 2,
    Right = 3,
}