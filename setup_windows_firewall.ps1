# Setup Windows Firewall for IGRIS - Run this once as Administrator
# Right-click → "Run with PowerShell" (as Admin)

# Check if running as Administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Host ""
    Write-Host "================================================================" -ForegroundColor Red
    Write-Host "  ERROR: This script requires Administrator privileges" -ForegroundColor Red
    Write-Host "================================================================" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please run as Administrator:" -ForegroundColor Yellow
    Write-Host "  1. Right-click this file" -ForegroundColor Yellow
    Write-Host "  2. Select 'Run with PowerShell'" -ForegroundColor Yellow
    Write-Host "  3. Click 'Yes' on UAC prompt" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Or run from PowerShell (Admin):" -ForegroundColor Yellow
    Write-Host "  powershell -ExecutionPolicy Bypass -File setup_windows_firewall.ps1" -ForegroundColor Yellow
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  IGRIS Windows Firewall Setup" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""

# Try to find IGRIS binary
$possiblePaths = @(
    "F:\igrisv3\target\release\igrisv3.exe",
    "F:\igrisv3\target\debug\igrisv3.exe",
    "$PSScriptRoot\target\release\igrisv3.exe",
    "$PSScriptRoot\target\debug\igrisv3.exe"
)

$igrisPath = $null
foreach ($path in $possiblePaths) {
    if (Test-Path $path) {
        $igrisPath = $path
        break
    }
}

if (-not $igrisPath) {
    Write-Host "ERROR: IGRIS binary not found" -ForegroundColor Red
    Write-Host ""
    Write-Host "Tried these locations:" -ForegroundColor Yellow
    foreach ($path in $possiblePaths) {
        Write-Host "  - $path" -ForegroundColor Gray
    }
    Write-Host ""
    Write-Host "Please build IGRIS first:" -ForegroundColor Yellow
    Write-Host "  cd F:\igrisv3" -ForegroundColor White
    Write-Host "  cargo build --release" -ForegroundColor White
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host "[OK] Found IGRIS binary at: $igrisPath" -ForegroundColor Green
Write-Host ""

# Check if firewall rules already exist
Write-Host "Checking existing firewall rules..." -ForegroundColor Cyan
$existingInbound = Get-NetFirewallRule -DisplayName "IGRIS File Share" -Direction Inbound -ErrorAction SilentlyContinue
$existingOutbound = Get-NetFirewallRule -DisplayName "IGRIS File Share" -Direction Outbound -ErrorAction SilentlyContinue

if ($existingInbound -and $existingOutbound) {
    Write-Host "[OK] Firewall rules already exist" -ForegroundColor Green
    Write-Host ""
    Write-Host "Inbound rule:  Enabled" -ForegroundColor Green
    Write-Host "Outbound rule: Enabled" -ForegroundColor Green
    Write-Host ""
    Write-Host "================================================================" -ForegroundColor Cyan
    Write-Host "  SUCCESS! IGRIS firewall already configured" -ForegroundColor Green
    Write-Host "================================================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "You can now run IGRIS:" -ForegroundColor White
    Write-Host "  cd F:\igrisv3" -ForegroundColor White
    Write-Host "  cargo run --release" -ForegroundColor White
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 0
}

# Remove old rules if they exist
if ($existingInbound) {
    Write-Host "Removing old inbound rule..." -ForegroundColor Yellow
    Remove-NetFirewallRule -DisplayName "IGRIS File Share" -Direction Inbound -ErrorAction SilentlyContinue
}
if ($existingOutbound) {
    Write-Host "Removing old outbound rule..." -ForegroundColor Yellow
    Remove-NetFirewallRule -DisplayName "IGRIS File Share" -Direction Outbound -ErrorAction SilentlyContinue
}

# Create inbound rule
Write-Host "Creating inbound firewall rule..." -ForegroundColor Cyan
try {
    New-NetFirewallRule `
        -DisplayName "IGRIS File Share" `
        -Direction Inbound `
        -Protocol UDP `
        -LocalPort 45679 `
        -Action Allow `
        -Program $igrisPath `
        -Profile Any `
        -ErrorAction Stop | Out-Null
    Write-Host "[OK] Inbound rule created successfully" -ForegroundColor Green
} catch {
    Write-Host "[ERROR] Failed to create inbound rule: $_" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

# Create outbound rule
Write-Host "Creating outbound firewall rule..." -ForegroundColor Cyan
try {
    New-NetFirewallRule `
        -DisplayName "IGRIS File Share" `
        -Direction Outbound `
        -Protocol UDP `
        -LocalPort 45679 `
        -Action Allow `
        -Program $igrisPath `
        -Profile Any `
        -ErrorAction Stop | Out-Null
    Write-Host "[OK] Outbound rule created successfully" -ForegroundColor Green
} catch {
    Write-Host "[ERROR] Failed to create outbound rule: $_" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

# Verify rules
Write-Host ""
Write-Host "Verifying firewall rules..." -ForegroundColor Cyan
$verifyInbound = Get-NetFirewallRule -DisplayName "IGRIS File Share" -Direction Inbound -ErrorAction SilentlyContinue
$verifyOutbound = Get-NetFirewallRule -DisplayName "IGRIS File Share" -Direction Outbound -ErrorAction SilentlyContinue

if ($verifyInbound -and $verifyOutbound) {
    Write-Host ""
    Write-Host "================================================================" -ForegroundColor Green
    Write-Host "  SUCCESS! IGRIS firewall configured" -ForegroundColor Green
    Write-Host "================================================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Firewall rules created:" -ForegroundColor White
    Write-Host "  - Inbound:  UDP port 45679 (Allow)" -ForegroundColor Green
    Write-Host "  - Outbound: UDP port 45679 (Allow)" -ForegroundColor Green
    Write-Host "  - Program:  $igrisPath" -ForegroundColor Gray
    Write-Host ""
    Write-Host "IGRIS can now accept incoming File Share connections!" -ForegroundColor Green
    Write-Host ""
    Write-Host "You can now run IGRIS:" -ForegroundColor White
    Write-Host "  cd F:\igrisv3" -ForegroundColor White
    Write-Host "  cargo run --release" -ForegroundColor White
    Write-Host ""
} else {
    Write-Host ""
    Write-Host "[WARNING] Could not verify firewall rules" -ForegroundColor Yellow
    Write-Host "Please check Windows Defender Firewall manually" -ForegroundColor Yellow
    Write-Host ""
}

Read-Host "Press Enter to exit"
