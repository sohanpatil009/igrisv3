# PROJECT SYNOPSIS

## IGRIS - Advanced Offline AI Voice Assistant

---

## ABSTRACT

IGRIS (Intelligent General-purpose Responsive Interactive System) is a fully offline, voice-activated AI assistant built using Rust and Dioxus 0.7 framework, designed to provide comprehensive hands-free control over desktop systems while ensuring complete data privacy. Unlike cloud-dependent voice assistants such as Siri, Alexa, or Google Assistant, IGRIS processes all voice commands locally using state-of-the-art machine learning models including OpenAI Whisper for speech recognition, SBERT for natural language understanding, and Piper for text-to-speech synthesis.

The system features a sophisticated voice processing pipeline that begins with wake word detection ("Arise"), followed by real-time speech-to-text conversion, semantic intent classification, and natural language response generation. IGRIS provides extensive functionality through a plugin-based architecture, enabling users to control applications, manage system settings (volume, brightness, connectivity), perform file operations, capture photos and videos, and set alarms and reminders through voice commands.

A key innovation of IGRIS is its unique self-presentation mode where the assistant explains its own architecture through animated slides with TTS narration, demonstrating its capabilities in an interactive manner. The system also features comprehensive camera control using FFmpeg integration for photo capture and video recording with real-time preview.

Built with performance and privacy as core principles, IGRIS operates efficiently on modest hardware (4GB RAM minimum) while maintaining real-time response latency under 500ms. The modular architecture ensures extensibility through Rust-based plugins, allowing users to adapt the assistant to specific workflows. The project demonstrates the feasibility of creating a powerful, privacy-focused voice assistant that operates entirely offline while maintaining high accuracy, low latency, and cross-platform compatibility across Windows, macOS, and Linux systems.

**Keywords:** Voice Assistant, Offline AI, Speech Recognition, Natural Language Processing, Rust, Privacy-Focused Computing, Cross-Platform Development, Semantic Understanding, Plugin Architecture

---

## 1. INTRODUCTION

### 1.1 Project Overview
IGRIS (Intelligent General-purpose Responsive Interactive System) is an advanced, fully offline voice-activated AI assistant designed to provide hands-free control over desktop systems. Built using Rust programming language and Dioxus 0.7 framework, IGRIS combines state-of-the-art machine learning models with a robust plugin architecture to deliver a comprehensive voice-controlled computing experience without requiring internet connectivity.

### 1.2 Motivation
Modern voice assistants like Siri, Alexa, and Google Assistant require constant internet connectivity and raise privacy concerns due to cloud-based processing. IGRIS addresses these limitations by providing:
- Complete offline functionality ensuring data privacy
- Low-latency response times through local processing
- Cross-platform compatibility (Windows, macOS, Linux)
- Extensible architecture for custom functionality

### 1.3 Objectives
- Develop a fully offline voice assistant with natural language understanding
- Implement secure cross-platform file sharing over LAN
- Create an extensible plugin system for custom commands
- Provide comprehensive system control through voice commands
- Ensure real-time performance with optimized resource usage

---

## 2. SYSTEM ARCHITECTURE

### 2.1 Core Components

#### 2.1.1 Voice Processing Pipeline
The voice processing system consists of four integrated modules:

**Wake Word Detection**
- Activation phrase: "Arise"
- Continuous audio monitoring with low CPU overhead
- Real-time detection with minimal false positives

**Speech-to-Text (STT)**
- OpenAI Whisper model (base-q8_0 quantized, 81MB)
- Offline transcription with high accuracy
- Support for natural speech patterns and accents

**Natural Language Understanding (NLU)**
- SBERT (Sentence-BERT) semantic embeddings (80MB)
- Intent classification with 0.45 similarity threshold
- Named Entity Recognition (NER) for parameter extraction
- Context-aware conversation memory

**Text-to-Speech (TTS)**
- Piper TTS engine with LibriTTS voice model (50MB)
- Natural-sounding voice synthesis
- Configurable speed and volume

#### 2.1.2 Plugin System
- Unified command routing architecture
- Built-in Rust plugins for browsers, utilities, media, office applications
- Smart app alias recognition (e.g., "Chrome" → "google chrome")
- Dynamic plugin loading and validation

#### 2.1.3 System Control
- Application lifecycle management (launch/close)
- System settings control (volume, brightness, WiFi, Bluetooth)
- Power management (sleep, shutdown, lock)
- File operations with multi-threaded search
- Camera control using FFmpeg integration

#### 2.1.4 Scheduler System
- Background alarm and reminder service
- Time-based notification triggers
- Persistent storage of scheduled tasks
- Voice-activated alarm management

### 2.2 Technology Stack

**Programming Language:** Rust 1.70+
- Memory safety without garbage collection
- Zero-cost abstractions for performance
- Concurrent programming with fearless concurrency

**UI Framework:** Dioxus 0.7
- Cross-platform native UI rendering
- Reactive state management
- Component-based architecture

**Machine Learning Models:**
- Whisper (Speech Recognition)
- SBERT all-MiniLM-L6-v2 (Semantic Understanding)
- Piper TTS (Voice Synthesis)

