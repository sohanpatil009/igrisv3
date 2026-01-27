# IGRIS Relay Setup for Windows
# Run this script to setup relay server on Windows

Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  IGRIS Relay Server Setup - Windows                       ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# Check if running as Administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Host "⚠️  This script needs Administrator privileges for firewall setup" -ForegroundColor Yellow
    Write-Host "   Right-click PowerShell and select 'Run as Administrator'" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Press any key to exit..."
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    exit
}

Write-Host "✓ Running as Administrator" -ForegroundColor Green
Write-Host ""

# Step 1: Check if Rust is installed
Write-Host "[1/5] Checking Rust installation..." -ForegroundColor Cyan
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    Write-Host "✓ Rust is installed" -ForegroundColor Green
} else {
    Write-Host "✗ Rust not found. Please install from: https://rustup.rs/" -ForegroundColor Red
    exit
}
Write-Host ""

# Step 2: Pull latest code
Write-Host "[2/5] Pulling latest code..." -ForegroundColor Cyan
git pull
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Code updated" -ForegroundColor Green
} else {
    Write-Host "⚠️  Git pull failed, continuing anyway..." -ForegroundColor Yellow
}
Write-Host ""

# Step 3: Build relay server
Write-Host "[3/5] Building relay server..." -ForegroundColor Cyan
Write-Host "   This may take a few minutes..." -ForegroundColor Gray
cargo build --bin relay_server --release
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Relay server built successfully" -ForegroundColor Green
} else {
    Write-Host "✗ Build failed" -ForegroundColor Red
    exit
}
Write-Host ""

# Step 4: Setup firewall
Write-Host "[4/5] Setting up firewall..." -ForegroundColor Cyan
& .\setup_windows_firewall.ps1
Write-Host ""

# Step 5: Get IP address
Write-Host "[5/5] Getting network information..." -ForegroundColor Cyan
$ip = (Get-NetIPAddress -AddressFamily IPv4 | Where-Object {$_.IPAddress -notlike "127.*" -and $_.IPAddress -notlike "169.254.*"} | Select-Object -First 1).IPAddress
Write-Host "✓ Windows IP Address: $ip" -ForegroundColor Green
Write-Host ""

# Summary
Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Green
Write-Host "║  Setup Complete!                                           ║" -ForegroundColor Green
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Green
Write-Host ""
Write-Host "Next Steps:" -ForegroundColor Yellow
Write-Host ""
Write-Host "Option 1: Use Windows as Relay Server" -ForegroundColor Cyan
Write-Host "  1. Run relay server:" -ForegroundColor White
Write-Host "     .\target\release\relay_server.exe" -ForegroundColor Gray
Write-Host ""
Write-Host "  2. On Mac, update relay address to:" -ForegroundColor White
Write-Host "     $ip`:45680" -ForegroundColor Gray
Write-Host ""
Write-Host "  3. Run main app on both devices" -ForegroundColor White
Write-Host ""
Write-Host "Option 2: Use Mac as Relay Server (Easier)" -ForegroundColor Cyan
Write-Host "  1. On Mac, run:" -ForegroundColor White
Write-Host "     ./target/release/relay_server" -ForegroundColor Gray
Write-Host ""
Write-Host "  2. Update Windows code with Mac's IP" -ForegroundColor White
Write-Host "     Edit: src\file_share\quic_relay.rs line 147" -ForegroundColor Gray
Write-Host ""
Write-Host "  3. Rebuild Windows:" -ForegroundColor White
Write-Host "     cargo build --release" -ForegroundColor Gray
Write-Host ""
Write-Host "Option 3: No Relay - Use Mac Hotspot (Easiest!)" -ForegroundColor Cyan
Write-Host "  1. Mac: Create Personal Hotspot" -ForegroundColor White
Write-Host "  2. Windows: Connect to Mac's hotspot" -ForegroundColor White
Write-Host "  3. Run apps normally - no relay needed!" -ForegroundColor White
Write-Host ""
Write-Host "Press any key to exit..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
