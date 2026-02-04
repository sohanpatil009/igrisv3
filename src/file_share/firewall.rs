// Firewall permission handling for file sharing

use anyhow::Result;
use std::process::Command;

/// Request firewall permission for the application
pub fn request_firewall_permission(app_name: &str, port: u16) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        request_windows_firewall_permission(app_name, port)
    }
    
    #[cfg(target_os = "macos")]
    {
        request_macos_firewall_permission(app_name)
    }
    
    #[cfg(target_os = "linux")]
    {
        request_linux_firewall_permission(app_name, port)
    }
}

#[cfg(target_os = "windows")]
fn request_windows_firewall_permission(app_name: &str, port: u16) -> Result<()> {
    use std::env;
    
    // Get current executable path
    let exe_path = env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy();
    
    // Check if rule already exists
    let check_output = Command::new("netsh")
        .args(&[
            "advfirewall",
            "firewall",
            "show",
            "rule",
            &format!("name={}", app_name),
        ])
        .output();
    
    if let Ok(output) = check_output {
        if String::from_utf8_lossy(&output.stdout).contains("No rules match") {
            // Rule doesn't exist, create it
            println!("Requesting firewall permission for {} on port {}...", app_name, port);
            
            // Add inbound rule
            let result = Command::new("netsh")
                .args(&[
                    "advfirewall",
                    "firewall",
                    "add",
                    "rule",
                    &format!("name={}", app_name),
                    "dir=in",
                    "action=allow",
                    &format!("program={}", exe_path_str),
                    "enable=yes",
                    &format!("localport={}", port),
                    "protocol=TCP",
                ])
                .status();
            
            match result {
                Ok(status) if status.success() => {
                    println!("✓ Firewall permission granted");
                    Ok(())
                }
                Ok(_) => {
                    eprintln!("⚠ Failed to add firewall rule. You may need to run as administrator.");
                    eprintln!("  Or manually allow port {} in Windows Firewall.", port);
                    Ok(()) // Don't fail, just warn
                }
                Err(e) => {
                    eprintln!("⚠ Could not configure firewall: {}", e);
                    eprintln!("  Please manually allow port {} in Windows Firewall.", port);
                    Ok(()) // Don't fail, just warn
                }
            }
        } else {
            // Rule already exists
            println!("✓ Firewall rule already exists");
            Ok(())
        }
    } else {
        eprintln!("⚠ Could not check firewall status");
        Ok(()) // Don't fail, just warn
    }
}

