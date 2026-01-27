# Check Firewall Status on Windows

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "  Firewall Status Check - Windows" -ForegroundColor Cyan
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""

# Check if running as Administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Host "WARNING: Not running as Administrator" -ForegroundColor Yellow
    Write-Host "Some checks may be limited" -ForegroundColor Yellow
    Write-Host ""
}

# Check firewall status for all profiles
Write-Host "1. Checking Windows Firewall Status..." -ForegroundColor Cyan
Write-Host ""

$profiles = Get-NetFirewallProfile
$allOff = $true
$allOn = $true

foreach ($profile in $profiles) {
    $status = if ($profile.Enabled) { "ON" } else { "OFF" }
    $color = if ($profile.Enabled) { "Green" } else { "Red" }
    $icon = if ($profile.Enabled) { "[OK]" } else { "[!!]" }
    
    Write-Host "   $icon $($profile.Name) Profile: $status" -ForegroundColor $color
    
    if ($profile.Enabled) {
        $allOff = $false
    } else {
        $allOn = $false
    }
}

Write-Host ""

# Check IGRIS firewall rules
Write-Host "2. Checking IGRIS Firewall Rules..." -ForegroundColor Cyan
Write-Host ""

$igrisRules = Get-NetFirewallRule -DisplayName "*IGRIS*" -ErrorAction SilentlyContinue

if ($igrisRules) {
    foreach ($rule in $igrisRules) {
        $status = if ($rule.Enabled) { "ENABLED" } else { "DISABLED" }
        $color = if ($rule.Enabled) { "Green" } else { "Yellow" }
        $icon = if ($rule.Enabled) { "[OK]" } else { "[!!]" }
        
        Write-Host "   $icon $($rule.DisplayName) ($($rule.Direction)): $status" -ForegroundColor $color
    }
} else {
    Write-Host "   [!!] No IGRIS firewall rules found" -ForegroundColor Yellow
}

Write-Host ""

# Check if IGRIS binary exists
Write-Host "3. Checking IGRIS Binary..." -ForegroundColor Cyan
Write-Host ""

$possiblePaths = @(
    "F:\igrisv3\target\release\igrisv3.exe",
    "F:\igrisv3\target\debug\igrisv3.exe"
)

$igrisFound = $false
foreach ($path in $possiblePaths) {
    if (Test-Path $path) {
        Write-Host "   [OK] Found: $path" -ForegroundColor Green
        $igrisFound = $true
        break
    }
}

if (-not $igrisFound) {
    Write-Host "   [!!] IGRIS binary not found" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ""

# Summary
Write-Host "SUMMARY:" -ForegroundColor Cyan
Write-Host ""

if ($allOn) {
    Write-Host "[OK] All firewall profiles are ON (Protected)" -ForegroundColor Green
} elseif ($allOff) {
    Write-Host "[!!] ALL FIREWALL PROFILES ARE OFF (UNPROTECTED!)" -ForegroundColor Red
    Write-Host ""
    Write-Host "To turn firewall ON:" -ForegroundColor Yellow
    Write-Host "  Set-NetFirewallProfile -Profile Domain,Public,Private -Enabled True" -ForegroundColor White
} else {
    Write-Host "[!!] Some firewall profiles are OFF (Partially Protected)" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "To turn all firewalls ON:" -ForegroundColor Yellow
    Write-Host "  Set-NetFirewallProfile -Profile Domain,Public,Private -Enabled True" -ForegroundColor White
}

if (-not $igrisRules) {
    Write-Host ""
    Write-Host "[!!] IGRIS firewall rules not configured" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "To configure IGRIS firewall:" -ForegroundColor Yellow
    Write-Host "  Run: .\setup_windows_firewall.ps1" -ForegroundColor White
}

Write-Host ""
Write-Host "Press Enter to exit..."
Read-Host
