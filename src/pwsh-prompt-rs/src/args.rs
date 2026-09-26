// Port of the C# Arguments record. Parses the command line passed after the
// "prompt" verb. Argument names, defaults, and TryParse semantics must match
// the C# implementation exactly.

use std::env;

const TERMINAL_WIDTH_OPTION: &str = "--terminal-width=";
const CURRENT_DIRECTORY_OPTION: &str = "--current-directory=";
const CURRENT_DIRECTORY_IS_FILESYSTEM_OPTION: &str = "--current-directory-is-filesystem=";
const LAST_COMMAND_DURATION_OPTION: &str = "--last-command-duration=";
const LAST_COMMAND_EXIT_CODE_OPTION: &str = "--last-command-exit-code=";
const LAST_COMMAND_STATE_OPTION: &str = "--last-command-state=";
const SIMPLE_MODE_OPTION: &str = "--simple";

#[derive(Debug, PartialEq)]
pub struct Arguments {
    pub terminal_width: i32,
    pub current_directory: String,
    pub current_directory_is_filesystem: bool,
    pub last_command_duration_ms: i32,
    pub last_command_exit_code: i32,
    pub last_command_state: bool,
    pub simple_mode: bool,
}

impl Arguments {
    /// Renders like the C# record's generated ToString, used by the debug
    /// output: "Arguments { TerminalWidth = 120, CurrentDirectory = ..., }".
    pub fn debug_string(&self) -> String {
        format!(
            "Arguments {{ TerminalWidth = {}, CurrentDirectory = {}, CurrentDirectoryIsFileSystem = {}, \
             LastCommandDurationMs = {}, LastCommandExitCode = {}, LastCommandState = {}, SimpleMode = {} }}",
            self.terminal_width,
            self.current_directory,
            bool_text(self.current_directory_is_filesystem),
            self.last_command_duration_ms,
            self.last_command_exit_code,
            bool_text(self.last_command_state),
            bool_text(self.simple_mode),
        )
    }
}

fn bool_text(value: bool) -> &'static str {
    if value { "True" } else { "False" }
}

/// int.TryParse(value, InvariantCulture) semantics: optional sign, ASCII
/// digits, leading/trailing whitespace allowed, i32 overflow fails.
pub fn try_parse_int(value: &str) -> Option<i32> {
    value.trim().parse::<i32>().ok()
}

/// bool.TryParse semantics: exactly "true" or "false", case-insensitive.
pub fn try_parse_bool(value: &str) -> Option<bool> {
    if value.eq_ignore_ascii_case("true") {
        Some(true)
    } else if value.eq_ignore_ascii_case("false") {
        Some(false)
    } else {
        None
    }
}

impl Arguments {
    pub fn parse(args: &[String]) -> Arguments {
        let mut terminal_width = 0;
        let mut current_directory = String::new();
        let mut current_directory_is_filesystem = true;
        let mut last_command_duration_ms = 0;
        let mut last_command_exit_code = 0;
        let mut last_command_state = true;
        let mut simple_mode = false;

        for arg in args {
            if arg == SIMPLE_MODE_OPTION {
                simple_mode = true;
                break;
            } else if let Some(result) = arg
                .strip_prefix(TERMINAL_WIDTH_OPTION)
                .and_then(try_parse_int)
            {
                terminal_width = result;
            } else if let Some(value) = arg.strip_prefix(CURRENT_DIRECTORY_OPTION) {
                current_directory = value.to_string();
            } else if let Some(result) = arg
                .strip_prefix(CURRENT_DIRECTORY_IS_FILESYSTEM_OPTION)
                .and_then(try_parse_bool)
            {
                current_directory_is_filesystem = result;
            } else if let Some(result) = arg
                .strip_prefix(LAST_COMMAND_DURATION_OPTION)
                .and_then(try_parse_int)
            {
                last_command_duration_ms = result;
            } else if let Some(result) = arg
                .strip_prefix(LAST_COMMAND_EXIT_CODE_OPTION)
                .and_then(try_parse_int)
            {
                last_command_exit_code = result;
            } else if let Some(result) = arg
                .strip_prefix(LAST_COMMAND_STATE_OPTION)
                .and_then(try_parse_bool)
            {
                last_command_state = result;
            }
        }
        if terminal_width == 0 {
            // Console.WindowWidth in C#, which throws when the output is
            // redirected, falling back to 80. terminal_size returns None in
            // the same redirected case.
            terminal_width = console_window_width();
        }

        if current_directory.is_empty() {
            current_directory = env::current_dir()
                .map(|d| d.to_string_lossy().into_owned())
                .unwrap_or_default();
        }

        Arguments {
            terminal_width,
            current_directory,
            current_directory_is_filesystem,
            last_command_duration_ms,
            last_command_exit_code,
            last_command_state,
            simple_mode,
        }
    }
}

