#!/bin/bash

# Quick start script for IGRIS File Share Backend

set -e

echo "🚀 Starting IGRIS File Share Backend..."
echo ""

# Check if binary exists
if [ ! -f "./fileshare" ]; then
    echo "❌ Binary not found. Building..."
    ./build.sh
    echo ""
fi

# Create config if not exists
if [ ! -f "config.json" ]; then
    echo "📝 Creating default config..."
    cp config.example.json config.json
    echo "✅ Config created: config.json"
    echo ""
fi

# Create downloads directory
mkdir -p downloads
echo "📁 Downloads directory: $(pwd)/downloads"
echo ""

# Get local IP
if command -v ip &> /dev/null; then
    LOCAL_IP=$(ip -4 addr show | grep -oP '(?<=inet\s)\d+(\.\d+){3}' | grep -v '127.0.0.1' | head -n1)
elif command -v ifconfig &> /dev/null; then
    LOCAL_IP=$(ifconfig | grep -Eo 'inet (addr:)?([0-9]*\.){3}[0-9]*' | grep -Eo '([0-9]*\.){3}[0-9]*' | grep -v '127.0.0.1' | head -n1)
else
    LOCAL_IP="unknown"
fi

echo "🌐 Local IP: $LOCAL_IP"
echo "🔌 Port: 53317"
echo ""
echo "📡 Make sure both devices are on the same mobile hotspot!"
echo ""
echo "Press Ctrl+C to stop"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Run the backend
./fileshare
