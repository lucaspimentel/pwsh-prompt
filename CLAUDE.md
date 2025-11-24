# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a custom PowerShell prompt written in C# that displays contextual information about the current directory, git repository, last command status, and system information. It's compiled as a native AOT binary for fast startup times.

## Project Structure

```
pwsh-prompt/
├── .github/
│   └── workflows/
│       ├── dotnet.yml        # CI workflow (build and test)
│       └── release.yml       # Release workflow (builds binaries on git tag)
├── src/
│   └── pwsh-prompt/          # Main C# project
│       ├── Modules/          # Segment implementations (ISegment)
│       ├── Program.cs        # Entry point and mode routing
│       ├── Arguments.cs      # Command-line argument parsing
│       ├── GitInfo.cs        # Git repository detection and caching
│       ├── Init.cs           # PowerShell initialization script generator
│       ├── ValueStringBuilder.cs  # High-performance string builder
│       └── pwsh-prompt.csproj
├── install-local.ps1         # Build from source and install locally
├── install-remote.ps1        # Download from GitHub releases and install
├── pwsh-prompt.slnx          # Solution file
├── CLAUDE.md                 # This file
└── README.md                 # User documentation
```

## Build and Installation

- **Restore dependencies**: `dotnet restore src/pwsh-prompt`
- **Build**: `dotnet build src/pwsh-prompt`
- **Quick build test**: `dotnet build src/pwsh-prompt -c Release -f net10.0` (verifies code compiles)
- **Publish (Windows)**: `dotnet publish src/pwsh-prompt -c release -r win-x64 --output ./publish`
- **Publish (Linux)**: `dotnet publish src/pwsh-prompt -c release -r linux-x64 --output ./publish`
- **Install**: `./install-local.ps1` - builds and installs to `~/.local/bin/pwsh-prompt`

The `install-local.ps1` script handles platform detection, building, and installation automatically.

## Architecture

The application has two modes:

1. **init mode**: Generates PowerShell initialization code that sets up the prompt function. This is invoked with `pwsh-prompt init` and the output is evaluated by PowerShell to install the prompt.

2. **prompt mode**: Invoked by PowerShell on each prompt render. Receives arguments about terminal state (width, current directory, last command info) and outputs ANSI-formatted prompt text using Spectre.Console markup.
   - **Normal mode**: `pwsh-prompt prompt [arguments]` - renders full prompt with all segments
   - **Simple mode**: `pwsh-prompt prompt --simple` - renders minimal prompt with only pathSegment, gitSegment, and hostSegment

### Core Components

- **src/pwsh-prompt/Program.cs**: Entry point that routes between init/prompt modes and orchestrates segment rendering. Contains conditional logic for simple vs normal mode rendering (lines 59-136).
- **src/pwsh-prompt/Arguments.cs**: Parses command-line arguments passed from PowerShell. When `--simple` is detected, parsing breaks early to skip unnecessary parameters (lines 36-39).
- **src/pwsh-prompt/Init.cs**: Generates PowerShell script that installs the prompt function
- **src/pwsh-prompt/GitInfo.cs**: Traverses directory tree to find .git folder and parse HEAD/config files to determine current branch
- **src/pwsh-prompt/ValueStringBuilder**: High-performance string building using stack-allocated buffers
- **install-local.ps1**: Cross-platform installation script that builds native binary and installs to `~/.local/bin`

### Segment System

All visual elements implement `ISegment` interface (in `src/pwsh-prompt/Modules/`):
- Each segment calculates its own `UnformattedLength` (for layout) and formats itself via `Append(ref ValueStringBuilder)`
- Segments use Spectre.Console markup syntax for colors (e.g., `[aqua]text[/]`)
- Main segments: `PathSegment`, `GitSegment`, `HostSegment`, `LastCommandExitCodeSegment`, `LastCommandDurationSegment`, `DateTimeSegment`, `OsSegment`, `PromptSegment`

The prompt rendering varies by mode:
- **Normal mode**: Two lines with automatic width calculation and filler space to right-align certain segments (exit code, duration, datetime). Includes OS and shell information on the second line.
- **Simple mode**: Single line with only path, git, and host segments. No filler, no second line, optimized for minimal visual footprint.

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
- **Simple mode**: Skips parsing unnecessary command-line parameters and creating unused segments for even faster execution

### Environment Variable Caching

**Important**: The C# process starts fresh on every prompt invocation, so static caching doesn't work. Instead, caching happens via PowerShell environment variables:

- **PowerShell side** (src/pwsh-prompt/Init.cs): Before invoking the C# binary, checks if still in same directory and if `.git/HEAD` file is unchanged. If so, passes cached git directory and branch name via `PROMPT_GIT_DIR_CACHED` and `PROMPT_GIT_BRANCH_CACHED` environment variables. After invocation, reads `PROMPT_GIT_DIR_OUT` and `PROMPT_GIT_BRANCH_OUT` from the C# process and stores them along with the HEAD file content for next prompt.

- **C# side** (src/pwsh-prompt/GitInfo.cs): `TryFindGitFolder()` and `GetBranchName()` check the `*_CACHED` environment variables first. If found, they skip all file I/O. After computing fresh values, they write to `*_OUT` environment variables for PowerShell to cache.

- **Cache invalidation**: The cache is invalidated when changing directories or when the `.git/HEAD` file content changes (which happens on checkout, commit, rebase, etc).

This approach eliminates file I/O on repeated prompts in the same branch while ensuring correctness when the branch changes.

## Releases

The project uses GitHub Actions to automatically create releases when a git tag is pushed.

### Creating a Release

1. **Tag the commit**: `git tag v1.0.0` (use semantic versioning)
2. **Push the tag**: `git push origin v1.0.0`
3. **GitHub Actions**: The release workflow (`.github/workflows/release.yml`) will automatically:
   - Build native binaries for Windows (win-x64) and Linux (linux-x64)
   - Create release archives (`.zip` for Windows, `.tar.gz` for Linux)
   - Generate SHA256 checksums
   - Create a GitHub release with the binaries and checksums attached
   - Auto-generate release notes from commits

### Installation Scripts

- **install-remote.ps1**: Downloads pre-built binaries from GitHub releases (no build tools required)
- **install-local.ps1**: Builds from source and installs locally
