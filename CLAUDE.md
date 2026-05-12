# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a custom PowerShell prompt written in C# that displays contextual information about the current directory, git repository, last command status, and system information. It's compiled as a native AOT binary for fast startup times.

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

- **Program.cs**: Entry point that routes between init/prompt modes and orchestrates segment rendering. Contains conditional logic for simple vs normal mode rendering.
- **Arguments.cs**: Parses command-line arguments passed from PowerShell. When `--simple` is detected, parsing breaks early to skip unnecessary parameters.
- **Init.cs**: Generates PowerShell script that installs the prompt function. Handles git info caching via environment variables and emits shell integration escape sequences (OSC 9;9, 133;A/B/C/D).
- **GitInfo.cs**: Traverses directory tree to find .git folder and parse HEAD/config files to determine current branch. Supports PR number display via `gh` CLI.
- **ValueStringBuilder.cs**: High-performance string building using stack-allocated buffers.
- **Settings.cs**: Defines compile-time settings and constants.
- **SegmentUtils.cs**: Shared segment rendering utilities.

### Segment System

All visual elements implement `ISegment` interface (in `src/pwsh-prompt/Segments/`):
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

- **PowerShell side** (src/pwsh-prompt/Init.cs): Discovers the git directory by walking up the directory tree (on directory change only). After the walk, compares the newly-discovered git dir against the previous one (`$gitDirChanged`). On every prompt, reads `.git/HEAD` once and compares to the stored content. If either the git dir changed or HEAD changed, the cached env vars are refreshed. Before invoking C#, passes cached values via `PROMPT_GIT_DIR_CACHED`, `PROMPT_GIT_BRANCH_CACHED`, `PROMPT_PR_NUMBER_CACHED`, and `PROMPT_PR_STATE_CACHED`.

- **C# side** (src/pwsh-prompt/GitInfo.cs, src/pwsh-prompt/Segments/GitSegment.cs): `TryFindGitFolder()`, `GetBranchName()`, and `GitSegment` check the `*_CACHED` environment variables first. If found, they skip all file I/O and subprocess calls.

- **Cache invalidation**: The cache is invalidated when the discovered `.git` directory changes (moving between repos, or in/out of a repo) or when the `.git/HEAD` file content changes (checkout, commit, rebase, etc). Moving between subdirectories of the same repo does **not** invalidate the cache.

- **Per-branch PR cache** (`$script:prCache` in the `lucas-prompt` dynamic module): a session-scoped hashtable keyed by branch name (or commit hash for detached HEAD) holds previously-fetched PR info. When the git dir / HEAD change triggers a PR refresh, the cache is consulted first; only a miss schedules a background `gh pr view` fetch (see next bullet). Both "has PR" and "no PR" results are cached so toggling between previously-visited branches always skips `gh`. Lives for the PowerShell session; no persistence and no TTL.

- **Async PR fetch** (`$script:prJob` + `Receive-PendingPrJob` in the same module): on a cache miss, the prompt spawns a single-slot background job (preferring `Start-ThreadJob`; falling back to `Start-Job`) that runs `gh pr view` and returns immediately. The job is tagged with the branch's cache key via an `Add-Member -NotePropertyName CacheKey`. The *next* prompt invocation calls `Receive-PendingPrJob` at the top: if completed, the result is written to `$script:prCache` and — if the user is still on the same branch — seeded directly into `$env:PROMPT_PR_NUMBER` / `$env:PROMPT_PR_STATE` so the downstream env-propagation path picks it up on the same prompt. If the user switches to a different branch before the job lands, the prompt cancels (`Stop-Job` / `Remove-Job -Force`) the stale job and spawns a fresh one for the new branch. UX consequence: on the first visit to an unseen branch in a session, the prompt renders without PR info; PR info appears on the next prompt (after any keypress / command).

- **Note**: Child process environment changes do not propagate back to the parent PowerShell process, so git metadata is discovered in PowerShell directly rather than relying on C# to write back `*_OUT` env vars. The per-branch PR cache and in-flight job slot live in `$script:`-scoped variables on the dynamic module for the same reason.

## Releases

The project uses GitHub Actions to automatically create releases when a git tag is pushed.

### Creating a Release

Use the `/ship` skill with the target version: `/ship 1.0.0`

This runs the full release flow (version bump, changelog, commit, tag, push, watch CI, update release notes) using `.ship.yml` config.

Alternatively, manually:

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
