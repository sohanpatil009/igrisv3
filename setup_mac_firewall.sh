#!/bin/bash
# Setup macOS Firewall for IGRIS - Run this once with sudo

set -e

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  IGRIS macOS Firewall Setup                               ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "⚠️  This script needs sudo privileges"
    echo "Please run: sudo ./setup_mac_firewall.sh"
    exit 1
fi

# Get the IGRIS binary path
IGRIS_PATH="$HOME/ai/igrisv3/target/release/igrisv3"

# Check if binary exists
if [ ! -f "$IGRIS_PATH" ]; then
    echo "❌ IGRIS binary not found at: $IGRIS_PATH"
    echo ""
    echo "Please build IGRIS first:"
    echo "  cd ~/ai/igrisv3"
    echo "  cargo build --release"
    exit 1
fi

echo "✓ Found IGRIS binary at: $IGRIS_PATH"
echo ""

# Check firewall status
echo "Checking firewall status..."
FIREWALL_STATUS=$(/usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate)

if echo "$FIREWALL_STATUS" | grep -q "disabled"; then
    echo "ℹ️  Firewall is disabled - no configuration needed"
    exit 0
fi

echo "✓ Firewall is enabled"
echo ""

# Add IGRIS to firewall
echo "Adding IGRIS to firewall..."
/usr/libexec/ApplicationFirewall/socketfilterfw --add "$IGRIS_PATH" 2>/dev/null || true

# Unblock IGRIS (allow incoming connections)
echo "Allowing incoming connections..."
/usr/libexec/ApplicationFirewall/socketfilterfw --unblockapp "$IGRIS_PATH"

# Verify
echo ""
echo "Verifying configuration..."
CHECK_RESULT=$(/usr/libexec/ApplicationFirewall/socketfilterfw --getappblocked "$IGRIS_PATH")

if echo "$CHECK_RESULT" | grep -q "permitted\|allowed"; then
    echo ""
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║  ✅ SUCCESS! IGRIS firewall configured                     ║"
    echo "╚════════════════════════════════════════════════════════════╝"
    echo ""
    echo "IGRIS can now accept incoming File Share connections!"
    echo ""
    echo "You can now run IGRIS normally:"
    echo "  cd ~/ai/igrisv3"
    echo "  cargo run --release"
else
    echo ""
    echo "⚠️  Could not verify firewall configuration"
    echo "Please add IGRIS manually:"
    echo "  1. System Settings → Network → Firewall → Options"
    echo "  2. Click + and add: $IGRIS_PATH"
    echo "  3. Set to 'Allow incoming connections'"
fi

echo ""
