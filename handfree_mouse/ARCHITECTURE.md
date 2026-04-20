# HandFree Mouse - Architecture

Technical architecture and design documentation.

## 🏗️ System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     HandFree Mouse System                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
        ┌─────────────────────────────────────────┐
        │         Camera Input (OpenCV)            │
        │  - Capture frames at 30 FPS              │
        │  - 640x480 resolution                    │
        └──────────────────┬──────────────────────┘
                           │
                           ▼
        ┌─────────────────────────────────────────┐
        │    Hand Tracking (MediaPipe)             │
        │  - Detect 21 hand landmarks              │
        │  - Real-time tracking                    │
        │  - Confidence filtering                  │
        └──────────────────┬──────────────────────┘
                           │
                           ▼
        ┌─────────────────────────────────────────┐
        │   Gesture Recognition (Custom)           │
        │  - Pinch detection                       │
        │  - Swipe detection                       │
        │  - Finger state analysis                 │
        │  - Gesture stabilization                 │
        └──────────────────┬──────────────────────┘
                           │
                           ▼
        ┌─────────────────────────────────────────┐
        │   Mouse Control (PyAutoGUI/Rust)         │
        │  - Cursor movement                       │
        │  - Click actions                         │
        │  - Scroll control                        │
        │  - System shortcuts                      │
        └─────────────────────────────────────────┘
```

## 📦 Component Architecture

### 1. Hand Tracker (`hand_tracker.py`)

**Responsibilities**:
- Camera frame capture
- Hand landmark detection
- FPS calculation
- Visual feedback

**Key Classes**:
```python
class HandTracker:
    - process_frame(frame) -> (annotated_frame, landmarks)
    - get_landmark_position(landmarks, id, shape) -> (x, y)
    - calculate_distance(landmarks, id1, id2) -> float
```

**MediaPipe Integration**:
- Uses `mediapipe.solutions.hands`
- Detects 21 landmarks per hand
- Provides 3D coordinates (x, y, z)

### 2. Gesture Recognizer (`gesture_recognizer.py`)

**Responsibilities**:
- Gesture classification
- Swipe detection
- Finger state analysis
- Gesture stabilization

**Key Classes**:
```python
class GestureRecognizer:
    - recognize(landmarks) -> Gesture
    - get_stable_gesture(gesture) -> Gesture
    - _is_pinch(landmarks) -> bool
    - _detect_swipe(landmarks) -> Gesture
    - _get_fingers_up(landmarks) -> List[int]
```

**Gesture Types**:
```python
class Gesture(Enum):
    NONE, POINT, PINCH, TWO_FINGER,
    OPEN_PALM, FIST, SWIPE_LEFT, SWIPE_RIGHT,
    SWIPE_UP, SWIPE_DOWN, PEACE, THUMBS_UP
```

### 3. Mouse Controller (`mouse_controller.py`)

**Responsibilities**:
- Cursor movement with smoothing
- Click handling
- Scroll control
- System shortcuts
- Drag & drop

**Key Classes**:
```python
class MouseController:
    - move_cursor(x, y, width, height)
    - handle_gesture(gesture)
    - scroll(delta_y)
    - adjust_brightness(delta)
```

**Control Mapping**:
| Gesture | Action | Implementation |
|---------|--------|----------------|
| Point | Move cursor | `pyautogui.moveTo()` |
| Pinch | Left click | `pyautogui.click()` |
| Pinch + Hold | Drag | `pyautogui.mouseDown/Up()` |
| Two Finger | Right click | `pyautogui.rightClick()` |
| Open Palm | Scroll | `pyautogui.scroll()` |
| Swipe Left/Right | Switch window | `alt+tab` |
| Swipe Up/Down | Volume | `volumeup/down` |

### 4. Main Application (`main.py`)

**Responsibilities**:
- Component orchestration
- Configuration management
- Main event loop
- UI rendering

**Key Classes**:
```python
class HandFreeMouse:
    - run() -> Main loop
    - _load_config(path) -> dict
    - stop() -> Cleanup
```

## 🔄 Data Flow

### Frame Processing Pipeline

```
Camera Frame (BGR)
    │
    ├─> Convert to RGB
    │
    ├─> MediaPipe Processing
    │   ├─> Hand Detection
    │   ├─> Landmark Extraction
    │   └─> Confidence Filtering
    │
    ├─> Gesture Recognition
    │   ├─> Distance Calculations
    │   ├─> Finger State Analysis
    │   ├─> Swipe Detection
    │   └─> Gesture Stabilization
    │
    ├─> Mouse Control
    │   ├─> Coordinate Mapping
    │   ├─> Smoothing
    │   ├─> Action Execution
    │   └─> State Management
    │
    └─> Visual Feedback
        ├─> Draw Landmarks
        ├─> Display Gesture
        └─> Show Status
