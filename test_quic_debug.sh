#!/bin/bash
# QUIC Connection Debug Test Script

echo "==================================="
echo "QUIC Connection Debug Test"
echo "==================================="
echo ""

# Check if running on Mac or Windows
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "Platform: macOS"
    DEVICE="Mac"
else
    echo "Platform: Windows/Linux"
    DEVICE="Windows"
fi

echo ""
echo "Step 1: Building release version..."
cargo build --release

echo ""
echo "Step 2: Starting IGRIS with detailed logging..."
echo "Watch for these key log messages:"
echo "  - [FileShare] on_connect() called"
echo "  - [FileShare] connect_direct_async() called"
echo "  - [ConnectionCoordinator] establish_quic_connection_with_handshake()"
echo "  - [QuicBridge] connect() called"
echo "  - [QuicBridge] Initiating QUIC connection..."
echo "  - [QuicBridge] QUIC connection established"
echo ""
echo "Press Ctrl+C to stop"
echo ""

cargo run --release 2>&1 | grep -E "\[FileShare\]|\[QuicBridge\]|\[ConnectionCoordinator\]|\[Discovery\]"
