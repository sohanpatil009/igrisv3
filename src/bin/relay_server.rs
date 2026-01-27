// src/bin/relay_server.rs - Standalone QUIC Relay Server

use igrisv3::file_share::relay_server::run_relay_server;

#[tokio::main]
async fn main() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  IGRIS QUIC Relay Server                                  ║");
    println!("║  Enables P2P file sharing through AP isolation            ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    
    // Default port for relay server
    let port = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(45680);
    
    match run_relay_server(port).await {
        Ok(_) => println!("Relay server stopped"),
        Err(e) => eprintln!("Relay server error: {}", e),
    }
}
