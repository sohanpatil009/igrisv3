// HandFree Mouse Plugin for IGRIS
// Voice-activated hand gesture mouse control

use super::*;

pub fn plugin() -> Plugin {
    Plugin {
        metadata: PluginMetadata {
            name: "handfree_mouse".to_string(),
            version: "1.0.0".to_string(),
            author: "IGRIS".to_string(),
            description: "AI-powered hand gesture mouse control using MediaPipe".to_string(),
            keywords: vec![
                "hand", "mouse", "gesture", "control", "handfree", 
                "touchless", "camera", "ai", "tracking"
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            enabled: true,
        },
        commands: vec![
            // Enable/Start commands
            cmd!(
                "enable hand mouse",
                "Enable hand gesture mouse control",
                &[
                    "enable hand mouse",
                    "start hand mouse",
                    "activate hand mouse",
                    "turn on hand mouse",
                    "enable gesture control",
                    "start gesture control",
                    "enable handfree mouse",
                    "start handfree mouse"
                ],
                ActionType::CustomFunction,
                "handfree_enable"
            ),
            
            // Disable/Stop commands
            cmd!(
                "disable hand mouse",
                "Disable hand gesture mouse control",
                &[
                    "disable hand mouse",
                    "stop hand mouse",
                    "deactivate hand mouse",
                    "turn off hand mouse",
                    "disable gesture control",
                    "stop gesture control",
                    "disable handfree mouse",
                    "stop handfree mouse"
                ],
                ActionType::CustomFunction,
                "handfree_disable"
            ),
            
            // Status command
            cmd!(
                "hand mouse status",
                "Check HandFree Mouse status",
                &[
                    "hand mouse status",
                    "is hand mouse enabled",
                    "check hand mouse",
                    "gesture control status"
                ],
                ActionType::CustomFunction,
                "handfree_status"
            ),
            
            // Calibration command
            cmd!(
                "calibrate hand mouse",
                "Calibrate HandFree Mouse settings",
                &[
                    "calibrate hand mouse",
                    "calibrate gesture control",
                    "adjust hand mouse",
                    "configure hand mouse"
                ],
                ActionType::CustomFunction,
                "handfree_calibrate"
            ),
        ],
    }
}
