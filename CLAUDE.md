# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a custom PowerShell prompt written in C# that displays contextual information about the current directory, git repository, last command status, and system information. It's compiled as a native AOT binary for fast startup times.

## Build Commands

- **Restore dependencies**: `dotnet restore`
- **Build**: `dotnet build`
- **Publish (Windows)**: `dotnet publish -c release -r win-x64`
- **Publish (Linux)**: `dotnet publish -c release -r linux-x64`

Published binaries are output to `bin/release/net9.0/{rid}/publish/pwsh-prompt.exe`

## Architecture

The application has two modes:

1. **init mode**: Generates PowerShell initialization code that sets up the prompt function. This is invoked with `pwsh-prompt init` and the output is evaluated by PowerShell to install the prompt.

2. **prompt mode**: Invoked by PowerShell on each prompt render. Receives arguments about terminal state (width, current directory, last command info) and outputs ANSI-formatted prompt text using Spectre.Console markup.

### Core Components

- **Program.cs**: Entry point that routes between init/prompt modes and orchestrates segment rendering
- **Arguments.cs**: Parses command-line arguments passed from PowerShell
- **Init.cs**: Generates PowerShell script that installs the prompt function
- **GitInfo.cs**: Traverses directory tree to find .git folder and parse HEAD/config files to determine current branch
- **ValueStringBuilder**: High-performance string building using stack-allocated buffers

### Segment System

All visual elements implement `ISegment` interface (in `Modules/`):
- Each segment calculates its own `UnformattedLength` (for layout) and formats itself via `Append(ref ValueStringBuilder)`
- Segments use Spectre.Console markup syntax for colors (e.g., `[aqua]text[/]`)
- Main segments: `PathSegment`, `GitSegment`, `HostSegment`, `LastCommandExitCodeSegment`, `LastCommandDurationSegment`, `DateTimeSegment`, `OsSegment`, `PromptSegment`

The prompt is rendered as two lines with automatic width calculation and filler space to right-align certain segments.

### Git Integration

- `GitInfo.TryFindGitFolder()`: Walks up directory tree looking for `.git` folder
- `GitInfo.GetBranchName()`: Parses `.git/HEAD` and `.git/config` to determine current branch name
- **Important**: Only works with regular git repositories (`.git` as directory). Does NOT currently support git worktrees where `.git` is a file pointing to the actual git directory.

## Performance Optimizations

- Native AOT compilation for instant startup
- Stack-allocated buffers via `ValueStringBuilder`
- Minimal allocations in hot paths
- Aggressive AOT trimming options configured in .csproj
