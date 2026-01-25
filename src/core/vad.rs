// src/vad.rs
// Voice Activity Detection for low-latency speech detection
// Eliminates fixed recording durations by detecting speech boundaries

use std::collections::VecDeque;

/// VAD configuration parameters
#[derive(Clone, Debug)]
pub struct VadConfig {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Frame size in samples (typically 10-30ms worth)
    pub frame_size: usize,
    /// Energy threshold for speech detection (0.0 - 1.0)
    pub energy_threshold: f32,
    /// Zero-crossing rate threshold
    pub zcr_threshold: f32,
    /// Number of consecutive speech frames to confirm speech start
    pub speech_start_frames: usize,
    /// Number of consecutive silence frames to confirm speech end
    pub speech_end_frames: usize,
    /// Minimum speech duration in milliseconds
    pub min_speech_ms: u32,
    /// Maximum speech duration in milliseconds (safety limit)
    pub max_speech_ms: u32,
    /// Pre-speech buffer in milliseconds (capture audio before speech detected)
    pub pre_speech_buffer_ms: u32,
    /// Post-speech buffer in milliseconds (capture audio after speech ends)
    pub post_speech_buffer_ms: u32,
}

impl Default for VadConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            frame_size: 480, // 30ms at 16kHz
            energy_threshold: 0.01,
            zcr_threshold: 0.3,
            speech_start_frames: 3,
            speech_end_frames: 15, // ~450ms of silence to end
            min_speech_ms: 300,
            max_speech_ms: 15000, // 15 seconds max
            pre_speech_buffer_ms: 300,
            post_speech_buffer_ms: 200,
        }
    }
}

impl VadConfig {
    /// Create config optimized for wake word detection (faster response)
    pub fn for_wake_word() -> Self {
        Self {
            sample_rate: 16000,
            frame_size: 320, // 20ms at 16kHz
            energy_threshold: 0.005, // Lower threshold - more sensitive
            zcr_threshold: 0.2, // Lower threshold - catch more speech
            speech_start_frames: 2, // Quick detection
            speech_end_frames: 15, // ~300ms silence - wait longer for full word
            min_speech_ms: 400, // "arise" takes ~400-600ms to say
            max_speech_ms: 2000, // 2 seconds max for wake word
            pre_speech_buffer_ms: 300, // Capture more of the beginning
            post_speech_buffer_ms: 200, // Capture the end
        }
    }

    /// Create config for command listening (balanced)
    pub fn for_commands() -> Self {
        Self {
            sample_rate: 16000,
            frame_size: 480, // 30ms at 16kHz
            energy_threshold: 0.01,
            zcr_threshold: 0.3,
            speech_start_frames: 3,
            speech_end_frames: 20, // ~600ms silence (allow pauses in commands)
            min_speech_ms: 300,
            max_speech_ms: 10000, // 10 seconds max
            pre_speech_buffer_ms: 300,
            post_speech_buffer_ms: 200,
        }
    }

    /// Create config for longer dictation
    pub fn for_dictation() -> Self {
        Self {
            sample_rate: 16000,
            frame_size: 480,
            energy_threshold: 0.012,
            zcr_threshold: 0.35,
            speech_start_frames: 3,
            speech_end_frames: 30, // ~900ms silence (longer pauses OK)
            min_speech_ms: 500,
            max_speech_ms: 30000, // 30 seconds max
            pre_speech_buffer_ms: 300,
            post_speech_buffer_ms: 300,
        }
    }

    fn frames_to_samples(&self, frames: usize) -> usize {
        frames * self.frame_size
    }

    fn ms_to_samples(&self, ms: u32) -> usize {
        (self.sample_rate as usize * ms as usize) / 1000
    }
}

/// VAD state machine states
#[derive(Clone, Debug, PartialEq)]
pub enum VadState {
    /// Waiting for speech to begin
    WaitingForSpeech,
    /// Confirming speech has started (need consecutive frames)
    ConfirmingSpeechStart { consecutive_frames: usize },
    /// Currently in speech segment
    InSpeech { duration_samples: usize },
    /// Confirming speech has ended (need consecutive silence frames)
    ConfirmingSpeechEnd {
        silence_frames: usize,
        speech_duration: usize,
    },
    /// Speech segment complete
    SpeechComplete,
}

