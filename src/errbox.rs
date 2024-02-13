#[macro_export]
macro_rules! errbox {
    ($s:expr) => {
        unsafe {
            windows::Win32::UI::WindowsAndMessaging::MessageBoxW(
                None,
                &windows::core::HSTRING::from(format!("{}", $s)),
                windows::core::w!("Error"),
                windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR,
            )
        }
    };
}
