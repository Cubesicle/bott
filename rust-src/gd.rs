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
        
        macro_rules! define_logging_fn {
            ($wrapper_fn_name:ident, $original_fn_name:ident) => {
                pub fn $wrapper_fn_name<S: AsRef<str>>(string: S) {
                    let s = CString::new(string.as_ref()).unwrap_or_default().into_raw();
                    unsafe { $original_fn_name(s) };
                }
            };
        }
        
        define_logging_fn!(debug, log_debug);
        define_logging_fn!(info, log_info);
        define_logging_fn!(warn, log_warn);
        define_logging_fn!(error, log_error);
    }
}

#[derive(Clone, Copy, Display, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[repr(C)]
pub enum PlayerButton {
    Jump = 1,
    Left = 2,
    Right = 3,
}