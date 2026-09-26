// Port of OsSegment: platform icon, raw (no color). The C# implementation
// checks the platform at runtime; Rust compiles the icon per target, which
// matches the per-RID release binaries.

pub struct OsSegment {
    value: &'static str,
}

impl OsSegment {
    pub fn new() -> Self {
        let value = if cfg!(windows) {
            " \u{E62A} " // Windows icon
        } else if cfg!(target_os = "linux") {
            " \u{E712} " // Linux icon
        } else if cfg!(target_os = "macos") {
            " \u{E711} " // Apple icon
        } else {
            " OS? "
        };
        OsSegment { value }
    }
}

impl super::Segment for OsSegment {
    fn name(&self) -> &'static str {
        "OsSegment"
    }

    fn unformatted_length(&self) -> i32 {
        crate::ansi::char_len(self.value)
    }

    fn append(&self, out: &mut String) {
        out.push_str(self.value);
    }

    fn display_text(&self) -> String {
        self.value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segments::Segment;

    #[test]
    fn renders_platform_icon_without_color() {
        let segment = OsSegment::new();
        let mut out = String::new();
        segment.append(&mut out);
        if cfg!(windows) {
            assert_eq!(out, " \u{E62A} ");
        }
        // Length is 3 for all glyphs, 5 for the " OS? " fallback.
        assert_eq!(segment.unformatted_length(), 3);
        assert_eq!(segment.display_text(), out);
    }
}
