#!/bin/bash
# Check IGRIS file share config location

echo "Checking IGRIS config locations..."
echo ""

# macOS
if [ -f "$HOME/Library/Application Support/IGRIS/file_share.json" ]; then
    echo "✓ Found config at: $HOME/Library/Application Support/IGRIS/file_share.json"
    echo "Device ID:"
    cat "$HOME/Library/Application Support/IGRIS/file_share.json" | grep -o '"id":"[^"]*"' | head -1
    echo ""
fi

# Current directory
if [ -f "./IGRIS/file_share.json" ]; then
    echo "✓ Found config at: ./IGRIS/file_share.json"
    echo "Device ID:"
    cat "./IGRIS/file_share.json" | grep -o '"id":"[^"]*"' | head -1
    echo ""
fi

echo "To delete config and regenerate unique ID:"
echo "rm ~/Library/Application\ Support/IGRIS/file_share.json"
echo ""
echo "Then restart IGRIS"
