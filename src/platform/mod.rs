// src/platform/mod.rs - Cross-platform abstraction layer

pub mod app_launcher;
pub mod file_system;
pub mod process_builder;
pub mod system_control;
pub mod macos_firewall;

pub use app_launcher::{AppLauncher, AppLauncherImpl};
pub use file_system::{FileSystemProvider, FileSystemProviderImpl};
pub use process_builder::ProcessBuilderExt;
pub use system_control::{SystemController, get_system_controller};
pub use macos_firewall::{check_and_prompt_firewall, show_firewall_help, is_igris_allowed, add_igris_to_firewall};
