"""
HandFree Mouse - Hand Tracking Engine
Uses MediaPipe for real-time hand landmark detection
"""

import cv2
import mediapipe as mp
import numpy as np
from typing import Optional, Tuple, List
import time


class HandTracker:
    """Real-time hand tracking using MediaPipe"""
    
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
        self.mp_hands = mp.solutions.hands
        self.mp_drawing = mp.solutions.drawing_utils
        self.mp_drawing_styles = mp.solutions.drawing_styles
        
        self.hands = self.mp_hands.Hands(
            static_image_mode=False,
            max_num_hands=max_num_hands,
            min_detection_confidence=min_detection_confidence,
            min_tracking_confidence=min_tracking_confidence
        )
        
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
        # Convert BGR to RGB
        rgb_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
        
        # Process frame
        self.results = self.hands.process(rgb_frame)
        
        # Calculate FPS
        self.frame_count += 1
        current_time = time.time()
        if current_time - self.last_time >= 1.0:
            self.fps = self.frame_count
            self.frame_count = 0
            self.last_time = current_time
        
        # Draw landmarks on frame
        annotated_frame = frame.copy()
        hand_landmarks_list = []
        
        if self.results.multi_hand_landmarks:
            for hand_landmarks in self.results.multi_hand_landmarks:
                # Draw landmarks
                self.mp_drawing.draw_landmarks(
                    annotated_frame,
                    hand_landmarks,
                    self.mp_hands.HAND_CONNECTIONS,
                    self.mp_drawing_styles.get_default_hand_landmarks_style(),
                    self.mp_drawing_styles.get_default_hand_connections_style()
                )
                
                # Extract landmark coordinates
                landmarks = []
                for landmark in hand_landmarks.landmark:
                    landmarks.append({
                        'x': landmark.x,
                        'y': landmark.y,
                        'z': landmark.z
                    })
                hand_landmarks_list.append(landmarks)
        
        # Draw FPS
        cv2.putText(
            annotated_frame,
            f"FPS: {self.fps}",
            (10, 30),
            cv2.FONT_HERSHEY_SIMPLEX,
            1,
            (0, 255, 0),
            2
        )
        
        return annotated_frame, hand_landmarks_list if hand_landmarks_list else None
    
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
