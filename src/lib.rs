pub mod windows_service;

pub mod app {
    use std::error::Error;
    use std::{collections::HashMap, env};

    pub const GPU_NAME: &str = "--name";
    pub const HIDE_WINDOW: &str = "--hide";
    pub const RUN_AS_SERVICE: &str = "--as-service";

    #[derive(Debug, Clone, Default)]
    pub struct Arguments {
        parsed_args: Option<HashMap<String, String>>,
    }

    impl Arguments {
        // Collect the command-line arguments
        pub fn init(&mut self) -> Result<(), Box<dyn Error>> {
            let args_raw: Vec<String> = env::args().skip(1).collect();
            let mut args_collected: HashMap<String, String> = HashMap::new();
            let mut last_switch_key: String = String::new();

            for arg in args_raw.into_iter() {
                match arg.trim() {
                    switch if switch.starts_with("--") => last_switch_key = String::from(switch),
                    _ => {
                        if !last_switch_key.is_empty() {
                            let new_arg = arg.clone();
                            let old_args = args_collected.get(last_switch_key.as_str());

                            if let Some(old_args) = old_args {
                                args_collected.insert(
                                    last_switch_key.clone(),
                                    [old_args, new_arg.as_str()].join(" "),
                                );
                            } else {
                                args_collected.insert(last_switch_key.clone(), new_arg.to_string());
                            }
                        }
                    }
                }
            }

            self.parsed_args = Some(args_collected);
            Ok(())
        }

        // Returns a clone of the parsed args.
        pub fn get_parsed_args(&self) -> Option<HashMap<String, String>> {
            self.parsed_args.clone()
        }

        pub fn get_value(&self, key: &str) -> Option<String> {
            if let Some(parsed_args) = &self.parsed_args {
                parsed_args.get(key).cloned()
            } else {
                None
            }
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
                let _ = ShowWindow(window, SW_HIDE);
            }
        }
    }

    pub fn run(args: &app::Arguments) -> Result<(), Box<dyn Error>> {
        // Check if the program is running with admin rights, if not, relaunch it as admin
        request_admin_privileges()?;
        println!("Running with elevated privileges!");

        let gpu_name = args.get_value(app::GPU_NAME).unwrap_or_else(|| {
            println!("No GPU name provided");
            println!("Press enter to exit");

            std::io::stdin().read_line(&mut String::new()).unwrap();
            std::process::exit(0);
        });

        let hide_window = args.get_value(app::HIDE_WINDOW);

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
