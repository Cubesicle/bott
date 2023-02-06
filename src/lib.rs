use std::ffi::c_void;
use windows::w;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HINSTANCE, MAX_PATH };
use windows::Win32::System::LibraryLoader::{FreeLibraryAndExitThread, GetModuleFileNameW };
use windows::Win32::System::SystemServices::*;
use windows::Win32::UI::Shell::{ PathFindFileNameW, StrCmpW };
use windows::Win32::UI::WindowsAndMessaging::{ MB_ICONERROR, MessageBoxW };

fn main_thread(hinst_dll: HINSTANCE) {
    let mut file_path_utf16 = [0; MAX_PATH as usize];
    unsafe { GetModuleFileNameW(None, &mut file_path_utf16); }

    let file_path = PCWSTR::from_raw(file_path_utf16.as_ptr());
    let file_name = unsafe { PCWSTR::from_raw(PathFindFileNameW(file_path).as_ptr()) };

    let is_gd = unsafe { StrCmpW(w!("GeometryDash.exe"), file_name) == 0 };
    if is_gd {
        println!("geometey dahs found!!1");
    } else {
        unsafe { MessageBoxW(None, w!("This is not Geometry Dash."), w!("wtf!!!"), MB_ICONERROR); }
    }

    unsafe { FreeLibraryAndExitThread(hinst_dll, 0); }
}

#[no_mangle]
extern "system" fn DllMain(hinst_dll: HINSTANCE, reason: u32, _: *mut c_void) -> bool {
    match reason {
        DLL_PROCESS_ATTACH => {
            unsafe { windows::Win32::System::Console::AllocConsole(); } // TODO remove
            std::thread::spawn(move || { main_thread(hinst_dll); });
            true
        },
        DLL_PROCESS_DETACH => true,
        _ => false
    }
}
