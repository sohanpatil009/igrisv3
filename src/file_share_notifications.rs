// File Share Notification System
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

#[derive(Clone, Debug)]
pub struct Notification {
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

static NOTIFICATION_QUEUE: Lazy<Arc<Mutex<Vec<Notification>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

/// Show a notification to the user
pub fn show_notification(title: &str, message: &str, notification_type: NotificationType) {
    // Add to queue for UI display
    if let Ok(mut queue) = NOTIFICATION_QUEUE.lock() {
        queue.push(Notification {
            title: title.to_string(),
            message: message.to_string(),
            notification_type: notification_type.clone(),
        });
    }
    
    // Log to console
    let prefix = match notification_type {
        NotificationType::Success => "✓",
        NotificationType::Info => "ℹ",
        NotificationType::Warning => "⚠",
        NotificationType::Error => "✗",
    };
    
    println!("[NOTIFICATION] {} {}: {}", prefix, title, message);
}

/// Get pending notifications
pub fn get_notifications() -> Vec<Notification> {
    if let Ok(mut queue) = NOTIFICATION_QUEUE.lock() {
        let notifications = queue.clone();
        queue.clear();
        notifications
    } else {
        Vec::new()
    }
}

/// Notify about transfer start
pub fn notify_transfer_started(device_name: &str, file_name: &str) {
    show_notification(
        "File Transfer Started",
        &format!("Sending {} to {}", file_name, device_name),
        NotificationType::Info,
    );
}

/// Notify about transfer completion
pub fn notify_transfer_completed(device_name: &str, file_name: &str) {
    show_notification(
        "Transfer Complete",
        &format!("Successfully sent {} to {}", file_name, device_name),
        NotificationType::Success,
    );
}

/// Notify about transfer failure
pub fn notify_transfer_failed(device_name: &str, file_name: &str, error: &str) {
    show_notification(
        "Transfer Failed",
        &format!("Failed to send {} to {}: {}", file_name, device_name, error),
        NotificationType::Error,
    );
}

/// Notify about incoming transfer
pub fn notify_incoming_transfer(device_name: &str, file_name: &str) {
    show_notification(
        "Incoming File",
        &format!("{} wants to send you {}", device_name, file_name),
        NotificationType::Info,
    );
}

/// Notify about received file
pub fn notify_file_received(device_name: &str, file_name: &str) {
    show_notification(
        "File Received",
        &format!("Received {} from {}", file_name, device_name),
        NotificationType::Success,
    );
}
