pub mod errbox;
pub mod gui;
pub mod hooks;

use std::ffi::c_void;
use windows::core::PCWSTR;
use windows::w;
use windows::Win32::Foundation::{HINSTANCE, MAX_PATH};
use windows::Win32::System::LibraryLoader::{FreeLibraryAndExitThread, GetModuleFileNameW};
use windows::Win32::System::SystemServices::*;
use windows::Win32::UI::Shell::{PathFindFileNameW, StrCmpW};

pub static mut EXITING: bool = false;

fn main_thread(hinst_dll: HINSTANCE) {
    unsafe {
        windows::Win32::System::Console::AllocConsole(); // TODO: remove
    }

    if is_gd() {
        println!("geometey dahs found!!1");

        hooks::load().unwrap_or_else(|err| errbox!(err));

        unsafe { while EXITING == false { } }

        hooks::unload().unwrap_or_else(|err| errbox!(err));
        println!("hooks unloaded");
        std::thread::sleep(std::time::Duration::from_secs(1));
    } else {
        errbox!("This is not Geometry Dash.");
    }

    unsafe {
        windows::Win32::System::Console::FreeConsole();
        FreeLibraryAndExitThread(hinst_dll, 0);
    }
}

fn is_gd() -> bool {
    let mut file_path_utf16 = [0; MAX_PATH as usize];
    unsafe { GetModuleFileNameW(None, &mut file_path_utf16); }

    let file_path = PCWSTR::from_raw(file_path_utf16.as_ptr());
    let file_name = unsafe { PCWSTR::from_raw(PathFindFileNameW(file_path).as_ptr()) };

    unsafe { StrCmpW(w!("GeometryDash.exe"), file_name) == 0 }
}

#[no_mangle]
extern "system" fn DllMain(hinst_dll: HINSTANCE, reason: u32, _: *mut c_void) -> bool {
    match reason {
        DLL_PROCESS_ATTACH => {
            std::thread::spawn(move || main_thread(hinst_dll));
            true
        }
        DLL_PROCESS_DETACH => {
            println!("dll unloaded");
            true
        }
        _ => false,
    }
}
