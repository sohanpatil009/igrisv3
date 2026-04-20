"""
HandFree Mouse - Main Application
Integrates hand tracking, gesture recognition, and mouse control
"""

import cv2
import json
import argparse
from pathlib import Path
from hand_tracker import HandTracker
from gesture_recognizer import GestureRecognizer, Gesture
from mouse_controller import MouseController


class HandFreeMouse:
    """Main application for hands-free mouse control"""
    
    def __init__(self, config_path: str = "config.json"):
        """
        Initialize HandFree Mouse
        
        Args:
            config_path: Path to configuration file
        """
        # Load configuration
        self.config = self._load_config(config_path)
        
        # Initialize components
        self.tracker = HandTracker(
            min_detection_confidence=self.config['tracking']['min_detection_confidence'],
            min_tracking_confidence=self.config['tracking']['min_tracking_confidence'],
            max_num_hands=self.config['tracking']['max_num_hands']
        )
        
        self.recognizer = GestureRecognizer(
            pinch_threshold=self.config['gestures']['pinch_threshold'],
            swipe_threshold=self.config['gestures']['swipe_threshold']
        )
        
        self.controller = MouseController(
            smoothing=self.config['mouse']['smoothing'],
            sensitivity=self.config['mouse']['sensitivity'],
            scroll_speed=self.config['mouse']['scroll_speed']
        )
        
        # State
        self.is_running = False
        self.is_paused = False
        self.show_ui = True
        
        print("HandFree Mouse initialized")
        print("Press 'q' to quit, 'p' to pause/resume, 'h' to hide/show UI")
    
    def _load_config(self, config_path: str) -> dict:
        """Load configuration from JSON file"""
        default_config = {
            "camera": {
                "device_id": 0,
                "width": 320,  # Reduced for better performance
                "height": 240,  # Reduced for better performance
                "fps": 30
            },
            "tracking": {
                "min_detection_confidence": 0.5,  # Lowered for faster detection
                "min_tracking_confidence": 0.3,  # Lowered for faster tracking
                "max_num_hands": 1
            },
            "mouse": {
                "smoothing": 0.7,  # Increased for smoother movement
                "sensitivity": 1.5,  # Increased for faster response
                "click_threshold": 0.03,
                "scroll_speed": 30  # Increased for faster scrolling
            },
            "gestures": {
                "pinch_threshold": 0.08,  # Slightly increased for reliability
                "swipe_threshold": 0.12,  # Slightly decreased for faster detection
                "hold_duration_ms": 300  # Reduced for faster drag
            }
        }
        
        try:
            with open(config_path, 'r') as f:
                config = json.load(f)
                # Merge with defaults
                for key in default_config:
                    if key not in config:
                        config[key] = default_config[key]
                return config
        except FileNotFoundError:
            print(f"Config file not found, using defaults")
            # Save default config
            with open(config_path, 'w') as f:
                json.dump(default_config, f, indent=2)
            return default_config
    
    def run(self):
        """Run the main application loop"""
        # Initialize camera
        cap = cv2.VideoCapture(self.config['camera']['device_id'])
        cap.set(cv2.CAP_PROP_FRAME_WIDTH, self.config['camera']['width'])
        cap.set(cv2.CAP_PROP_FRAME_HEIGHT, self.config['camera']['height'])
        cap.set(cv2.CAP_PROP_FPS, self.config['camera']['fps'])
        
        if not cap.isOpened():
            print("Error: Could not open camera")
            return
        
        self.is_running = True
        prev_hand_y = None
        
        try:
            while self.is_running:
                ret, frame = cap.read()
                if not ret:
                    print("Failed to grab frame")
                    break
                
                # Process frame
                annotated_frame, hand_landmarks = self.tracker.process_frame(frame)
                
                # Handle hand detection
                if hand_landmarks and not self.is_paused:
                    landmarks = hand_landmarks[0]  # Use first hand
                    
                    # Get index finger tip position for cursor
                    index_tip = landmarks[8]
                    self.controller.move_cursor(
                        index_tip['x'],
                        index_tip['y'],
                        frame.shape[1],
                        frame.shape[0]
                    )
                    
                    # Recognize gesture
                    gesture = self.recognizer.recognize(landmarks)
                    stable_gesture = self.recognizer.get_stable_gesture(gesture)
                    
                    # Handle gesture
                    if stable_gesture != Gesture.NONE:
                        self.controller.handle_gesture(stable_gesture)
                    
                    # Handle scroll with open palm
                    if stable_gesture == Gesture.OPEN_PALM:
                        palm_y = sum(lm['y'] for lm in landmarks) / len(landmarks)
                        if prev_hand_y is not None:
                            delta_y = prev_hand_y - palm_y
                            self.controller.scroll(delta_y)
                        prev_hand_y = palm_y
                    else:
                        prev_hand_y = None
                    
                    # Draw gesture info
                    if self.show_ui:
                        cv2.putText(
                            annotated_frame,
                            f"Gesture: {stable_gesture.value}",
                            (10, 70),
                            cv2.FONT_HERSHEY_SIMPLEX,
                            0.7,
                            (0, 255, 255),
                            2
                        )
                else:
                    prev_hand_y = None
                
                # Draw status
                if self.show_ui:
                    status = "PAUSED" if self.is_paused else "ACTIVE"
                    color = (0, 165, 255) if self.is_paused else (0, 255, 0)
                    cv2.putText(
                        annotated_frame,
                        f"Status: {status}",
                        (10, 110),
                        cv2.FONT_HERSHEY_SIMPLEX,
                        0.7,
                        color,
                        2
                    )
                
                # Show frame
                if self.show_ui:
                    cv2.imshow('HandFree Mouse', annotated_frame)
                
                # Handle keyboard input
                key = cv2.waitKey(1) & 0xFF
                if key == ord('q'):
                    break
                elif key == ord('p'):
                    self.is_paused = not self.is_paused
                    print(f"{'Paused' if self.is_paused else 'Resumed'}")
                elif key == ord('h'):
                    self.show_ui = not self.show_ui
                    if not self.show_ui:
                        cv2.destroyAllWindows()
        
        finally:
            self.stop()
            cap.release()
            cv2.destroyAllWindows()
    
    def stop(self):
        """Stop the application"""
        self.is_running = False
        self.controller.reset_state()
        self.tracker.close()
        print("HandFree Mouse stopped")


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description='HandFree Mouse - Gesture Control')
    parser.add_argument(
        '--config',
        type=str,
        default='config.json',
        help='Path to configuration file'
    )
    parser.add_argument(
        '--no-ui',
        action='store_true',
        help='Run without UI window'
    )
    
    args = parser.parse_args()
    
    # Create and run application
    app = HandFreeMouse(config_path=args.config)
    if args.no_ui:
        app.show_ui = False
    
    try:
        app.run()
    except KeyboardInterrupt:
        print("\nInterrupted by user")
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()
