// src/ui/mod.rs - UI Components

pub mod settings;
pub mod search_results;
pub mod camera_panel;
pub mod presentation;

pub use settings::{SettingsPanel, SettingsButton};
pub use search_results::{SearchResultsPanel, SearchResultItem};
pub use camera_panel::CameraPanel;
pub use presentation::{PresentationPanel, start_presentation, stop_presentation, is_presentation_active, is_presentation_open};
