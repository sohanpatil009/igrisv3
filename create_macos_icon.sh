#!/bin/bash
# Create macOS .icns icon from SVG

echo "Creating macOS icon (.icns) from SVG..."

# Check if we have a PNG source (easier than SVG)
if [ ! -f "icons/igris_icon.svg" ]; then
    echo "❌ icons/igris_icon.svg not found!"
    exit 1
fi

# Create iconset directory
mkdir -p icons/igris_icon.iconset

# We need to convert SVG to PNG first using a different method
# Try using rsvg-convert if available, otherwise use ImageMagick
if command -v rsvg-convert &> /dev/null; then
    echo "Using rsvg-convert..."
    rsvg-convert -w 1024 -h 1024 icons/igris_icon.svg -o icons/base_icon.png
elif command -v convert &> /dev/null; then
    echo "Using ImageMagick convert..."
    convert -background none -resize 1024x1024 icons/igris_icon.svg icons/base_icon.png
else
    echo "⚠️  Neither rsvg-convert nor ImageMagick found."
    echo "Installing rsvg-convert via Homebrew..."
    brew install librsvg
    rsvg-convert -w 1024 -h 1024 icons/igris_icon.svg -o icons/base_icon.png
fi

# Check if base icon was created
if [ ! -f "icons/base_icon.png" ]; then
    echo "❌ Failed to create base PNG from SVG"
    exit 1
fi

echo "✓ Base PNG created"

# Generate all required icon sizes for macOS
echo "Generating icon sizes..."

sips -z 16 16     icons/base_icon.png --out icons/igris_icon.iconset/icon_16x16.png
sips -z 32 32     icons/base_icon.png --out icons/igris_icon.iconset/icon_16x16@2x.png
sips -z 32 32     icons/base_icon.png --out icons/igris_icon.iconset/icon_32x32.png
sips -z 64 64     icons/base_icon.png --out icons/igris_icon.iconset/icon_32x32@2x.png
sips -z 128 128   icons/base_icon.png --out icons/igris_icon.iconset/icon_128x128.png
sips -z 256 256   icons/base_icon.png --out icons/igris_icon.iconset/icon_128x128@2x.png
sips -z 256 256   icons/base_icon.png --out icons/igris_icon.iconset/icon_256x256.png
sips -z 512 512   icons/base_icon.png --out icons/igris_icon.iconset/icon_256x256@2x.png
sips -z 512 512   icons/base_icon.png --out icons/igris_icon.iconset/icon_512x512.png
sips -z 1024 1024 icons/base_icon.png --out icons/igris_icon.iconset/icon_512x512@2x.png

echo "✓ All icon sizes generated"

# Convert iconset to icns
echo "Creating .icns file..."
iconutil -c icns icons/igris_icon.iconset -o icons/igris_icon.icns

if [ -f "icons/igris_icon.icns" ]; then
    echo "✅ SUCCESS! Created icons/igris_icon.icns"
    ls -lh icons/igris_icon.icns
    
    # Cleanup
    rm -rf icons/igris_icon.iconset
    rm -f icons/base_icon.png
    
    echo ""
    echo "Icon ready for macOS! Update Dioxus.toml to use it."
else
    echo "❌ Failed to create .icns file"
    exit 1
fi
