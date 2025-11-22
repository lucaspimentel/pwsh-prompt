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

## Architecture

The prompt uses a modular segment system where each visual element (path, git branch, exit code, etc.) implements the `ISegment` interface. The application runs in two modes:

1. **init mode**: Generates PowerShell initialization code to install the prompt
2. **prompt mode**: Renders the actual prompt on every invocation

### Performance

Since this runs on every prompt render, performance is critical:

- **Native AOT compilation** eliminates JIT overhead for instant startup
- **Environment variable caching** avoids repeated git file I/O (cache invalidates on directory change or `.git/HEAD` modification)
- **Stack-allocated buffers** minimize heap allocations via `ValueStringBuilder`
- **Smart truncation** calculates exact widths for optimal path display

The PowerShell profile caches git information between prompts, avoiding file system operations when the branch hasn't changed.

## License

MIT License - see [LICENSE](LICENSE) file for details.

Copyright (c) 2025 Lucas Pimentel
