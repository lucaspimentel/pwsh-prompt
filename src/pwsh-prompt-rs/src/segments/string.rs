// Port of StringSegment: literal passthrough used for the filler spaces and
// the " pwsh" shell label.

pub struct StringSegment {
    value: String,
}

impl StringSegment {
    pub fn new(value: impl Into<String>) -> Self {
        StringSegment {
            value: value.into(),
        }
    }
}

impl super::Segment for StringSegment {
    fn name(&self) -> &'static str {
        "StringSegment"
    }

    fn unformatted_length(&self) -> i32 {
        crate::ansi::utf16_len(&self.value)
    }

    fn append(&self, out: &mut String) {
        out.push_str(&self.value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segments::Segment;

    #[test]
    fn renders_value() {
        let segment = StringSegment::new(" pwsh");
        let mut out = String::new();
        segment.append(&mut out);
        assert_eq!(out, " pwsh");
        assert_eq!(segment.unformatted_length(), 5);
        assert_eq!(segment.display_text(), " pwsh");
    }
}
