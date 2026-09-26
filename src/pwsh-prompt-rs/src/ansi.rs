// Direct ANSI escape sequences, replacing Spectre.Console markup.
//
// The C# implementation renders Spectre markup tags ([aqua], [blue], etc.)
// through AnsiConsole.Markup with a forced TrueColor profile. The escape
// sequences Spectre emits were captured from the published C# binary:
// standard named colors resolve to indexed codes in the 16-color range, while
// arbitrary hex colors like #ff7fff resolve to truecolor. The constants below
// reproduce the exact byte sequences.

/// Markup tag "aqua" (Spectre Color(0, 255, 255)) -> indexed bright cyan.
pub const AQUA: &str = "\x1b[38;5;14m";
/// Markup tag "blue" (Spectre Color(0, 0, 255)) -> indexed bright blue.
pub const BLUE: &str = "\x1b[38;5;12m";
/// Markup tag "red" (Spectre Color(255, 0, 0)) -> indexed bright red.
pub const RED: &str = "\x1b[38;5;9m";
/// Markup tag "yellow" (Spectre Color(255, 255, 0)) -> indexed bright yellow.
pub const YELLOW: &str = "\x1b[38;5;11m";
/// Markup tag "lime" (Spectre Color(0, 255, 0)) -> indexed bright green.
pub const LIME: &str = "\x1b[38;5;10m";
/// Spectre "grey" used by the debug output (Color(128, 128, 128)) -> indexed.
pub const GREY: &str = "\x1b[38;5;8m";
/// Markup tag "#ff7fff" -> truecolor (not a named Spectre color).
pub const MAGENTA_FF7FFF: &str = "\x1b[38;2;255;127;255m";
/// Markup tag "[/]" and AnsiConsole color resets.
pub const RESET: &str = "\x1b[0m";

/// The newline written by Console.WriteLine / AnsiConsole.WriteLine.
pub const fn platform_newline() -> &'static str {
    if cfg!(windows) { "\r\n" } else { "\n" }
}

/// Number of Unicode code points in a string: the display-length measure
/// shared by every segment's UnformattedLength, so astral-plane characters
/// (surrogate pairs) count as one column like any other character.
pub fn char_len(s: &str) -> i32 {
    s.chars().count() as i32
}

/// Writes plain text to stdout without any transformation (the prompt itself;
/// Spectre's Markup call performs no wrapping at the widths used here, since
/// each prompt line is at most half the profile width).
pub fn write_raw(text: &str) {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let _ = out.write_all(text.as_bytes());
    let _ = out.flush();
}

/// A run of text with an optional color style, emitted to the debug output.
#[derive(Clone)]
pub struct DebugSpan {
    pub style: Option<&'static str>,
    pub text: String,
}

impl DebugSpan {
    pub fn plain(text: impl Into<String>) -> Self {
        DebugSpan {
            style: None,
            text: text.into(),
        }
    }

    pub fn styled(style: &'static str, text: impl Into<String>) -> Self {
        DebugSpan {
            style: Some(style),
            text: text.into(),
        }
    }
}

/// Holds whether DEBUG_PROMPT output is enabled. Constructed once in main and
/// threaded through the code that renders debug lines, keeping those functions
/// free of global state. Every method is a no-op when debug output is
/// disabled. Lines are written verbatim: no wrapping, no markup escaping.
pub struct DebugSink {
    enabled: bool,
}

/// Debug output writer: emits each span with its style and terminates the
/// line with the platform newline.
impl DebugSink {
    /// Creates a sink that writes when `enabled` is true.
    pub fn new(enabled: bool) -> Self {
        DebugSink { enabled }
    }

    /// A sink that never writes; used by callers that do not render debug
    /// output (tests, non-debug code paths).
    #[cfg(test)]
    pub fn disabled() -> Self {
        DebugSink { enabled: false }
    }

    /// True when DEBUG_PROMPT output should be rendered.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Writes an unstyled line.
    pub fn plain_line(&self, text: &str) {
        self.write_line(&[DebugSpan::plain(text)]);
    }

    /// Writes a fully yellow line.
    pub fn yellow_line(&self, text: &str) {
        self.write_line(&[DebugSpan::styled(YELLOW, text)]);
    }

    /// Writes a composed line of styled spans.
    pub fn debug_line(&self, spans: &[DebugSpan]) {
        self.write_line(spans);
    }

    fn write_line(&self, spans: &[DebugSpan]) {
        if !self.enabled {
            return;
        }
        let mut out = String::new();
        for span in spans {
            match span.style {
                Some(style) => {
                    out.push_str(style);
                    out.push_str(&span.text);
                    out.push_str(RESET);
                }
                None => out.push_str(&span.text),
            }
        }
        out.push_str(platform_newline());
        write_raw(&out);
    }
}

/// Writes an unstyled line without wrapping; used for plain CLI output
/// (--version, usage) that never goes through the debug profile width.
pub fn write_plain_line(text: &str) {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let _ = out.write_all(text.as_bytes());
    let _ = out.write_all(platform_newline().as_bytes());
    let _ = out.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(spans: &[DebugSpan]) -> String {
        let mut out = String::new();
        for span in spans {
            match span.style {
                Some(style) => {
                    out.push_str(style);
                    out.push_str(&span.text);
                    out.push_str(RESET);
                }
                None => out.push_str(&span.text),
            }
        }
        out
    }

    #[test]
    fn plain_spans_render_verbatim() {
        let spans = [DebugSpan::plain("hello world")];
        assert_eq!(render(&spans), "hello world");
    }

    #[test]
    fn styled_spans_wrap_color_and_reset() {
        let spans = [DebugSpan::styled(YELLOW, "note")];
        assert_eq!(render(&spans), format!("{YELLOW}note{RESET}"));
    }

    #[test]
    fn mixed_spans_keep_each_style() {
        let spans = [
            DebugSpan::styled(YELLOW, "prefix"),
            DebugSpan::plain(": \"text\""),
            DebugSpan::styled(GREY, "[1.00ms]"),
        ];
        assert_eq!(
            render(&spans),
            format!("{YELLOW}prefix{RESET}: \"text\"{GREY}[1.00ms]{RESET}")
        );
    }

    #[test]
    fn char_len_counts_code_points() {
        // The clock glyph U+F0955 is one char occupying one column.
        assert_eq!(char_len(" \u{F0955} "), 3);
        assert_eq!(char_len("abc"), 3);
        assert_eq!(char_len(" \u{276F} "), 3);
    }
}
