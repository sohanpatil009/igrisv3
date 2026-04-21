// src/core/mod.rs - Core voice processing modules

pub mod stt;
pub mod tts;
pub mod vad;
pub mod wake_word;
pub mod audio_capture;
pub mod about;
pub mod gemini;

// Re-exports for convenience
pub use stt::{init_whisper_context, transcribe_audio, hybrid_transcribe_audio};
pub use tts::{speak, speak_compat, TTS_ENGINE};
pub use audio_capture::{capture_audio_vad, CaptureConfig, CaptureResult, CaptureMode};
pub use wake_word::listen_for_wake_word;
pub use about::{IgrisAbout, AboutSection, is_about_query, wants_detailed_info};
pub use gemini::{enhanced_web_search, enhance_voice_command, GeminiClient};
