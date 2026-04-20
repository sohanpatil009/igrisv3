use dioxus::prelude::*;
use rfd::AsyncFileDialog;

#[component]
pub fn FastSwapPanel() -> Element {
    let status_message = use_signal(|| String::from("FastSwap Ready"));

    // Note: FastSwap server is started in main.rs
    // This panel connects to the running FastSwap manager

    rsx! {
        div {
            class: "file-share-panel",
            style: "padding: 20px; background: #1a1a2e; border-radius: 12px; color: white;",

            // Header
            div {
                style: "margin-bottom: 20px;",
                h2 {
                    style: "margin: 0 0 10px 0; color: #a855f7;",
                    "⚡ FastSwap File Sharing"
                }
                p {
                    style: "margin: 0; color: #888; font-size: 14px;",
                    "{status_message}"
                }
            }

            // Instructions
            div {
                style: "margin-top: 30px; padding: 15px; background: #0a0a0a; border-radius: 8px; border-left: 4px solid #a855f7;",
                h4 {
                    style: "margin: 0 0 10px 0; color: #a855f7;",
                    "FastSwap Integration"
                }
                div {
                    style: "font-size: 14px; color: #888; line-height: 1.6;",
                    "FastSwap server is running on port 53317"
                    br {}
                    "Use LocalSend apps on other devices to connect and share files."
                    br {}
                    "Compatible with LocalSend v2.0 protocol for cross-platform sharing."
                }
            }
        }
    }
}
