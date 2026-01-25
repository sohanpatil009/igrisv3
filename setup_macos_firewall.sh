#!/bin/bash
# macOS Firewall Setup for IGRIS - One-Time Configuration

echo "🔐 IGRIS macOS Firewall Setup"
echo "=============================="
echo ""
echo "This script will configure your firewall to allow IGRIS"
echo "without disabling the entire firewall."
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "❌ Please run with sudo:"
    echo "   sudo ./setup_macos_firewall.sh"
    exit 1
fi

# Get current firewall status
FIREWALL_STATUS=$(/usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate)
echo "Current Firewall Status: $FIREWALL_STATUS"
echo ""

# Find IGRIS binary
IGRIS_PATH=""
if [ -f "./target/debug/igrisv3" ]; then
    IGRIS_PATH="$(pwd)/target/debug/igrisv3"
elif [ -f "./target/release/igrisv3" ]; then
    IGRIS_PATH="$(pwd)/target/release/igrisv3"
else
    echo "⚠️  IGRIS binary not found. Please build first:"
    echo "   cargo build"
    exit 1
fi

echo "Found IGRIS at: $IGRIS_PATH"
echo ""

# Add IGRIS to firewall allow list
echo "Step 1: Adding IGRIS to firewall allow list..."
/usr/libexec/ApplicationFirewall/socketfilterfw --add "$IGRIS_PATH" 2>/dev/null
if [ $? -eq 0 ]; then
    echo "✅ IGRIS added to firewall"
else
    echo "⚠️  Already in firewall or error occurred"
fi

# Unblock the application
echo ""
echo "Step 2: Unblocking IGRIS..."
/usr/libexec/ApplicationFirewall/socketfilterfw --unblock "$IGRIS_PATH" 2>/dev/null
echo "✅ IGRIS unblocked"

# Allow signed apps automatically
echo ""
echo "Step 3: Enabling signed app auto-allow..."
/usr/libexec/ApplicationFirewall/socketfilterfw --setallowsigned on
echo "✅ Signed apps allowed"

# Enable logging (optional, for debugging)
echo ""
echo "Step 4: Enabling firewall logging (optional)..."
read -p "Enable firewall logging for debugging? (y/n): " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    /usr/libexec/ApplicationFirewall/socketfilterfw --setloggingmode on
    echo "✅ Logging enabled"
    echo "   View logs: log show --predicate 'process == \"socketfilterfw\"' --last 5m"
else
    echo "⏭️  Logging skipped"
fi

# Restart firewall to apply changes
echo ""
echo "Step 5: Restarting firewall..."
/usr/libexec/ApplicationFirewall/socketfilterfw --setglobalstate off
sleep 1
/usr/libexec/ApplicationFirewall/socketfilterfw --setglobalstate on
echo "✅ Firewall restarted"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Setup Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "IGRIS is now allowed through the firewall."
echo "You can now run: dx serve"
echo ""
echo "To verify:"
echo "  /usr/libexec/ApplicationFirewall/socketfilterfw --listapps | grep igris"
echo ""
echo "To remove IGRIS from firewall later:"
echo "  sudo /usr/libexec/ApplicationFirewall/socketfilterfw --remove $IGRIS_PATH"
echo ""