/// Result of processing an audio frame
#[derive(Clone, Debug)]
pub enum VadEvent {
    /// No significant event, continue collecting
    Continue,
    /// Speech has started
    SpeechStarted,
    /// Speech is ongoing
    SpeechOngoing,
    /// Speech has ended - ready to process
    SpeechEnded,
    /// Maximum duration reached - forced end
    MaxDurationReached,
}

/// Voice Activity Detector
pub struct Vad {
    config: VadConfig,
    state: VadState,
    /// Ring buffer for pre-speech audio
    pre_buffer: VecDeque<f32>,
    /// Collected speech audio
    speech_buffer: Vec<f32>,
    /// Running energy average for adaptive thresholding
    energy_avg: f32,
    /// Energy history for adaptive threshold
    energy_history: VecDeque<f32>,
    /// Frame counter for statistics
    frame_count: u64,
}

impl Vad {
    pub fn new(config: VadConfig) -> Self {
        let pre_buffer_size = config.ms_to_samples(config.pre_speech_buffer_ms);

        Self {
            config,
            state: VadState::WaitingForSpeech,
            pre_buffer: VecDeque::with_capacity(pre_buffer_size),
            speech_buffer: Vec::with_capacity(16000 * 10), // Pre-allocate for 10 seconds
            energy_avg: 0.0,
            energy_history: VecDeque::with_capacity(100),
            frame_count: 0,
        }
    }

    /// Reset VAD state for new detection
    pub fn reset(&mut self) {
        self.state = VadState::WaitingForSpeech;
        self.pre_buffer.clear();
        self.speech_buffer.clear();
        self.frame_count = 0;
        // Keep energy_avg and energy_history for adaptive thresholding
    }

    /// Get current VAD state
    pub fn state(&self) -> &VadState {
        &self.state
    }

    /// Check if currently detecting speech
    pub fn is_speech_active(&self) -> bool {
        matches!(
            self.state,
            VadState::InSpeech { .. } | VadState::ConfirmingSpeechEnd { .. }
        )
    }

    /// Get collected speech audio (call after SpeechEnded event)
    pub fn get_speech_audio(&self) -> Vec<f32> {
        self.speech_buffer.clone()
    }

