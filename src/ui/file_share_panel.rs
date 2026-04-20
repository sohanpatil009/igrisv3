use dioxus::prelude::*;
use crate::localshare::{Device, Transfer};
use rfd::AsyncFileDialog;

#[component]
pub fn FileSharePanel() -> Element {
    let mut devices = use_signal(|| Vec::<Device>::new());
    let mut transfers = use_signal(|| Vec::<Transfer>::new());
    let mut selected_device = use_signal(|| None::<String>);
    let mut status_message = use_signal(|| String::from("Initializing LocalShare..."));
    let mut sending = use_signal(|| false);

    // Note: LocalShare server is started in main.rs
    // This panel connects to the running LocalShare manager

    rsx! {
        div {
            class: "file-share-panel",
            style: "padding: 20px; background: #1a1a2e; border-radius: 12px; color: white;",

            // Header
            div {
                style: "margin-bottom: 20px;",
                h2 {
                    style: "margin: 0 0 10px 0; color: #a855f7;",
                    "📡 LocalShare File Sharing"
                }
                p {
                    style: "margin: 0; color: #888; font-size: 14px;",
                    "LocalShare server is running on port 53317"
                }
            }

            // Instructions
            div {
                style: "margin-top: 30px; padding: 15px; background: #0a0a0a; border-radius: 8px; border-left: 4px solid #a855f7;",
                h4 {
                    style: "margin: 0 0 10px 0; color: #a855f7;",
                    "LocalShare Integration"
                }
                div {
                    style: "font-size: 14px; color: #888; line-height: 1.6;",
                    "LocalShare server is now integrated directly into IGRIS."
                    br {}
                    "Use LocalSend apps on other devices to connect and share files."
                    br {}
                    "Server is running on port 53317 with device discovery enabled."
                }
            }
        }
    }
}
