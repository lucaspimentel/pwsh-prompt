# TODO

- [ ] Make the prompt customizable (show/hide segments, change order, colors, icons)
  - Current segments: `PathSegment`, `GitSegment`, `HostSegment`, `LastCommandExitCodeSegment`, `LastCommandDurationSegment`, `DateTimeSegment`, `OsSegment`, `PromptSegment`, `NewLineSegment`, `StringSegment`
  - `ISegment` interface (`src/pwsh-prompt/Segments/ISegment.cs`) has `UnformattedLength` and `Append()` — no color/icon properties yet
  - Segment rendering order is hardcoded in `Program.cs` (normal mode vs simple mode)
  - Consider a config file format (JSON/TOML) to define segment list, order, and per-segment options (color, icon, visibility)
  - Need to balance customizability with startup performance (native AOT, minimal allocations)
