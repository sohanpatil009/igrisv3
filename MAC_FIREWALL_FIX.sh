#!/bin/bash
# macOS Firewall Fix for IGRIS Discovery

echo "🔍 Checking macOS Firewall Status..."
echo ""

# Check firewall status
FIREWALL_STATUS=$(sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate)
echo "Firewall Status: $FIREWALL_STATUS"
echo ""

# Check if IGRIS is allowed
echo "📋 Checking if IGRIS is in firewall rules..."
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --listapps | grep -i igris
echo ""

# Option 1: Temporarily disable firewall (NOT RECOMMENDED for production)
echo "Option 1: Temporarily Disable Firewall (Testing Only)"
echo "Command: sudo /usr/libexec/ApplicationFirewall/socketfilterfw --setglobalstate off"
echo ""

# Option 2: Add IGRIS to allowed apps (RECOMMENDED)
echo "Option 2: Add IGRIS to Firewall Allow List (Recommended)"
echo "Command: sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add /path/to/igris"
echo ""

# Option 3: Allow signed apps
echo "Option 3: Allow All Signed Apps"
echo "Command: sudo /usr/libexec/ApplicationFirewall/socketfilterfw --setallowsigned on"
echo ""

# Test multicast
echo "🧪 Testing Multicast Reception..."
echo "Run this in another terminal:"
echo "  echo 'test' | nc -u 239.255.45.67 45678"
echo ""
echo "Then run this to listen:"
echo "  nc -u -l 45678"
echo ""

# Check network interfaces
echo "🌐 Network Interfaces:"
ifconfig | grep -E "^[a-z]|inet " | grep -v "127.0.0.1"
echo ""

echo "✅ Done! Choose an option above to fix firewall."
