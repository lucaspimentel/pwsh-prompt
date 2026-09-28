// Entry point. Modes (matching the C# Program.Main switch):
//   --version          prints the version
//   init               prints the PowerShell init script
//   prompt [args...]   renders the prompt
//   (no arguments)     prints usage
//   anything else      prints usage (both implementations treat unknown
//                      verbs like empty input)

mod ansi;
mod args;
mod env;
mod git_info;
mod init;
mod path_utils;
mod segments;
mod settings;

use ansi::{DebugSink, DebugSpan};
use env::RealEnv;
use segments::{Segment, StringSegment};

fn version_string() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn main() {
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|a| a.to_string_lossy().into_owned())
        .collect();

    match args.as_slice() {
        [only] if only == "--version" => {
            ansi::write_plain_line(&version_string());
        }
        [only] if only == "init" => {
            ansi::write_raw(&init::render());
        }
        [verb, rest @ ..] if verb == "prompt" => {
            run_prompt(rest);
        }
        [] => {
            ansi::write_plain_line("Usage: Prompt init");
            ansi::write_plain_line("       Prompt prompt [arguments]");
            ansi::write_plain_line("       Prompt --version");
        }
        _ => {
            // Unknown verb: print usage like empty input instead of
            // silently doing nothing.
            ansi::write_plain_line("Usage: Prompt init");
            ansi::write_plain_line("       Prompt prompt [arguments]");
            ansi::write_plain_line("       Prompt --version");
        }
    }
}

fn run_prompt(args: &[String]) {
    let state = args::Arguments::parse(args);
    let sink = ansi::DebugSink::new(settings::debug());
    let env = RealEnv;
    let debug = sink.enabled();

    // Per-segment construction timings, shown when DEBUG_PROMPT=1. Matches
    // the C# SegmentTimer dictionary (keyed by type name).
    let mut timings: Vec<(&'static str, f64)> = Vec::new();

    if debug {
        sink.plain_line(&format!("Literal arguments: \"prompt {}\"", args.join(" ")));
        sink.plain_line(&format!("Parsed arguments: {}", state.debug_string()));
        let current_directory = std::env::current_dir()
            .map(|d| d.to_string_lossy().into_owned())
            .unwrap_or_default();
        sink.plain_line(&format!("Current directory: {current_directory}"));
    }

    let mut measure = |name: &'static str, start: std::time::Instant| {
        if debug {
            timings.push((name, start.elapsed().as_secs_f64() * 1000.0));
        }
    };

    let segments: Vec<Box<dyn Segment>> = if state.simple_mode {
        let start = std::time::Instant::now();
        let host_segment = segments::host::HostSegment::new();
        measure("HostSegment", start);

        let start = std::time::Instant::now();
        let git_segment = segments::git::GitSegment::new(&state.current_directory, &env, &sink);
        measure("GitSegment", start);

        let max_path_length = state.terminal_width
            - host_segment.unformatted_length()
            - git_segment.unformatted_length()
            - 1;

        let start = std::time::Instant::now();
        let path_segment = segments::path::PathSegment::new(
            &state.current_directory,
            state.current_directory_is_filesystem,
            max_path_length,
            true,
            &env,
            &sink,
        );
        measure("PathSegment", start);

        vec![
            Box::new(path_segment),
            Box::new(git_segment),
            Box::new(host_segment),
        ]
    } else {
        let start = std::time::Instant::now();
        let host_segment = segments::host::HostSegment::new();
        measure("HostSegment", start);

        let start = std::time::Instant::now();
        let git_segment = segments::git::GitSegment::new(&state.current_directory, &env, &sink);
        measure("GitSegment", start);

        let start = std::time::Instant::now();
        let last_command_exit_code_segment =
            segments::last_command_exit_code::LastCommandExitCodeSegment::new(
                state.last_command_exit_code,
                state.last_command_state,
            );
        measure("LastCommandExitCodeSegment", start);

        let start = std::time::Instant::now();
        let last_command_duration_segment =
            segments::last_command_duration::LastCommandDurationSegment::new(
                state.last_command_duration_ms,
                settings::LAST_COMMAND_DURATION_THRESHOLD_MS,
            );
        measure("LastCommandDurationSegment", start);

        let start = std::time::Instant::now();
        let date_time_segment = segments::date_time::DateTimeSegment::new();
        measure("DateTimeSegment", start);

        let start = std::time::Instant::now();
        let os_segment = segments::os::OsSegment::new();
        measure("OsSegment", start);

        let shell_segment = StringSegment::new(" pwsh");

        let start = std::time::Instant::now();
        let prompt_segment = segments::prompt::PromptSegment::new(settings::PROMPT);
        measure("PromptSegment", start);

        let max_path_length = state.terminal_width
            - host_segment.unformatted_length()
            - git_segment.unformatted_length()
            - last_command_exit_code_segment.unformatted_length()
            - last_command_duration_segment.unformatted_length()
            - date_time_segment.unformatted_length()
            - 3;

        let start = std::time::Instant::now();
        let path_segment = segments::path::PathSegment::new(
            &state.current_directory,
            state.current_directory_is_filesystem,
            max_path_length,
            false,
            &env,
            &sink,
        );
        measure("PathSegment", start);

        let filler_width = state.terminal_width
            - host_segment.unformatted_length()
            - path_segment.unformatted_length()
            - git_segment.unformatted_length()
            - last_command_exit_code_segment.unformatted_length()
            - last_command_duration_segment.unformatted_length()
            - date_time_segment.unformatted_length()
            - 2;

        let filler_segment = StringSegment::new(if filler_width <= 0 {
            String::new()
        } else {
            " ".repeat(usize::try_from(filler_width).unwrap_or(0))
        });

        vec![
            Box::new(segments::new_line::NewLineSegment),
            Box::new(path_segment),
            Box::new(git_segment),
            Box::new(host_segment),
            Box::new(filler_segment),
            Box::new(last_command_exit_code_segment),
            Box::new(last_command_duration_segment),
            Box::new(date_time_segment),
            Box::new(segments::new_line::NewLineSegment),
            Box::new(os_segment),
            Box::new(shell_segment),
            Box::new(prompt_segment),
        ]
    };

    let mut prompt = String::new();
    combine_segments(
        &segments,
        state.terminal_width,
        &mut prompt,
        &timings,
        &sink,
    );
    // AnsiConsole.Markup writes the prompt without a trailing newline.
    ansi::write_raw(&prompt);
}

