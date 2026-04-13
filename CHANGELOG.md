# Changelog

## [0.4.1] - 2026-04-13

### Changed
- Simplify PR prefix in git segment display

## [0.4.0] - 2026-04-10

### Added
- Add Windows Terminal shell integration escape sequences (OSC 9;9, 133;A/B/C/D) for new-tab-same-directory, scrollbar marks, command navigation, and output selection

## [0.3.1] - 2026-04-08

### Added
- Add .ship.yml release config

### Fixed
- Fix padding regression in DateTimeSegment
- Fix PR number not updating on branch switch

## [0.3.0] - 2026-04-07

### Added
- Add `--version` CLI parameter
- Add PR number to prompt via `gh` CLI
- Add checksum verification to install-remote.ps1

### Changed
- Simplify segment prefixes and formatting
- Update nuget references
- Fix namespace name

## [0.2.2-beta] - 2026-03-15

### Changed
- Update GitHub Actions to latest versions and pin to commit SHAs for supply-chain security
- Refactor install-local.ps1 and add Directory.Build.props
- Tune AOT optimization settings

### Added
- Add Scoop installation instructions to README
- Add CI and Release workflow badges to README
- Add -Force and -Update params to install-local.ps1
- Add .claude local files to .gitignore
- Add CHANGELOG.md
- Add TODO.md

### Fixed
- Fix docs: Modules/ → Segments/, remove stale line numbers

## [0.2.1-beta] - 2026-01-06

### Fixed
- Fix date format: use MM for month instead of mm for minutes
- Use -NoProfile to run ps1 scripts
- Make ps1 scripts executable

### Changed
- Bump Microsoft.Extensions.Primitives from 10.0.0 to 10.0.1

## [0.2.0-beta] - 2025-11-24

### Changed
- Rename Modules/ to Segments/ for clarity
- Fix root namespace

### Added
- Add multi-platform CI testing for Windows and Linux

## [0.1.0] - 2025-11-24

### Added
- Custom PowerShell prompt with ANSI-formatted segments
- Segments: path, git branch, host, last command exit code, last command duration, datetime, OS, prompt indicator
- Native AOT compilation for fast startup
- Git integration with branch detection for regular repos and worktrees
- Environment variable caching for git info to minimize file I/O
- Simple mode (--simple) for minimal single-line prompt
- Path truncation and path mappings support
- Nerd Font icons
- Automated release workflow with Windows and Linux binaries
- Local and remote installation scripts
- MIT License
