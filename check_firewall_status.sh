#!/bin/bash
# Check Firewall Status on Mac

echo "╔════════════════════════════════════════════════════════════╗"
echo "║  Firewall Status Check - macOS                            ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Check if running with sudo
if [ "$EUID" -ne 0 ]; then 
    echo "⚠️  Need sudo to check firewall status"
    echo "Run: sudo ./check_firewall_status.sh"
    echo ""
fi

# Check firewall status
echo "1. Checking macOS Firewall Status..."
FIREWALL_STATUS=$(/usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate 2>/dev/null)

if echo "$FIREWALL_STATUS" | grep -q "enabled"; then
    echo "   ✅ Firewall: ENABLED (ON)"
    FIREWALL_ON=true
elif echo "$FIREWALL_STATUS" | grep -q "disabled"; then
    echo "   ❌ Firewall: DISABLED (OFF)"
    FIREWALL_ON=false
else
    echo "   ⚠️  Could not determine firewall status"
    FIREWALL_ON=false
fi

echo ""

# Check stealth mode
echo "2. Checking Stealth Mode..."
STEALTH_STATUS=$(/usr/libexec/ApplicationFirewall/socketfilterfw --getstealthmode 2>/dev/null)
if echo "$STEALTH_STATUS" | grep -q "enabled"; then
    echo "   ✅ Stealth Mode: ENABLED"
else
    echo "   ❌ Stealth Mode: DISABLED"
fi

echo ""

# Check IGRIS app status
echo "3. Checking IGRIS Permissions..."
IGRIS_PATHS=(
    "$HOME/ai/igrisv3/target/release/igrisv3"
    "$HOME/ai/igrisv3/target/debug/igrisv3"
)

IGRIS_FOUND=false
for path in "${IGRIS_PATHS[@]}"; do
    if [ -f "$path" ]; then
        echo "   Found IGRIS at: $path"
        IGRIS_FOUND=true
        
        if [ "$EUID" -eq 0 ]; then
            APP_STATUS=$(/usr/libexec/ApplicationFirewall/socketfilterfw --getappblocked "$path" 2>/dev/null)
            if echo "$APP_STATUS" | grep -q "permitted\|allowed"; then
                echo "   ✅ IGRIS: ALLOWED (incoming connections permitted)"
            elif echo "$APP_STATUS" | grep -q "blocked"; then
                echo "   ❌ IGRIS: BLOCKED (incoming connections denied)"
            else
                echo "   ⚠️  IGRIS: NOT IN FIREWALL LIST"
            fi
        fi
    fi
done

if [ "$IGRIS_FOUND" = false ]; then
    echo "   ⚠️  IGRIS binary not found"
fi

echo ""
echo "════════════════════════════════════════════════════════════"
echo ""

# Summary
if [ "$FIREWALL_ON" = true ]; then
    echo "📊 SUMMARY: Firewall is ON (Protected)"
    echo ""
    echo "Your Mac is protected by the firewall."
    if [ "$IGRIS_FOUND" = true ]; then
        echo "Run with sudo to check IGRIS permissions."
    fi
else
    echo "⚠️  SUMMARY: Firewall is OFF (Unprotected!)"
    echo ""
    echo "Your Mac is NOT protected by the firewall!"
    echo ""
    echo "To turn it ON:"
    echo "  sudo /usr/libexec/ApplicationFirewall/socketfilterfw --setglobalstate on"
    echo ""
    echo "Or use System Settings:"
    echo "  System Settings → Network → Firewall → Turn On"
fi

echo ""