/// Program.CombineSegments equivalent: iterates the segments, resetting the
/// remaining width on newline segments; a segment longer than the remaining
/// width marks the line full, skipping all subsequent segments on that line
/// until the next newline.
fn combine_segments(
    segments: &[Box<dyn Segment>],
    width: i32,
    out: &mut String,
    timings: &[(&'static str, f64)],
    sink: &DebugSink,
) {
    if sink.enabled() {
        sink.debug_line(&[DebugSpan::plain("")]);

        for segment in segments {
            let type_name = segment.name();
            let mut spans = vec![DebugSpan::styled(
                ansi::YELLOW,
                format!("{} ({})", type_name, segment.unformatted_length()),
            )];

            if let Some((_, ms)) = timings.iter().find(|(name, _)| *name == type_name) {
                // The C# timing string has a leading space before the grey tag.
                spans.push(DebugSpan::plain(" "));
                spans.push(DebugSpan::styled(ansi::GREY, format!("[{ms:.2}ms]")));
            }

            if !segment.is_newline() {
                spans.push(DebugSpan::plain(format!(
                    ": \"{}\"",
                    segment.display_text()
                )));
            }

            sink.debug_line(&spans);
        }
    }

    let mut remaining_width = width;
    let mut line_full = false;

    for segment in segments {
        if segment.is_newline() {
            // reset for next line
            remaining_width = width;
            line_full = false;
        }

        if segment.unformatted_length() > remaining_width {
            line_full = true;
        }

        if !line_full {
            segment.append(out);
            remaining_width -= segment.unformatted_length();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::EnvSource;
    use segments::date_time::DateTimeSegment;
    use segments::host::HostSegment;
    use segments::new_line::NewLineSegment;
    use segments::os::OsSegment;
    use segments::path::PathSegment;
    use segments::prompt::PromptSegment;
    use segments::{
        last_command_duration::LastCommandDurationSegment,
        last_command_exit_code::LastCommandExitCodeSegment,
    };

    fn combine(segments: Vec<Box<dyn Segment>>, width: i32) -> String {
        let sink = DebugSink::disabled();
        let mut out = String::new();
        combine_segments(&segments, width, &mut out, &[], &sink);
        out
    }

    /// Builds the normal-mode segment list with fixed inputs; the datetime
    /// segment is constructed from a fixed time so the output is
    /// deterministic.
    fn fixed_normal_segments(
        current_directory: &str,
        terminal_width: i32,
        is_file_system: bool,
        env: &dyn EnvSource,
    ) -> Vec<Box<dyn Segment>> {
        let host_segment = HostSegment::new();
        let sink = DebugSink::disabled();
        let git_segment = segments::git::GitSegment::new(current_directory, env, &sink);
        let last_command_exit_code_segment = LastCommandExitCodeSegment::new(0, true);
        let last_command_duration_segment =
            LastCommandDurationSegment::new(0, settings::LAST_COMMAND_DURATION_THRESHOLD_MS);
        let date_time_segment = DateTimeSegment::new();
        let os_segment = OsSegment::new();
        let shell_segment = StringSegment::new(" pwsh");
        let prompt_segment = PromptSegment::new(settings::PROMPT);

        let max_path_length = terminal_width
            - host_segment.unformatted_length()
            - git_segment.unformatted_length()
            - last_command_exit_code_segment.unformatted_length()
            - last_command_duration_segment.unformatted_length()
            - date_time_segment.unformatted_length()
            - 3;

        let path_segment = PathSegment::new(
            current_directory,
            is_file_system,
            max_path_length,
            false,
            env,
            &sink,
        );

        let filler_width = terminal_width
            - host_segment.unformatted_length()
            - path_segment.unformatted_length()
            - git_segment.unformatted_length()
            - last_command_exit_code_segment.unformatted_length()
            - last_command_duration_segment.unformatted_length()
            - date_time_segment.unformatted_length()
            - 2;

        let filler_segment = StringSegment::new(if filler_width <= 0 {
            String::new()
        } else {
            " ".repeat(usize::try_from(filler_width).unwrap_or(0))
        });

        vec![
            Box::new(NewLineSegment),
            Box::new(path_segment),
            Box::new(git_segment),
            Box::new(host_segment),
            Box::new(filler_segment),
            Box::new(last_command_exit_code_segment),
            Box::new(last_command_duration_segment),
            Box::new(date_time_segment),
            Box::new(NewLineSegment),
            Box::new(os_segment),
            Box::new(shell_segment),
            Box::new(prompt_segment),
        ]
    }

    #[test]
    fn combine_skips_overflow_until_newline() {
        // Two lines; the second line's first segment is longer than the
        // width, so everything until the next newline is skipped.
        let wide = StringSegment::new("x".repeat(20));
        let skipped = StringSegment::new("skipped");
        let after = StringSegment::new("after");
        let out = combine(
            vec![
                Box::new(NewLineSegment),
                Box::new(wide),
                Box::new(skipped),
                Box::new(NewLineSegment),
                Box::new(after),
            ],
            10,
        );
        // The wide segment marks the line full and is skipped; the same
        // applies to every segment after it on that line.
        assert_eq!(out, format!("{nl}{nl}after", nl = ansi::platform_newline()));
    }

    #[test]
    fn combine_resets_on_newline() {
        let a = StringSegment::new("aaaa");
        let b = StringSegment::new("bbbb");
        let out = combine(vec![Box::new(a), Box::new(NewLineSegment), Box::new(b)], 5);
        assert_eq!(out, format!("aaaa{nl}bbbb", nl = ansi::platform_newline()));
    }

    #[test]
    fn combine_normal_layout_shape() {
        // Non-repo directory outside the user's home so the path renders
        // verbatim.
        let dir = std::env::temp_dir().join("pwsh-prompt-tests-combine");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let dir_string = dir.to_string_lossy().into_owned();

        let env = crate::env::FakeEnv::empty();
        let segments = fixed_normal_segments(&dir_string, 120, true, &env);
        let out = combine(segments, 120);

        // Two lines separated by the platform newline; the list starts with a
        // newline so the first split piece is empty.
        let parts: Vec<&str> = out.split(ansi::platform_newline()).collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "");
        assert!(parts[1].starts_with(ansi::AQUA));
        // Line 1 ends with the datetime segment (no trailing color reset).
        assert!(parts[1].ends_with("AM ") || parts[1].ends_with("PM "));
        assert!(parts[2].contains(" pwsh"));
        assert!(parts[2].ends_with(&format!("{} \u{276F} {}", ansi::LIME, ansi::RESET)));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn version_string_is_plain_semver() {
        assert_eq!(version_string(), env!("CARGO_PKG_VERSION"));
    }
}
