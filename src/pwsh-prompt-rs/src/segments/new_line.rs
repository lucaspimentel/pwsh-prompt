// Port of NewLineSegment: appends the platform newline (CRLF on Windows, LF
// on Unix, matching the C# Environment.NewLine), contributes no width.

use crate::ansi::platform_newline;

pub struct NewLineSegment;

impl super::Segment for NewLineSegment {
    fn name(&self) -> &'static str {
        "NewLineSegment"
    }

    fn is_newline(&self) -> bool {
        true
    }

    fn unformatted_length(&self) -> i32 {
        0
    }

    fn append(&self, out: &mut String) {
        out.push_str(platform_newline());
    }
}
