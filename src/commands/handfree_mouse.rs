// HandFree Mouse command handler for IGRIS
// Integrates Python-based hand gesture control with voice commands

use std::process::{Command, Child};
use std::sync::{Arc, Mutex};
use anyhow::Result;

// Global state for HandFree Mouse process
static HANDFREE_PROCESS: once_cell::sync::Lazy<Arc<Mutex<Option<Child>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

static HANDFREE_ENABLED: once_cell::sync::Lazy<Arc<Mutex<bool>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(false)));

/// Handle HandFree Mouse voice commands
pub fn handle_handfree_command(command: &str) -> Result<String> {
    let cmd_lower = command.to_lowercase();
    
    if cmd_lower.contains("enable") || cmd_lower.contains("start") || cmd_lower.contains("activate") {
        start_handfree_mouse()
    } else if cmd_lower.contains("disable") || cmd_lower.contains("stop") || cmd_lower.contains("deactivate") {
        stop_handfree_mouse()
    } else if cmd_lower.contains("status") {
        get_handfree_status()
    } else if cmd_lower.contains("calibrate") {
        calibrate_handfree_mouse()
    } else {
        Ok("Unknown HandFree Mouse command. Try 'enable hand mouse' or 'disable hand mouse'".to_string())
    }
}

/// Start HandFree Mouse gesture control
fn start_handfree_mouse() -> Result<String> {
    let mut process_guard = HANDFREE_PROCESS.lock().unwrap();
    let mut enabled_guard = HANDFREE_ENABLED.lock().unwrap();
    
    // Check if already running
    if process_guard.is_some() {
        return Ok("HandFree Mouse is already running".to_string());
    }
    
    // Get Python executable path
    let python_cmd = if cfg!(windows) {
        "python"
    } else {
        "python3"
    };
    
    // Get HandFree Mouse script path
    let script_path = std::env::current_dir()
        .unwrap()
        .join("handfree_mouse")
        .join("python")
        .join("main.py");
    
    if !script_path.exists() {
        return Err(anyhow::anyhow!(
            "HandFree Mouse not found. Please ensure handfree_mouse/python/main.py exists"
        ));
    }
    
    // Start Python process
    let child = Command::new(python_cmd)
        .arg(script_path)
        .arg("--no-ui")  // Run without UI window
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to start HandFree Mouse: {}", e))?;
    
    *process_guard = Some(child);
    *enabled_guard = true;
    
    tracing::info!("HandFree Mouse started");
    Ok("HandFree Mouse enabled. Control your mouse with hand gestures!".to_string())
}

/// Stop HandFree Mouse gesture control
fn stop_handfree_mouse() -> Result<String> {
    let mut process_guard = HANDFREE_PROCESS.lock().unwrap();
    let mut enabled_guard = HANDFREE_ENABLED.lock().unwrap();
    
    if let Some(mut child) = process_guard.take() {
        // Try to kill the process gracefully
        match child.kill() {
            Ok(_) => {
                let _ = child.wait();
                *enabled_guard = false;
                tracing::info!("HandFree Mouse stopped");
                Ok("HandFree Mouse disabled".to_string())
            }
            Err(e) => {
                tracing::error!("Failed to stop HandFree Mouse: {}", e);
                Err(anyhow::anyhow!("Failed to stop HandFree Mouse: {}", e))
            }
        }
    } else {
        Ok("HandFree Mouse is not running".to_string())
    }
}

/// Get HandFree Mouse status
fn get_handfree_status() -> Result<String> {
    let enabled = *HANDFREE_ENABLED.lock().unwrap();
    
    if enabled {
        Ok("HandFree Mouse is currently enabled and running".to_string())
    } else {
        Ok("HandFree Mouse is currently disabled".to_string())
    }
}

/// Calibrate HandFree Mouse settings
fn calibrate_handfree_mouse() -> Result<String> {
    // For now, just provide instructions
    // In the future, this could open a calibration UI
    Ok("To calibrate HandFree Mouse, edit handfree_mouse/python/config.json and adjust sensitivity, smoothing, and threshold values".to_string())
}

/// Check if HandFree Mouse is enabled
pub fn is_handfree_enabled() -> bool {
    *HANDFREE_ENABLED.lock().unwrap()
}

/// Cleanup on shutdown
pub fn cleanup_handfree_mouse() {
    let _ = stop_handfree_mouse();
}
