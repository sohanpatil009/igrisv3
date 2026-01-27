// src/platform/firewall.rs - Firewall Permission Management

use std::process::Command;

/// Check and request firewall permissions for QUIC
pub fn request_firewall_permission() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        request_macos_firewall_permission()
    }
    
    #[cfg(target_os = "windows")]
    {
        request_windows_firewall_permission()
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        println!("[Firewall] Linux detected - firewall rules may need manual configuration");
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn request_macos_firewall_permission() -> Result<(), String> {
    use std::env;
    
    println!("[Firewall] Checking macOS firewall permissions...");
    
    // Get the current executable path
    let exe_path = env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    
    println!("[Firewall] Executable path: {:?}", exe_path);
    
    // Check if firewall is enabled
    let firewall_status = Command::new("/usr/libexec/ApplicationFirewall/socketfilterfw")
        .arg("--getglobalstate")
        .output();
    
    match firewall_status {
        Ok(output) => {
            let status = String::from_utf8_lossy(&output.stdout);
            if status.contains("disabled") {
                println!("[Firewall] macOS firewall is disabled - no action needed");
                return Ok(());
            }
            println!("[Firewall] macOS firewall is enabled");
        }
        Err(e) => {
            println!("[Firewall] Could not check firewall status: {}", e);
        }
    }
    
    // Check if app is already allowed
    let check_app = Command::new("/usr/libexec/ApplicationFirewall/socketfilterfw")
        .arg("--getappblocked")
        .arg(&exe_path)
        .output();
    
    match check_app {
        Ok(output) => {
            let result = String::from_utf8_lossy(&output.stdout);
            if result.contains("permitted") || result.contains("allowed") {
                println!("[Firewall] ✓ App already has firewall permission");
                return Ok(());
            }
        }
        Err(e) => {
            println!("[Firewall] Could not check app permission: {}", e);
        }
    }
    
    println!("[Firewall] Requesting firewall permission...");
    println!("[Firewall] macOS will show a permission dialog when you first use File Share");
    println!("[Firewall] Please click 'Allow' when prompted");
    
    // The permission dialog will appear automatically when we bind to the UDP port
    // We don't need to manually trigger it - macOS handles this
    
    Ok(())
}

#[cfg(target_os = "windows")]
fn request_windows_firewall_permission() -> Result<(), String> {
    use std::env;
    
    println!("[Firewall] Checking Windows firewall permissions...");
    
    // Get the current executable path
    let exe_path = env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    
    let exe_path_str = exe_path.to_string_lossy();
    println!("[Firewall] Executable path: {}", exe_path_str);
    
    // Check if firewall rule already exists
    let check_rule = Command::new("netsh")
        .args(&["advfirewall", "firewall", "show", "rule", "name=IGRIS File Share"])
        .output();
    
    match check_rule {
        Ok(output) => {
            let result = String::from_utf8_lossy(&output.stdout);
            if result.contains("IGRIS File Share") {
                println!("[Firewall] ✓ Firewall rule already exists");
                return Ok(());
            }
        }
        Err(e) => {
            println!("[Firewall] Could not check firewall rule: {}", e);
        }
    }
    
    println!("[Firewall] Firewall rule not found - attempting to create...");
    
    // Try to add firewall rule (requires admin)
    let add_inbound = Command::new("netsh")
        .args(&[
            "advfirewall", "firewall", "add", "rule",
            "name=IGRIS File Share",
            "dir=in",
            "action=allow",
            "protocol=UDP",
            "localport=45679",
            &format!("program={}", exe_path_str),
        ])
        .output();
    
    match add_inbound {
        Ok(output) => {
            if output.status.success() {
                println!("[Firewall] ✓ Inbound firewall rule created successfully");
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                if error.contains("administrator") || error.contains("elevated") {
                    println!("[Firewall] ⚠️  Need administrator privileges to add firewall rule");
                    println!("[Firewall] Please run as administrator or manually add firewall rule:");
                    println!("[Firewall]   1. Open Windows Defender Firewall");
                    println!("[Firewall]   2. Click 'Advanced settings'");
                    println!("[Firewall]   3. Add Inbound Rule for UDP port 45679");
                    return Err("Administrator privileges required for firewall rule".to_string());
                } else {
                    println!("[Firewall] Failed to create inbound rule: {}", error);
                }
            }
        }
        Err(e) => {
            println!("[Firewall] Error creating inbound rule: {}", e);
        }
    }
    
    // Add outbound rule
    let add_outbound = Command::new("netsh")
        .args(&[
            "advfirewall", "firewall", "add", "rule",
            "name=IGRIS File Share",
            "dir=out",
            "action=allow",
            "protocol=UDP",
            "localport=45679",
            &format!("program={}", exe_path_str),
        ])
        .output();
    
    match add_outbound {
        Ok(output) => {
            if output.status.success() {
                println!("[Firewall] ✓ Outbound firewall rule created successfully");
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                println!("[Firewall] Failed to create outbound rule: {}", error);
            }
        }
        Err(e) => {
            println!("[Firewall] Error creating outbound rule: {}", e);
        }
    }
    
    Ok(())
}

/// Show firewall setup instructions to user
pub fn show_firewall_instructions() {
    #[cfg(target_os = "macos")]
    {
        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!("║  macOS Firewall Permission Required                       ║");
        println!("╚════════════════════════════════════════════════════════════╝");
        println!();
        println!("  When you use File Share for the first time, macOS will");
        println!("  show a dialog asking for network permission.");
        println!();
        println!("  Please click 'Allow' to enable file sharing.");
        println!();
        println!("  If you don't see the dialog:");
        println!("  1. Open System Settings → Network → Firewall");
        println!("  2. Click Options");
        println!("  3. Add IGRIS and set to 'Allow incoming connections'");
        println!();
    }
    
    #[cfg(target_os = "windows")]
    {
        println!("\n╔════════════════════════════════════════════════════════════╗");
        println!("║  Windows Firewall Permission Required                     ║");
        println!("╚════════════════════════════════════════════════════════════╝");
        println!();
        println!("  To enable file sharing, run as Administrator:");
        println!();
        println!("  PowerShell (as Admin):");
        println!("  New-NetFirewallRule -DisplayName \"IGRIS File Share\" \\");
        println!("    -Direction Inbound -Protocol UDP -LocalPort 45679 \\");
        println!("    -Action Allow");
        println!();
        println!("  Or manually:");
        println!("  1. Open Windows Defender Firewall");
        println!("  2. Advanced settings → Inbound Rules → New Rule");
        println!("  3. Port → UDP → 45679 → Allow");
        println!();
    }
}
