pub mod bot;
pub mod errbox;
pub mod gd;
pub mod gui;
pub mod hooks;

use std::ffi::c_void;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use lazy_static::lazy_static;
use log::info;
use windows::core::{w, HSTRING};
use windows::Win32::Foundation::{HINSTANCE, MAX_PATH};
use windows::Win32::System::LibraryLoader::{FreeLibraryAndExitThread, GetModuleFileNameW};
use windows::Win32::System::SystemServices::*;
use windows::Win32::UI::Shell::StrCmpW;

lazy_static! {
    static ref EXE_PATH: PathBuf = {
        let mut buf = [0; MAX_PATH as usize];
        unsafe { GetModuleFileNameW(None, &mut buf) };
        PathBuf::from(String::from_utf16(&buf).unwrap().trim().to_string())
    };
    static ref EXITING: AtomicBool = AtomicBool::new(false);
}

fn main_thread(hinst_dll: HINSTANCE) {
    if is_gd() {
        std::env::set_var("RUST_LOG", "trace");
        pretty_env_logger::init_timed();
        info!("geometey dahs found!!1");

        unsafe { gui::GUI.init() };
        hooks::load().unwrap();

        while EXITING.load(Ordering::Relaxed) == false {}

        hooks::unload().unwrap();
    } else {
        errbox!("This is not Geometry Dash.");
    }

    unload(hinst_dll);
}

fn is_gd() -> bool {
    EXE_PATH
        .file_name()
        .map(|s| unsafe { StrCmpW(&HSTRING::from(s), w!("GeometryDash.exe")) } == 0)
        .unwrap_or(false)
}

fn unload(hinst_dll: HINSTANCE) {
    info!("Unloading Bott.");
    unsafe { FreeLibraryAndExitThread(hinst_dll, 0) };
}

#[no_mangle]
extern "stdcall" fn DllMain(hinst_dll: HINSTANCE, reason: u32, _: *mut c_void) -> bool {
    match reason {
        DLL_PROCESS_ATTACH => {
            let orig_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |panic_info| {
                orig_hook(panic_info);
                errbox!(panic_info);
                let _ = hooks::unload();
                unload(hinst_dll);
            }));

            std::thread::spawn(move || main_thread(hinst_dll));

            true
        }
        DLL_PROCESS_DETACH => true,
        _ => false,
    }
}
