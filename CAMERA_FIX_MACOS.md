# Camera Fix for macOS

## Problem
Camera operations fail on macOS with FFmpeg because macOS requires explicit camera permissions.

## Root Cause
macOS security blocks camera access by default. Apps (including terminal apps) need user permission to access the camera.

## Solution

### Option 1: Grant Terminal Camera Access (Quick Fix)

1. **Open System Settings**
   - Click Apple menu → System Settings
   - Go to **Privacy & Security** → **Camera**

2. **Grant Permission**
   - Find your terminal app (Terminal, iTerm2, etc.)
   - Toggle it **ON** to allow camera access
   - If your terminal isn't listed, you may need to run the app first

3. **Restart Terminal**
   - Close and reopen your terminal
   - Run IGRIS again

### Option 2: Build as macOS App Bundle (Recommended)

Build IGRIS as a proper macOS app with Info.plist permissions:

```bash
# Build the app bundle
dx bundle --release

# The app will be in target/release/bundle/macos/
# Run it from Finder or:
open target/release/bundle/macos/IGRIS.app
```

The app bundle includes the required permissions in Info.plist:
- `NSCameraUsageDescription` - Camera access for photos/videos
- `NSMicrophoneUsageDescription` - Microphone access for voice commands
- `NSCameraUseContinuityCameraDeviceType` - Continuity camera support

## Testing

Run the test script to check camera access:

```bash
./test_camera.sh
```

This will:
1. Check if FFmpeg is installed
2. List available cameras
3. Attempt to capture a test photo
4. Show detailed error messages if it fails

## Changes Made

### 1. Updated FFmpeg Arguments (src/media/ffmpeg_camera/mod.rs)

**Photo Capture:**
```rust
// macOS: Use "0:none" for video-only (no audio for photos)
"-i", "0:none"
```

**Video Recording:**
```rust
// macOS: Use "0:0" for video device 0 + audio device 0
"-video_size", "1280x720",
"-framerate", "30",
"-i", "0:0"
```

### 2. Added Info.plist Permissions (Dioxus.toml)

```toml
[bundle.macos]
info_plist = """
<key>NSCameraUsageDescription</key>
<string>IGRIS needs access to your camera to take photos and record videos.</string>
<key>NSMicrophoneUsageDescription</key>
<string>IGRIS needs access to your microphone for voice commands and video recording.</string>
<key>NSCameraUseContinuityCameraDeviceType</key>
<true/>
"""
```

## Detected Devices

Your Mac has:
- **Video**: FaceTime HD Camera (device index 0)
- **Audio**: MacBook Air Microphone (device index 0)

## Common Issues

### "Input/output error" when capturing
- **Cause**: No camera permission granted
- **Fix**: Grant camera access in System Settings (see Option 1 above)

### "Device not found"
- **Cause**: Wrong device index
- **Fix**: Run `ffmpeg -f avfoundation -list_devices true -i ""` to see available devices

### Camera works in other apps but not IGRIS
- **Cause**: Terminal doesn't have camera permission
- **Fix**: Grant permission to your terminal app specifically

## Verification

After granting permissions, test with:

```bash
# Quick test - capture a photo
ffmpeg -f avfoundation -video_size 1280x720 -framerate 30 -i "0:none" -frames:v 1 -y ~/test.jpg

# Check if photo was created
ls -lh ~/test.jpg
open ~/test.jpg
```

If this works, IGRIS camera will work too!

## Notes

- Camera permissions are per-app on macOS
- Running from terminal requires terminal to have camera access
- Building as app bundle (.app) is more user-friendly
- First camera access will show a system permission dialog
- Permissions persist after being granted once
