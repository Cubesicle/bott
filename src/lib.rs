pub mod bot;
pub mod errbox;
pub mod gd;
pub mod gui;
pub mod hooks;

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};

use lazy_static::lazy_static;
use log::info;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, MAX_PATH};
use windows::Win32::System::LibraryLoader::{
    FreeLibraryAndExitThread, GetModuleFileNameW,
};
use windows::Win32::System::SystemServices::*;
use windows::Win32::UI::Shell::{PathFindFileNameW, StrCmpW};

lazy_static! {
    static ref EXITING: AtomicBool = AtomicBool::new(false);
}

fn main_thread(hinst_dll: HINSTANCE) {
    if is_gd() {
        std::env::set_var("RUST_LOG", "trace");
        unsafe {
            windows::Win32::System::Console::AllocConsole();
        }
        pretty_env_logger::init_timed();
        //egui_logger::init().unwrap();
        info!("geometey dahs found!!1");

        gui::GUI.write().unwrap().init();
        hooks::load().unwrap();

        while EXITING.load(Ordering::Relaxed) == false {}

        hooks::unload().unwrap();
    } else {
        errbox!("This is not Geometry Dash.");
    }

    unload(hinst_dll);
}

fn is_gd() -> bool {
    let mut file_path_utf16 = [0; MAX_PATH as usize];
    unsafe { GetModuleFileNameW(None, &mut file_path_utf16) };

    let file_path = PCWSTR::from_raw(file_path_utf16.as_ptr());
    let file_name =
        unsafe { PCWSTR::from_raw(PathFindFileNameW(file_path).as_ptr()) };

    unsafe { StrCmpW(w!("GeometryDash.exe"), file_name) == 0 }
}

fn unload(hinst_dll: HINSTANCE) {
    info!("Unloading rBot.");
    unsafe { FreeLibraryAndExitThread(hinst_dll, 0) };
}

#[no_mangle]
extern "stdcall" fn DllMain(
    hinst_dll: HINSTANCE,
    reason: u32,
    _: *mut c_void,
) -> bool {
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
