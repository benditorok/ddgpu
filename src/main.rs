use ddgpu::app;

fn main() {
    // Run with cargo run -- --name "NVIDIA GeForce RTX 3050 Laptop GPU" --hide false

    let args = app::Arguments {
        gpu_name: app::GPU_NAME,
        hide_window: app::HIDE_WINDOW,
        run_as_service: app::RUN_AS_SERVICE,
    };

    let as_service_default = String::from("False");

    let parsed_args = args.parse_args().unwrap();
    let run_as_service = parsed_args
        .get(app::RUN_AS_SERVICE)
        .to_owned()
        .unwrap_or(&as_service_default);

    if run_as_service.as_str() == "true" {
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