/// Console.WindowWidth equivalent: the attached console's window width, or 80
/// when unavailable (redirected output).
fn console_window_width() -> i32 {
    terminal_size::terminal_size()
        .map(|(width, _)| i32::from(width.0))
        .unwrap_or(80)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn empty_args_use_fallbacks() {
        let a = Arguments::parse(&args(&[]));
        assert_eq!(a.terminal_width, console_window_width());
        assert!(a.current_directory_is_filesystem);
        assert_eq!(a.last_command_duration_ms, 0);
        assert_eq!(a.last_command_exit_code, 0);
        assert!(a.last_command_state);
        assert!(!a.simple_mode);
        assert_eq!(
            a.current_directory,
            env::current_dir().unwrap().to_string_lossy().into_owned()
        );
    }

    #[test]
    fn parses_all_options() {
        let a = Arguments::parse(&args(&[
            "--terminal-width=120",
            "--current-directory=C:\\repo",
            "--current-directory-is-filesystem=false",
            "--last-command-duration=4500",
            "--last-command-exit-code=130",
            "--last-command-state=false",
        ]));
        assert_eq!(a.terminal_width, 120);
        assert_eq!(a.current_directory, "C:\\repo");
        assert!(!a.current_directory_is_filesystem);
        assert_eq!(a.last_command_duration_ms, 4500);
        assert_eq!(a.last_command_exit_code, 130);
        assert!(!a.last_command_state);
        assert!(!a.simple_mode);
    }

    #[test]
    fn simple_breaks_parsing_early() {
        let a = Arguments::parse(&args(&["--simple", "--terminal-width=50"]));
        assert!(a.simple_mode);
        // Width was not parsed because --simple breaks the loop first.
        assert_eq!(a.terminal_width, console_window_width());
    }

    #[test]
    fn invalid_values_keep_defaults() {
        let a = Arguments::parse(&args(&[
            "--terminal-width=abc",
            "--current-directory-is-filesystem=yes",
            "--last-command-duration=1x",
            "--last-command-exit-code=99999999999999",
            "--last-command-state=0",
        ]));
        assert_eq!(a.terminal_width, console_window_width());
        assert!(a.current_directory_is_filesystem);
        assert_eq!(a.last_command_duration_ms, 0);
        assert_eq!(a.last_command_exit_code, 0);
        assert!(a.last_command_state);
    }

    #[test]
    fn int_parse_matches_invariant_try_parse() {
        assert_eq!(try_parse_int(" 123 "), Some(123));
        assert_eq!(try_parse_int("+42"), Some(42));
        assert_eq!(try_parse_int("-7"), Some(-7));
        assert_eq!(try_parse_int("007"), Some(7));
        assert_eq!(try_parse_int("1,234"), None);
        assert_eq!(try_parse_int("2147483648"), None);
        assert_eq!(try_parse_int(""), None);
    }

    #[test]
    fn bool_parse_matches_try_parse() {
        assert_eq!(try_parse_bool("true"), Some(true));
        assert_eq!(try_parse_bool("TRUE"), Some(true));
        assert_eq!(try_parse_bool("False"), Some(false));
        assert_eq!(try_parse_bool("1"), None);
        assert_eq!(try_parse_bool("yes"), None);
    }

    #[test]
    fn empty_current_directory_falls_back() {
        let a = Arguments::parse(&args(&["--current-directory="]));
        assert_eq!(
            a.current_directory,
            env::current_dir().unwrap().to_string_lossy().into_owned()
        );
    }

    #[test]
    fn debug_string_matches_record_to_string() {
        let a = Arguments::parse(&args(&[
            "--terminal-width=120",
            "--current-directory=C:\\repo",
            "--last-command-duration=4500",
            "--last-command-exit-code=130",
            "--last-command-state=false",
        ]));
        assert_eq!(
            a.debug_string(),
            "Arguments { TerminalWidth = 120, CurrentDirectory = C:\\repo, CurrentDirectoryIsFileSystem = True, \
             LastCommandDurationMs = 4500, LastCommandExitCode = 130, LastCommandState = False, SimpleMode = False }"
        );
    }
}
