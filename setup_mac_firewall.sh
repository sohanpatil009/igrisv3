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

# Try multiple possible paths
POSSIBLE_PATHS=(
    "$HOME/ai/igrisv3/target/release/igrisv3"
    "$HOME/ai/igrisv3/target/debug/igrisv3"
    "$(pwd)/target/release/igrisv3"
    "$(pwd)/target/debug/igrisv3"
)

IGRIS_PATH=""
for path in "${POSSIBLE_PATHS[@]}"; do
    if [ -f "$path" ]; then
        IGRIS_PATH="$path"
        break
    fi
done

# Check if binary exists
if [ -z "$IGRIS_PATH" ]; then
    echo "❌ IGRIS binary not found"
    echo ""
    echo "Tried these locations:"
    for path in "${POSSIBLE_PATHS[@]}"; do
        echo "  - $path"
    done
    echo ""
    echo "Please build IGRIS first:"
    echo "  cd ~/ai/igrisv3"
    echo "  cargo build --release"
    echo ""
    echo "Or if running with 'cargo run', build debug version:"
    echo "  cargo build"
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
    echo "You can now run IGRIS:"
    echo "  cd ~/ai/igrisv3"
    echo "  cargo run --release"
    echo ""
    echo "Or for development:"
    echo "  cargo run"
else
    echo ""
    echo "⚠️  Could not verify firewall configuration"
    echo "Please add IGRIS manually:"
    echo "  1. System Settings → Network → Firewall → Options"
    echo "  2. Click + and add: $IGRIS_PATH"
    echo "  3. Set to 'Allow incoming connections'"
fi

echo ""
