# Publish MSIX to Microsoft Store using MSStoreCLI
# Prerequisites: winget install "Microsoft Store Developer CLI"
# Docs: https://learn.microsoft.com/en-us/windows/apps/publish/msstore-dev-cli/overview

param(
    [switch]$SkipBuild,
    [switch]$DryRun,
    [switch]$UpdateMetadata,
    [switch]$GetMetadata,
    [string]$AppId = ""  # Your Store App ID (e.g., "9NXXXXX")
)

$ErrorActionPreference = "Stop"
$MetadataFile = ".\store-metadata.json"

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

# Check authentication
Write-Host "`n=== Checking authentication ===" -ForegroundColor Green
$configCheck = msstore apps list 2>&1
if ($configCheck -match "not configured" -or $configCheck -match "authenticate" -or $LASTEXITCODE -ne 0) {
    Write-Host "MSStoreCLI not configured. Running setup..." -ForegroundColor Yellow
    msstore configure
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Configuration failed"
        exit 1
    }
}

# Mode: Get metadata from existing submission
if ($GetMetadata) {
    if (-not $AppId) {
        Write-Host "`nAvailable apps:" -ForegroundColor Cyan
        msstore apps list
        Write-Host "`nUsage: .\publish-store.ps1 -GetMetadata -AppId <APP_ID>" -ForegroundColor Yellow
        exit 0
    }

    Write-Host "`n=== Fetching metadata for $AppId ===" -ForegroundColor Green
    msstore submission get $AppId | Out-File -FilePath $MetadataFile -Encoding utf8
    Write-Host "Metadata saved to: $MetadataFile" -ForegroundColor Cyan
    exit 0
}

# Mode: Update metadata only
if ($UpdateMetadata) {
    if (-not $AppId) {
        Write-Host "Error: -AppId required for metadata update" -ForegroundColor Red
        Write-Host "Usage: .\publish-store.ps1 -UpdateMetadata -AppId <APP_ID>" -ForegroundColor Yellow
        exit 1
    }

    if (-not (Test-Path $MetadataFile)) {
        Write-Host "Error: $MetadataFile not found" -ForegroundColor Red
        Write-Host "First run: .\publish-store.ps1 -GetMetadata -AppId $AppId" -ForegroundColor Yellow
        exit 1
    }

    Write-Host "`n=== Updating metadata for $AppId ===" -ForegroundColor Green

    if ($DryRun) {
        Write-Host "[DRY RUN] Would update metadata from: $MetadataFile" -ForegroundColor Yellow
    } else {
        msstore submission updateMetadata $AppId $MetadataFile
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Metadata update failed"
            exit 1
        }
        Write-Host "Metadata updated successfully" -ForegroundColor Green
    }
    exit 0
}

# Default mode: Build and publish MSIX

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

# Step 3: Publish
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

Write-Host "`n=== Available commands ===" -ForegroundColor Yellow
Write-Host "  .\publish-store.ps1                        # Build + publish MSIX"
Write-Host "  .\publish-store.ps1 -SkipBuild             # Publish existing MSIX"
Write-Host "  .\publish-store.ps1 -GetMetadata -AppId X  # Download current metadata"
Write-Host "  .\publish-store.ps1 -UpdateMetadata -AppId X  # Push metadata changes"
Write-Host "  msstore submission status                  # Check submission status"
