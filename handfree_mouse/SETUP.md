# HandFree Mouse - Setup Guide

Complete setup instructions for HandFree Mouse gesture control system.

## 📋 Prerequisites

### System Requirements
- **OS**: Windows 10+, macOS 10.13+, or Linux
- **Python**: 3.8 or higher
- **Rust**: 1.70 or higher (already installed for IGRIS)
- **Camera**: Webcam or external USB camera
- **RAM**: 2GB minimum, 4GB recommended
- **CPU**: Multi-core processor recommended

### Check Prerequisites

```bash
# Check Python version
python --version
# Should show: Python 3.8.x or higher

# Check Rust version
cargo --version
# Should show: cargo 1.70.x or higher

# Check pip
pip --version
```

## 🚀 Installation

### Step 1: Navigate to Directory

```bash
cd igrisv3/handfree_mouse
```

### Step 2: Install Python Dependencies

```bash
# Install all required packages
pip install -r python/requirements.txt

# Or install individually
pip install opencv-python mediapipe numpy pyautogui pynput
```

### Step 3: Build Rust Bindings (Optional)

The Rust bindings provide better performance but are optional.

```bash
cd rust
cargo build --release
cd ..
```

### Step 4: Test Installation

```bash
# Test hand tracking
python python/hand_tracker.py

# Test gesture recognition
python python/gesture_recognizer.py

# Test mouse controller
python python/mouse_controller.py
```

## 🎮 Quick Start

### Run Standalone

```bash
# Run with default settings
python python/main.py

# Run with custom config
python python/main.py --config my_config.json

# Run without UI (headless mode)
python python/main.py --no-ui
```

### Controls

- **Q** - Quit application
- **P** - Pause/Resume gesture control
- **H** - Hide/Show UI window

## ⚙️ Configuration

Edit `python/config.json` to customize settings:

### Camera Settings

```json
"camera": {
  "device_id": 0,        // Camera index (0=default, 1=external)
  "width": 640,          // Frame width
  "height": 480,         // Frame height
  "fps": 30              // Frames per second
}
```

### Tracking Settings

```json
"tracking": {
  "min_detection_confidence": 0.7,  // Hand detection threshold
  "min_tracking_confidence": 0.5,   // Hand tracking threshold
  "max_num_hands": 1                // Number of hands to track
}
```

### Mouse Settings

```json
"mouse": {
  "smoothing": 0.5,      // Cursor smoothing (0-1, higher=smoother)
  "sensitivity": 1.0,    // Mouse sensitivity multiplier
  "click_threshold": 0.03,  // Distance for click detection
  "scroll_speed": 20     // Scroll speed in pixels
}
```

### Gesture Settings

```json
"gestures": {
  "pinch_threshold": 0.05,      // Distance for pinch detection
  "swipe_threshold": 0.15,      // Distance for swipe detection
  "hold_duration_ms": 500       // Hold duration for drag
}
```

## 🔧 Troubleshooting

### Camera Issues

**Problem**: Camera not detected
```bash
# List available cameras
python -c "import cv2; print([i for i in range(10) if cv2.VideoCapture(i).isOpened()])"

# Try different device_id in config.json
"device_id": 1  # or 2, 3, etc.
```

**Problem**: Low FPS / Laggy
```bash
# Reduce resolution in config.json
"width": 320,
"height": 240
```

### Hand Tracking Issues

**Problem**: Hand not detected
- Ensure good lighting
- Keep hand in camera view
- Adjust `min_detection_confidence` (lower = more sensitive)

**Problem**: Jittery tracking
- Increase `smoothing` value (0.5 → 0.8)
- Increase `min_tracking_confidence`

### Gesture Issues

**Problem**: Gestures not recognized
- Adjust threshold values in config
- Check gesture guide in README
- Ensure fingers are clearly visible

**Problem**: False positives
- Increase threshold values
- Improve lighting conditions
- Keep background simple

### Python Package Issues

