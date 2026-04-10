# TODO

- [ ] Make the prompt customizable (show/hide segments, change order, colors, icons)
  - Current segments: `PathSegment`, `GitSegment`, `HostSegment`, `LastCommandExitCodeSegment`, `LastCommandDurationSegment`, `DateTimeSegment`, `OsSegment`, `PromptSegment`, `NewLineSegment`, `StringSegment`
  - `ISegment` interface (`src/pwsh-prompt/Segments/ISegment.cs`) has `UnformattedLength` and `Append()` — no color/icon properties yet
  - Segment rendering order is hardcoded in `Program.cs` (normal mode vs simple mode)
  - Consider a config file format (JSON/TOML) to define segment list, order, and per-segment options (color, icon, visibility)
  - Need to balance customizability with startup performance (native AOT, minimal allocations)
- [x] Add Windows Terminal shell integration escape sequences in the PowerShell wrapper (`Init.cs`-generated script)
  - OSC 9;9 — emit CWD so new tabs/panes open in the same directory
  - OSC 133;A — mark prompt start (enables scrollbar marks, jump-between-commands, select-command-output)
  - OSC 133;B — mark command input start (end of prompt)
  - OSC 133;D — mark previous command finished with exit code (must fire before C# binary invocation)
  - These are protocol-level markers, not visual — they belong in the shell wrapper, not the C# rendering
  - Ref: [Shell Integration](https://learn.microsoft.com/en-us/windows/terminal/tutorials/shell-integration), [New Tab Same Directory](https://learn.microsoft.com/en-us/windows/terminal/tutorials/new-tab-same-directory)
