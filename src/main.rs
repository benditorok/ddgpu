use std::{process::Command, time::Duration};

fn main() {
    #[cfg(target_os = "windows")]
    ddgpu::request_admin_privileges();

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
