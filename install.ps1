# Omitl installer for Windows (PowerShell)
# Usage: irm https://raw.githubusercontent.com/yourusername/omitl/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo    = "yourusername/omitl"
$Target  = "omitl-windows-x86_64"
$InstallDir = "$env:USERPROFILE\.omitl\bin"

# ── Fetch latest release tag ──────────────────────────────────────────────
Write-Host "Fetching latest release..."
$Release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
$Tag = $Release.tag_name

if (-not $Tag) {
    Write-Error "Could not determine latest version."
    exit 1
}

Write-Host "Installing omitl $Tag for Windows x86_64..."

# ── Download ──────────────────────────────────────────────────────────────
$Url = "https://github.com/$Repo/releases/download/$Tag/$Target.zip"
$Tmp = Join-Path $env:TEMP "omitl-install"
New-Item -ItemType Directory -Force -Path $Tmp | Out-Null
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

$ZipPath = Join-Path $Tmp "$Target.zip"
Invoke-WebRequest -Uri $Url -OutFile $ZipPath
Expand-Archive -Path $ZipPath -DestinationPath $Tmp -Force

Move-Item -Force (Join-Path $Tmp "omitl.exe") (Join-Path $InstallDir "omitl.exe")
Remove-Item -Recurse -Force $Tmp

# ── Add to PATH if not already present ───────────────────────────────────
$CurrentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($CurrentPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$CurrentPath;$InstallDir", "User")
    Write-Host "Added $InstallDir to your PATH."
    Write-Host "Restart your terminal for the change to take effect."
}

Write-Host ""
Write-Host "omitl $Tag installed to $InstallDir\omitl.exe"
Write-Host "Run 'omitl --help' to get started."
