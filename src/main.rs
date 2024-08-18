fn main() {
    // Run with cargo run -- --name 'NVIDIA GeForce RTX 3050 Laptop GPU'

    #[cfg(target_os = "windows")]
    if let Err(run_err) = ddgpu::on_windows::run() {
        println!("Program failed to run! Error: {}", run_err);
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