    /// Take collected speech audio (moves ownership)
    pub fn take_speech_audio(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.speech_buffer)
    }

    /// Process a frame of audio samples
    pub fn process_frame(&mut self, samples: &[f32]) -> VadEvent {
        self.frame_count += 1;

        // Calculate frame features
        let energy = calculate_energy(samples);
        let zcr = calculate_zero_crossing_rate(samples);

        // Update adaptive threshold
        self.update_energy_history(energy);
        let adaptive_threshold = self.get_adaptive_threshold();

        // Determine if this frame contains speech
        let is_speech = energy > adaptive_threshold && zcr < self.config.zcr_threshold;

        // State machine processing - extract values to avoid borrow issues
        let current_state = self.state.clone();

        match current_state {
            VadState::WaitingForSpeech => {
                // Always keep pre-buffer filled
                self.add_to_pre_buffer(samples);

                if is_speech {
                    self.state = VadState::ConfirmingSpeechStart {
                        consecutive_frames: 1,
                    };
                }
                VadEvent::Continue
            }

            VadState::ConfirmingSpeechStart { consecutive_frames } => {
                self.add_to_pre_buffer(samples);

                if is_speech {
                    let new_count = consecutive_frames + 1;
                    if new_count >= self.config.speech_start_frames {
                        // Speech confirmed - transfer pre-buffer to speech buffer
                        self.speech_buffer.extend(self.pre_buffer.iter().copied());
                        self.speech_buffer.extend_from_slice(samples);
                        self.state = VadState::InSpeech {
                            duration_samples: self.speech_buffer.len(),
                        };
                        VadEvent::SpeechStarted
                    } else {
                        self.state = VadState::ConfirmingSpeechStart {
                            consecutive_frames: new_count,
                        };
                        VadEvent::Continue
                    }
                } else {
                    // Reset - false alarm
                    self.state = VadState::WaitingForSpeech;
                    VadEvent::Continue
                }
            }

            VadState::InSpeech { duration_samples } => {
                self.speech_buffer.extend_from_slice(samples);
                let new_duration = duration_samples + samples.len();

                // Check max duration
                let max_samples = self.config.ms_to_samples(self.config.max_speech_ms);
                if new_duration >= max_samples {
                    self.state = VadState::SpeechComplete;
                    return VadEvent::MaxDurationReached;
                }

                if is_speech {
                    self.state = VadState::InSpeech {
                        duration_samples: new_duration,
                    };
                    VadEvent::SpeechOngoing
                } else {
                    // Potential end of speech
                    self.state = VadState::ConfirmingSpeechEnd {
                        silence_frames: 1,
                        speech_duration: new_duration,
                    };
                    VadEvent::SpeechOngoing
                }
            }

            VadState::ConfirmingSpeechEnd {
                silence_frames,
                speech_duration,
            } => {
                self.speech_buffer.extend_from_slice(samples);
                let new_duration = speech_duration + samples.len();

                // Check max duration
                let max_samples = self.config.ms_to_samples(self.config.max_speech_ms);
                if new_duration >= max_samples {
                    self.state = VadState::SpeechComplete;
                    return VadEvent::MaxDurationReached;
                }

                if is_speech {
                    // Back to speech
                    self.state = VadState::InSpeech {
                        duration_samples: new_duration,
                    };
                    VadEvent::SpeechOngoing
                } else {
                    let new_silence = silence_frames + 1;
                    if new_silence >= self.config.speech_end_frames {
                        // Check minimum duration
                        let min_samples = self.config.ms_to_samples(self.config.min_speech_ms);
                        if new_duration >= min_samples {
                            self.state = VadState::SpeechComplete;
                            VadEvent::SpeechEnded
                        } else {
                            // Too short - likely noise, reset
                            self.reset();
                            VadEvent::Continue
                        }
                    } else {
                        self.state = VadState::ConfirmingSpeechEnd {
                            silence_frames: new_silence,
                            speech_duration: new_duration,
                        };
                        VadEvent::SpeechOngoing
                    }
                }
            }

            VadState::SpeechComplete => {
                // Already complete - shouldn't receive more frames
                VadEvent::SpeechEnded
            }
        }
    }

    /// Process samples one at a time (for streaming)
    /// Returns event when a full frame has been processed
    pub fn process_sample(&mut self, sample: f32, frame_buffer: &mut Vec<f32>) -> Option<VadEvent> {
        frame_buffer.push(sample);

        if frame_buffer.len() >= self.config.frame_size {
            let event = self.process_frame(frame_buffer);
            frame_buffer.clear();
            Some(event)
        } else {
            None
        }
    }

    fn add_to_pre_buffer(&mut self, samples: &[f32]) {
        let max_size = self.config.ms_to_samples(self.config.pre_speech_buffer_ms);

        for &sample in samples {
            if self.pre_buffer.len() >= max_size {
                self.pre_buffer.pop_front();
            }
            self.pre_buffer.push_back(sample);
        }
    }

    fn update_energy_history(&mut self, energy: f32) {
        const HISTORY_SIZE: usize = 100;

        if self.energy_history.len() >= HISTORY_SIZE {
            self.energy_history.pop_front();
        }
        self.energy_history.push_back(energy);

        // Update running average
        if !self.energy_history.is_empty() {
            self.energy_avg =
                self.energy_history.iter().sum::<f32>() / self.energy_history.len() as f32;
        }
    }

    fn get_adaptive_threshold(&self) -> f32 {
        // Use higher of fixed threshold or 2x average background energy
        let adaptive = self.energy_avg * 2.5;
        self.config.energy_threshold.max(adaptive)
    }

    /// Get speech duration in milliseconds (if in speech or complete)
    pub fn speech_duration_ms(&self) -> Option<u32> {
        let samples = match &self.state {
            VadState::InSpeech { duration_samples } => Some(*duration_samples),
            VadState::ConfirmingSpeechEnd {
                speech_duration, ..
            } => Some(*speech_duration),
            VadState::SpeechComplete => Some(self.speech_buffer.len()),
            _ => None,
        };

        samples.map(|s| (s as u32 * 1000) / self.config.sample_rate)
    }
}

