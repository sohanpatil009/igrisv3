# Test IGRIS Relay on Windows
# Quick test script to verify relay setup

Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  IGRIS Relay Test - Windows                               ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

$allGood = $true

# Check relay server binary
Write-Host "[1/4] Checking relay server binary..." -ForegroundColor Cyan
if (Test-Path ".\target\release\relay_server.exe") {
    Write-Host "✓ Relay server found" -ForegroundColor Green
} else {
    Write-Host "✗ Relay server not found" -ForegroundColor Red
    Write-Host "   Run: cargo build --bin relay_server --release" -ForegroundColor Yellow
    $allGood = $false
}
Write-Host ""

# Check main app binary
Write-Host "[2/4] Checking main app binary..." -ForegroundColor Cyan
if (Test-Path ".\target\release\igrisv3.exe") {
    Write-Host "✓ Main app found" -ForegroundColor Green
} else {
    Write-Host "✗ Main app not found" -ForegroundColor Red
    Write-Host "   Run: cargo build --release" -ForegroundColor Yellow
    $allGood = $false
}
Write-Host ""

# Check firewall
Write-Host "[3/4] Checking firewall rules..." -ForegroundColor Cyan
$rules = Get-NetFirewallRule | Where-Object {$_.DisplayName -like "*IGRIS*"}
if ($rules) {
    Write-Host "✓ Firewall rules configured" -ForegroundColor Green
    foreach ($rule in $rules) {
        Write-Host "   - $($rule.DisplayName)" -ForegroundColor Gray
    }
} else {
    Write-Host "⚠️  No IGRIS firewall rules found" -ForegroundColor Yellow
    Write-Host "   Run: .\setup_windows_firewall.ps1 (as Admin)" -ForegroundColor Yellow
}
Write-Host ""

# Get network info
Write-Host "[4/4] Network information..." -ForegroundColor Cyan
$ip = (Get-NetIPAddress -AddressFamily IPv4 | Where-Object {$_.IPAddress -notlike "127.*" -and $_.IPAddress -notlike "169.254.*"} | Select-Object -First 1).IPAddress
if ($ip) {
    Write-Host "✓ Windows IP: $ip" -ForegroundColor Green
} else {
    Write-Host "⚠️  Could not determine IP address" -ForegroundColor Yellow
}
Write-Host ""

# Summary
if ($allGood) {
    Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Green
    Write-Host "║  All checks passed! Ready to test.                        ║" -ForegroundColor Green
    Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Green
    Write-Host ""
    Write-Host "To start relay server:" -ForegroundColor Yellow
    Write-Host "  .\target\release\relay_server.exe" -ForegroundColor White
    Write-Host ""
    Write-Host "To start main app:" -ForegroundColor Yellow
    Write-Host "  .\target\release\igrisv3.exe" -ForegroundColor White
    Write-Host ""
    Write-Host "Share this IP with Mac for relay connection:" -ForegroundColor Yellow
    Write-Host "  $ip`:45680" -ForegroundColor White
} else {
    Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Red
    Write-Host "║  Some checks failed. Please fix issues above.             ║" -ForegroundColor Red
    Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Red
}

Write-Host ""
Write-Host "Press any key to exit..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
