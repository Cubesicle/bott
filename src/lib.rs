use std::ffi::c_void;
use windows::Win32::{ Foundation::HINSTANCE, System::SystemServices::* };

#[no_mangle]
pub extern "stdcall" fn DllMain(_: HINSTANCE, reason: u32, _: *mut c_void) -> bool {
    match reason {
        DLL_PROCESS_ATTACH => {
            unsafe {
                windows::Win32::System::Console::AllocConsole();
            }
            println!("dll has been injected!!!!");
            true
        },
        DLL_PROCESS_DETACH => true,
        _ => false
    }
}
