<#
.SYNOPSIS
    Botox Installer & Updater for Windows (PowerShell)
    https://github.com/botoxMD/botox-cli

.DESCRIPTION
    Installs or updates the Botox single-binary executable on Windows.
    Automatically detects architecture, fetches the latest release from GitHub,
    extracts botox.exe, places it in the user's Programs folder, and configures PATH.

.EXAMPLE
    irm https://raw.githubusercontent.com/botoxMD/botox-cli/main/install.ps1 | iex
    .\install.ps1 -Version v0.2.0 -Force
#>

[CmdletBinding()]
param (
    [string]$Version = "",
    [string]$InstallDir = "",
    [switch]$Force
)

$ErrorActionPreference = "Stop"

$Repo = "botoxMD/botox-cli"
$BinName = "botox.exe"

function Write-Step {
    param([string]$Message)
    Write-Host "==> " -ForegroundColor Blue -NoNewline
    Write-Host $Message -ForegroundColor White
}

function Write-Success {
    param([string]$Message)
    Write-Host "==> " -ForegroundColor Green -NoNewline
    Write-Host $Message -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host "warning: " -ForegroundColor Yellow -NoNewline
    Write-Host $Message
}

# 1. Architecture Check
$Arch = $env:PROCESSOR_ARCHITECTURE
if ($Arch -ne "AMD64" -and $Arch -ne "x86_64") {
    Write-Error "Unsupported Windows architecture: $Arch. Pre-built Windows binaries are currently provided for x86_64 (AMD64)."
    exit 1
}
$Target = "x86_64-pc-windows-msvc"

# 2. Resolve Tag
if (-not $Version) {
    Write-Step "Checking latest release..."
    try {
        $Req = [System.Net.WebRequest]::Create("https://github.com/$Repo/releases/latest")
        $Req.AllowAutoRedirect = $false
        $Resp = $Req.GetResponse()
        $Location = $Resp.GetResponseHeader("Location")
        $Resp.Close()
        if ($Location -match "/tag/([^/]+)") {
            $Tag = $Matches[1]
        }
    } catch {
        # Fallback to GitHub API
        try {
            $Release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -Headers @{ "User-Agent" = "botox-installer" }
            $Tag = $Release.tag_name
        } catch {
            Write-Error "Could not determine latest version: $_"
            exit 1
        }
    }
} else {
    $Tag = $Version
    if (-not $Tag.StartsWith("v")) {
        $Tag = "v$Tag"
    }
}

$CleanVersion = $Tag.TrimStart("v")

# 3. Check existing installation for update
$ExistingPath = $null
$Command = Get-Command "botox" -ErrorAction SilentlyContinue
if ($Command) {
    $ExistingPath = $Command.Source
    $CurrentVersionOutput = & $ExistingPath --version 2>$null
    if ($CurrentVersionOutput -match "botox\s+([0-9\.]+)") {
        $InstalledVer = $Matches[1]
        if ($InstalledVer -eq $CleanVersion -and -not $Force) {
            Write-Success "Botox is already up to date!"
            Write-Host "  Version:  $InstalledVer" -ForegroundColor Green
            Write-Host "  Location: $ExistingPath"
            Write-Host "  (Run with -Force to reinstall)"
            return
        }
    }
}

# 4. Determine Destination
if ($InstallDir) {
    $DestDir = $InstallDir
} elseif ($ExistingPath) {
    $DestDir = Split-Path -Parent $ExistingPath
} else {
    $DestDir = Join-Path $env:LOCALAPPDATA "Programs\botox"
}

if (-not (Test-Path $DestDir)) {
    New-Item -ItemType Directory -Path $DestDir -Force | Out-Null
}

# 5. Download and Extract
$ArchiveName = "botox-$Tag-$Target.zip"
$DownloadUrl = "https://github.com/$Repo/releases/download/$Tag/$ArchiveName"
$TempZip = Join-Path ([System.IO.Path]::GetTempPath()) ([System.IO.Path]::GetRandomFileName() + ".zip")
$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.IO.Path]::GetRandomFileName())

try {
    Write-Step "Downloading Botox $Tag for Windows ($Target)..."
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempZip -UseBasicParsing

    Write-Step "Extracting executable..."
    Expand-Archive -Path $TempZip -DestinationPath $TempDir -Force

    $ExtractedBin = Join-Path $TempDir $BinName
    if (-not (Test-Path $ExtractedBin)) {
        # Check subdirectories if nested
        $Found = Get-ChildItem -Path $TempDir -Filter $BinName -Recurse
        if ($Found) {
            $ExtractedBin = $Found[0].FullName
        } else {
            Write-Error "$BinName was not found in release archive."
            exit 1
        }
    }

    $DestFile = Join-Path $DestDir $BinName
    Write-Step "Installing to $DestFile..."
    Copy-Item -Path $ExtractedBin -Destination $DestFile -Force
} finally {
    if (Test-Path $TempZip) { Remove-Item $TempZip -Force -ErrorAction SilentlyContinue }
    if (Test-Path $TempDir) { Remove-Item $TempDir -Recurse -Force -ErrorAction SilentlyContinue }
}

# 6. Verify and Path Setup
$VerifiedVer = & (Join-Path $DestDir $BinName) --version 2>$null
Write-Success "Successfully installed $VerifiedVer at $DestDir\$BinName!"

# Check if in User PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
$PathDirs = $UserPath -split ";"
if ($PathDirs -notcontains $DestDir) {
    Write-Warn "$DestDir is not currently in your User PATH."
    Write-Step "Adding $DestDir to User PATH..."
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$DestDir", "User")
    $env:Path = "$env:Path;$DestDir"
    Write-Success "Added $DestDir to User PATH. Restart open terminal windows for changes to take effect."
} else {
    Write-Host "`nRun 'botox --help' to get started." -ForegroundColor Cyan
}
