// src/platform/macos_firewall.rs - macOS Firewall Management (Safe & Minimal)

#[cfg(target_os = "macos")]
use std::process::Command;

/// Check if IGRIS is allowed in macOS firewall
#[cfg(target_os = "macos")]
pub fn is_igris_allowed() -> Result<bool, String> {
    let output = Command::new("/usr/libexec/ApplicationFirewall/socketfilterfw")
        .arg("--listapps")
        .output()
        .map_err(|e| format!("Failed to check firewall: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Check if igrisv3 is in the list
    Ok(stdout.contains("igrisv3") || stdout.contains("IGRIS"))
}

/// Add IGRIS to firewall allow list (requires sudo/admin privileges)
/// This is a ONE-TIME setup, not done automatically
#[cfg(target_os = "macos")]
pub fn add_igris_to_firewall() -> Result<(), String> {
    // Get current executable path
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get exe path: {}", e))?;
    
    println!("[Firewall] Adding IGRIS to firewall allow list...");
    println!("[Firewall] Path: {}", exe_path.display());
    
    // This requires admin privileges - will prompt user for password
    let output = Command::new("osascript")
        .arg("-e")
        .arg(format!(
            "do shell script \"/usr/libexec/ApplicationFirewall/socketfilterfw --add {}\" with administrator privileges",
            exe_path.display()
        ))
        .output()
        .map_err(|e| format!("Failed to add to firewall: {}", e))?;
    
    if output.status.success() {
        println!("[Firewall] ✅ IGRIS added to firewall allow list");
        
        // Unblock the app
        let _ = Command::new("osascript")
            .arg("-e")
            .arg(format!(
                "do shell script \"/usr/libexec/ApplicationFirewall/socketfilterfw --unblock {}\" with administrator privileges",
                exe_path.display()
            ))
            .output();
        
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to add to firewall: {}", stderr))
    }
}

/// Check firewall status and show user-friendly message
#[cfg(target_os = "macos")]
pub fn check_and_prompt_firewall() -> Result<(), String> {
    // Check if firewall is enabled
    let output = Command::new("/usr/libexec/ApplicationFirewall/socketfilterfw")
        .arg("--getglobalstate")
        .output()
        .map_err(|e| format!("Failed to check firewall status: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    if stdout.contains("enabled") || stdout.contains("on") {
        println!("[Firewall] macOS Firewall is enabled");
        
        // Check if IGRIS is allowed
        if !is_igris_allowed()? {
            println!("");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("⚠️  FIREWALL SETUP REQUIRED");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("");
            println!("IGRIS needs firewall permission for file sharing.");
            println!("");
            println!("Option 1: Automatic Setup (Recommended)");
            println!("  Run: sudo ./setup_macos_firewall.sh");
            println!("");
            println!("Option 2: Manual Setup");
            println!("  1. Open System Settings → Network → Firewall");
            println!("  2. Click 'Options'");
            println!("  3. Click '+' and add IGRIS");
            println!("  4. Select 'Allow incoming connections'");
            println!("");
            println!("Option 3: Programmatic Setup (Will prompt for password)");
            println!("  Press 'y' to add IGRIS to firewall now");
            println!("");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("");
            
            // Don't automatically add - let user decide
            return Err("Firewall permission required".to_string());
        } else {
            println!("[Firewall] ✅ IGRIS is allowed in firewall");
        }
    } else {
        println!("[Firewall] macOS Firewall is disabled");
    }
    
    Ok(())
}

/// Show firewall help message
#[cfg(target_os = "macos")]
pub fn show_firewall_help() {
    println!("");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔐 macOS Firewall Configuration");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("");
    println!("IGRIS needs to accept incoming connections for:");
    println!("  • Device discovery (UDP port 45678)");
    println!("  • File transfers (TCP port 45679)");
    println!("");
    println!("✅ Safe Method: Add IGRIS to firewall allow list");
    println!("   This keeps your firewall ON and secure!");
    println!("");
    println!("Run setup script:");
    println!("  sudo ./setup_macos_firewall.sh");
    println!("");
    println!("Or manually:");
    println!("  System Settings → Network → Firewall → Options");
    println!("  Add IGRIS and allow incoming connections");
    println!("");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("");
}

// Stub implementations for non-macOS platforms
#[cfg(not(target_os = "macos"))]
pub fn is_igris_allowed() -> Result<bool, String> {
    Ok(true) // No firewall check needed on other platforms
}

#[cfg(not(target_os = "macos"))]
pub fn check_and_prompt_firewall() -> Result<(), String> {
    Ok(()) // No-op on other platforms
}

#[cfg(not(target_os = "macos"))]
pub fn show_firewall_help() {
    // No-op on other platforms
}

#[cfg(not(target_os = "macos"))]
pub fn add_igris_to_firewall() -> Result<(), String> {
    Ok(()) // No-op on other platforms
}
