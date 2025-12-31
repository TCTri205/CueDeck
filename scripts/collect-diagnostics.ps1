# CueDeck Diagnostic Collection Script
# Usage: .\collect-diagnostics.ps1

Write-Host "CueDeck Diagnostic Collection" -ForegroundColor Cyan
Write-Host "==============================" -ForegroundColor Cyan

$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$outputDir = "cuedeck-debug-$timestamp"

# Create output directory
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
Write-Host "Created: $outputDir" -ForegroundColor Green

# 1. System Information
Write-Host "Collecting system info..." -ForegroundColor Yellow
$sysInfo = @{
    OS = [System.Environment]::OSVersion.VersionString
    PowerShell = $PSVersionTable.PSVersion.ToString()
    Timestamp = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss")
}
$sysInfo | ConvertTo-Json | Out-File "$outputDir/system-info.json"

# 2. Rust/Cargo version
Write-Host "Collecting Rust info..." -ForegroundColor Yellow
try {
    $rustVersion = & rustc --version 2>&1
    $cargoVersion = & cargo --version 2>&1
    @{
        Rust = $rustVersion
        Cargo = $cargoVersion
    } | ConvertTo-Json | Out-File "$outputDir/rust-info.json"
} catch {
    "Rust not found" | Out-File "$outputDir/rust-info.txt"
}

# 3. CueDeck workspace check
Write-Host "Checking workspace..." -ForegroundColor Yellow
if (Test-Path ".cuedeck") {
    Get-ChildItem -Path ".cuedeck" -Recurse | Select-Object FullName, Length, LastWriteTime | 
        ConvertTo-Json | Out-File "$outputDir/workspace-structure.json"
} else {
    "No .cuedeck directory found" | Out-File "$outputDir/workspace-structure.txt"
}

# 4. Config file
Write-Host "Copying config..." -ForegroundColor Yellow
if (Test-Path ".cuedeck/config.toml") {
    Copy-Item ".cuedeck/config.toml" "$outputDir/config.toml"
}

# 5. Logs
Write-Host "Collecting logs..." -ForegroundColor Yellow
if (Test-Path ".cuedeck/logs") {
    New-Item -ItemType Directory -Force -Path "$outputDir/logs" | Out-Null
    Get-ChildItem ".cuedeck/logs/*.log" -ErrorAction SilentlyContinue | 
        ForEach-Object { Copy-Item $_.FullName "$outputDir/logs/" }
}

# 6. Run cue doctor if available
Write-Host "Running diagnostics..." -ForegroundColor Yellow
try {
    $doctorOutput = & cargo run --bin cue -- doctor 2>&1
    $doctorOutput | Out-File "$outputDir/cue-doctor.txt"
} catch {
    "cue doctor not available" | Out-File "$outputDir/cue-doctor.txt"
}

# 7. Create archive
Write-Host "Creating archive..." -ForegroundColor Yellow
$archiveName = "cuedeck-debug-$timestamp.zip"
Compress-Archive -Path $outputDir -DestinationPath $archiveName -Force

# Cleanup
Remove-Item -Recurse -Force $outputDir

Write-Host ""
Write-Host "Diagnostics saved to: $archiveName" -ForegroundColor Green
Write-Host "Please attach this file when reporting issues." -ForegroundColor Cyan
