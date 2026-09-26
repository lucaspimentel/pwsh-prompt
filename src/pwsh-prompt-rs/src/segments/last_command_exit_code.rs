// Port of LastCommandExitCodeSegment: a red icon plus the decimal exit code,
// shown when the exit code is nonzero and the command state is failure.
//
// Quirk reproduced: the constructor clears the text when lastCommandState is
// true, but Append only checks the exit code, so a nonzero exit code with
// state = true still emits the (empty) red markup range.

pub struct LastCommandExitCodeSegment {
    last_command_exit_code: i32,
    unformatted_string: String,
}

impl LastCommandExitCodeSegment {
    fn prefix() -> &'static str {
        " \u{E654} " // x-circle glyph
    }

    pub fn new(last_command_exit_code: i32, last_command_state: bool) -> Self {
        let unformatted_string = if last_command_exit_code == 0 || last_command_state {
            String::new()
        } else {
            format!("{}{}", Self::prefix(), last_command_exit_code)
        };
        LastCommandExitCodeSegment {
            last_command_exit_code,
            unformatted_string,
        }
    }
}

impl super::Segment for LastCommandExitCodeSegment {
    fn name(&self) -> &'static str {
        "LastCommandExitCodeSegment"
    }

    fn unformatted_length(&self) -> i32 {
        if self.unformatted_string.is_empty() {
            0
        } else {
            crate::ansi::utf16_len(&self.unformatted_string)
        }
    }

    fn append(&self, out: &mut String) {
        if self.last_command_exit_code == 0 {
            return;
        }

        out.push_str(crate::ansi::RED);
        out.push_str(&self.unformatted_string);
        out.push_str(crate::ansi::RESET);
    }

    fn display_text(&self) -> String {
        if self.last_command_exit_code == 0 {
            return String::new();
        }
        format!("[red]{}[/]", self.unformatted_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segments::Segment;

    #[test]
    fn exit_code_scenarios() {
        // Success: nothing at all.
        let segment = LastCommandExitCodeSegment::new(0, true);
        let mut out = String::new();
        segment.append(&mut out);
        assert_eq!(out, "");
        assert_eq!(segment.unformatted_length(), 0);

        // Failure state, nonzero code: icon plus decimal code.
        let segment = LastCommandExitCodeSegment::new(130, false);
        let mut out = String::new();
        segment.append(&mut out);
        assert_eq!(out, "\x1b[38;5;9m \u{E654} 130\x1b[0m");
        assert_eq!(segment.unformatted_length(), 6);

        // State true with nonzero code: empty text but the red range is
        // still emitted (matches the C# Append logic).
        let segment = LastCommandExitCodeSegment::new(5, true);
        let mut out = String::new();
        segment.append(&mut out);
        assert_eq!(out, "\x1b[38;5;9m\x1b[0m");
        assert_eq!(segment.unformatted_length(), 0);

        // Negative exit code renders with a minus sign.
        let segment = LastCommandExitCodeSegment::new(-3, false);
        let mut out = String::new();
        segment.append(&mut out);
        assert_eq!(out, "\x1b[38;5;9m \u{E654} -3\x1b[0m");
    }
}