#[cfg(target_os = "macos")]
fn request_macos_firewall_permission(app_name: &str) -> Result<()> {
    use std::env;
    
    // Get current executable path
    let exe_path = env::current_exe()?;
    let exe_path_str = exe_path.to_string_lossy();
    
    println!("Checking macOS firewall configuration for {}...", app_name);
    
    // Check if application firewall is enabled
    let check_status = Command::new("sudo")
        .args(&[
            "/usr/libexec/ApplicationFirewall/socketfilterfw",
            "--getglobalstate",
        ])
        .output();
    
    if let Ok(output) = check_status {
        let status_str = String::from_utf8_lossy(&output.stdout);
        
        if status_str.contains("enabled") {
            println!("Application Firewall is enabled");
            
            // Check if app is already allowed
            let check_app = Command::new("sudo")
                .args(&[
                    "/usr/libexec/ApplicationFirewall/socketfilterfw",
                    "--getappblocked",
                    &exe_path_str,
                ])
                .output();
            
            if let Ok(app_output) = check_app {
                let app_status = String::from_utf8_lossy(&app_output.stdout);
                
                if app_status.contains("blocked") || app_status.contains("not found") {
                    println!("Adding {} to firewall allowlist...", app_name);
                    
                    // Add application to firewall
                    let add_result = Command::new("sudo")
                        .args(&[
                            "/usr/libexec/ApplicationFirewall/socketfilterfw",
                            "--add",
                            &exe_path_str,
                        ])
                        .status();
                    
                    // Unblock the application
                    let unblock_result = Command::new("sudo")
                        .args(&[
                            "/usr/libexec/ApplicationFirewall/socketfilterfw",
                            "--unblockapp",
                            &exe_path_str,
                        ])
                        .status();
                    
                    match (add_result, unblock_result) {
                        (Ok(add_status), Ok(unblock_status)) 
                            if add_status.success() && unblock_status.success() => {
                            println!("✓ Application added to firewall allowlist");
                            
                            // Restart firewall to apply changes
                            let _ = Command::new("sudo")
                                .args(&[
                                    "/usr/libexec/ApplicationFirewall/socketfilterfw",
                                    "--setglobalstate",
                                    "on",
                                ])
                                .status();
                            
                            Ok(())
                        }
                        _ => {
                            eprintln!("⚠ Could not modify firewall settings.");
                            eprintln!("  You may need to manually allow {} in System Preferences > Security & Privacy > Firewall", app_name);
                            Ok(()) // Don't fail, just warn
                        }
                    }
                } else {
                    println!("✓ Application already allowed in firewall");
                    Ok(())
                }
            } else {
                eprintln!("⚠ Could not check application firewall status");
                Ok(())
            }
        } else {
            println!("✓ Application Firewall is disabled (no action needed)");
            Ok(())
        }
    } else {
        eprintln!("⚠ Could not check firewall status");
        eprintln!("  macOS will show a system dialog when network access is needed.");
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn request_linux_firewall_permission(app_name: &str, port: u16) -> Result<()> {
    println!("Configuring Linux firewall for {} on port {}...", app_name, port);
    
    // Try UFW (Ubuntu/Debian)
    if let Ok(output) = Command::new("which").arg("ufw").output() {
        if output.status.success() {
            return configure_ufw(app_name, port);
        }
    }
    
    // Try firewalld (Fedora/RHEL/CentOS)
    if let Ok(output) = Command::new("which").arg("firewall-cmd").output() {
        if output.status.success() {
            return configure_firewalld(app_name, port);
        }
    }
    
    // Try iptables (fallback)
    if let Ok(output) = Command::new("which").arg("iptables").output() {
        if output.status.success() {
            return configure_iptables(app_name, port);
        }
    }
    
    // No known firewall found
    println!("⚠ No known firewall detected (ufw, firewalld, or iptables)");
    println!("  If you have a firewall, please manually allow TCP port {}", port);
    Ok(())
}

#[cfg(target_os = "linux")]
fn configure_ufw(app_name: &str, port: u16) -> Result<()> {
    println!("Detected UFW firewall");
    
    // Check if UFW is active
    let status_output = Command::new("sudo")
        .args(&["ufw", "status"])
        .output();
    
    if let Ok(output) = status_output {
        let status_str = String::from_utf8_lossy(&output.stdout);
        
        if status_str.contains("inactive") {
            println!("✓ UFW is inactive (no action needed)");
            return Ok(());
        }
        
        println!("UFW is active, adding rule for port {}...", port);
        
        // Add rule for the port
        let result = Command::new("sudo")
            .args(&[
                "ufw",
                "allow",
                &format!("{}/tcp", port),
                "comment",
                app_name,
            ])
            .status();
        
        match result {
            Ok(status) if status.success() => {
                println!("✓ UFW rule added successfully");
                Ok(())
            }
            _ => {
                eprintln!("⚠ Could not add UFW rule. Please run manually:");
                eprintln!("  sudo ufw allow {}/tcp comment '{}'", port, app_name);
                Ok(())
            }
        }
    } else {
        eprintln!("⚠ Could not check UFW status");
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn configure_firewalld(app_name: &str, port: u16) -> Result<()> {
    println!("Detected firewalld");
    
    // Check if firewalld is running
    let status_output = Command::new("sudo")
        .args(&["firewall-cmd", "--state"])
        .output();
    
    if let Ok(output) = status_output {
        let status_str = String::from_utf8_lossy(&output.stdout);
        
        if !status_str.contains("running") {
            println!("✓ firewalld is not running (no action needed)");
            return Ok(());
        }
        
        println!("firewalld is running, adding rule for port {}...", port);
        
        // Add port to firewalld
        let result = Command::new("sudo")
            .args(&[
                "firewall-cmd",
                "--permanent",
                &format!("--add-port={}/tcp", port),
            ])
            .status();
        
        // Reload firewalld
        let reload_result = Command::new("sudo")
            .args(&["firewall-cmd", "--reload"])
            .status();
        
        match (result, reload_result) {
            (Ok(add_status), Ok(reload_status)) 
                if add_status.success() && reload_status.success() => {
                println!("✓ firewalld rule added successfully");
                Ok(())
            }
            _ => {
                eprintln!("⚠ Could not add firewalld rule. Please run manually:");
                eprintln!("  sudo firewall-cmd --permanent --add-port={}/tcp", port);
                eprintln!("  sudo firewall-cmd --reload");
                Ok(())
            }
        }
    } else {
        eprintln!("⚠ Could not check firewalld status");
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn configure_iptables(app_name: &str, port: u16) -> Result<()> {
    println!("Detected iptables");
    
    // Check if rule already exists
    let check_output = Command::new("sudo")
        .args(&[
            "iptables",
            "-C",
            "INPUT",
            "-p",
            "tcp",
            "--dport",
            &port.to_string(),
            "-j",
            "ACCEPT",
        ])
        .output();
    
    if let Ok(output) = check_output {
        if output.status.success() {
            println!("✓ iptables rule already exists");
            return Ok(());
        }
    }
    
    println!("Adding iptables rule for port {}...", port);
    
    // Add iptables rule
    let result = Command::new("sudo")
        .args(&[
            "iptables",
            "-A",
            "INPUT",
            "-p",
            "tcp",
            "--dport",
            &port.to_string(),
            "-j",
            "ACCEPT",
            "-m",
            "comment",
            "--comment",
            app_name,
        ])
        .status();
    
    match result {
        Ok(status) if status.success() => {
            println!("✓ iptables rule added successfully");
            println!("⚠ Note: iptables rules are not persistent by default.");
            println!("  Consider using iptables-persistent or saving rules manually.");
            Ok(())
        }
        _ => {
            eprintln!("⚠ Could not add iptables rule. Please run manually:");
            eprintln!("  sudo iptables -A INPUT -p tcp --dport {} -j ACCEPT -m comment --comment '{}'", port, app_name);
            Ok(())
        }
    }
}

/// Show a user-friendly dialog explaining firewall requirements
pub fn show_firewall_info_dialog() -> bool {
    #[cfg(target_os = "windows")]
    {
        use rfd::MessageDialog;
        
        MessageDialog::new()
            .set_title("Network Permission Required")
            .set_description(
                "IGRIS File Share needs network access to:\n\n\
                 • Discover nearby devices\n\
                 • Send and receive files\n\n\
                 Windows Firewall may ask for permission.\n\
                 Please click 'Allow access' when prompted."
            )
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
        
        true
    }
    
    #[cfg(target_os = "macos")]
    {
        use rfd::MessageDialog;
        
        MessageDialog::new()
            .set_title("Network Permission Required")
            .set_description(
                "IGRIS File Share needs network access to:\n\n\
                 • Discover nearby devices\n\
                 • Send and receive files\n\n\
                 macOS may show a system dialog asking for permission.\n\
                 Please click 'Allow' when prompted.\n\n\
                 If the firewall is enabled, you may need to allow\n\
                 the app in System Preferences > Security & Privacy > Firewall."
            )
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
        
        true
    }
    
    #[cfg(target_os = "linux")]
    {
        use rfd::MessageDialog;
        
        MessageDialog::new()
            .set_title("Network Permission Required")
            .set_description(
                "IGRIS File Share needs network access to:\n\n\
                 • Discover nearby devices (mDNS on port 5353)\n\
                 • Send and receive files (TCP port 53317)\n\n\
                 If you have a firewall enabled (UFW, firewalld, iptables),\n\
                 you may need to allow these ports.\n\n\
                 The app will attempt to configure your firewall automatically,\n\
                 but you may be prompted for your password (sudo)."
            )
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
        
        true
    }
}
