# pwsh-prompt

A fast, customizable PowerShell prompt written in C# and compiled as a native AOT binary for instant startup.

## Features

- **Lightning fast**: Native AOT compilation means zero startup delay
- **Git integration**: Shows current branch with smart caching
- **Path display**: Intelligently truncates long paths to fit terminal width
- **Command feedback**: Displays exit codes and execution duration
- **System info**: Shows OS and shell information
- **Simple mode**: Minimal prompt option for even faster rendering

## Installation

Run the installation script:

```powershell
./install.ps1
```

This will build the native binary and install it to `~/.local/bin/pwsh-prompt`.

Then add this line to your PowerShell profile (`$PROFILE`):

```powershell
Invoke-Expression (& ~/.local/bin/pwsh-prompt init)
```

### Simple Mode

For a minimal prompt with just path, git, and host information:

```powershell
# Edit the init output or modify your profile to use --simple
pwsh-prompt prompt --simple
```

## Building Manually

```powershell
# Quick build test
dotnet build -c Release -f net10.0

# Publish for Windows
dotnet publish -c Release -r win-x64 --output ./publish

# Publish for Linux
dotnet publish -c Release -r linux-x64 --output ./publish
```

## How It Works

The prompt runs as a native executable on every prompt render. To achieve maximum performance:

- **Native AOT compilation** eliminates JIT overhead
- **Environment variable caching** avoids repeated file I/O for git information
- **Stack-allocated buffers** minimize heap allocations
- **Smart truncation** calculates exact width for path display

Git information is cached in PowerShell environment variables between prompts. The cache is invalidated when you change directories or when `.git/HEAD` changes (checkout, commit, rebase, etc.).

## License

Personal project by Lucas Pimentel.
