#[cfg(target_os = "windows")]
pub mod on_windows {
    use std::{collections::HashMap, env, fmt::Error, process::Command, time::Duration};
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

    pub fn run() -> Result<(), Error> {
        // Check if the program is running with admin rights, if not, relaunch it as admin
        request_admin_privileges();
        println!("Running with elevated privileges!");

        // Collect the command-line arguments
        let args_raw: Vec<String> = env::args().skip(1).collect();
        let mut args_collected: HashMap<&str, String> = HashMap::new();
        let mut last_switch_key: &str = "";

        for arg in args_raw.iter() {
            match arg.trim() {
                "--name" => {
                    last_switch_key = "--name";
                }
                _ => {
                    if !last_switch_key.is_empty() {
                        let mut old_args = args_collected.get(last_switch_key);

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

        let gpu_name = args_collected.get("--name");

        if let Some(gpu_name) = gpu_name {
            println!("GPU name: {}", gpu_name);
        } else {
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
                .output()
                .expect("Failed to execute command");

            // Convert the output bytes to a string
            let power_connected = std::str::from_utf8(&get_power_command.stdout)
                .expect("Failed to convert output to string")
                .trim();

            // Print the parsed output
            println!("power_connected: {}", power_connected);

            //let power_status = power_status.stdout == [1] as [u8; 1];
            match power_connected {
                "False" => {
                    Command::new("powershell")
                .arg("-Command")
                .arg(format!("Get-PnpDevice | Where-Object {{ $_.FriendlyName -like \"{}\" }} | Disable-PnpDevice -Confirm:$false", gpu_name))
                .spawn()
                .expect("Failed to execute command");
                }
                "True" => {
                    Command::new("powershell")
            .arg("-Command")
            // Use this line instead of the one below Get-PnpDevice | Where-Object { $_.FriendlyName -like "*$gpuName*" } | Enable-PnpDevice -Confirm:$false}
            // where gpuName is gpu_name from above
            .arg(format!("Get-PnpDevice | Where-Object {{ $_.FriendlyName -like \"{}\" }} | Enable-PnpDevice -Confirm:$false", gpu_name))
            .spawn()
            .expect("Failed to execute command");
                }
                _ => println!("Invalid!"),
            }

            std::thread::sleep(std::time::Duration::from_secs(10));
        }
    }
}
