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

/// UTF-16 code unit count of a string, matching C# string.Length semantics
/// used by every segment's UnformattedLength.
pub fn utf16_len(s: &str) -> i32 {
    s.chars().map(char::len_utf16).sum::<usize>() as i32
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

/// A run of text with an optional color style, as the C# debug output renders
/// through AnsiConsole.Write / MarkupInterpolated.
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

/// A visible character with its style, used by the debug line wrapper.
#[derive(Clone)]
struct Char {
    c: char,
    style: Option<&'static str>,
}

/// Holds the DEBUG_PROMPT output settings: whether debug output is enabled and
/// the wrap width (the AnsiConsole profile width, terminalWidth * 2). It is
/// constructed once in main and threaded through the code that renders debug
/// lines, keeping those functions free of global state. Every method is a
/// no-op when debug output is disabled.
pub struct DebugSink {
    enabled: bool,
    width: i32,
}

/// AnsiConsole.WriteLine for a composed line of styled spans: word-wraps the
/// visible text at the profile width (terminalWidth * 2), re-emitting each
/// style at the start of every wrapped line and resetting at its end.
impl DebugSink {
    /// Creates a sink that writes when `enabled` is true, wrapping lines at
    /// `width` columns.
    pub fn new(enabled: bool, width: i32) -> Self {
        DebugSink { enabled, width }
    }

    /// A sink that never writes; used by callers that do not render debug
    /// output (tests, non-debug code paths).
    #[cfg(test)]
    pub fn disabled() -> Self {
        DebugSink {
            enabled: false,
            width: 0,
        }
    }

    /// True when DEBUG_PROMPT output should be rendered.
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Writes an unstyled line (AnsiConsole.WriteLine with plain text).
    pub fn plain_line(&self, text: &str) {
        self.write_line(&[DebugSpan::plain(text)]);
    }

    /// Writes a fully yellow line (AnsiConsole.MarkupLineInterpolated with a
    /// single [yellow]...[/] span).
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
        use std::io::Write;
        let stdout = std::io::stdout();
        let mut out = stdout.lock();

        for line in wrap_spans(spans, self.width) {
            for span in line {
                match span.style {
                    Some(style) => {
                        let _ = write!(out, "{style}{}{RESET}", span.text);
                    }
                    None => {
                        let _ = write!(out, "{}", span.text);
                    }
                }
            }
            let _ = write!(out, "{}", platform_newline());
        }
        let _ = out.flush();
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

/// Splits the spans into wrapped lines. The visible text is split into word
/// tokens, each carrying its trailing separator space; the line breaks before
/// a token that would overflow the width. A token longer than the width (only
/// reachable in DEBUG output with extremely long paths) is hard-split into
/// width-sized chunks that fill their own lines, with subsequent words
/// continuing after the final partial chunk.
fn wrap_spans(spans: &[DebugSpan], width: i32) -> Vec<Vec<DebugSpan>> {
    if width <= 0 {
        return vec![spans.to_vec()];
    }

    // Flatten to word tokens, each carrying its trailing separator space (the
    // last word has none). Spaces are standalone tokens when doubled, and
    // carry the style of the span they belong to (the C# output keeps the
    // trailing space inside the color range).
    let mut words: Vec<Vec<Char>> = Vec::new();
    let mut current: Vec<Char> = Vec::new();
    for span in spans {
        for c in span.text.chars() {
            if c == ' ' {
                current.push(Char {
                    c: ' ',
                    style: span.style,
                });
                words.push(std::mem::take(&mut current));
            } else {
                current.push(Char {
                    c,
                    style: span.style,
                });
            }
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    wrap_words(words, width)
}

fn wrap_words(words: Vec<Vec<Char>>, width: i32) -> Vec<Vec<DebugSpan>> {
    let mut lines: Vec<Vec<DebugSpan>> = Vec::new();
    let mut line_words: Vec<Vec<Char>> = Vec::new();
    let mut line_len: usize = 0;

    let width_usize = width as usize;

    for word in words {
        let word_len = word.len();

        if !line_words.is_empty() && line_len + word_len > width_usize {
            lines.push(join_words(&line_words));
            line_words.clear();
            line_len = 0;
        }

        if line_words.is_empty() && word_len > width_usize {
            // Hard-split into width-sized chunks; full chunks fill their own
            // line, and the final partial chunk stays open for subsequent
            // words.
            let mut start = 0;
            while start < word.len() {
                let end = (start + width_usize).min(word.len());
                let chunk = word[start..end].to_vec();
                if end == word.len() {
                    line_words.push(chunk);
                    line_len = end - start;
                } else {
                    lines.push(join_words(&[chunk]));
                }
                start = end;
            }
        } else {
            line_words.push(word);
            line_len += word_len;
        }
    }

    if !line_words.is_empty() {
        lines.push(join_words(&line_words));
    }
    if lines.is_empty() {
        lines.push(Vec::new());
    }
    lines
}

fn join_words(words: &[Vec<Char>]) -> Vec<DebugSpan> {
    // words are (char, style) runs; group consecutive same-style runs.
    let mut spans: Vec<DebugSpan> = Vec::new();
    for word in words {
        for ch in word {
            let (c, style) = (ch.c, ch.style);
            match spans.last_mut() {
                Some(span) if span.style == style => span.text.push(c),
                _ => spans.push(match style {
                    Some(style) => DebugSpan::styled(style, c.to_string()),
                    None => DebugSpan::plain(c.to_string()),
                }),
            }
        }
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(spans: &[DebugSpan], width: i32) -> String {
        let mut out = String::new();
        for line in wrap_spans(spans, width) {
            for span in line {
                match span.style {
                    Some(style) => {
                        out.push_str(style);
                        out.push_str(&span.text);
                        out.push_str(RESET);
                    }
                    None => out.push_str(&span.text),
                }
            }
            out.push('|');
        }
        out
    }

    #[test]
    fn short_line_not_wrapped() {
        let spans = [DebugSpan::plain("hello world")];
        assert_eq!(render(&spans, 40), "hello world|");
    }

    #[test]
    fn wraps_before_word_exceeding_width_keeping_trailing_space() {
        // 10 + 1 + 10 = 21 > 20 -> the second word moves to the next line;
        // the separator space stays at the end of the first line.
        let spans = [DebugSpan::plain("aaaaaaaaaa bbbbbbbbbb")];
        assert_eq!(render(&spans, 20), "aaaaaaaaaa |bbbbbbbbbb|");
    }

    #[test]
    fn wraps_like_the_parsed_arguments_line() {
        // Modeled on the C# capture: the line broke at 231 columns (of 240)
        // keeping the trailing space, before the word "SimpleMode".
        let spans = [DebugSpan::plain(format!(
            "{} SimpleMode = False }}",
            "a".repeat(230)
        ))];
        let out = render(&spans, 240);
        let pieces: Vec<&str> = out.split('|').collect();
        assert_eq!(pieces[0].chars().count(), 231); // includes trailing space
        assert_eq!(pieces[1], "SimpleMode = False }");
    }

    #[test]
    fn hard_splits_long_words() {
        let spans = [DebugSpan::plain("aaaaaaaaaaaaaaaaaaaa bb")];
        // The long word carries its trailing space on the final chunk, and
        // "bb" joins that chunk's line.
        assert_eq!(render(&spans, 8), "aaaaaaaa|aaaaaaaa|aaaa bb|");
    }

    #[test]
    fn styled_span_reapplied_per_line() {
        let spans = [DebugSpan::styled(YELLOW, "aaaa bbbb cccc")];
        // Each word carries its trailing space; "aaaa " fills line 1, and
        // line 2 fits both remaining tokens exactly.
        assert_eq!(
            render(&spans, 9),
            format!("{YELLOW}aaaa {RESET}|{YELLOW}bbbb cccc{RESET}|")
        );
    }

    #[test]
    fn mixed_spans_wrap_with_style_tracking() {
        let spans = [
            DebugSpan::styled(YELLOW, "prefix"),
            DebugSpan::plain(": \""),
            DebugSpan::plain("looooooongword12345678"),
            DebugSpan::plain("\""),
        ];
        let out = render(&spans, 10);
        // The plain spans concatenate into one word (": \"looooooong...")
        // with no separating space, exactly as the C# renderer treats the
        // continuous line; the word hard-splits and the final chunk joins
        // with the closing quote.
        assert_eq!(
            out,
            format!("{YELLOW}prefix{RESET}: |\"looooooon|gword12345|678\"|")
        );
    }

    #[test]
    fn utf16_len_counts_surrogate_pairs_as_two_units() {
        // The clock glyph U+F0955 is one char but two UTF-16 code units.
        assert_eq!(utf16_len(" \u{F0955} "), 4);
        assert_eq!(utf16_len("abc"), 3);
        assert_eq!(utf16_len(" \u{276F} "), 3);
    }
}
