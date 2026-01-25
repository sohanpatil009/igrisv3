#!/bin/bash
# Quick Fix for Mac Discovery & Connection Issues

echo "🔧 IGRIS Mac Quick Fix"
echo "====================="
echo ""

# Step 1: Check and disable firewall temporarily
echo "Step 1: Checking macOS Firewall..."
FIREWALL_STATUS=$(sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate 2>/dev/null)
echo "Current status: $FIREWALL_STATUS"

if [[ "$FIREWALL_STATUS" == *"on"* ]]; then
    echo ""
    read -p "Firewall is ON. Disable temporarily for testing? (y/n): " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        sudo /usr/libexec/ApplicationFirewall/socketfilterfw --setglobalstate off
        echo "✅ Firewall disabled"
    fi
fi

echo ""
echo "Step 2: Checking Network Interfaces..."
echo "Your IP addresses:"
ifconfig | grep -E "^[a-z]|inet " | grep -v "127.0.0.1" | grep -A 1 "status: active"

echo ""
echo "Step 3: Testing Multicast Reception..."
echo "Run this command in another terminal to test:"
echo "  nc -u -l 45678"
echo ""
echo "Then from Windows, send test packet:"
echo "  echo 'test' | nc -u 239.255.45.67 45678"
echo ""

echo "Step 4: Check if ports are open..."
echo "Discovery port (45678):"
lsof -i :45678 2>/dev/null || echo "  Not in use (OK)"
echo "Bridge port (45679):"
lsof -i :45679 2>/dev/null || echo "  Not in use (OK)"

echo ""
echo "✅ Quick fix complete!"
echo ""
echo "Now run: dx serve"
echo ""
echo "To re-enable firewall later:"
echo "  sudo /usr/libexec/ApplicationFirewall/socketfilterfw --setglobalstate on"