/// Calculate Root Mean Square energy of audio frame
fn calculate_energy(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
    (sum_squares / samples.len() as f32).sqrt()
}

/// Calculate Zero Crossing Rate of audio frame
fn calculate_zero_crossing_rate(samples: &[f32]) -> f32 {
    if samples.len() < 2 {
        return 0.0;
    }

    let crossings: usize = samples
        .windows(2)
        .filter(|w| (w[0] >= 0.0) != (w[1] >= 0.0))
        .count();

    crossings as f32 / (samples.len() - 1) as f32
}

/// Simple energy-based speech detection (for quick checks)
pub fn is_speech_frame(samples: &[f32], threshold: f32) -> bool {
    calculate_energy(samples) > threshold
}

/// Detect if audio buffer contains speech
pub fn contains_speech(samples: &[f32], config: &VadConfig) -> bool {
    let mut speech_frames = 0;
    let total_frames = samples.len() / config.frame_size;

    for chunk in samples.chunks(config.frame_size) {
        if chunk.len() == config.frame_size {
            let energy = calculate_energy(chunk);
            let zcr = calculate_zero_crossing_rate(chunk);

            if energy > config.energy_threshold && zcr < config.zcr_threshold {
                speech_frames += 1;
            }
        }
    }

    // Consider speech if >10% of frames contain speech
    speech_frames > total_frames / 10
}

/// Trim silence from beginning and end of audio
pub fn trim_silence(samples: &[f32], config: &VadConfig) -> Vec<f32> {
    if samples.is_empty() {
        return Vec::new();
    }

    let frame_size = config.frame_size;
    let threshold = config.energy_threshold;

    // Find first speech frame
    let mut start_idx = 0;
    for (i, chunk) in samples.chunks(frame_size).enumerate() {
        if calculate_energy(chunk) > threshold {
            start_idx = i.saturating_sub(1) * frame_size; // Include one frame before
            break;
        }
    }

    // Find last speech frame
    let mut end_idx = samples.len();
    for (i, chunk) in samples.chunks(frame_size).enumerate().rev() {
        if calculate_energy(chunk) > threshold {
            end_idx = ((i + 2) * frame_size).min(samples.len()); // Include one frame after
            break;
        }
    }

    if start_idx >= end_idx {
        return Vec::new();
    }

    samples[start_idx..end_idx].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy_calculation() {
        let silent = vec![0.0f32; 100];
        let loud = vec![0.5f32; 100];

        assert!(calculate_energy(&silent) < 0.001);
        assert!(calculate_energy(&loud) > 0.4);
    }

    #[test]
    fn test_zcr_calculation() {
        // No crossings
        let positive = vec![0.5f32; 100];
        assert!(calculate_zero_crossing_rate(&positive) < 0.01);

        // Maximum crossings (alternating)
        let alternating: Vec<f32> = (0..100)
            .map(|i| if i % 2 == 0 { 0.5 } else { -0.5 })
            .collect();
        assert!(calculate_zero_crossing_rate(&alternating) > 0.9);
    }

    #[test]
    fn test_vad_state_machine() {
        let config = VadConfig::for_wake_word();
        let mut vad = Vad::new(config.clone());

        // Simulate silence
        let silence = vec![0.001f32; config.frame_size];
        for _ in 0..10 {
            let event = vad.process_frame(&silence);
            assert!(matches!(event, VadEvent::Continue));
        }

        // Simulate speech
        let speech = vec![0.3f32; config.frame_size];
        let mut speech_started = false;
        for _ in 0..10 {
            let event = vad.process_frame(&speech);
            if matches!(event, VadEvent::SpeechStarted) {
                speech_started = true;
                break;
            }
        }
        assert!(speech_started);
    }
}