# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a custom PowerShell prompt written in C# that displays contextual information about the current directory, git repository, last command status, and system information. It's compiled as a native AOT binary for fast startup times.

## Build Commands

- **Restore dependencies**: `dotnet restore`
- **Build**: `dotnet build`
- **Publish (Windows)**: `dotnet publish -c release -r win-x64`
- **Publish (Linux)**: `dotnet publish -c release -r linux-x64`

Published binaries are output to `bin/release/net10.0/{rid}/publish/pwsh-prompt.exe`

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

- `GitInfo.TryFindGitFolder()`: Walks up directory tree looking for `.git` folder or file (for worktrees)
- `GitInfo.GetBranchName()`: Parses `.git/HEAD` and `.git/config` to determine current branch name
- Supports both regular repositories (`.git` as directory) and git worktrees (`.git` as file containing `gitdir:` path)
- For worktrees, the config file is found in the main repository (two directories up from the worktree git directory)

## Performance Optimizations

**Critical**: This tool runs on every prompt render, so performance is paramount.

- Native AOT compilation for instant startup
- Stack-allocated buffers via `ValueStringBuilder`
- Minimal allocations in hot paths
- Aggressive AOT trimming options configured in .csproj

### Environment Variable Caching

**Important**: The C# process starts fresh on every prompt invocation, so static caching doesn't work. Instead, caching happens via PowerShell environment variables:

- **PowerShell side** (Init.cs): Before invoking the C# binary, checks if still in same directory and if `.git/HEAD` file is unchanged. If so, passes cached git directory and branch name via `PROMPT_GIT_DIR_CACHED` and `PROMPT_GIT_BRANCH_CACHED` environment variables. After invocation, reads `PROMPT_GIT_DIR_OUT` and `PROMPT_GIT_BRANCH_OUT` from the C# process and stores them along with the HEAD file content for next prompt.

- **C# side** (GitInfo.cs): `TryFindGitFolder()` and `GetBranchName()` check the `*_CACHED` environment variables first. If found, they skip all file I/O. After computing fresh values, they write to `*_OUT` environment variables for PowerShell to cache.

- **Cache invalidation**: The cache is invalidated when changing directories or when the `.git/HEAD` file content changes (which happens on checkout, commit, rebase, etc).

This approach eliminates file I/O on repeated prompts in the same branch while ensuring correctness when the branch changes.
