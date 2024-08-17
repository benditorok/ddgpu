use std::{process::Command, time::Duration};
use windows::{
    core::PCWSTR,
    Win32::Foundation::{HINSTANCE, HWND},
    Win32::UI::Shell::ShellExecuteW,
    Win32::UI::WindowsAndMessaging::SW_SHOW,
};

pub fn request_admin_privileges() {
    // Check if the program is running with admin rights
    let is_admin = Command::new("net")
        .arg("session")
        .output()
        .expect("Failed to check admin status")
        .status
        .success();

    if !is_admin {
        // Get the path of the current executable
        let exe_path = std::env::current_exe().unwrap();
        let exe_path_str = exe_path.to_str().unwrap();

        // Convert the executable path and "runas" verb to wide strings
        let exe_path_wide: Vec<u16> = exe_path_str
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let runas_wide: Vec<u16> = "runas".encode_utf16().chain(std::iter::once(0)).collect();

        // Get the current command-line arguments
        let args: Vec<String> = std::env::args().skip(1).collect();
        let args_wide: Vec<u16> = args
            .join(" ")
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        // Attempt to relaunch the process with elevated privileges
        let result = unsafe {
            ShellExecuteW(
                HWND::default(),
                PCWSTR(runas_wide.as_ptr()), // Verb ("runas" for elevation)
                PCWSTR(exe_path_wide.as_ptr()), // Path to the executable
                PCWSTR(args_wide.as_ptr()),  // Pass the arguments
                PCWSTR::null(),              // Directory (None)
                SW_SHOW,                     // Show window flag
            )
        };

        // Check if the operation was successful (HINSTANCE is a pointer, compare against 32)
        if result.0 as isize <= 32 {
            panic!("Failed to request admin privileges.");
        }

        // Exit the current process as it will be relaunched
        std::process::exit(0);
    }
}
