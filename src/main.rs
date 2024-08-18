fn main() {
    #[cfg(target_os = "windows")]
    ddgpu::run_windows();

    #[cfg(not(target_os = "windows"))]
    {
        println!("Application only runs on windwos currently!");
        std::process::exit(0);
    }
}
