// src/plugins/builtin/file_share.rs
// File sharing plugin

use super::*;

pub fn plugin() -> Plugin {
    Plugin {
        metadata: PluginMetadata {
            name: "file_share".to_string(),
            version: "1.0.0".to_string(),
            author: "IGRIS".to_string(),
            description: "Device discovery and file sharing".to_string(),
            keywords: vec!["file", "share", "transfer", "device", "discover", "send", "receive"]
                .into_iter().map(String::from).collect(),
            enabled: true,
        },
        commands: vec![
            cmd!("discover devices", "Scans for nearby devices", &["discover devices", "scan devices", "find devices", "search devices"], ActionType::FileShare, "discover"),
            cmd!("show devices", "Shows discovered devices", &["show devices", "list devices", "nearby devices"], ActionType::FileShare, "list"),
            cmd!("my devices", "Shows trusted devices", &["my devices", "trusted devices", "saved devices"], ActionType::FileShare, "trusted"),
            cmd!("connect device", "Connects to a device", &["connect device", "connect to", "pair device"], ActionType::FileShare, "connect"),
            cmd!("send file", "Sends a file to device", &["send file", "share file", "transfer file"], ActionType::FileShare, "send"),
            cmd!("stop sharing", "Stops file sharing", &["stop sharing", "stop discovery", "disconnect"], ActionType::FileShare, "stop"),
        ],
    }
}
