use super::{app, on_windows};

#[cfg(target_os = "windows")]
pub fn run_windows_service() -> windows_service::Result<()> {
    use std::time::Duration;
    use windows_service::service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    };
    use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
    use windows_service::service_dispatcher;

    const SERVICE_NAME: &str = "ddgpu";

    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;

    extern "system" fn ffi_service_main(_arguments: u32, _reserved: *mut *mut u16) {
        if let Err(e) = run_service() {
            eprintln!("Service failed with: {}", e);
        }
    }

    fn run_service() -> windows_service::Result<()> {
        // TODO recieve this from main
        let mut args = app::Arguments::default();
        args.init().unwrap_or_else(|e| {
            eprintln!("Failed to parse arguments: {:?}", e);
            std::io::stdin().read_line(&mut String::new()).unwrap();
            std::process::exit(0);
        });
        let status_handle = service_control_handler::register(SERVICE_NAME, service_handler)?;

        status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::StartPending,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::from_secs(5),
            process_id: None,
        })?;

        status_handle.set_service_status(ServiceStatus {
            current_state: ServiceState::Running,
            service_type: ServiceType::OWN_PROCESS,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::from_secs(5),
            process_id: None,
        })?;

        // Service runs until stopped
        if let Err(e) = on_windows::run(&args) {
            println!("Program failed to run! Error: {}", e);
            println!("Press enter to exit");

            std::io::stdin().read_line(&mut String::new()).unwrap();
            std::process::exit(0);
        }

        // After loop ends, set service status to stopped
        status_handle.set_service_status(ServiceStatus {
            current_state: ServiceState::Stopped,
            service_type: ServiceType::OWN_PROCESS,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::from_secs(5),
            process_id: None,
        })?;

        Ok(())
    }

    fn service_handler(control: ServiceControl) -> ServiceControlHandlerResult {
        match control {
            // TODO cannot stop service from Services
            ServiceControl::Stop => {
                // Handle stop event
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    }

    Ok(())
}
