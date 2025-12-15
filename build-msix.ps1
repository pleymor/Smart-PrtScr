# Build MSIX package for Microsoft Store submission
# Requires Windows SDK (makeappx.exe)

param(
    [switch]$SkipBuild,
    [string]$Architecture = "x64"
)

$ErrorActionPreference = "Stop"

# Configuration
$AppName = "smart-prtscr"
$Version = "2.0.2.0"
$OutputDir = ".\msix-output"
$PackageDir = "$OutputDir\package"

# Find makeappx.exe from Windows SDK
$sdkPaths = @(
    "${env:ProgramFiles(x86)}\Windows Kits\10\bin\10.0.22621.0\x64\makeappx.exe",
    "${env:ProgramFiles(x86)}\Windows Kits\10\bin\10.0.22000.0\x64\makeappx.exe",
    "${env:ProgramFiles(x86)}\Windows Kits\10\bin\10.0.19041.0\x64\makeappx.exe",
    "${env:ProgramFiles(x86)}\Windows Kits\10\bin\10.0.18362.0\x64\makeappx.exe"
)

$makeappx = $null
foreach ($path in $sdkPaths) {
    if (Test-Path $path) {
        $makeappx = $path
        break
    }
}

if (-not $makeappx) {
    # Try to find any version
    $sdkBase = "${env:ProgramFiles(x86)}\Windows Kits\10\bin"
    if (Test-Path $sdkBase) {
        $versions = Get-ChildItem $sdkBase -Directory | Sort-Object Name -Descending
        foreach ($ver in $versions) {
            $candidate = Join-Path $ver.FullName "x64\makeappx.exe"
            if (Test-Path $candidate) {
                $makeappx = $candidate
                break
            }
        }
    }
}

if (-not $makeappx) {
    Write-Error "makeappx.exe not found. Please install Windows SDK from https://developer.microsoft.com/windows/downloads/windows-sdk/"
    exit 1
}

Write-Host "Using makeappx: $makeappx" -ForegroundColor Cyan

# Step 1: Build Tauri app
if (-not $SkipBuild) {
    Write-Host "`n=== Building Tauri application ===" -ForegroundColor Green
    npm run tauri:build
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Tauri build failed"
        exit 1
    }
}

# Step 2: Create package directory structure
Write-Host "`n=== Creating MSIX package structure ===" -ForegroundColor Green

if (Test-Path $OutputDir) {
    Remove-Item $OutputDir -Recurse -Force
}
New-Item -ItemType Directory -Path $PackageDir -Force | Out-Null
New-Item -ItemType Directory -Path "$PackageDir\icons" -Force | Out-Null

# Step 3: Copy built application files
$buildDir = ".\src-tauri\target\release"
if (-not (Test-Path $buildDir)) {
    Write-Error "Build directory not found: $buildDir"
    exit 1
}

Write-Host "Copying application files..." -ForegroundColor Yellow

# Copy main executable
Copy-Item "$buildDir\$AppName.exe" "$PackageDir\" -Force

# Copy WebView2Loader.dll if present
if (Test-Path "$buildDir\WebView2Loader.dll") {
    Copy-Item "$buildDir\WebView2Loader.dll" "$PackageDir\" -Force
}

# Copy any additional DLLs
Get-ChildItem "$buildDir\*.dll" | ForEach-Object {
    Copy-Item $_.FullName "$PackageDir\" -Force
}

# Step 4: Copy icons
Write-Host "Copying icons..." -ForegroundColor Yellow
$iconFiles = @(
    "StoreLogo.png",
    "Square44x44Logo.png",
    "Square71x71Logo.png",
    "Square150x150Logo.png",
    "Square310x310Logo.png"
)

foreach ($icon in $iconFiles) {
    $src = ".\src-tauri\icons\$icon"
    if (Test-Path $src) {
        Copy-Item $src "$PackageDir\icons\" -Force
    } else {
        Write-Warning "Icon not found: $icon"
    }
}

# Step 5: Copy and update manifest
Write-Host "Copying manifest..." -ForegroundColor Yellow
Copy-Item ".\src-tauri\AppxManifest.xml" "$PackageDir\" -Force

# Step 6: Create MSIX package
Write-Host "`n=== Creating MSIX package ===" -ForegroundColor Green
$msixFile = "$OutputDir\SmartPrtScr_${Version}_${Architecture}.msix"

& $makeappx pack /d $PackageDir /p $msixFile /o

if ($LASTEXITCODE -ne 0) {
    Write-Error "makeappx failed"
    exit 1
}

Write-Host "`n=== Build Complete ===" -ForegroundColor Green
Write-Host "MSIX package created: $msixFile" -ForegroundColor Cyan

Write-Host "`n=== Next Steps ===" -ForegroundColor Yellow
Write-Host "1. The MSIX is unsigned. For Store submission, upload it to Partner Center"
Write-Host "   (Microsoft will sign it with your Store certificate)"
Write-Host ""
Write-Host "2. For local testing, you can self-sign with:"
Write-Host "   - Create a test certificate (if needed):"
Write-Host "     New-SelfSignedCertificate -Type Custom -Subject 'CN=TestCert' -KeyUsage DigitalSignature -FriendlyName 'Test Cert' -CertStoreLocation 'Cert:\CurrentUser\My'"
Write-Host "   - Sign the package:"
Write-Host "     signtool sign /fd SHA256 /a /f cert.pfx /p password $msixFile"
Write-Host ""
Write-Host "3. Upload to Partner Center: https://partner.microsoft.com/dashboard"
