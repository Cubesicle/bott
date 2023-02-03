use std::ffi::c_void;
use windows::Win32::{ 
    Foundation::HINSTANCE,
    System::LibraryLoader::FreeLibraryAndExitThread,
    System::SystemServices::*,
};

fn main_thread(hinst_dll: HINSTANCE) {
    println!("yippee!!");
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
