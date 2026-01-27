#!/bin/bash
# Auto-fix Firewall - Turn ON if OFF and configure IGRIS

set -e

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  Firewall Auto-Fix - macOS                                ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "❌ This script needs sudo privileges"
    echo ""
    echo "Run: sudo ./fix_firewall.sh"
    exit 1
fi

echo "Checking firewall status..."
echo ""

# Check current firewall status
FIREWALL_STATUS=$(/usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate)

if echo "$FIREWALL_STATUS" | grep -q "disabled"; then
    echo "❌ Firewall is currently OFF"
    echo ""
    echo "Turning firewall ON..."
    /usr/libexec/ApplicationFirewall/socketfilterfw --setglobalstate on
    echo "✅ Firewall is now ON"
    echo ""
else
    echo "✅ Firewall is already ON"
    echo ""
fi

# Enable stealth mode for extra security
echo "Enabling stealth mode..."
/usr/libexec/ApplicationFirewall/socketfilterfw --setstealthmode on
echo "✅ Stealth mode enabled"
echo ""

# Find IGRIS binary
echo "Looking for IGRIS binary..."
IGRIS_PATHS=(
    "$HOME/ai/igrisv3/target/release/igrisv3"
    "$HOME/ai/igrisv3/target/debug/igrisv3"
    "$(pwd)/target/release/igrisv3"
    "$(pwd)/target/debug/igrisv3"
)

IGRIS_PATH=""
for path in "${IGRIS_PATHS[@]}"; do
    if [ -f "$path" ]; then
        IGRIS_PATH="$path"
        echo "✅ Found IGRIS at: $path"
        break
    fi
done

if [ -z "$IGRIS_PATH" ]; then
    echo "⚠️  IGRIS binary not found"
    echo ""
    echo "Build IGRIS first:"
    echo "  cd ~/ai/igrisv3"
    echo "  cargo build --release"
    echo ""
else
    echo ""
    echo "Configuring IGRIS in firewall..."
    
    # Add IGRIS to firewall
    /usr/libexec/ApplicationFirewall/socketfilterfw --add "$IGRIS_PATH" 2>/dev/null || true
    
    # Allow incoming connections
    /usr/libexec/ApplicationFirewall/socketfilterfw --unblockapp "$IGRIS_PATH"
    
    # Verify
    APP_STATUS=$(/usr/libexec/ApplicationFirewall/socketfilterfw --getappblocked "$IGRIS_PATH")
    
    if echo "$APP_STATUS" | grep -q "permitted\|allowed"; then
        echo "✅ IGRIS configured: Incoming connections ALLOWED"
    else
        echo "⚠️  Could not verify IGRIS configuration"
    fi
    echo ""
fi

# Final verification
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Final Status:"
echo ""

FINAL_STATUS=$(/usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate)
if echo "$FINAL_STATUS" | grep -q "enabled"; then
    echo "✅ Firewall: ON (Protected)"
else
    echo "❌ Firewall: OFF (Check failed)"
fi

STEALTH_STATUS=$(/usr/libexec/ApplicationFirewall/socketfilterfw --getstealthmode)
if echo "$STEALTH_STATUS" | grep -q "enabled"; then
    echo "✅ Stealth Mode: ON"
else
    echo "❌ Stealth Mode: OFF"
fi

if [ -n "$IGRIS_PATH" ]; then
    echo "✅ IGRIS: Configured for File Share"
fi

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║  ✅ Firewall Configuration Complete!                      ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "Your Mac is now protected and IGRIS can accept connections."
echo ""
