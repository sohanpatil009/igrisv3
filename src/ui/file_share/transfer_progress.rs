// src/ui/file_share/transfer_progress.rs - Transfer Progress UI

use dioxus::prelude::*;

/// Transfer display info (self-contained)
#[derive(Clone, PartialEq, Debug)]
pub struct TransferDisplay {
    pub id: String,
    pub filename: String,
    pub size: String,
    pub transferred: String,
    pub progress: f32,
    pub speed: String,
    pub eta: String,
    pub direction: String,
    pub direction_icon: String,
    pub status: String,
    pub status_color: String,
    pub is_pending: bool,
    pub is_active: bool,
    pub is_incoming: bool,
    pub can_cancel: bool,
}

/// Transfer Progress - Shows active and recent transfers
#[component]
pub fn TransferProgress(
    transfers: Vec<TransferDisplay>,
    on_accept: EventHandler<String>,
    on_reject: EventHandler<String>,
    on_cancel: EventHandler<String>,
    on_close: EventHandler<()>,
) -> Element {
    let handle_accept = move |transfer_id: String| {
        on_accept.call(transfer_id);
    };
    
    let handle_reject = move |transfer_id: String| {
        on_reject.call(transfer_id);
    };
    
    let handle_cancel = move |transfer_id: String| {
        on_cancel.call(transfer_id);
    };
    
    let handle_close = move |_| {
        println!("[FileShare UI] TransferProgress close button clicked");
        on_close.call(());
    };
    
    let active_count = transfers.iter().filter(|t| t.is_active).count();
    let pending_count = transfers.iter().filter(|t| t.is_pending).count();
    
    rsx! {
        div {
            style: "background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%); border-radius: 16px; padding: 24px; min-width: 400px;",
            
            // Header
            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;",
                
                div {
                    h2 { style: "color: #fff; margin: 0; font-size: 20px;", "📤 Transfers" }
                    if active_count > 0 || pending_count > 0 {
                        div { style: "color: #64748b; font-size: 13px; margin-top: 4px;", "{active_count} active, {pending_count} pending" }
                    }
                }
                
                button {
                    style: "background: transparent; border: none; color: #888; cursor: pointer; font-size: 20px;",
                    onclick: handle_close,
                    "✕"
                }
            }
            
            // Transfer list
            div {
                style: "max-height: 400px; overflow-y: auto;",
                
                if transfers.is_empty() {
                    div {
                        style: "text-align: center; color: #64748b; padding: 40px;",
                        div { style: "font-size: 48px; margin-bottom: 16px;", "📁" }
                        div { "No transfers" }
                    }
                }
                
                for transfer in transfers.iter() {
                    {
                        let transfer_id = transfer.id.clone();
                        let transfer_id2 = transfer.id.clone();
                        let transfer_id3 = transfer.id.clone();
                        
                        rsx! {
                            div {
                                key: "{transfer.id}",
                                style: "background: #1e293b; border-radius: 12px; padding: 16px; margin-bottom: 12px;",
                                
                                // Header
                                div {
                                    style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px;",
                                    
                                    div {
                                        style: "display: flex; align-items: center; gap: 10px;",
                                        span { style: "font-size: 20px;", "{transfer.direction_icon}" }
                                        div {
                                            div { style: "color: #fff; font-weight: 500; font-size: 14px; max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;", "{transfer.filename}" }
                                            div { style: "color: #64748b; font-size: 12px;", "{transfer.direction} • {transfer.size}" }
                                        }
                                    }
                                    
                                    span { style: "color: {transfer.status_color}; font-size: 12px; font-weight: 500;", "{transfer.status}" }
                                }
                                
                                // Progress bar
                                if transfer.is_active {
                                    div {
                                        style: "margin-bottom: 12px;",
                                        
                                        div {
                                            style: "background: #0f172a; border-radius: 4px; height: 8px; overflow: hidden;",
                                            div { style: "background: linear-gradient(90deg, #3b82f6, #8b5cf6); height: 100%; width: {transfer.progress}%; transition: width 0.3s ease;" }
                                        }
                                        
                                        div {
                                            style: "display: flex; justify-content: space-between; margin-top: 8px; color: #64748b; font-size: 12px;",
                                            span { "{transfer.transferred} / {transfer.size}" }
                                            span { "{transfer.progress:.1}%" }
                                        }
                                        
                                        div {
                                            style: "display: flex; justify-content: space-between; margin-top: 4px; color: #94a3b8; font-size: 12px;",
                                            span { "⚡ {transfer.speed}" }
                                            span { "⏱️ {transfer.eta}" }
                                        }
                                    }
                                }
                                
                                // Actions
                                if transfer.is_pending && transfer.is_incoming {
                                    div {
                                        style: "display: flex; gap: 8px;",
                                        button {
                                            style: "background: #22c55e; color: white; border: none; padding: 10px 20px; border-radius: 8px; cursor: pointer; flex: 1; font-weight: 500;",
                                            onclick: move |_| handle_accept(transfer_id.clone()),
                                            "✓ Accept"
                                        }
                                        button {
                                            style: "background: #ef4444; color: white; border: none; padding: 10px 20px; border-radius: 8px; cursor: pointer; flex: 1; font-weight: 500;",
                                            onclick: move |_| handle_reject(transfer_id2.clone()),
                                            "✕ Reject"
                                        }
                                    }
                                } else if transfer.can_cancel {
                                    button {
                                        style: "background: #334155; color: #94a3b8; border: none; padding: 8px 16px; border-radius: 6px; cursor: pointer; width: 100%; font-size: 13px;",
                                        onclick: move |_| handle_cancel(transfer_id3.clone()),
                                        "Cancel Transfer"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
