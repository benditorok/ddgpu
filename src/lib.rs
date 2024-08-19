pub mod windows_service;

pub mod app {
    use std::error::Error;
    use std::{collections::HashMap, env};

    pub const GPU_NAME: &str = "--name";
    pub const HIDE_WINDOW: &str = "--hide";
    pub const RUN_AS_SERVICE: &str = "--as-service";

    pub struct Arguments<'keys> {
        pub gpu_name: &'keys str,
        pub hide_window: &'keys str,
        pub run_as_service: &'keys str,
    }

    impl<'keys> Arguments<'keys> {
        // Collect the command-line arguments
        pub fn parse_args(&self) -> Result<HashMap<&'keys str, String>, Box<dyn Error>> {
            let args_raw: Vec<String> = env::args().skip(1).collect();
            let mut args_collected: HashMap<&str, String> = HashMap::new();
            let mut last_switch_key: &str = "";

            for arg in args_raw.iter() {
                match arg.trim() {
                    i if i == self.gpu_name => last_switch_key = self.gpu_name,
                    i if i == self.hide_window => last_switch_key = self.hide_window,
                    i if i == self.run_as_service => last_switch_key = self.run_as_service,
                    _ => {
                        if !last_switch_key.is_empty() {
                            let old_args = args_collected.get(last_switch_key);

                            if let Some(old_args) = old_args {
                                args_collected
                                    .insert(last_switch_key, [old_args, arg.trim()].join(" "));
                            } else {
                                args_collected.insert(last_switch_key, arg.trim().to_string());
                            }
                        }
                    }
                }
            }

            Ok(args_collected)
        }
    }
}

#[cfg(target_os = "windows")]
pub mod on_windows {
    use super::app;
    use std::error::Error;
    use std::{
        collections::HashMap,
        env,
        process::Command,
        thread,
        time::{self, Duration},
    };
    use windows::Win32::System::Console::GetConsoleWindow;
    use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, HIDE_WINDOW, SW_HIDE};
    use windows::{
        core::PCWSTR,
        Win32::Foundation::{HINSTANCE, HWND},
        Win32::UI::Shell::ShellExecuteW,
        Win32::UI::WindowsAndMessaging::SW_SHOW,
    };
    pub fn request_admin_privileges() -> Result<(), Box<dyn Error>> {
        // Check if the program is running with admin rights
        let is_admin = Command::new("net")
            .arg("session")
            .output()
            .expect("Failed to check admin status")
            .status
            .success();

        if !is_admin {
            // Get the path of the current executable
            let exe_path = env::current_exe()?;
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

        Ok(())
    }

    fn hide_console_window() {
        // Get the handle to the console window
        let window: HWND = unsafe { GetConsoleWindow() };

        // Check if the window handle is not null
        if window.0 as isize != 0 {
            // Hide the console window
            unsafe {
                ShowWindow(window, SW_HIDE);
            }
        }
    }

    pub fn run(args: &app::Arguments) -> Result<(), Box<dyn Error>> {
        // Check if the program is running with admin rights, if not, relaunch it as admin
        request_admin_privileges()?;
        println!("Running with elevated privileges!");

        let parsed_args = args.parse_args()?;
        let gpu_name = parsed_args.get(app::GPU_NAME);

        if let Some(gpu_name) = gpu_name {
            println!("GPU name: {}", gpu_name);
        } else {
            println!("No GPU name provided");
            println!("Press enter to exit");

            std::io::stdin().read_line(&mut String::new()).unwrap();
            std::process::exit(0);
        }

        let gpu_name = gpu_name.unwrap();

        let hide_window = parsed_args.get(app::HIDE_WINDOW);

        if let Some(hide_window) = hide_window {
            let hide_window = hide_window == "true";

            if hide_window {
                hide_console_window();
            }
        }

        loop {
            // Get the current power status
            let get_power_command = Command::new("powershell")
                .arg("-Command")
                .arg(r#"(Get-WmiObject -Class BatteryStatus -Namespace "root\wmi").PowerOnline"#)
                .output()
                .expect("Failed to execute command");
            let power_connected = std::str::from_utf8(&get_power_command.stdout)
                .expect("Failed to convert output to string")
                .trim();
            let power_connected = power_connected == "True";

            if power_connected {
                println!("AC power connected.");
            } else {
                println!("Running on battery.");
            }

            // Get the current status of the GPU
            let get_gpu_status_command = Command::new("powershell")
                .arg("-Command")
                .arg(format!("Get-PnpDevice | Where-Object {{ $_.FriendlyName -like \"{}\" }} | Select-Object -Property Status", gpu_name))
                .output()
                .expect("Failed to execute command");
            let gpu_status = std::str::from_utf8(&get_gpu_status_command.stdout)
                .expect("Failed to convert output to string")
                .trim();

            if gpu_status.contains("OK") {
                println!("The GPU is enabled.");
            } else if gpu_status.contains("Error") {
                println!("The GPU is disabled.");
            } else {
                return Err("Unable to determine the GPU status.".into());
            }

            let gpu_status = gpu_status.contains("OK");

            // Check if the power status has changed
            // GPU should only be enabled if the power is connected
            if gpu_status != power_connected {
                match power_connected {
                    false => {
                        println!("Disabling the GPU...");

                        Command::new("powershell")
                        .arg("-Command")
                        .arg(format!("Get-PnpDevice | Where-Object {{ $_.FriendlyName -like \"{}\" }} | Disable-PnpDevice -Confirm:$false", gpu_name))
                        .spawn()
                        .expect("Failed to execute command");
                    }
                    true => {
                        println!("Enabling the GPU...");

                        Command::new("powershell")
                            .arg("-Command")
                            .arg(format!("Get-PnpDevice | Where-Object {{ $_.FriendlyName -like \"{}\" }} | Enable-PnpDevice -Confirm:$false", gpu_name))
                            .spawn()
                            .expect("Failed to execute command");
                    }
                }
            }

            thread::sleep(time::Duration::from_secs(10));
        }
    }
}