**Additional Technologies:**
- FFmpeg (Camera and media processing)

### 2.3 Data Flow Architecture

```
User Voice Input
    ↓
Voice Activity Detection (VAD)
    ↓
Wake Word Detection → [Standby Mode Loop]
    ↓
Speech-to-Text (Whisper)
    ↓
Natural Language Understanding (SBERT)
    ↓
Named Entity Recognition (NER)
    ↓
Plugin System Router
    ↓
Command Handler Execution
    ↓
Text-to-Speech Response (Piper)
    ↓
Audio Output
```

---

## 3. FUNCTIONAL REQUIREMENTS

### 3.1 Voice Commands
- Application control (open/close applications)
- System settings management (volume, brightness, connectivity)
- File operations (create, delete, search)
- Camera control (photo capture, video recording)
- Alarm and reminder management
- Self-presentation mode

### 3.2 User Interface
- Dark theme with gradient background
- Animated voice activity visualization
- Real-time transcription display
- Settings panel for configuration
- Camera preview interface
- Interactive presentation mode

### 3.3 Configuration Management
- Persistent configuration storage
- Adjustable recognition sensitivity
- Customizable TTS parameters
- Hotkey configuration

---

## 4. NON-FUNCTIONAL REQUIREMENTS

### 4.1 Performance
- Real-time voice processing with <500ms latency
- Efficient resource usage (4GB RAM minimum, 8GB recommended)
- Multi-threaded file search for large directories
- Optimized model inference

### 4.2 Security
- Local processing ensuring data privacy
- No external network dependencies
- Secure credential handling

### 4.3 Reliability
- Graceful error handling and recovery
- Automatic model download on first run
- Fallback mechanisms for failed commands
- Persistent state management

### 4.4 Usability
- Natural language command interface
- Visual feedback for all operations
- Comprehensive voice responses
- Intuitive UI design

### 4.5 Portability
- Cross-platform support (Windows, macOS, Linux)
- Platform-specific optimizations
- Consistent behavior across operating systems

---

## 5. IMPLEMENTATION DETAILS

### 5.1 Module Structure

**Core Modules:**
- `core/stt.rs` - Whisper integration
- `core/tts.rs` - Piper TTS engine
- `core/vad.rs` - Voice activity detection
- `core/wake_word.rs` - Wake word detection

**NLU Modules:**
- `nlu/engine.rs` - Intent matching engine
- `nlu/sbert.rs` - Semantic embeddings
- `nlu/ner.rs` - Entity extraction
- `nlu/context.rs` - Conversation memory

**Command Handlers:**
- `commands/system.rs` - System control
- `commands/files.rs` - File operations
- `commands/ffmpeg_camera.rs` - Camera control
- `commands/reminders.rs` - Alarms & reminders

**Plugin System:**
- `plugins/system.rs` - Plugin manager
- `plugins/builtin/` - Built-in plugin implementations

**UI Components:**
- `ui/settings.rs` - Settings panel
- `ui/camera_panel.rs` - Camera interface
- `ui/presentation/` - Self-presentation mode

### 5.2 Key Algorithms

**Intent Matching:**
- Cosine similarity between user input and command embeddings
- Threshold-based classification (0.45 default)
- Fallback to keyword matching for low-confidence results

**File Search:**
- Multi-threaded directory traversal
- Pattern matching with wildcard support
- Recursive search with depth limiting

---

## 6. TESTING AND VALIDATION

### 6.1 Testing Strategy
- Unit tests for individual modules
- Integration tests for voice pipeline
- Performance benchmarking for model inference
- Cross-platform compatibility testing

### 6.2 Quality Assurance
- Code review and static analysis
- Memory leak detection
- Stress testing for concurrent operations
- User acceptance testing

---

## 7. FUTURE ENHANCEMENTS

- Multi-language support for international users
- Custom wake word training capability
- Mobile companion application
- Voice command history and analytics
- Advanced context-aware conversations
- Integration with smart home devices
- Enhanced camera features with filters and effects

---

## 8. CONCLUSION

IGRIS represents a significant advancement in offline voice assistant technology, combining cutting-edge machine learning models with robust system integration. The project demonstrates the feasibility of creating a fully functional, privacy-focused voice assistant that operates entirely offline while maintaining high performance and extensibility. The modular architecture and plugin system ensure that IGRIS can be adapted to various use cases and extended with custom functionality.

---

## 9. REFERENCES

1. OpenAI Whisper - Speech Recognition Model
2. Sentence-BERT (SBERT) - Semantic Textual Similarity
3. Piper TTS - Neural Text-to-Speech
4. Dioxus Framework - Cross-platform UI Development
5. Rust Programming Language - Systems Programming
6. FFmpeg - Multimedia Framework

---

## PROJECT DETAILS

**Project Name:** IGRIS - Advanced Offline AI Voice Assistant

**Technology:** Rust, Dioxus 0.7, Machine Learning (Whisper, SBERT, Piper)

**Platform:** Cross-platform (Windows, macOS, Linux)

**Development Status:** Active Development

**License:** MIT License

**Repository:** [GitHub Repository URL]

---

**Prepared for:** IEEE Project Submission

**Date:** January 2026
