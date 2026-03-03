// Go Backend Auto-Start Manager
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

static GO_BACKEND_PROCESS: Lazy<Arc<Mutex<Option<Child>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));

/// Start the Go file share backend
pub fn start_go_backend() -> Result<(), String> {
    let mut process_guard = GO_BACKEND_PROCESS.lock()
        .map_err(|e| format!("Failed to lock process: {}", e))?;
    
    // Check if already running
    if let Some(ref mut child) = *process_guard {
        // Check if process is still alive
        match child.try_wait() {
            Ok(None) => {
                // Process is still running
                println!("[GO_BACKEND] Already running");
                return Ok(());
            }
            Ok(Some(status)) => {
                println!("[GO_BACKEND] Previous process exited with: {}", status);
            }
            Err(e) => {
                println!("[GO_BACKEND] Error checking process: {}", e);
            }
        }
    }
    
    // Determine binary path based on OS
    #[cfg(target_os = "windows")]
    let binary_path = "./go-fileshare/fileshare.exe";
    
    #[cfg(not(target_os = "windows"))]
    let binary_path = "./go-fileshare/fileshare";
    
    // Check if binary exists
    if !std::path::Path::new(binary_path).exists() {
        return Err(format!(
            "Go backend binary not found at {}. Please build it first with: cd go-fileshare && ./build.sh",
            binary_path
        ));
    }
    
    // Start the process
    match Command::new(binary_path)
        .spawn()
    {
        Ok(child) => {
            println!("[GO_BACKEND] Started successfully (PID: {})", child.id());
            *process_guard = Some(child);
            
            // Give it time to initialize
            std::thread::sleep(std::time::Duration::from_secs(2));
            
            Ok(())
        }
        Err(e) => {
            Err(format!("Failed to start Go backend: {}", e))
        }
    }
}

/// Stop the Go file share backend
pub fn stop_go_backend() {
    if let Ok(mut process_guard) = GO_BACKEND_PROCESS.lock() {
        if let Some(mut child) = process_guard.take() {
            match child.kill() {
                Ok(_) => println!("[GO_BACKEND] Stopped successfully"),
                Err(e) => eprintln!("[GO_BACKEND] Failed to stop: {}", e),
            }
        }
    }
}

/// Check if Go backend is running
pub fn is_running() -> bool {
    if let Ok(mut process_guard) = GO_BACKEND_PROCESS.lock() {
        if let Some(ref mut child) = *process_guard {
            match child.try_wait() {
                Ok(None) => return true, // Still running
                _ => {}
            }
        }
    }
    false
}

/// Restart the Go backend
pub fn restart_go_backend() -> Result<(), String> {
    stop_go_backend();
    std::thread::sleep(std::time::Duration::from_millis(500));
    start_go_backend()
}
