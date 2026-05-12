# pwsh-prompt

[![CI](https://github.com/lucaspimentel/pwsh-prompt/actions/workflows/ci.yml/badge.svg)](https://github.com/lucaspimentel/pwsh-prompt/actions/workflows/ci.yml) [![Release](https://github.com/lucaspimentel/pwsh-prompt/actions/workflows/release.yml/badge.svg)](https://github.com/lucaspimentel/pwsh-prompt/actions/workflows/release.yml)

A fast, minimal shell prompt for PowerShell, written in C#.

Pre-built binaries support Windows and Linux. macOS is supported when built from source.

## Features

- **Lightning fast**: Native AOT compilation means zero startup delay
- **Git integration**: Shows current branch and PR number with status icons (open, draft, closed) and smart caching
- **Path display**: Intelligently truncates long paths to fit terminal width
- **Command feedback**: Displays exit codes and execution duration
- **System info**: Shows OS and shell information
- **Simple mode**: Minimal prompt option for even faster rendering
- **Windows Terminal integration**: Shell integration escape sequences for new-tab-same-directory, scrollbar marks, and command navigation

## Installation

### Scoop (Windows)

```powershell
scoop bucket add lucaspimentel https://github.com/lucaspimentel/scoop-bucket
scoop install pwsh-prompt
```

### Install from GitHub Release (Recommended)

No build tools or .NET SDK required:

```powershell
irm https://raw.githubusercontent.com/lucaspimentel/pwsh-prompt/main/install-remote.ps1 | iex
```

Or to install a specific version:

```powershell
$version = "1.0.0"; irm https://raw.githubusercontent.com/lucaspimentel/pwsh-prompt/main/install-remote.ps1 | iex
```

### Install from Source

Clone the repository and run the installation script:

```powershell
./install-local.ps1
```

This will build the native binary and install it to `~/.local/bin/pwsh-prompt`.

Options:
- `-Force` — skip confirmation prompts and overwrite existing installation
- `-Update` — pull latest changes from the remote before building

```powershell
# Pull latest and install without prompts
./install-local.ps1 -Update -Force
```

### Setup

Add this line to your PowerShell profile (`$PROFILE`):

```powershell
Invoke-Expression (& ~/.local/bin/pwsh-prompt init)
```

### Simple Mode

For a minimal prompt with just path, git, and host information:

```powershell
# Edit the init output or modify your profile to use --simple
pwsh-prompt prompt --simple
```

### Version

To check the installed version:

```powershell
pwsh-prompt --version
```

## Building Manually

```powershell
# Quick build test
dotnet build src/pwsh-prompt -c Release -f net10.0

# Publish for Windows
dotnet publish src/pwsh-prompt -c Release -r win-x64 --output ./publish

# Publish for Linux
dotnet publish src/pwsh-prompt -c Release -r linux-x64 --output ./publish

# Or build via solution
dotnet build pwsh-prompt.slnx
```

## Project Structure

```
pwsh-prompt/
├── src/
│   └── pwsh-prompt/          # Main C# project
│       ├── Segments/         # Segment implementations (ISegment)
│       ├── Program.cs        # Entry point and mode routing
│       ├── Arguments.cs      # Command-line argument parsing
│       ├── GitInfo.cs        # Git repository detection and caching
│       ├── Init.cs           # PowerShell initialization script generator
│       └── ...               # Other core files
├── install-local.ps1         # Build from source and install
├── install-remote.ps1        # Download from GitHub releases and install
└── pwsh-prompt.slnx          # Solution file
```

## Architecture

The prompt uses a modular segment system where each visual element (path, git branch, exit code, etc.) implements the `ISegment` interface. The application runs in two modes:

1. **init mode**: Generates PowerShell initialization code to install the prompt
2. **prompt mode**: Renders the actual prompt on every invocation

### Performance

Since this runs on every prompt render, performance is critical:

- **Native AOT compilation** eliminates JIT overhead for instant startup
- **Environment variable caching** avoids repeated git file I/O and `gh pr view` subprocess calls (cache invalidates when the discovered `.git` directory changes or `.git/HEAD` is modified — `cd` within the same repo reuses the cache)
- **Per-branch PR cache** keeps PR info for every branch visited in the current shell session, so toggling between branches doesn't re-run `gh pr view`
- **Stack-allocated buffers** minimize heap allocations via `ValueStringBuilder`
- **Smart truncation** calculates exact widths for optimal path display

The PowerShell profile caches git information between prompts, avoiding file system operations when the branch hasn't changed.

## License

MIT License - see [LICENSE](LICENSE) file for details.

Copyright (c) 2025 Lucas Pimentel
