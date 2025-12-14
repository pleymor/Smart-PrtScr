# Smart PrtScr - Automated Demo Script
# Records a demo of the app features using simulated input
#
# Prerequisites:
# 1. App must be running (npm run tauri:dev or built exe)
# 2. OBS Studio open with "Start Recording" hotkey set to Ctrl+F9
# 3. Screen resolution: 1920x1080 recommended
#
# Usage: .\record-demo.ps1

Add-Type -AssemblyName System.Windows.Forms

# Helper functions
function Wait($ms) {
    Start-Sleep -Milliseconds $ms
}

function MoveMouse($x, $y, $steps = 20, $delay = 10) {
    $current = [System.Windows.Forms.Cursor]::Position
    $startX = $current.X
    $startY = $current.Y

    for ($i = 1; $i -le $steps; $i++) {
        $newX = $startX + (($x - $startX) * $i / $steps)
        $newY = $startY + (($y - $startY) * $i / $steps)
        [System.Windows.Forms.Cursor]::Position = New-Object System.Drawing.Point($newX, $newY)
        Wait $delay
    }
}

function Click() {
    $signature = @"
    [DllImport("user32.dll", CharSet = CharSet.Auto, CallingConvention = CallingConvention.StdCall)]
    public static extern void mouse_event(uint dwFlags, uint dx, uint dy, uint cButtons, uint dwExtraInfo);
"@
    $mouse = Add-Type -MemberDefinition $signature -Name "MouseEvent" -Namespace "Win32" -PassThru
    $mouse::mouse_event(0x0002, 0, 0, 0, 0) # Left down
    Wait 50
    $mouse::mouse_event(0x0004, 0, 0, 0, 0) # Left up
}

function DragTo($x, $y, $steps = 30, $delay = 15) {
    $signature = @"
    [DllImport("user32.dll", CharSet = CharSet.Auto, CallingConvention = CallingConvention.StdCall)]
    public static extern void mouse_event(uint dwFlags, uint dx, uint dy, uint cButtons, uint dwExtraInfo);
"@
    $mouse = Add-Type -MemberDefinition $signature -Name "MouseDrag" -Namespace "Win32Drag" -PassThru

    $mouse::mouse_event(0x0002, 0, 0, 0, 0) # Left down
    Wait 50
    MoveMouse $x $y $steps $delay
    Wait 50
    $mouse::mouse_event(0x0004, 0, 0, 0, 0) # Left up
}

function PressKey($key) {
    [System.Windows.Forms.SendKeys]::SendWait($key)
}

# Screen dimensions (adjust if different)
$screenWidth = 1920
$screenHeight = 1080

Write-Host "=== Smart PrtScr Demo Recording ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "Prerequisites:" -ForegroundColor Yellow
Write-Host "  1. Smart PrtScr must be running"
Write-Host "  2. OBS ready with Ctrl+F9 = Start/Stop Recording"
Write-Host "  3. Close all other windows for clean demo"
Write-Host ""
Write-Host "Starting in 5 seconds..." -ForegroundColor Green
Write-Host "Press Ctrl+C to cancel"
Wait 5000

# ============================================
# START RECORDING (OBS hotkey)
# ============================================
Write-Host ">> Starting OBS recording..." -ForegroundColor Cyan
PressKey "^{F9}"
Wait 2000

# ============================================
# SCENE 1: Trigger screenshot with PrintScreen
# ============================================
Write-Host ">> Scene 1: Triggering PrintScreen..." -ForegroundColor Cyan
PressKey "{PRTSC}"
Wait 1500

# ============================================
# SCENE 2: Draw a selection rectangle
# ============================================
Write-Host ">> Scene 2: Drawing selection..." -ForegroundColor Cyan
# Start position (top-left of selection)
$startX = 400
$startY = 250
# End position (bottom-right of selection)
$endX = 1100
$endY = 650

MoveMouse $startX $startY
Wait 300
DragTo $endX $endY 40 20
Wait 1000

# ============================================
# SCENE 3: Resize using SE handle (bottom-right)
# ============================================
Write-Host ">> Scene 3: Resizing selection..." -ForegroundColor Cyan
# Move to SE handle
MoveMouse $endX $endY
Wait 500
# Drag to make it larger
DragTo ($endX + 150) ($endY + 100) 30 20
Wait 800

# ============================================
# SCENE 4: Resize using NW handle (top-left)
# ============================================
Write-Host ">> Scene 4: Resizing from top-left..." -ForegroundColor Cyan
MoveMouse $startX $startY
Wait 500
DragTo ($startX - 80) ($startY - 60) 30 20
Wait 800

# ============================================
# SCENE 5: Move the selection
# ============================================
Write-Host ">> Scene 5: Moving selection..." -ForegroundColor Cyan
# Calculate center of current selection
$centerX = ($startX - 80 + $endX + 150) / 2
$centerY = ($startY - 60 + $endY + 100) / 2
MoveMouse $centerX $centerY
Wait 500
# Drag to new position
DragTo ($centerX + 200) ($centerY - 100) 40 20
Wait 1000

# ============================================
# SCENE 6: Confirm with Enter
# ============================================
Write-Host ">> Scene 6: Confirming selection..." -ForegroundColor Cyan
Wait 500
PressKey "{ENTER}"
Wait 2000

# ============================================
# SCENE 7: Show save dialog briefly
# ============================================
Write-Host ">> Scene 7: Save dialog visible..." -ForegroundColor Cyan
Wait 2000

# Press Escape to cancel (we don't actually save in demo)
PressKey "{ESCAPE}"
Wait 1000

# ============================================
# STOP RECORDING
# ============================================
Write-Host ">> Stopping OBS recording..." -ForegroundColor Cyan
PressKey "^{F9}"
Wait 500

Write-Host ""
Write-Host "=== Demo recording complete! ===" -ForegroundColor Green
Write-Host "Check OBS for the recorded video." -ForegroundColor Yellow
