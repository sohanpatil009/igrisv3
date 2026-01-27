# Auto-fix Firewall - Turn ON if OFF and configure IGRIS
# Run as Administrator

# Check if running as Administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Host ""
    Write-Host "================================================================" -ForegroundColor Red
    Write-Host "  ERROR: Administrator privileges required" -ForegroundColor Red
    Write-Host "================================================================" -ForegroundColor Red
    Write-Host ""
    Write-Host "Right-click this file and select 'Run with PowerShell'" -ForegroundColor Yellow
    Write-Host "Or run from PowerShell (Admin):" -ForegroundColor Yellow
    Write-Host "  powershell -ExecutionPolicy Bypass -File fix_firewall.ps1" -ForegroundColor White
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  Firewall Auto-Fix - Windows" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""

# Check and fix firewall status for all profiles
Write-Host "Checking firewall status..." -ForegroundColor Cyan
Write-Host ""

$profiles = Get-NetFirewallProfile
$anyOff = $false

foreach ($profile in $profiles) {
    if (-not $profile.Enabled) {
        Write-Host "[FIX] $($profile.Name) Profile is OFF - Turning ON..." -ForegroundColor Yellow
        Set-NetFirewallProfile -Name $profile.Name -Enabled True
        $anyOff = $true
    } else {
        Write-Host "[OK] $($profile.Name) Profile is already ON" -ForegroundColor Green
    }
}

if ($anyOff) {
    Write-Host ""
    Write-Host "[OK] All firewall profiles are now ON" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "[OK] All firewall profiles were already ON" -ForegroundColor Green
}

Write-Host ""

# Find IGRIS binary
Write-Host "Looking for IGRIS binary..." -ForegroundColor Cyan
Write-Host ""

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
        Write-Host "[OK] Found IGRIS at: $path" -ForegroundColor Green
        break
    }
}

if (-not $igrisPath) {
    Write-Host "[!!] IGRIS binary not found" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Build IGRIS first:" -ForegroundColor Yellow
    Write-Host "  cd F:\igrisv3" -ForegroundColor White
    Write-Host "  cargo build --release" -ForegroundColor White
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 1
}

Write-Host ""

# Check and configure IGRIS firewall rules
Write-Host "Configuring IGRIS firewall rules..." -ForegroundColor Cyan
Write-Host ""

# Remove old rules if they exist
$existingRules = Get-NetFirewallRule -DisplayName "IGRIS File Share" -ErrorAction SilentlyContinue
if ($existingRules) {
    Write-Host "[FIX] Removing old IGRIS rules..." -ForegroundColor Yellow
    Remove-NetFirewallRule -DisplayName "IGRIS File Share" -ErrorAction SilentlyContinue
}

# Create inbound rule
Write-Host "[FIX] Creating inbound rule..." -ForegroundColor Yellow
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
    Write-Host "[OK] Inbound rule created" -ForegroundColor Green
} catch {
    Write-Host "[ERROR] Failed to create inbound rule: $_" -ForegroundColor Red
}

# Create outbound rule
Write-Host "[FIX] Creating outbound rule..." -ForegroundColor Yellow
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
    Write-Host "[OK] Outbound rule created" -ForegroundColor Green
} catch {
    Write-Host "[ERROR] Failed to create outbound rule: $_" -ForegroundColor Red
}

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""

# Final verification
Write-Host "Final Status:" -ForegroundColor Cyan
Write-Host ""

$finalProfiles = Get-NetFirewallProfile
$allOn = $true
foreach ($profile in $finalProfiles) {
    $status = if ($profile.Enabled) { "ON" } else { "OFF" }
    $color = if ($profile.Enabled) { "Green" } else { "Red" }
    Write-Host "  $($profile.Name) Profile: $status" -ForegroundColor $color
    if (-not $profile.Enabled) {
        $allOn = $false
    }
}

Write-Host ""

$finalRules = Get-NetFirewallRule -DisplayName "IGRIS File Share" -ErrorAction SilentlyContinue
if ($finalRules) {
    Write-Host "  IGRIS Rules: Configured" -ForegroundColor Green
    foreach ($rule in $finalRules) {
        Write-Host "    - $($rule.Direction): Enabled" -ForegroundColor Green
    }
} else {
    Write-Host "  IGRIS Rules: Not Found" -ForegroundColor Red
}

Write-Host ""

if ($allOn -and $finalRules) {
    Write-Host "================================================================" -ForegroundColor Green
    Write-Host "  SUCCESS! Firewall Configuration Complete" -ForegroundColor Green
    Write-Host "================================================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Your Windows is now protected and IGRIS can accept connections." -ForegroundColor White
} else {
    Write-Host "================================================================" -ForegroundColor Yellow
    Write-Host "  WARNING: Some issues remain" -ForegroundColor Yellow
    Write-Host "================================================================" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Please check the status above and fix manually if needed." -ForegroundColor Yellow
}

Write-Host ""
Read-Host "Press Enter to exit"
