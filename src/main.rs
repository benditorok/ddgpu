use ddgpu::app;

fn main() {
    // Run with cargo run -- --name "NVIDIA GeForce RTX 3050 Laptop GPU" --hide false

    let args = app::Arguments {
        gpu_name: app::GPU_NAME,
        hide_window: app::HIDE_WINDOW,
    };

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
