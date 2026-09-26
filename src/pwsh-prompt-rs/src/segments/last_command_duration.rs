// Port of LastCommandDurationSegment: the last command's wall-clock time,
// shown when it reaches the threshold.
//
// Format branches (invariant culture):
//   >= 60s   -> "Nm Ss" ("1024m 0s")
//   >= 1s    -> "0.#" seconds, one decimal with trailing zeros and the
//               decimal point trimmed, rounded half away from zero ("4.5s",
//               "60s")
//   < 1s     -> "{ms}ms" ("30ms")

const PREFIX: &str = " \u{F0955} "; // clock glyph

pub struct LastCommandDurationSegment {
    last_command_duration_ms: i32,
    threshold_ms: i32,
    unformatted_string: String,
}

impl LastCommandDurationSegment {
    pub fn new(last_command_duration_ms: i32, threshold_ms: i32) -> Self {
        let unformatted_string = if last_command_duration_ms < threshold_ms {
            String::new()
        } else {
            format_duration(last_command_duration_ms)
        };
        LastCommandDurationSegment {
            last_command_duration_ms,
            threshold_ms,
            unformatted_string,
        }
    }
}

fn format_duration(last_command_duration_ms: i32) -> String {
    if last_command_duration_ms >= 60_000 {
        // 1,440m 59s
        let minutes = last_command_duration_ms / 60_000;
        let seconds = (last_command_duration_ms % 60_000) / 1_000;

        let minutes_text = minutes.to_string();
        format!("{PREFIX}{minutes_text}m {seconds}s")
    } else if last_command_duration_ms >= 1_000 {
        // 59.9s
        let total_seconds = f64::from(last_command_duration_ms % 60_000) / 1000.0;
        format!("{PREFIX}{}s", format_zero_dot_hash(total_seconds))
    } else {
        // 999ms
        format!("{PREFIX}{last_command_duration_ms}ms")
    }
}

/// C# "0.#" format: one decimal, rounded half away from zero, trailing zeros
/// and an all-zero fraction dropped ("60.0" -> "60").
fn format_zero_dot_hash(value: f64) -> String {
    let scaled = (value * 10.0).round() as i64;
    let whole = scaled / 10;
    let fraction = scaled % 10;
    if fraction == 0 {
        format!("{whole}")
    } else if fraction < 0 {
        format!("{whole}.{}", -fraction)
    } else {
        format!("{whole}.{fraction}")
    }
}

impl super::Segment for LastCommandDurationSegment {
    fn name(&self) -> &'static str {
        "LastCommandDurationSegment"
    }

    fn unformatted_length(&self) -> i32 {
        if self.unformatted_string.is_empty() {
            0
        } else {
            // Code-point length: the clock glyph is one char in one column.
            crate::ansi::char_len(&self.unformatted_string)
        }
    }

    fn append(&self, out: &mut String) {
        if self.last_command_duration_ms < self.threshold_ms {
            return;
        }

        out.push_str(crate::ansi::YELLOW);
        out.push_str(&self.unformatted_string);
        out.push_str(crate::ansi::RESET);
    }

    fn display_text(&self) -> String {
        if self.last_command_duration_ms < self.threshold_ms {
            return String::new();
        }
        format!("[yellow]{}[/]", self.unformatted_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segments::Segment;

    fn rendered(ms: i32) -> String {
        let segment = LastCommandDurationSegment::new(ms, 30);
        let mut out = String::new();
        segment.append(&mut out);
        out
    }

    #[test]
    fn below_threshold_is_empty() {
        assert_eq!(rendered(0), "");
        assert_eq!(rendered(29), "");
        let segment = LastCommandDurationSegment::new(29, 30);
        assert_eq!(segment.unformatted_length(), 0);
    }

    #[test]
    fn millisecond_branch() {
        assert_eq!(rendered(30), "\x1b[38;5;11m \u{F0955} 30ms\x1b[0m");
        assert_eq!(rendered(999), "\x1b[38;5;11m \u{F0955} 999ms\x1b[0m");
        // 4 UTF-16 units prefix + "30ms" minus the glyph adjustment.
        let segment = LastCommandDurationSegment::new(30, 30);
        assert_eq!(segment.unformatted_length(), 7);
    }

    #[test]
    fn seconds_branch() {
        assert_eq!(rendered(1000), "\x1b[38;5;11m \u{F0955} 1s\x1b[0m");
        assert_eq!(rendered(1050), "\x1b[38;5;11m \u{F0955} 1.1s\x1b[0m");
        assert_eq!(rendered(4550), "\x1b[38;5;11m \u{F0955} 4.6s\x1b[0m");
        assert_eq!(rendered(59999), "\x1b[38;5;11m \u{F0955} 60s\x1b[0m");
    }

    #[test]
    fn minutes_branch() {
        assert_eq!(rendered(60000), "\x1b[38;5;11m \u{F0955} 1m 0s\x1b[0m");
        assert_eq!(rendered(61000), "\x1b[38;5;11m \u{F0955} 1m 1s\x1b[0m");
        assert_eq!(rendered(1440000), "\x1b[38;5;11m \u{F0955} 24m 0s\x1b[0m");
        assert_eq!(
            rendered(61440000),
            "\x1b[38;5;11m \u{F0955} 1024m 0s\x1b[0m"
        );
        assert_eq!(
            rendered(123456789),
            "\x1b[38;5;11m \u{F0955} 2057m 36s\x1b[0m"
        );
    }

    #[test]
    fn length_counts_glyph_as_one_column() {
        // " \u{F0955} 1024m 0s" is 11 code points, each rendering in one
        // column.
        let segment = LastCommandDurationSegment::new(61440000, 30);
        assert_eq!(segment.unformatted_length(), 11);
    }

    #[test]
    fn zero_dot_hash_rounds_half_away_from_zero() {
        assert_eq!(format_zero_dot_hash(4.55), "4.6");
        assert_eq!(format_zero_dot_hash(1.05), "1.1");
        assert_eq!(format_zero_dot_hash(59.999), "60");
        assert_eq!(format_zero_dot_hash(1.049), "1");
        assert_eq!(format_zero_dot_hash(0.95), "1");
        assert_eq!(format_zero_dot_hash(0.0), "0");
    }
}
