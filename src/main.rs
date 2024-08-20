use ddgpu::app;

fn main() {
    // Run with cargo run -- --name "NVIDIA GeForce RTX 3050 Laptop GPU" --hide false

    let mut args = app::Arguments::default();
    args.init().unwrap_or_else(|e| {
        eprintln!("Failed to parse arguments: {:?}", e);
        std::io::stdin().read_line(&mut String::new()).unwrap();
        std::process::exit(0);
    });

    let as_service_default = String::from("False");
    let as_service = args
        .get_value(app::RUN_AS_SERVICE)
        .unwrap_or(as_service_default);

    if as_service.as_str() == "true" {
        #[cfg(target_os = "windows")]
        if let Err(e) = ddgpu::windows_service::run_windows_service() {
            eprintln!("Failed to run as a service: {:?}", e);
        }

        #[cfg(not(target_os = "windows"))]
        {
            println!("Application only runs on windwos currently!");
            std::process::exit(0);
        }
    } else {
        #[cfg(target_os = "windows")]
        if let Err(e) = ddgpu::on_windows::run(&args) {
            println!("Program failed to run! Error: {}", e);
            println!("Press enter to exit");

            std::io::stdin().read_line(&mut String::new()).unwrap();
            std::process::exit(0);
        }

        #[cfg(not(target_os = "windows"))]
        {
            println!("Application only runs on windwos currently!");
            std::process::exit(0);
        }
    }
}
