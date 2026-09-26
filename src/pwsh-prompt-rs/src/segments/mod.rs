// Segment trait, the Rust equivalent of the C# ISegment interface: every
// visual element reports its display width (in UTF-16 code units, matching
// C# string.Length) and appends its rendered text (including ANSI escapes)
// to an output string.

pub mod date_time;
pub mod git;
pub mod host;
pub mod last_command_duration;
pub mod last_command_exit_code;
pub mod new_line;
pub mod os;
pub mod path;
pub mod prompt;
pub mod string;

pub use string::StringSegment;

pub trait Segment {
    /// The C# type name, used by the debug listing.
    fn name(&self) -> &'static str;

    /// True only for NewLineSegment; the layout engine resets the remaining
    /// width and the line-full flag on newline segments.
    fn is_newline(&self) -> bool {
        false
    }

    /// Display width in UTF-16 code units (C# string.Length semantics).
    fn unformatted_length(&self) -> i32;

    /// Appends the rendered text (markup resolved to ANSI escapes) to out.
    fn append(&self, out: &mut String);

    /// Text shown in the debug listing. Mirrors the C# ToString overrides:
    /// HostSegment returns only the hostname and OsSegment the raw value;
    /// every other segment renders via append.
    fn display_text(&self) -> String {
        let mut text = String::new();
        self.append(&mut text);
        text
    }
}
