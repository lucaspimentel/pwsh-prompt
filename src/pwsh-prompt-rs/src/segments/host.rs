// Port of HostSegment: lowercase machine name in blue. Note the C#
// implementation appends a trailing space inside the colored range that is
// NOT counted by UnformattedLength; reproduced faithfully.

pub struct HostSegment {
    hostname: String,
}

impl HostSegment {
    pub fn new() -> Self {
        // Environment.MachineName.ToLowerInvariant(): the NetBIOS name on
        // Windows, the hostname on Unix.
        let hostname = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        HostSegment { hostname }
    }

    fn prefix() -> &'static str {
        "  \u{F108}  " // computer glyph
    }
}

impl super::Segment for HostSegment {
    fn name(&self) -> &'static str {
        "HostSegment"
    }

    fn unformatted_length(&self) -> i32 {
        let prefix = Self::prefix();
        crate::ansi::utf16_len(prefix) + crate::ansi::utf16_len(&self.hostname)
    }

    fn append(&self, out: &mut String) {
        out.push_str(crate::ansi::BLUE);
        out.push_str(Self::prefix());
        out.push_str(&self.hostname);
        out.push(' ');
        out.push_str(crate::ansi::RESET);
    }

    fn display_text(&self) -> String {
        // The C# HostSegment.ToString override returns only the hostname.
        self.hostname.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segments::Segment;

    #[test]
    fn renders_blue_hostname() {
        let segment = HostSegment::new();
        let mut out = String::new();
        segment.append(&mut out);
        assert!(out.starts_with("\x1b[38;5;12m  \u{F108}  "));
        assert!(out.ends_with(" \x1b[0m"));
        // UnformattedLength excludes the trailing space appended by Append.
        assert_eq!(
            segment.unformatted_length(),
            crate::ansi::utf16_len(HostSegment::prefix())
                + crate::ansi::utf16_len(&segment.display_text())
        );
        // ToString override: only the hostname.
        assert!(!segment.display_text().contains('\u{F108}'));
    }
}
