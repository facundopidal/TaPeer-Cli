$ErrorActionPreference = "Stop"

$Repo = "facundopidal/tapeer-cli"
$Target = "x86_64-pc-windows-msvc"
$DownloadUrl = "https://github.com/$Repo/releases/latest/download/tapeer-$Target.zip"

$InstallDir = "$env:LOCALAPPDATA\tapeer\bin"
$TempDir = [System.IO.Path]::GetTempPath()
$ZipPath = Join-Path $TempDir "tapeer.zip"
$ExtractPath = Join-Path $TempDir "tapeer-extracted-$([Guid]::NewGuid().ToString().Substring(0,8))"

Write-Host "==> Downloading TaPeer CLI for Windows..." -ForegroundColor Cyan

try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath -UseBasicParsing
} catch {
    Write-Error "Failed to download TaPeer CLI: $_"
    exit 1
}

if (!(Test-Path -Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

Write-Host "==> Extracting files..." -ForegroundColor Cyan
Expand-Archive -Path $ZipPath -DestinationPath $ExtractPath -Force

$BinaryPath = Join-Path $ExtractPath "tapeer.exe"
if (Test-Path -Path $BinaryPath) {
    Move-Item -Path $BinaryPath -Destination (Join-Path $InstallDir "tapeer.exe") -Force
} else {
    # Check if subfolder was created
    $Found = Get-ChildItem -Path $ExtractPath -Filter "tapeer.exe" -Recurse | Select-Object -First 1
    if ($Found) {
        Move-Item -Path $Found.FullName -Destination (Join-Path $InstallDir "tapeer.exe") -Force
    } else {
        Write-Error "tapeer.exe not found in downloaded archive."
        exit 1
    }
}

# Cleanup temporary files
Remove-Item -Path $ZipPath -Force -ErrorAction SilentlyContinue
Remove-Item -Path $ExtractPath -Recurse -Force -ErrorAction SilentlyContinue

# Ensure $InstallDir is in user's PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    $NewPath = if ([string]::IsNullOrEmpty($UserPath)) { $InstallDir } else { "$UserPath;$InstallDir" }
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    $env:Path += ";$InstallDir"
    Write-Host "==> Added $InstallDir to user PATH." -ForegroundColor Yellow
}

Write-Host "`n==> ✓ TaPeer CLI installed successfully to $InstallDir\tapeer.exe!" -ForegroundColor Green
Write-Host "==> Run 'tapeer --help' to get started." -ForegroundColor Cyan
