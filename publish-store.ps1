# Publish MSIX to Microsoft Store using MSStoreCLI
# Prerequisites: winget install "Microsoft Store Developer CLI"

param(
    [switch]$SkipBuild,
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

# Check if msstore CLI is installed
$msstore = Get-Command msstore -ErrorAction SilentlyContinue
if (-not $msstore) {
    Write-Host "MSStoreCLI not found. Installing..." -ForegroundColor Yellow
    winget install "Microsoft Store Developer CLI"

    # Refresh PATH
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")

    $msstore = Get-Command msstore -ErrorAction SilentlyContinue
    if (-not $msstore) {
        Write-Error "Failed to install MSStoreCLI. Please restart your terminal and try again."
        exit 1
    }
}

Write-Host "Using MSStoreCLI: $($msstore.Source)" -ForegroundColor Cyan

# Step 1: Build MSIX if needed
if (-not $SkipBuild) {
    Write-Host "`n=== Building MSIX package ===" -ForegroundColor Green
    & .\build-msix.ps1
    if ($LASTEXITCODE -ne 0) {
        Write-Error "MSIX build failed"
        exit 1
    }
}

# Step 2: Find the MSIX file
$msixFile = Get-ChildItem ".\msix-output\*.msix" | Sort-Object LastWriteTime -Descending | Select-Object -First 1
if (-not $msixFile) {
    Write-Error "No MSIX file found in msix-output/. Run without -SkipBuild first."
    exit 1
}

Write-Host "`nMSIX package: $($msixFile.Name)" -ForegroundColor Cyan

# Step 3: Check authentication
Write-Host "`n=== Checking authentication ===" -ForegroundColor Green
$configCheck = msstore apps list 2>&1
if ($configCheck -match "not configured" -or $configCheck -match "authenticate") {
    Write-Host "MSStoreCLI not configured. Running setup..." -ForegroundColor Yellow
    msstore configure
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Configuration failed"
        exit 1
    }
}

# Step 4: Publish
Write-Host "`n=== Publishing to Microsoft Store ===" -ForegroundColor Green

if ($DryRun) {
    Write-Host "[DRY RUN] Would publish: $($msixFile.FullName)" -ForegroundColor Yellow
    Write-Host "[DRY RUN] Command: msstore publish `"$($msixFile.FullName)`"" -ForegroundColor Yellow
} else {
    msstore publish $msixFile.FullName
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Publishing failed"
        exit 1
    }

    Write-Host "`n=== Publication submitted ===" -ForegroundColor Green
    Write-Host "Check status with: msstore submission status" -ForegroundColor Cyan
}
