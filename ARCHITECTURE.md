# IGRIS - System Architecture & Flow Diagrams

## Table of Contents
1. [High-Level Architecture](#high-level-architecture)
2. [Voice Processing Pipeline](#voice-processing-pipeline)
3. [Plugin System Architecture](#plugin-system-architecture)
4. [NLU Engine Flow](#nlu-engine-flow)
5. [Camera Control Flow](#camera-control-flow)
6. [Reminder System Flow](#reminder-system-flow)
7. [Component Interaction Diagram](#component-interaction-diagram)
8. [Data Flow Architecture](#data-flow-architecture)

---

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         IGRIS SYSTEM                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │   UI Layer   │    │  Core Voice  │    │   Plugin     │       │
│  │  (Dioxus)    │◄──►│   Pipeline   │◄──►│   System     │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│         │                    │                    │             │
│         │                    │                    │             │
│  ┌──────▼────────────────────▼────────────────────▼──────┐      │
│  │              Configuration Manager                    │      │
│  └───────────────────────────────────────────────────────┘      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Layer Breakdown

**1. UI Layer (Dioxus 0.7)**
- Main application window with animated orb
- Settings panel
- Camera preview interface
- Presentation mode UI
- Real-time transcription display

**2. Core Voice Pipeline**
- Wake Word Detection
- Voice Activity Detection (VAD)
- Speech-to-Text (Whisper)
- Natural Language Understanding (SBERT)
- Text-to-Speech (Piper)

**3. Plugin System**
- Built-in Rust plugins
- Command routing
- App lifecycle management
- System control integration

**4. Configuration Manager**
- JSON-based settings storage
- Runtime configuration updates
- Model path management

---

## Voice Processing Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│                    VOICE PROCESSING FLOW                         │
└─────────────────────────────────────────────────────────────────┘

    Audio Input (Microphone)
           │
           ▼
    ┌──────────────┐
    │     VAD      │  ◄── Detects speech activity
    │   (Voice     │      Filters silence
    │   Activity)  │
    └──────┬───────┘
           │
           ▼
    ┌──────────────┐
    │  Wake Word   │  ◄── Listens for "Arise"
    │  Detection   │      Low CPU overhead
    └──────┬───────┘      Continuous monitoring
           │
           │ [Wake Word Detected]
           ▼
    ┌──────────────┐
    │   Whisper    │  ◄── Converts speech to text
    │     STT      │      base-q8_0 quantized model
    │              │      Max 15 seconds listening
    └──────┬───────┘
           │
           │ [Transcribed Text]
           ▼
    ┌──────────────┐
    │  SBERT NLU   │  ◄── Semantic understanding
    │   Engine     │      Intent classification
    │              │      Similarity threshold: 0.45
    └──────┬───────┘
           │
           │ [Intent + Entities]
           ▼
    ┌──────────────┐
    │     NER      │  ◄── Extract parameters
    │  (Named      │      Numbers, times, names
    │   Entity)    │      File patterns
    └──────┬───────┘
           │
           │ [Structured Command]
           ▼
    ┌──────────────┐
    │   Plugin     │  ◄── Route to handler
    │   Router     │      Execute command
    │              │      Generate response
    └──────┬───────┘
           │
           │ [Response Text]
           ▼
    ┌──────────────┐
    │  Piper TTS   │  ◄── Convert text to speech
    │              │      LibriTTS voice model
    │              │      Configurable speed/volume
    └──────┬───────┘
           │
           ▼
    Audio Output (Speakers)
```

---

## Plugin System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      PLUGIN SYSTEM                              │
└─────────────────────────────────────────────────────────────────┘

                    ┌──────────────────┐
                    │  Plugin Manager  │
                    │   (system.rs)    │
                    └────────┬─────────┘
                             │
                ┌────────────┼────────────┐
                │            │            │
                ▼            ▼            ▼
         ┌──────────┐ ┌──────────┐ ┌──────────┐
         │ Built-in │ │ Command  │ │  Plugin  │
         │ Plugins  │ │ Routing  │ │Validation│
         └────┬─────┘ └────┬─────┘ └────┬─────┘
              │            │            │
              └────────────┼────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        │                  │                  │
        ▼                  ▼                  ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│   Browsers   │  │  Utilities   │  │    Media     │
│ - Chrome     │  │ - Calculator │  │ - VLC        │
│ - Firefox    │  │ - Notepad    │  │ - Spotify    │
│ - Edge       │  │ - Terminal   │  │ - Photos     │
└──────────────┘  └──────────────┘  └──────────────┘

        ▼                  ▼                  ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│    Office    │  │    Gaming    │  │   Creative   │
│ - Word       │  │ - Steam      │  │ - Photoshop  │
│ - Excel      │  │ - Discord    │  │ - Blender    │
│ - PowerPoint │  │ - Epic       │  │ - Figma      │
└──────────────┘  └──────────────┘  └──────────────┘

        ▼                  ▼                  ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│   Camera     │  │   Reminders  │  │System Control│
│ - Photo      │  │ - Alarms     │  │ - Volume     │
│ - Video      │  │ - Timers     │  │ - Brightness │
│ - Preview    │  │ - Scheduler  │  │ - WiFi/BT    │
└──────────────┘  └──────────────┘  └──────────────┘

        ▼
┌──────────────┐
│    Files     │
│ - Create     │
│ - Delete     │
│ - Search     │
└──────────────┘
```

### Plugin Interface

```rust
pub trait Plugin {
    fn name(&self) -> &str;
    fn can_handle(&self, intent: &str) -> bool;
    fn execute(&self, command: &Command) -> Result<Response>;
    fn aliases(&self) -> Vec<String>;
}
```

---

## NLU Engine Flow

```
┌─────────────────────────────────────────────────────────────────┐
│              NATURAL LANGUAGE UNDERSTANDING                     │
└─────────────────────────────────────────────────────────────────┘

    User Input: "Open Chrome"
           │
           ▼
    ┌──────────────────┐
    │  Text Cleaning   │  ◄── Lowercase, trim
    │  & Normalization │      Remove punctuation
    └────────┬─────────┘
             │
             ▼
    ┌──────────────────┐
    │ SBERT Embedding  │  ◄── Convert to 384-dim vector
    │  (all-MiniLM-L6) │      Semantic representation
    └────────┬─────────┘
             │
             ▼
    ┌──────────────────┐
    │ Intent Database  │  ◄── Pre-computed embeddings
    │  - open_app      │      for all intents
    │  - close_app     │
    │  - system_cmd    │
    │  - file_op       │
    │  - camera        │
    │  - reminder      │
    └────────┬─────────┘
             │
             ▼
    ┌──────────────────┐
    │ Cosine Similarity│  ◄── Compare input vs intents
    │   Calculation    │      Find best match
    └────────┬─────────┘
             │
             ▼
    ┌──────────────────┐
    │   Threshold      │  ◄── Score >= 0.45 ?
    │   Check (0.45)   │      Yes: Accept intent
    └────────┬─────────┘      No: Fallback to keywords
             │
             ├─── [High Confidence] ───┐
             │                         │
             └─── [Low Confidence] ────┤
                                       │
                                       ▼
                              ┌──────────────────┐
                              │  Named Entity    │
                              │  Recognition     │
                              │  - App names     │
                              │  - Numbers       │
                              │  - Time values   │
                              │  - File patterns │
                              └────────┬─────────┘
                                       │
                                       ▼
                              ┌──────────────────┐
                              │ Structured       │
                              │ Command Object   │
                              │ {                │
                              │   intent: "open" │
                              │   target: "chrome"│
                              │ }                │
                              └──────────────────┘
```

---

## Camera Control Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    CAMERA CONTROL SYSTEM                         │
└─────────────────────────────────────────────────────────────────┘

    Voice Command: "Take a photo"
           │
           ▼
    ┌──────────────────┐
    │  Camera Plugin   │  ◄── Receives command
    │   Activation     │
    └────────┬─────────┘
             │
             ▼
    ┌──────────────────┐
    │  FFmpeg Device   │  ◄── Detect available cameras
    │   Detection      │      List video devices
    └────────┬─────────┘
             │
             ▼
    ┌──────────────────┐
    │  Camera Preview  │  ◄── Show live feed in UI
    │   UI Launch      │      Dioxus component
    └────────┬─────────┘
             │
             ├─── [Photo Mode] ────────┐
             │                         │
             └─── [Video Mode] ────────┤
                                       │
                                       ▼
                              ┌──────────────────┐
                              │  FFmpeg Capture  │
                              │  - Photo: Single │
                              │    frame grab    │
                              │  - Video: Stream │
                              │    recording     │
                              └────────┬─────────┘
                                       │
                                       ▼
                              ┌──────────────────┐
                              │  File Storage    │
                              │  - Photos/       │
                              │  - Videos/       │
                              │  Timestamp name  │
                              └────────┬─────────┘
                                       │
                                       ▼
                              ┌──────────────────┐
                              │  TTS Response    │
                              │  "Photo saved"   │
                              │  "Recording..."  │
                              └──────────────────┘
```

---

## Reminder System Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    REMINDER & ALARM SYSTEM                       │
└─────────────────────────────────────────────────────────────────┘

    Voice: "Set alarm for 7 AM"
           │
           ▼
    ┌──────────────────┐
    │  NER Extraction  │  ◄── Parse time: "7 AM"
    │  Time Parser     │      Convert to DateTime
    └────────┬─────────┘
             │
             ▼
    ┌──────────────────┐
    │  Reminder Object │  ◄── Create reminder
    │  Creation        │      {
    │                  │        time: 07:00,
    │                  │        message: "Alarm",
    │                  │        type: Alarm
    │                  │      }
    └────────┬─────────┘
             │
             ▼
    ┌──────────────────┐
    │  Persistent      │  ◄── Save to storage
    │  Storage         │      JSON file
    └────────┬─────────┘
             │
             ▼
    ┌──────────────────┐
    │  Background      │  ◄── Tokio async task
    │  Scheduler       │      Checks every 10 sec
    │  Thread          │
    └────────┬─────────┘
             │
             │ [Time Check Loop]
             │
             ▼
    ┌──────────────────┐
    │  Time Comparison │  ◄── Current >= Target?
    │  Current vs      │
    │  Target Time     │
    └────────┬─────────┘
             │
             ├─── [Not Yet] ──► Continue Loop
             │
             └─── [Time Reached] ───┐
                                    │
                                    ▼
                           ┌──────────────────┐
                           │  Trigger Alarm   │
                           │  - Play sound    │
                           │  - Show UI notif │
                           │  - TTS announce  │
                           └────────┬─────────┘
                                    │
                                    ▼
                           ┌──────────────────┐
                           │  Remove from     │
                           │  Active List     │
                           └──────────────────┘
```

---

## Component Interaction Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                  COMPONENT INTERACTIONS                         │
└─────────────────────────────────────────────────────────────────┘

                    ┌──────────────┐
                    │     User     │
                    └──────┬───────┘
                           │ Voice Input
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                      Main UI (Dioxus)                        │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐              │
│  │   Orb      │  │  Settings  │  │   Camera   │              │
│  │ Animation  │  │   Panel    │  │   Panel    │              │
│  └────────────┘  └────────────┘  └────────────┘              │
└──────┬───────────────────────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────────────────────┐
│                   Voice Pipeline Manager                     │
│  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐            │
│  │ VAD  │→ │ Wake │→ │ STT  │→ │ NLU  │→ │ TTS  │            │
│  └──────┘  └──────┘  └──────┘  └──────┘  └──────┘            │
└──────┬───────────────────────────────────────────────────────┘
       │
       ▼
┌──────────────────────────────────────────────────────────────┐
│                     Plugin Router                            │
└──────┬───────────────────────────────────────────────────────┘
       │
       ├──────────────┬──────────────┬──────────────┐
       ▼              ▼              ▼              ▼
┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐
│   System   │ │   Camera   │ │  Reminder  │ │   Files    │
│  Control   │ │   Plugin   │ │   Plugin   │ │   Plugin   │
└─────┬──────┘ └─────┬──────┘ └─────┬──────┘ └─────┬──────┘
      │              │              │              │
      ▼              ▼              ▼              ▼
┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐
│   OS API   │ │   FFmpeg   │ │ Scheduler  │ │  File I/O  │
└────────────┘ └────────────┘ └────────────┘ └────────────┘
```

---

## Data Flow Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      DATA FLOW DIAGRAM                          │
└─────────────────────────────────────────────────────────────────┘

[Audio Stream] ──────────────────────────────────────┐
                                                      │
                                                      ▼
                                            ┌──────────────────┐
                                            │   Audio Buffer   │
                                            │   (Ring Buffer)  │
                                            └────────┬─────────┘
                                                     │
                                                     ▼
                                            ┌──────────────────┐
                                            │  VAD Processing  │
                                            │  (Silence Filter)│
                                            └────────┬─────────┘
                                                     │
                                    ┌────────────────┼────────────────┐
                                    │                │                │
                                    ▼                ▼                ▼
                            [Silence]        [Speech Detected]  [Wake Word]
                                │                   │                │
                                └───────────────────┼────────────────┘
                                                    │
                                                    ▼
                                          ┌──────────────────┐
                                          │  Whisper Model   │
                                          │  (STT Engine)    │
                                          └────────┬─────────┘
                                                   │
                                                   │ [Text String]
                                                   ▼
                                          ┌──────────────────┐
                                          │  SBERT Model     │
                                          │  (Embeddings)    │
                                          └────────┬─────────┘
                                                   │
                                                   │ [Vector 384-dim]
                                                   ▼
                                          ┌──────────────────┐
                                          │ Intent Matcher   │
                                          │ (Cosine Sim)     │
                                          └────────┬─────────┘
                                                   │
                                                   │ [Intent + Score]
                                                   ▼
                                          ┌──────────────────┐
                                          │  NER Extractor   │
                                          │  (Regex/Rules)   │
                                          └────────┬─────────┘
                                                   │
                                                   │ [Command Object]
                                                   ▼
                                          ┌──────────────────┐
                                          │  Plugin Router   │
                                          └────────┬─────────┘
                                                   │
                        ┌──────────────────────────┼──────────────────────────┐
                        │                          │                          │
                        ▼                          ▼                          ▼
              ┌──────────────────┐      ┌──────────────────┐      ┌──────────────────┐
              │  System Commands │      │  File Operations │      │  Camera Control  │
              │  - Volume        │      │  - Create        │      │  - Capture       │
              │  - Brightness    │      │  - Search        │      │  - Record        │
              └────────┬─────────┘      └────────┬─────────┘      └────────┬─────────┘
                       │                         │                         │
                       └─────────────────────────┼─────────────────────────┘
                                                 │
                                                 │ [Response Text]
                                                 ▼
                                        ┌──────────────────┐
                                        │   Piper TTS      │
                                        │   (Synthesis)    │
                                        └────────┬─────────┘
                                                 │
                                                 │ [Audio Samples]
                                                 ▼
                                        ┌──────────────────┐
                                        │  Audio Output    │
                                        │  (Speakers)      │
                                        └──────────────────┘
```

---

## State Management

```
┌─────────────────────────────────────────────────────────────────┐
│                      APPLICATION STATE                           │
└─────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│                      Global State                            │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  use_signal(AppState)                                  │  │
│  │  - is_awake: bool                                      │  │
│  │  - is_listening: bool                                  │  │
│  │  - current_transcription: String                       │  │
│  │  - last_response: String                               │  │
│  │  - camera_active: bool                                 │  │
│  │  - active_reminders: Vec<Reminder>                     │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
         │
         ├──► UI Components (read state)
         │
         └──► Voice Pipeline (write state)


┌──────────────────────────────────────────────────────────────┐
│                    Configuration State                       │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  Config (loaded from pkg/config.json)                  │  │
│  │  - personality: String                                 │  │
│  │  - recognition_sensitivity: f32                        │  │
│  │  - tts_speed: f32                                      │  │
│  │  - tts_volume: f32                                     │  │
│  │  - hotkey: String                                      │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
         │
         └──► Persisted to disk on change
```

---

## Threading Model

```
┌─────────────────────────────────────────────────────────────────┐
│                      THREAD ARCHITECTURE                         │
└─────────────────────────────────────────────────────────────────┘

Main Thread (UI)
    │
    ├──► Dioxus Renderer
    │    └──► React to state changes
    │
    └──► Event Loop
         └──► Handle user interactions

Audio Thread (Tokio)
    │
    ├──► Microphone Capture
    │    └──► Continuous audio stream
    │
    ├──► VAD Processing
    │    └──► Real-time speech detection
    │
    └──► Audio Playback
         └──► TTS output

Voice Processing Thread (Tokio)
    │
    ├──► Wake Word Detection
    │    └──► Low-latency monitoring
    │
    ├──► STT Processing (Whisper)
    │    └──► CPU-intensive inference
    │
    └──► TTS Synthesis (Piper)
         └──► Audio generation

NLU Thread (Tokio)
    │
    ├──► SBERT Inference
    │    └──► Embedding calculation
    │
    └──► Intent Matching
         └──► Database lookup

Plugin Execution Thread (Tokio)
    │
    ├──► Command Execution
    │    └──► OS API calls
    │
    └──► File Operations
         └──► I/O operations

Scheduler Thread (Tokio)
    │
    └──► Reminder Checker
         └──► Periodic time checks (10s interval)

Camera Thread (Tokio)
    │
    ├──► FFmpeg Process
    │    └──► Video capture
    │
    └──► Frame Processing
         └──► Preview updates
```

---

## Model Loading & Initialization

```
┌─────────────────────────────────────────────────────────────────┐
│                    INITIALIZATION FLOW                           │
└─────────────────────────────────────────────────────────────────┘

Application Start
    │
    ▼
┌──────────────────┐
│  Check Models    │  ◄── Verify pkg/ directory
│  Existence       │      Check for required files
└────────┬─────────┘
         │
         ├─── [Models Missing] ───┐
         │                        │
         └─── [Models Present] ───┤
                                  │
                                  ▼
                         ┌──────────────────┐
                         │  Setup Manager   │
                         │  - Download STT  │
                         │  - Download TTS  │
                         │  - Download NLU  │
                         │  - Download FFmpeg│
                         └────────┬─────────┘
                                  │
                                  ▼
                         ┌──────────────────┐
                         │  Load Models     │
                         │  into Memory     │
                         │  - Whisper       │
                         │  - SBERT         │
                         │  - Piper         │
                         └────────┬─────────┘
                                  │
                                  ▼
                         ┌──────────────────┐
                         │  Initialize      │
                         │  Audio Devices   │
                         │  - Mic input     │
                         │  - Speaker output│
                         └────────┬─────────┘
                                  │
                                  ▼
                         ┌──────────────────┐
                         │  Load Config     │
                         │  (config.json)   │
                         └────────┬─────────┘
                                  │
                                  ▼
                         ┌──────────────────┐
                         │  Initialize      │
                         │  Plugins         │
                         │  (Built-in)      │
                         └────────┬─────────┘
                                  │
                                  ▼
                         ┌──────────────────┐
                         │  Start Scheduler │
                         │  Thread          │
                         └────────┬─────────┘
                                  │
                                  ▼
                         ┌──────────────────┐
                         │  Launch UI       │
                         │  (Dioxus)        │
                         └────────┬─────────┘
                                  │
                                  ▼
                         ┌──────────────────┐
                         │  Enter Voice     │
                         │  Loop (Standby)  │
                         └──────────────────┘
```

---

## Error Handling Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                      ERROR HANDLING                              │
└─────────────────────────────────────────────────────────────────┘

Error Occurs
    │
    ▼
┌──────────────────┐
│  Error Type      │
│  Detection       │
└────────┬─────────┘
         │
         ├─── [STT Error] ──────────┐
         │                          │
         ├─── [NLU Error] ──────────┤
         │                          │
         ├─── [Plugin Error] ───────┤
         │                          │
         ├─── [Audio Error] ────────┤
         │                          │
         └─── [System Error] ───────┤
                                    │
                                    ▼
                           ┌──────────────────┐
                           │  Log Error       │
                           │  (Console/File)  │
                           └────────┬─────────┘
                                    │
                                    ▼
                           ┌──────────────────┐
                           │  Generate        │
                           │  User-Friendly   │
                           │  Message         │
                           └────────┬─────────┘
                                    │
                                    ▼
                           ┌──────────────────┐
                           │  TTS Response    │
                           │  "Sorry, I       │
                           │  couldn't..."    │
                           └────────┬─────────┘
                                    │
                                    ▼
                           ┌──────────────────┐
                           │  Fallback        │
                           │  Behavior        │
                           │  - Retry         │
                           │  - Skip          │
                           │  - Reset state   │
                           └──────────────────┘
```

---

## Performance Optimization

### Model Optimization
- **Whisper**: Quantized to Q8_0 (81MB vs 140MB)
- **SBERT**: Cached embeddings for common intents
- **Piper**: Streaming synthesis for low latency

### Threading Strategy
- Audio processing on dedicated thread
- Non-blocking UI updates
- Async command execution

### Memory Management
- Lazy model loading
- Audio buffer ring structure
- Periodic cleanup of old transcriptions

---

## Security Considerations

```
┌─────────────────────────────────────────────────────────────────┐
│                      SECURITY LAYERS                             │
└─────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│  Layer 1: Local Processing                                   │
│  - All voice data processed locally                          │
│  - No external network calls                                 │
│  - No cloud dependencies                                     │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│  Layer 2: File System Access                                 │
│  - Sandboxed file operations                                 │
│  - User permission checks                                    │
│  - Path validation                                           │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│  Layer 3: Command Validation                                 │
│  - Intent verification                                       │
│  - Parameter sanitization                                    │
│  - Plugin authorization                                      │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│  Layer 4: Configuration Security                             │
│  - Encrypted sensitive settings                              │
│  - Secure credential storage                                 │
│  - Read-only model files                                     │
└──────────────────────────────────────────────────────────────┘
```

---

**End of Architecture Documentation**

*For implementation details, refer to source code in `src/` directory*
