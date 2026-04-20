"""
HandFree Mouse - Gesture Recognition
Recognizes hand gestures from MediaPipe landmarks
"""

import numpy as np
from typing import List, Optional, Dict
from enum import Enum


class Gesture(Enum):
    """Supported hand gestures"""
    NONE = "none"
    POINT = "point"              # Index finger extended
    PINCH = "pinch"              # Thumb + index finger together
    TWO_FINGER = "two_finger"    # Index + middle finger extended
    OPEN_PALM = "open_palm"      # All fingers extended
    FIST = "fist"                # All fingers closed
    SWIPE_LEFT = "swipe_left"
    SWIPE_RIGHT = "swipe_right"
    SWIPE_UP = "swipe_up"
    SWIPE_DOWN = "swipe_down"
    PEACE = "peace"              # Index + middle finger V shape
    THUMBS_UP = "thumbs_up"


class GestureRecognizer:
    """Recognize gestures from hand landmarks"""
    
    def __init__(
        self,
        pinch_threshold: float = 0.05,
        swipe_threshold: float = 0.15,
        finger_threshold: float = 0.1
    ):
        """
        Initialize gesture recognizer
        
        Args:
            pinch_threshold: Distance threshold for pinch detection
            swipe_threshold: Distance threshold for swipe detection
            finger_threshold: Distance threshold for finger extension
        """
        self.pinch_threshold = pinch_threshold
        self.swipe_threshold = swipe_threshold
        self.finger_threshold = finger_threshold
        
        # Previous hand position for swipe detection
        self.prev_position = None
        self.gesture_history = []
        self.max_history = 5
        
    def recognize(self, landmarks: List[dict]) -> Gesture:
        """
        Recognize gesture from hand landmarks
        
        Args:
            landmarks: List of 21 hand landmarks
            
        Returns:
            Detected gesture
        """
        if not landmarks or len(landmarks) != 21:
            return Gesture.NONE
        
        # Check for pinch
        if self._is_pinch(landmarks):
            return Gesture.PINCH
        
        # Check for swipe
        swipe = self._detect_swipe(landmarks)
        if swipe != Gesture.NONE:
            return swipe
        
        # Check finger states
        fingers_up = self._get_fingers_up(landmarks)
        
        # Classify based on finger states
        if fingers_up == [0, 1, 0, 0, 0]:  # Only index
            return Gesture.POINT
        elif fingers_up == [0, 1, 1, 0, 0]:  # Index + middle
            if self._is_peace_sign(landmarks):
                return Gesture.PEACE
            return Gesture.TWO_FINGER
        elif fingers_up == [1, 1, 1, 1, 1]:  # All fingers
            return Gesture.OPEN_PALM
        elif fingers_up == [0, 0, 0, 0, 0]:  # No fingers
            return Gesture.FIST
        elif fingers_up == [1, 0, 0, 0, 0]:  # Only thumb
            if self._is_thumbs_up(landmarks):
                return Gesture.THUMBS_UP
        
        return Gesture.NONE
    
    def _is_pinch(self, landmarks: List[dict]) -> bool:
        """Check if thumb and index finger are pinched"""
        thumb_tip = landmarks[4]
        index_tip = landmarks[8]
        
        distance = np.sqrt(
            (thumb_tip['x'] - index_tip['x']) ** 2 +
            (thumb_tip['y'] - index_tip['y']) ** 2 +
            (thumb_tip['z'] - index_tip['z']) ** 2
        )
        
        return distance < self.pinch_threshold
    
    def _detect_swipe(self, landmarks: List[dict]) -> Gesture:
        """Detect swipe gestures"""
        # Use wrist position for swipe detection
        wrist = landmarks[0]
        current_pos = np.array([wrist['x'], wrist['y']])
        
        if self.prev_position is None:
            self.prev_position = current_pos
            return Gesture.NONE
        
        # Calculate movement
        delta = current_pos - self.prev_position
        distance = np.linalg.norm(delta)
        
        gesture = Gesture.NONE
        
        if distance > self.swipe_threshold:
            # Determine swipe direction
            angle = np.arctan2(delta[1], delta[0]) * 180 / np.pi
            
            if -45 <= angle < 45:
                gesture = Gesture.SWIPE_RIGHT
            elif 45 <= angle < 135:
                gesture = Gesture.SWIPE_DOWN
            elif -135 <= angle < -45:
                gesture = Gesture.SWIPE_UP
            else:
                gesture = Gesture.SWIPE_LEFT
        
        self.prev_position = current_pos
        return gesture
    
    def _get_fingers_up(self, landmarks: List[dict]) -> List[int]:
        """
        Check which fingers are extended
        
        Returns:
            List of 5 integers (0=down, 1=up) for [thumb, index, middle, ring, pinky]
        """
        fingers = []
        
        # Thumb (special case - check x-axis)
        if landmarks[4]['x'] < landmarks[3]['x']:  # Right hand
            fingers.append(1)
        else:
            fingers.append(0)
        
        # Other fingers (check y-axis)
        finger_tips = [8, 12, 16, 20]  # Index, middle, ring, pinky tips
        finger_pips = [6, 10, 14, 18]  # Corresponding PIP joints
        
        for tip, pip in zip(finger_tips, finger_pips):
            if landmarks[tip]['y'] < landmarks[pip]['y']:
                fingers.append(1)
            else:
                fingers.append(0)
        
        return fingers
    
    def _is_peace_sign(self, landmarks: List[dict]) -> bool:
        """Check if index and middle fingers form V shape"""
        index_tip = landmarks[8]
        middle_tip = landmarks[12]
        
        # Calculate distance between fingertips
        distance = np.sqrt(
            (index_tip['x'] - middle_tip['x']) ** 2 +
            (index_tip['y'] - middle_tip['y']) ** 2
        )
        
        # Peace sign has fingers spread apart
        return distance > 0.08
    
    def _is_thumbs_up(self, landmarks: List[dict]) -> bool:
        """Check if thumb is pointing up"""
        thumb_tip = landmarks[4]
        thumb_mcp = landmarks[2]
        wrist = landmarks[0]
        
        # Thumb should be above wrist and pointing up
        return (thumb_tip['y'] < wrist['y'] and 
                thumb_tip['y'] < thumb_mcp['y'])
    
    def get_stable_gesture(self, gesture: Gesture) -> Gesture:
        """
        Get stable gesture by filtering noise
        
        Args:
            gesture: Current detected gesture
            
        Returns:
            Stable gesture after filtering
        """
        self.gesture_history.append(gesture)
        
        # Keep only recent history
        if len(self.gesture_history) > self.max_history:
            self.gesture_history.pop(0)
        
        # Return most common gesture in history
        if len(self.gesture_history) >= 3:
            from collections import Counter
            most_common = Counter(self.gesture_history).most_common(1)[0][0]
            return most_common
        
        return gesture


def main():
    """Test gesture recognition"""
    print("Gesture Recognizer Test")
    print("Available gestures:")
    for gesture in Gesture:
        print(f"  - {gesture.value}")


if __name__ == "__main__":
    main()
