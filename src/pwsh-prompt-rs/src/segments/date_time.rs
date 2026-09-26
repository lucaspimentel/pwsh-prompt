// Port of DateTimeSegment: local date/time formatted as " yyyy-MM-dd h:mm tt "
// (12-hour clock, no leading zero on the hour, uppercase invariant AM/PM,
// matching the C# invariant/en-US rendering).
//
// Width quirk reproduced: UnformattedLength adds 1 when Hour % 12 >= 10,
// because the "h" specifier renders two digits at hours 10 and 11. Hours 0
// and 12 also render two digits ("12") but do not get the +1; the C#
// implementation behaves the same way.

use chrono::{Datelike, Local, Timelike};

const FORMAT_LENGTH: i32 = 20; // " yyyy-MM-dd h:mm tt " in UTF-16 units

pub struct DateTimeSegment {
    rendered: String,
    hour: u32,
}

pub struct DateTimeRendered {
    pub text: String,
    pub hour: u32,
}

impl DateTimeSegment {
    pub fn new() -> Self {
        let now = Local::now();
        let rendered =
            render_datetime(now.year(), now.month(), now.day(), now.hour(), now.minute());
        DateTimeSegment {
            rendered: rendered.text,
            hour: rendered.hour,
        }
    }
}

/// Renders the format string; `hour` is the 24-hour input hour, kept for the
/// width quirk (see module docs).
pub fn render_datetime(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
) -> DateTimeRendered {
    let hour_12 = match hour % 12 {
        0 => 12,
        h => h,
    };
    let meridiem = if hour < 12 { "AM" } else { "PM" };
    DateTimeRendered {
        text: format!(" {year:04}-{month:02}-{day:02} {hour_12}:{minute:02} {meridiem} "),
        hour,
    }
}

impl super::Segment for DateTimeSegment {
    fn name(&self) -> &'static str {
        "DateTimeSegment"
    }

    fn unformatted_length(&self) -> i32 {
        if self.hour % 12 < 10 {
            FORMAT_LENGTH
        } else {
            FORMAT_LENGTH + 1
        }
    }

    fn append(&self, out: &mut String) {
        out.push_str(&self.rendered);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segments::Segment;

    fn render(hour: u32) -> String {
        render_datetime(2026, 9, 26, hour, 8).text
    }

    #[test]
    fn renders_twelve_hour_clock() {
        assert_eq!(render(0), " 2026-09-26 12:08 AM ");
        assert_eq!(render(1), " 2026-09-26 1:08 AM ");
        assert_eq!(render(9), " 2026-09-26 9:08 AM ");
        assert_eq!(render(11), " 2026-09-26 11:08 AM ");
        assert_eq!(render(12), " 2026-09-26 12:08 PM ");
        assert_eq!(render(13), " 2026-09-26 1:08 PM ");
        assert_eq!(render(23), " 2026-09-26 11:08 PM ");
    }

    #[test]
    fn length_quirk_matches_csharp() {
        let length_for = |hour: u32| {
            let rendered = render_datetime(2026, 9, 26, hour, 8);
            let segment = DateTimeSegment {
                rendered: rendered.text,
                hour: rendered.hour,
            };
            segment.unformatted_length()
        };
        // Template length is 20; hours 10 and 11 render a two-digit hour.
        assert_eq!(length_for(9), 20);
        assert_eq!(length_for(10), 21);
        assert_eq!(length_for(11), 21);
        // Hours 0 and 12 render "12" (two digits) but keep 20, matching C#.
        assert_eq!(length_for(0), 20);
        assert_eq!(length_for(12), 20);
        // Rendered text at hour 11 is 21 characters.
        assert_eq!(crate::ansi::utf16_len(&render(11)), 21);
    }

    #[test]
    fn appends_without_color() {
        let rendered = render_datetime(2026, 1, 2, 3, 4);
        let segment = DateTimeSegment {
            rendered: rendered.text,
            hour: rendered.hour,
        };
        let mut out = String::new();
        segment.append(&mut out);
        assert_eq!(out, " 2026-01-02 3:04 AM ");
    }
}
