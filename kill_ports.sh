#!/bin/bash
# Script to kill any processes using IGRIS file sharing ports

echo "🔍 Checking for processes on ports 45678, 45679, 45680..."

for port in 45678 45679 45680; do
    pid=$(lsof -ti :$port 2>/dev/null)
    if [ ! -z "$pid" ]; then
        echo "⚠️  Found process $pid on port $port - killing it..."
        kill -9 $pid 2>/dev/null
        echo "✅ Killed process on port $port"
    else
        echo "✓ Port $port is free"
    fi
done

echo ""
echo "✅ All ports cleared! You can now run IGRIS."
echo ""
echo "Run: cargo run --release"