**Problem**: ModuleNotFoundError
```bash
# Reinstall packages
pip install --upgrade -r python/requirements.txt

# Check installation
pip list | grep opencv
pip list | grep mediapipe
```

**Problem**: OpenCV import error
```bash
# Uninstall and reinstall
pip uninstall opencv-python opencv-python-headless
pip install opencv-python
```

### Rust Build Issues

**Problem**: PyO3 build fails
```bash
# Install Python development headers
# Ubuntu/Debian
sudo apt-get install python3-dev

# macOS (via Homebrew)
brew install python

# Windows
# Ensure Python is installed from python.org
```

**Problem**: Enigo compilation error
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

## 🔌 IGRIS Integration

### Add to IGRIS Voice Commands

1. Create plugin file: `igrisv3/src/plugins/builtin/handfree_mouse.rs`

2. Add voice commands:
```rust
"enable hand mouse" -> Start gesture control
"disable hand mouse" -> Stop gesture control
"calibrate hand mouse" -> Calibrate tracking
```

3. Register plugin in `igrisv3/src/plugins/system.rs`

### Python-Rust Bridge

The Rust bindings can be called from Python:

```python
# Import Rust module
import handfree_mouse_rust

# Use Rust functions (faster than PyAutoGUI)
handfree_mouse_rust.move_cursor(100, 200)
handfree_mouse_rust.left_click()
handfree_mouse_rust.scroll(10, "vertical")
```

## 📊 Performance Optimization

### Reduce Latency

1. **Lower resolution**: 320x240 instead of 640x480
2. **Reduce smoothing**: 0.3 instead of 0.5
3. **Use Rust bindings**: Faster than PyAutoGUI
4. **Close other apps**: Free up CPU/RAM

### Improve Accuracy

1. **Better lighting**: Bright, even lighting
2. **Simple background**: Solid color background
3. **Higher confidence**: Increase detection thresholds
4. **Calibration**: Adjust sensitivity for your setup

## 🧪 Testing

### Unit Tests

```bash
# Test individual components
python python/hand_tracker.py --test
python python/gesture_recognizer.py --test
python python/mouse_controller.py --test
```

### Integration Test

```bash
# Run full application in test mode
python python/main.py --config test_config.json
```

### Rust Tests

```bash
cd rust
cargo test
```

## 📝 Development

### Adding Custom Gestures

1. **Define gesture** in `gesture_recognizer.py`:
```python
class Gesture(Enum):
    MY_GESTURE = "my_gesture"

def _detect_my_gesture(self, landmarks):
    # Your detection logic
    return True
```

2. **Add handler** in `mouse_controller.py`:
```python
def _handle_my_gesture(self):
    # Your action
    print("My gesture detected!")
```

3. **Register** in `handle_gesture()` method

### Debugging

Enable verbose logging:

```python
# Add to main.py
import logging
logging.basicConfig(level=logging.DEBUG)
```

## 🔒 Security & Privacy

- All processing is local (no cloud/internet)
- Camera frames are not saved or recorded
- No data collection or telemetry
- Opt-in activation (disabled by default)

## 📚 Additional Resources

- [MediaPipe Hands Documentation](https://google.github.io/mediapipe/solutions/hands.html)
- [OpenCV Python Tutorials](https://docs.opencv.org/4.x/d6/d00/tutorial_py_root.html)
- [PyAutoGUI Documentation](https://pyautogui.readthedocs.io/)
- [PyO3 User Guide](https://pyo3.rs/)

## 💡 Tips & Best Practices

1. **Lighting**: Use bright, even lighting for best results
2. **Distance**: Keep hand 30-60cm from camera
3. **Background**: Use simple, solid-color background
4. **Calibration**: Adjust sensitivity for your setup
5. **Practice**: Gestures become more natural with practice

## 🆘 Getting Help

If you encounter issues:

1. Check this setup guide
2. Review troubleshooting section
3. Check GitHub issues
4. Create new issue with:
   - OS and Python version
   - Error messages
   - Config file
   - Steps to reproduce

---

**Ready to go!** Run `python python/main.py` to start.
