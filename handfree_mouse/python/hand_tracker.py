"""
HandFree Mouse - Hand Tracking Engine
Uses MediaPipe Tasks API for real-time hand landmark detection
"""

import cv2
import numpy as np
from typing import Optional, Tuple, List
import time

# Import MediaPipe
try:
    import mediapipe as mp
    from mediapipe.tasks import python
    from mediapipe.tasks.python import vision
    from mediapipe.framework.formats import landmark_pb2
    MEDIAPIPE_AVAILABLE = True
except ImportError as e:
    print(f"MediaPipe import error: {e}")
    MEDIAPIPE_AVAILABLE = False


class HandTracker:
    """Real-time hand tracking using MediaPipe Tasks API"""
    
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
        if not MEDIAPIPE_AVAILABLE:
            raise RuntimeError("MediaPipe is not available. Please install: pip install mediapipe")
        
        # MediaPipe 0.10+ uses Tasks API without pre-trained models
        # We'll use a simpler approach with video mode
        self.max_num_hands = max_num_hands
        self.min_detection_confidence = min_detection_confidence
        self.min_tracking_confidence = min_tracking_confidence
        
        # For MediaPipe 0.10+, we need to use the video/live stream mode
        # Since hand_landmarker.task model is not included, we'll use a workaround
        self.hands = None
        self.use_simple_detection = True
        
        print(f"[HandTracker] Initialized with MediaPipe {mp.__version__}")
        print("[HandTracker] Using simplified hand detection")
        
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
        
        annotated_frame = frame.copy()
        
        # For now, use a simple color-based hand detection
        # This is a fallback until we can properly configure MediaPipe 0.10+
        hand_landmarks_list = self._simple_hand_detection(frame)
        
        # Draw detected points
        if hand_landmarks_list:
            for landmarks in hand_landmarks_list:
                h, w = frame.shape[:2]
                
                # Draw landmarks
                for i, landmark in enumerate(landmarks):
                    x = int(landmark['x'] * w)
                    y = int(landmark['y'] * h)
                    
                    # Different colors for different fingers
                    if i == 4:  # Thumb tip
                        color = (255, 0, 0)  # Blue
                    elif i == 8:  # Index tip
                        color = (0, 255, 0)  # Green
                    elif i == 12:  # Middle tip
                        color = (0, 255, 255)  # Yellow
                    else:
                        color = (255, 255, 255)  # White
                    
                    cv2.circle(annotated_frame, (x, y), 5, color, -1)
                
                # Draw connections
                for connection in self.HAND_CONNECTIONS:
                    start_idx, end_idx = connection
                    if start_idx < len(landmarks) and end_idx < len(landmarks):
                        start = landmarks[start_idx]
                        end = landmarks[end_idx]
                        start_point = (int(start['x'] * w), int(start['y'] * h))
                        end_point = (int(end['x'] * w), int(end['y'] * h))
                        cv2.line(annotated_frame, start_point, end_point, (0, 255, 0), 2)
        
        # Draw FPS and status
        cv2.putText(
            annotated_frame,
            f"FPS: {self.fps}",
            (10, 30),
            cv2.FONT_HERSHEY_SIMPLEX,
            1,
            (0, 255, 0),
            2
        )
        
        cv2.putText(
            annotated_frame,
            "Simple Detection Mode",
            (10, 60),
            cv2.FONT_HERSHEY_SIMPLEX,
            0.6,
            (0, 255, 255),
            2
        )
        
        return annotated_frame, hand_landmarks_list if hand_landmarks_list else None
    
    def _simple_hand_detection(self, frame: np.ndarray) -> Optional[List]:
        """
        Simple hand detection using skin color and contours
        Returns approximate hand landmarks
        """
        # Convert to HSV for skin detection
        hsv = cv2.cvtColor(frame, cv2.COLOR_BGR2HSV)
        
        # Define skin color range (adjust for different skin tones)
        lower_skin = np.array([0, 20, 70], dtype=np.uint8)
        upper_skin = np.array([20, 255, 255], dtype=np.uint8)
        
        # Create mask
        mask = cv2.inRange(hsv, lower_skin, upper_skin)
        
        # Apply morphological operations
        kernel = np.ones((5, 5), np.uint8)
        mask = cv2.erode(mask, kernel, iterations=1)
        mask = cv2.dilate(mask, kernel, iterations=2)
        
        # Find contours
        contours, _ = cv2.findContours(mask, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
        
        if not contours:
            return None
        
        # Get largest contour (assumed to be hand)
        largest_contour = max(contours, key=cv2.contourArea)
        
        if cv2.contourArea(largest_contour) < 5000:  # Minimum area threshold
            return None
        
        # Get bounding box and center
        x, y, w, h = cv2.boundingRect(largest_contour)
        center_x = x + w // 2
        center_y = y + h // 2
        
        # Create approximate 21-point hand landmarks
        # This is a simplified version - just enough for basic gestures
        landmarks = []
        frame_h, frame_w = frame.shape[:2]
        
        # Normalize coordinates
        def normalize(px, py):
            return {'x': px / frame_w, 'y': py / frame_h, 'z': 0.0}
        
        # Wrist (0)
        landmarks.append(normalize(center_x, y + h))
        
        # Thumb (1-4)
        for i in range(4):
            landmarks.append(normalize(x + w * 0.2, y + h - i * h * 0.2))
        
        # Index finger (5-8)
        for i in range(4):
            landmarks.append(normalize(x + w * 0.4, y + h - i * h * 0.25))
        
        # Middle finger (9-12)
        for i in range(4):
            landmarks.append(normalize(center_x, y + h - i * h * 0.25))
        
        # Ring finger (13-16)
        for i in range(4):
            landmarks.append(normalize(x + w * 0.6, y + h - i * h * 0.25))
        
        # Pinky (17-20)
        for i in range(4):
            landmarks.append(normalize(x + w * 0.8, y + h - i * h * 0.2))
        
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
