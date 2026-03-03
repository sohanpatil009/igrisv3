// File Share Voice Command Handler
use crate::file_share_client::FileShareClient;
use std::collections::HashMap;

pub async fn handle_file_share_command(
    intent_name: &str,
    entities: &HashMap<String, String>,
) -> String {
    let client = FileShareClient::new(53317);

    // Check if backend is running
    if !client.is_running().await {
        return "File share service is not running. Please start the Go backend first.".to_string();
    }

    match intent_name {
        "file_share_devices" => {
            match client.get_devices().await {
                Ok(devices) => {
                    if devices.is_empty() {
                        "No devices found. Make sure both devices are on the same mobile hotspot.".to_string()
                    } else {
                        let device_list: Vec<String> = devices
                            .iter()
                            .map(|d| format!("{} at {}", d.alias, d.ip))
                            .collect();
                        format!(
                            "Found {} device{}. {}",
                            devices.len(),
                            if devices.len() == 1 { "" } else { "s" },
                            device_list.join(", ")
                        )
                    }
                }
                Err(e) => format!("Failed to get devices: {}", e),
            }
        }

        "file_share_transfers" => {
            match client.get_transfers().await {
                Ok(transfers) => {
                    if transfers.is_empty() {
                        "No active transfers.".to_string()
                    } else {
                        let active_count = transfers.iter().filter(|t| t.status == "in_progress").count();
                        let completed_count = transfers.iter().filter(|t| t.status == "completed").count();
                        
                        format!(
                            "You have {} transfer{}. {} in progress, {} completed.",
                            transfers.len(),
                            if transfers.len() == 1 { "" } else { "s" },
                            active_count,
                            completed_count
                        )
                    }
                }
                Err(e) => format!("Failed to get transfers: {}", e),
            }
        }

        "file_share_send" => {
            // This requires UI interaction for file picker
            "Please use the file share panel to select and send files.".to_string()
        }

        "file_share_cancel" => {
            match client.get_transfers().await {
                Ok(transfers) => {
                    let active_transfers: Vec<_> = transfers
                        .iter()
                        .filter(|t| t.status == "in_progress")
                        .collect();

                    if active_transfers.is_empty() {
                        "No active transfers to cancel.".to_string()
                    } else if active_transfers.len() == 1 {
                        let session_id = &active_transfers[0].session_id;
                        match client.cancel_transfer(session_id).await {
                            Ok(_) => "Transfer cancelled successfully.".to_string(),
                            Err(e) => format!("Failed to cancel transfer: {}", e),
                        }
                    } else {
                        format!(
                            "You have {} active transfers. Please use the file share panel to cancel specific transfers.",
                            active_transfers.len()
                        )
                    }
                }
                Err(e) => format!("Failed to get transfers: {}", e),
            }
        }

        _ => "Unknown file share command.".to_string(),
    }
}
