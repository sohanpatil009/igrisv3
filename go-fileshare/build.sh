#!/bin/bash

# Build script for IGRIS File Share Go Backend

set -e

echo "🔨 Building IGRIS File Share Backend..."

# Get dependencies
echo "📦 Downloading dependencies..."
go mod download

# Build for current platform
echo "🏗️  Building for current platform..."
go build -o fileshare -ldflags="-s -w" .

echo "✅ Build complete: ./fileshare"
echo ""
echo "Run with: ./fileshare"
echo "Or with custom settings: ./fileshare -name \"My Desktop\" -port 53317"
