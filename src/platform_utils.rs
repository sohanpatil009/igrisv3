// src/platform_utils.rs - Convenient wrappers for platform abstractions

use crate::platform::{AppLauncherImpl, FileSystemProviderImpl};
use std::sync::OnceLock;

static APP_LAUNCHER: OnceLock<Box<dyn crate::platform::AppLauncher>> = OnceLock::new();
static FILE_SYSTEM: OnceLock<Box<dyn crate::platform::FileSystemProvider>> = OnceLock::new();

/// Get the platform-specific app launcher
pub fn get_app_launcher() -> &'static dyn crate::platform::AppLauncher {
    APP_LAUNCHER
        .get_or_init(|| AppLauncherImpl::new())
        .as_ref()
}

/// Get the platform-specific file system provider
pub fn get_file_system() -> &'static dyn crate::platform::FileSystemProvider {
    FILE_SYSTEM
        .get_or_init(|| FileSystemProviderImpl::new())
        .as_ref()
}

// Re-export ProcessBuilderExt for convenience
pub use crate::platform::process_builder::ProcessBuilderExt;
