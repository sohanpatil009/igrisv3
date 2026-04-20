"""
HandFree Mouse - Mouse Controller
Controls mouse cursor and system actions based on gestures
"""

import pyautogui
import numpy as np
from typing import Tuple, Optional
import time
import platform

# Platform-specific imports
if platform.system() == "Windows":
    try:
        import screen_brightness_control as sbc
    except ImportError:
        sbc = None
else:
    sbc = None

from gesture_recognizer import Gesture


class MouseController:
    """Control mouse and system based on hand gestures"""
    
    def __init__(
        self,
        smoothing: float = 0.5,
        sensitivity: float = 1.0,
        scroll_speed: int = 20
    ):
        """
        Initialize mouse controller
        
        Args:
            smoothing: Cursor smoothing factor (0-1, higher = smoother)
            sensitivity: Mouse sensitivity multiplier
            scroll_speed: Scroll speed in pixels
        """
        self.smoothing = smoothing
        self.sensitivity = sensitivity
        self.scroll_speed = scroll_speed
        
        # Get screen size
        self.screen_width, self.screen_height = pyautogui.size()
        
        # Disable PyAutoGUI failsafe
        pyautogui.FAILSAFE = False
        
        # State tracking
        self.prev_cursor_pos = None
        self.is_dragging = False
        self.last_click_time = 0
        self.click_cooldown = 0.3  # seconds
        
        # Gesture state
        self.prev_gesture = Gesture.NONE
        self.gesture_start_time = 0
        self.hold_duration = 0.5  # seconds
        
        print(f"Mouse Controller initialized")
        print(f"Screen size: {self.screen_width}x{self.screen_height}")
    
    def move_cursor(
        self,
        hand_x: float,
        hand_y: float,
        frame_width: int,
        frame_height: int
    ):
        """
        Move cursor based on hand position
        
        Args:
            hand_x: Hand x position (0-1 normalized)
            hand_y: Hand y position (0-1 normalized)
            frame_width: Camera frame width
            frame_height: Camera frame height
        """
        # Map hand position to screen coordinates
        # Flip x-axis for mirror effect
        target_x = int((1 - hand_x) * self.screen_width * self.sensitivity)
        target_y = int(hand_y * self.screen_height * self.sensitivity)
        
        # Clamp to screen bounds
        target_x = max(0, min(self.screen_width - 1, target_x))
        target_y = max(0, min(self.screen_height - 1, target_y))
        
        # Apply smoothing
        if self.prev_cursor_pos is not None:
            prev_x, prev_y = self.prev_cursor_pos
            target_x = int(prev_x * self.smoothing + target_x * (1 - self.smoothing))
            target_y = int(prev_y * self.smoothing + target_y * (1 - self.smoothing))
        
        # Move cursor
        pyautogui.moveTo(target_x, target_y, duration=0)
        self.prev_cursor_pos = (target_x, target_y)
    
    def handle_gesture(self, gesture: Gesture):
        """
        Handle detected gesture
        
        Args:
            gesture: Detected gesture
        """
        current_time = time.time()
        
        # Check if gesture changed
        if gesture != self.prev_gesture:
            self.gesture_start_time = current_time
            self.prev_gesture = gesture
        
        # Calculate hold duration
        hold_time = current_time - self.gesture_start_time
        
        # Handle gestures
        if gesture == Gesture.PINCH:
            self._handle_pinch(hold_time)
        elif gesture == Gesture.TWO_FINGER:
            self._handle_two_finger()
        elif gesture == Gesture.OPEN_PALM:
            self._handle_scroll()
        elif gesture == Gesture.SWIPE_LEFT:
            self._handle_swipe_left()
        elif gesture == Gesture.SWIPE_RIGHT:
            self._handle_swipe_right()
        elif gesture == Gesture.SWIPE_UP:
            self._handle_swipe_up()
        elif gesture == Gesture.SWIPE_DOWN:
            self._handle_swipe_down()
        elif gesture == Gesture.FIST:
            self._handle_fist()
        elif gesture == Gesture.PEACE:
            self._handle_peace()
        elif gesture == Gesture.THUMBS_UP:
            self._handle_thumbs_up()
        else:
            # Release drag if no gesture
            if self.is_dragging:
                pyautogui.mouseUp()
                self.is_dragging = False
    
    def _handle_pinch(self, hold_time: float):
        """Handle pinch gesture (left click or drag)"""
        current_time = time.time()
        
        if hold_time > self.hold_duration:
            # Hold pinch = drag
            if not self.is_dragging:
                pyautogui.mouseDown()
                self.is_dragging = True
                print("Drag started")
        else:
            # Quick pinch = click
            if current_time - self.last_click_time > self.click_cooldown:
                if not self.is_dragging:
                    pyautogui.click()
                    self.last_click_time = current_time
                    print("Left click")
    
    def _handle_two_finger(self):
        """Handle two-finger gesture (right click)"""
        current_time = time.time()
        
        if current_time - self.last_click_time > self.click_cooldown:
            pyautogui.rightClick()
            self.last_click_time = current_time
            print("Right click")
    
    def _handle_scroll(self):
        """Handle scroll gesture"""
        if self.prev_cursor_pos is not None:
            # Scroll based on vertical hand movement
            # This is handled in the main loop with hand position delta
            pass
    
    def scroll(self, delta_y: float):
        """
        Scroll based on hand movement
        
        Args:
            delta_y: Vertical movement delta
        """
        scroll_amount = int(delta_y * self.scroll_speed)
        if abs(scroll_amount) > 5:
            pyautogui.scroll(scroll_amount)
            print(f"Scroll: {scroll_amount}")
    
    def _handle_swipe_left(self):
        """Handle swipe left (previous window/tab)"""
        pyautogui.hotkey('alt', 'shift', 'tab')
        print("Swipe left - Previous window")
    
    def _handle_swipe_right(self):
        """Handle swipe right (next window/tab)"""
        pyautogui.hotkey('alt', 'tab')
        print("Swipe right - Next window")
    
    def _handle_swipe_up(self):
        """Handle swipe up (volume up)"""
        pyautogui.press('volumeup')
        print("Swipe up - Volume up")
    
    def _handle_swipe_down(self):
        """Handle swipe down (volume down)"""
        pyautogui.press('volumedown')
        print("Swipe down - Volume down")
    
    def _handle_fist(self):
        """Handle fist gesture (pause control)"""
        print("Fist - Pause")
        # Could be used to pause gesture control
    
    def _handle_peace(self):
        """Handle peace sign (screenshot)"""
        pyautogui.hotkey('win', 'shift', 's')
        print("Peace - Screenshot")
    
    def _handle_thumbs_up(self):
        """Handle thumbs up (like/confirm)"""
        print("Thumbs up - Confirm")
    
    def adjust_brightness(self, delta: int):
        """
        Adjust screen brightness
        
        Args:
            delta: Brightness change (-100 to 100)
        """
        if sbc is not None:
            try:
                current = sbc.get_brightness()[0]
                new_brightness = max(0, min(100, current + delta))
                sbc.set_brightness(new_brightness)
                print(f"Brightness: {new_brightness}%")
            except Exception as e:
                print(f"Brightness control error: {e}")
        else:
            print("Brightness control not available")
    
    def reset_state(self):
        """Reset controller state"""
        if self.is_dragging:
            pyautogui.mouseUp()
            self.is_dragging = False
        self.prev_cursor_pos = None
        self.prev_gesture = Gesture.NONE


def main():
    """Test mouse controller"""
    print("Mouse Controller Test")
    controller = MouseController()
    
    # Test cursor movement
    print("\nTesting cursor movement...")
    for i in range(10):
        x = 0.5 + 0.1 * np.sin(i * 0.5)
        y = 0.5 + 0.1 * np.cos(i * 0.5)
        controller.move_cursor(x, y, 640, 480)
        time.sleep(0.1)
    
    print("\nTest complete")


if __name__ == "__main__":
    main()
