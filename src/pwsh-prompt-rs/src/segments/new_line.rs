// Port of NewLineSegment: appends a literal CRLF (the C# implementation uses
// a hardcoded "\r\n" on every platform), contributes no width.

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
        out.push_str("\r\n");
    }
}
