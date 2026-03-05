#!/usr/bin/env -S pwsh -NoProfile -File
#Requires -Version 7.0

<#
.SYNOPSIS
    Builds and installs pwsh-prompt to ~/.local/bin.

.DESCRIPTION
    This script builds the native executable and installs it to ~/.local/bin/pwsh-prompt.
    After installation, you can use the prompt in your PowerShell profile.

    Requirements:
    - PowerShell 7.0+
    - .NET 10 SDK

.PARAMETER Force
    Skip confirmation prompts and overwrite existing installation.

.PARAMETER Update
    Pull the latest changes from the remote before building.

.EXAMPLE
    ./install-local.ps1

    Builds and installs pwsh-prompt to ~/.local/bin

.EXAMPLE
    ./install-local.ps1 -Force

    Builds and installs, overwriting any existing installation.

.EXAMPLE
    ./install-local.ps1 -Update -Force

    Pull latest changes, then build and install.
#>

[CmdletBinding()]
param(
    [switch]$Force,

    [switch]$Update
)

$ErrorActionPreference = 'Stop'

# Check if the local clone is up-to-date with remote
Write-Host "Checking if repository is up-to-date..." -ForegroundColor Cyan
try {
    Push-Location $PSScriptRoot
    $branch = & git rev-parse --abbrev-ref HEAD 2>&1
    & git fetch origin $branch --quiet 2>&1
    $local = & git rev-parse HEAD 2>&1
    $remote = & git rev-parse "origin/$branch" 2>&1

    if ($local -ne $remote) {
        $behind = & git rev-list --count "HEAD..origin/$branch" 2>&1
        $ahead = & git rev-list --count "origin/$branch..HEAD" 2>&1
        $status = @()
        if ([int]$behind -gt 0) { $status += "$behind commit(s) behind" }
        if ([int]$ahead -gt 0) { $status += "$ahead commit(s) ahead of" }
        Write-Host "Warning: Local branch '$branch' is $($status -join ' and ') origin/$branch." -ForegroundColor Yellow

        if ($Update) {
            Write-Host "Pulling latest changes..." -ForegroundColor Cyan
            & git pull --quiet 2>&1
            if ($LASTEXITCODE -ne 0) {
                Write-Host "Error: git pull failed. Resolve conflicts and try again." -ForegroundColor Red
                exit 1
            }
            Write-Host "Repository updated successfully." -ForegroundColor Cyan
        } elseif (-not $Force) {
            $response = Read-Host "Continue anyway? (y/N)"
            if ($response -notmatch '^[Yy]') {
                Write-Host "Installation cancelled. Run 'git pull' or use -Update to update." -ForegroundColor Cyan
                exit 0
            }
        }
    } else {
        Write-Host "Repository is up-to-date with origin/$branch." -ForegroundColor Cyan
    }
} catch {
    Write-Host "Warning: Could not check remote status: $_" -ForegroundColor Yellow
    Write-Host "Continuing with installation..." -ForegroundColor Yellow
} finally {
    Pop-Location
}

# Determine platform-specific runtime identifier
$rid = if ($IsWindows) {
    'win-x64'
} elseif ($IsMacOS) {
    if ([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture -eq 'Arm64') {
        'osx-arm64'
    } else {
        'osx-x64'
    }
} elseif ($IsLinux) {
    'linux-x64'
} else {
    throw "Unsupported platform"
}

Write-Host "Building pwsh-prompt for $rid..." -ForegroundColor Cyan

# Build the project
$projectPath = Join-Path -Path $PSScriptRoot -ChildPath 'src' -AdditionalChildPath 'pwsh-prompt', 'pwsh-prompt.csproj'
$publishPath = Join-Path $PSScriptRoot 'publish'

dotnet publish $projectPath -c Release -r $rid --output $publishPath

if ($LASTEXITCODE -ne 0) {
    throw "Build failed with exit code $LASTEXITCODE"
}

# Determine executable name
$exeName = if ($IsWindows) { 'pwsh-prompt.exe' } else { 'pwsh-prompt' }
$exePath = Join-Path $publishPath $exeName

if (-not (Test-Path $exePath)) {
    throw "Executable not found at $exePath"
}

Write-Host "Build successful!" -ForegroundColor Green

# Create installation directory
$installDir = Join-Path -Path $HOME -ChildPath '.local' -AdditionalChildPath 'bin'

if (-not (Test-Path $installDir)) {
    Write-Host "Creating installation directory at $installDir..." -ForegroundColor Cyan
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

# Copy executable
$installPath = Join-Path $installDir $exeName

if (Test-Path $installPath) {
    if (-not $Force) {
        Write-Host "pwsh-prompt is already installed at: $installPath" -ForegroundColor Yellow
        $response = Read-Host "Overwrite existing installation? (y/N)"
        if ($response -notmatch '^[Yy]') {
            Write-Host "Installation cancelled." -ForegroundColor Cyan
            exit 0
        }
    }
    Write-Host "Removing existing installation..." -ForegroundColor Yellow
    Remove-Item $installPath -Force
}

Write-Host "Copying executable to $installPath..." -ForegroundColor Cyan
Copy-Item $exePath $installPath

# Make executable on Unix systems
if (-not $IsWindows) {
    chmod +x $installPath
}

Write-Host ""
Write-Host "Installation complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Executable installed to: $installPath" -ForegroundColor Cyan
Write-Host ""
Write-Host "To use the prompt, add this to your PowerShell profile:" -ForegroundColor Cyan
Write-Host "    Invoke-Expression (& '$installPath' init)" -ForegroundColor White
Write-Host ""
Write-Host "To edit your profile, run:" -ForegroundColor Cyan
Write-Host "    code `$PROFILE" -ForegroundColor White
