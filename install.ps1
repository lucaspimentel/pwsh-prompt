#!/usr/bin/env pwsh
#Requires -Version 7.0

<#
.SYNOPSIS
    Builds and installs pwsh-prompt to ~/.local/bin.

.DESCRIPTION
    This script builds the native executable and installs it to ~/.local/bin/pwsh-prompt.
    After installation, you can use the prompt in your PowerShell profile.

.EXAMPLE
    ./install.ps1
#>

[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

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
$projectPath = Join-Path $PSScriptRoot 'pwsh-prompt.csproj'
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
$installDir = Join-Path $HOME '.local' 'bin'

if (-not (Test-Path $installDir)) {
    Write-Host "Creating installation directory at $installDir..." -ForegroundColor Cyan
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

# Copy executable
$installPath = Join-Path $installDir $exeName

if (Test-Path $installPath) {
    Write-Host "Removing existing installation at $installPath..." -ForegroundColor Yellow
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
