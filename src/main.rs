use std::{process::Command, time::Duration};
use windows::{
    core::PCWSTR,
    Win32::Foundation::{HINSTANCE, HWND},
    Win32::UI::Shell::ShellExecuteW,
    Win32::UI::WindowsAndMessaging::SW_SHOW,
};
fn main() {
    #[cfg(target_os = "windows")]
    request_admin_privileges();

    #[cfg(not(target_os = "windows"))]
    {
        println!("Application only runs on windwos currently!");
        std::process::exit(0);
    }

    println!("Running with elevated privileges!");

    //let gpu_name = std::env::args().nth(1).expect("No GPU name provided");
    let gpu_name = std::env::args().nth(1);

    if gpu_name.is_none() {
        println!("No GPU name provided");
        println!("Press enter to exit");

        std::io::stdin().read_line(&mut String::new()).unwrap();
        std::process::exit(0);
    }

    let gpu_name = gpu_name.unwrap();

    loop {
        let get_power_command = Command::new("powershell")
            .arg("-Command")
            .arg(r#"(Get-WmiObject -Class BatteryStatus -Namespace "root\wmi").PowerOnline"#)
            .spawn()
            .expect("Failed to execute command");

        let power_status = get_power_command
            .wait_with_output()
            .expect("Failed to get power status");

        let power_status = power_status.stdout == "True".as_bytes();
        println!("Power status: {}", power_status);

        if power_status {
            // Disable NVIDIA GPU
            Command::new("powershell")
                .arg("-Command")
                .arg(format!("Get-PnpDevice | Where-Object {{ $_.FriendlyName -like \"*{}*\" }} | Disable-PnpDevice -Confirm:$false", gpu_name))                
                .spawn()
                .expect("Failed to execute command");
        } else {
            // Enable NVIDIA GPU
            Command::new("powershell")
                .arg("-Command")
                // Use this line instead of the one below Get-PnpDevice | Where-Object { $_.FriendlyName -like "*$gpuName*" } | Enable-PnpDevice -Confirm:$false}
                // where gpuName is gpu_name from above
                .arg(format!("Get-PnpDevice | Where-Object {{ $_.FriendlyName -like \"*{}*\" }} | Enable-PnpDevice -Confirm:$false", gpu_name))                
                .spawn()
                .expect("Failed to execute command");
        }

        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}

/* fn request_admin_privileges() {
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

        // Attempt to relaunch the process with elevated privileges
        let result = unsafe {
            ShellExecuteW(
                HWND::default(),
                PCWSTR(runas_wide.as_ptr()),
                PCWSTR(exe_path_wide.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOW,
            )
        };

        // Check if the operation was successful
        if result.0 as isize <= 32 {
            panic!("Failed to request admin privileges.");
        }

        // Exit the current process as it will be relaunched
        std::process::exit(0);
    }
} */

fn request_admin_privileges() {
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

        // Attempt to relaunch the process with elevated privileges and pass in env vars
        let result = Command::new(exe_path_str)
            .env("RUNAS_ADMIN", "1")
            .spawn()
            .expect("Failed to request admin privileges.");

        let result = unsafe {
            ShellExecuteW(
                HWND::default(),                // Parent window handle (None)
                PCWSTR(runas_wide.as_ptr()),    // Verb ("runas" for elevation)
                PCWSTR(exe_path_wide.as_ptr()), // Path to the executable
                PCWSTR::null(),                 // Parameters (None)
                PCWSTR::null(),                 // Directory (None)
                SW_SHOW,                        // Show window flag
            )
        };

        // Check if the operation was successful (HINSTANCE is a pointer, compare against 32)
        if result.0 as isize <= 32 {
            panic!("Failed to request admin privileges.");
        }

        // If ShellExecuteW succeeds, the original process should exit
        std::process::exit(0);
    }
}