```

### Coordinate Transformation

```
Hand Landmark (0-1 normalized)
    │
    ├─> Flip X-axis (mirror effect)
    │   x' = 1 - x
    │
    ├─> Scale to screen size
    │   screen_x = x' * screen_width
    │   screen_y = y * screen_height
    │
    ├─> Apply sensitivity
    │   screen_x *= sensitivity
    │   screen_y *= sensitivity
    │
    ├─> Apply smoothing
    │   x_smooth = prev_x * α + screen_x * (1-α)
    │   y_smooth = prev_y * α + screen_y * (1-α)
    │
    └─> Clamp to bounds
        x_final = clamp(x_smooth, 0, screen_width)
        y_final = clamp(y_smooth, 0, screen_height)
```

## 🧮 Algorithms

### Pinch Detection

```python
def is_pinch(landmarks):
    thumb_tip = landmarks[4]
    index_tip = landmarks[8]
    
    distance = sqrt(
        (thumb_tip.x - index_tip.x)² +
        (thumb_tip.y - index_tip.y)² +
        (thumb_tip.z - index_tip.z)²
    )
    
    return distance < threshold  # 0.05
```

### Swipe Detection

```python
def detect_swipe(landmarks, prev_position):
    wrist = landmarks[0]
    current_pos = [wrist.x, wrist.y]
    
    delta = current_pos - prev_position
    distance = norm(delta)
    
    if distance > threshold:  # 0.15
        angle = atan2(delta[1], delta[0]) * 180 / π
        
        if -45 <= angle < 45:
            return SWIPE_RIGHT
        elif 45 <= angle < 135:
            return SWIPE_DOWN
        elif -135 <= angle < -45:
            return SWIPE_UP
        else:
            return SWIPE_LEFT
    
    return NONE
```

### Finger Extension Detection

```python
def get_fingers_up(landmarks):
    fingers = []
    
    # Thumb (check x-axis)
    if landmarks[4].x < landmarks[3].x:
        fingers.append(1)  # Extended
    else:
        fingers.append(0)  # Closed
    
    # Other fingers (check y-axis)
    tips = [8, 12, 16, 20]  # Index, middle, ring, pinky
    pips = [6, 10, 14, 18]  # PIP joints
    
    for tip, pip in zip(tips, pips):
        if landmarks[tip].y < landmarks[pip].y:
            fingers.append(1)  # Extended
        else:
            fingers.append(0)  # Closed
    
    return fingers  # [thumb, index, middle, ring, pinky]
```

### Gesture Stabilization

```python
def get_stable_gesture(gesture, history):
    history.append(gesture)
    
    if len(history) > max_history:
        history.pop(0)
    
    if len(history) >= 3:
        # Return most common gesture
        return mode(history)
    
    return gesture
```

## 🔌 Python-Rust FFI

### Architecture

```
Python Layer (High-level)
    │
    ├─> PyO3 Bindings
    │
    └─> Rust Layer (Low-level)
        ├─> enigo (Mouse control)
        └─> Native OS APIs
```

### Function Mapping

| Python | Rust | Performance Gain |
|--------|------|------------------|
| `pyautogui.moveTo()` | `enigo.move_mouse()` | 2-3x faster |
| `pyautogui.click()` | `enigo.button()` | 2x faster |
| `pyautogui.scroll()` | `enigo.scroll()` | 1.5x faster |

### Usage Example

```python
# Python code
import handfree_mouse_rust as rust

# Use Rust functions (faster)
rust.move_cursor(100, 200)
rust.left_click()
rust.scroll(10, "vertical")

# Fallback to PyAutoGUI if Rust not available
try:
    import handfree_mouse_rust as rust
    USE_RUST = True
except ImportError:
    import pyautogui
    USE_RUST = False
```

## 📊 Performance Characteristics

### Latency Breakdown

| Component | Latency | Optimization |
|-----------|---------|--------------|
| Camera capture | 33ms (30 FPS) | Use 60 FPS camera |
| MediaPipe processing | 10-15ms | GPU acceleration |
| Gesture recognition | 1-2ms | Optimized algorithms |
| Mouse control | 1-5ms | Rust bindings |
| **Total** | **45-55ms** | **~20 FPS effective** |

### Resource Usage

| Resource | Usage | Notes |
|----------|-------|-------|
| CPU | 15-25% | Single core |
| RAM | 200MB | MediaPipe models |
| GPU | Optional | MediaPipe can use GPU |
| Camera | 640x480@30fps | Configurable |

## 🔒 Security Considerations

### Privacy
- All processing is local
- No network communication
- No data storage
- No recording

### Safety
- Failsafe mechanisms
- Gesture confirmation
- Pause/resume control
- Visual indicators

## 🚀 Future Enhancements

### Planned Features
1. **GPU Acceleration** - Use CUDA for MediaPipe
2. **Multi-Hand Support** - Track both hands
3. **Custom Gestures** - User-defined gestures
4. **Gesture Macros** - Record and replay
5. **ML Training** - Personalized gesture recognition
6. **Voice Integration** - IGRIS voice commands

### Performance Improvements
1. **Async Processing** - Parallel frame processing
2. **Model Optimization** - Quantized MediaPipe models
3. **Caching** - Cache gesture patterns
4. **Predictive Tracking** - Kalman filtering

---

**Architecture Version**: 1.0
**Last Updated**: April 2026
