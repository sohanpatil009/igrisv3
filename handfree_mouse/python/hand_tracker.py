"""
HandFree Mouse - Hand Tracking Engine
Uses OpenCV for hand detection (MediaPipe-free fallback)
"""

import cv2
import numpy as np
from typing import Optional, Tuple, List
import time


class HandTracker:
    """Real-time hand tracking using OpenCV"""
    
    def __init__(
        self,
        min_detection_confidence: float = 0.7,
        min_tracking_confidence: float = 0.5,
        max_num_hands: int = 1
    ):
        """
        Initialize hand tracker
        
        Args:
            min_detection_confidence: Minimum confidence for hand detection
            min_tracking_confidence: Minimum confidence for hand tracking
            max_num_hands: Maximum number of hands to track
        """
        self.max_num_hands = max_num_hands
        self.min_detection_confidence = min_detection_confidence
        self.min_tracking_confidence = min_tracking_confidence
        
        print("[HandTracker] Using OpenCV-based hand detection")
        print("[HandTracker] No MediaPipe required")
        
        self.results = None
        self.frame_count = 0
        self.fps = 0
        self.last_time = time.time()
        
        # Hand connections for drawing
        self.HAND_CONNECTIONS = [
            (0, 1), (1, 2), (2, 3), (3, 4),  # Thumb
            (0, 5), (5, 6), (6, 7), (7, 8),  # Index
            (0, 9), (9, 10), (10, 11), (11, 12),  # Middle
            (0, 13), (13, 14), (14, 15), (15, 16),  # Ring
            (0, 17), (17, 18), (18, 19), (19, 20),  # Pinky
            (5, 9), (9, 13), (13, 17)  # Palm
        ]
        
        self.results = None
        self.frame_count = 0
        self.fps = 0
        self.last_time = time.time()
        
    def process_frame(self, frame: np.ndarray) -> Tuple[np.ndarray, Optional[List]]:
        """
        Process a single frame and detect hands
        
        Args:
            frame: Input BGR image from camera
            
        Returns:
            Tuple of (annotated_frame, hand_landmarks)
        """
        # Calculate FPS
        self.frame_count += 1
        current_time = time.time()
        if current_time - self.last_time >= 1.0:
            self.fps = self.frame_count
            self.frame_count = 0
            self.last_time = current_time
        
        # Detect hands (optimized)
        hand_landmarks_list = self._simple_hand_detection(frame)
        
        # Only create annotated frame if needed (for UI mode)
        annotated_frame = frame
        
        return annotated_frame, hand_landmarks_list if hand_landmarks_list else None
    
    def _simple_hand_detection(self, frame: np.ndarray) -> Optional[List]:
        """
        Optimized hand detection using skin color and contours
        Returns approximate hand landmarks
        """
        # Resize frame for faster processing
        small_frame = cv2.resize(frame, (320, 240))
        
        # Convert to YCrCb color space (better for skin detection)
        ycrcb = cv2.cvtColor(small_frame, cv2.COLOR_BGR2YCrCb)
        
        # Skin color range in YCrCb
        lower_skin = np.array([0, 133, 77], dtype=np.uint8)
        upper_skin = np.array([255, 173, 127], dtype=np.uint8)
        
        # Create mask
        mask = cv2.inRange(ycrcb, lower_skin, upper_skin)
        
        # Fast morphological operations
        kernel = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (3, 3))
        mask = cv2.morphologyEx(mask, cv2.MORPH_CLOSE, kernel, iterations=1)
        mask = cv2.morphologyEx(mask, cv2.MORPH_OPEN, kernel, iterations=1)
        
        # Find contours (faster with RETR_EXTERNAL)
        contours, _ = cv2.findContours(mask, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
        
        if not contours:
            return None
        
        # Get largest contour
        largest_contour = max(contours, key=cv2.contourArea)
        area = cv2.contourArea(largest_contour)
        
        if area < 2000:  # Minimum area threshold (adjusted for smaller frame)
            return None
        
        # Get convex hull for better hand shape
        hull = cv2.convexHull(largest_contour)
        
        # Get bounding box
        x, y, w, h = cv2.boundingRect(hull)
        
        # Scale back to original frame size
        scale_x = frame.shape[1] / 320
        scale_y = frame.shape[0] / 240
        x = int(x * scale_x)
        y = int(y * scale_y)
        w = int(w * scale_x)
        h = int(h * scale_y)
        
        # Calculate key points
        center_x = x + w // 2
        center_y = y + h // 2
        top_y = y
        bottom_y = y + h
        
        # Create approximate 21-point hand landmarks (optimized)
        landmarks = []
        frame_h, frame_w = frame.shape[:2]
        
        # Normalize coordinates
        def normalize(px, py):
            return {
                'x': max(0, min(1, px / frame_w)),
                'y': max(0, min(1, py / frame_h)),
                'z': 0.0
            }
        
        # Wrist (0)
        landmarks.append(normalize(center_x, bottom_y))
        
        # Thumb (1-4) - left side
        thumb_x = x + int(w * 0.15)
        for i in range(4):
            landmarks.append(normalize(thumb_x, bottom_y - int(i * h * 0.22)))
        
        # Index finger (5-8) - pointing up
        index_x = x + int(w * 0.35)
        for i in range(4):
            landmarks.append(normalize(index_x, bottom_y - int(i * h * 0.28)))
        
        # Middle finger (9-12) - center, tallest
        for i in range(4):
            landmarks.append(normalize(center_x, bottom_y - int(i * h * 0.30)))
        
        # Ring finger (13-16)
        ring_x = x + int(w * 0.65)
        for i in range(4):
            landmarks.append(normalize(ring_x, bottom_y - int(i * h * 0.28)))
        
        # Pinky (17-20) - right side, shortest
        pinky_x = x + int(w * 0.85)
        for i in range(4):
            landmarks.append(normalize(pinky_x, bottom_y - int(i * h * 0.22)))
        
        return [landmarks]
    
    def get_landmark_position(
        self,
        landmarks: List[dict],
        landmark_id: int,
        frame_shape: Tuple[int, int]
    ) -> Tuple[int, int]:
        """
        Get pixel coordinates of a specific landmark
        
        Args:
            landmarks: List of hand landmarks
            landmark_id: ID of the landmark (0-20)
            frame_shape: (height, width) of the frame
            
        Returns:
            (x, y) pixel coordinates
        """
        if 0 <= landmark_id < len(landmarks):
            landmark = landmarks[landmark_id]
            h, w = frame_shape
            x = int(landmark['x'] * w)
            y = int(landmark['y'] * h)
            return x, y
        return 0, 0
    
    def calculate_distance(
        self,
        landmarks: List[dict],
        id1: int,
        id2: int
    ) -> float:
        """
        Calculate Euclidean distance between two landmarks
        
        Args:
            landmarks: List of hand landmarks
            id1: First landmark ID
            id2: Second landmark ID
            
        Returns:
            Distance between landmarks (normalized 0-1)
        """
        if 0 <= id1 < len(landmarks) and 0 <= id2 < len(landmarks):
            p1 = landmarks[id1]
            p2 = landmarks[id2]
            
            distance = np.sqrt(
                (p1['x'] - p2['x']) ** 2 +
                (p1['y'] - p2['y']) ** 2 +
                (p1['z'] - p2['z']) ** 2
            )
            return distance
        return 0.0
    
    def close(self):
        """Release resources"""
        self.hands.close()


def main():
    """Test hand tracking with webcam"""
    print("HandFree Mouse - Hand Tracker Test")
    print("Press 'q' to quit")
    
    # Initialize camera
    cap = cv2.VideoCapture(0)
    cap.set(cv2.CAP_PROP_FRAME_WIDTH, 640)
    cap.set(cv2.CAP_PROP_FRAME_HEIGHT, 480)
    
    # Initialize tracker
    tracker = HandTracker()
    
    try:
        while True:
            ret, frame = cap.read()
            if not ret:
                print("Failed to grab frame")
                break
            
            # Process frame
            annotated_frame, hand_landmarks = tracker.process_frame(frame)
            
            # Display hand info
            if hand_landmarks:
                for i, landmarks in enumerate(hand_landmarks):
                    # Get index finger tip position
                    index_tip = tracker.get_landmark_position(
                        landmarks, 8, frame.shape[:2]
                    )
                    
                    # Draw index finger position
                    cv2.circle(annotated_frame, index_tip, 10, (255, 0, 0), -1)
                    
                    # Calculate pinch distance
                    pinch_dist = tracker.calculate_distance(landmarks, 4, 8)
                    
                    # Display info
                    cv2.putText(
                        annotated_frame,
                        f"Hand {i+1} - Pinch: {pinch_dist:.3f}",
                        (10, 70 + i * 40),
                        cv2.FONT_HERSHEY_SIMPLEX,
                        0.7,
                        (255, 255, 0),
                        2
                    )
            
            # Show frame
            cv2.imshow('HandFree Mouse - Hand Tracker', annotated_frame)
            
            # Check for quit
            if cv2.waitKey(1) & 0xFF == ord('q'):
                break
    
    finally:
        cap.release()
        cv2.destroyAllWindows()
        tracker.close()
        print("Hand tracker closed")


if __name__ == "__main__":
    main()
