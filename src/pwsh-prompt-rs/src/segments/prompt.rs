// Port of PromptSegment: the prompt glyph from Settings, colored lime.

pub struct PromptSegment {
    prompt: &'static str,
}

impl PromptSegment {
    pub fn new(prompt: &'static str) -> Self {
        PromptSegment { prompt }
    }
}

impl super::Segment for PromptSegment {
    fn name(&self) -> &'static str {
        "PromptSegment"
    }

    fn unformatted_length(&self) -> i32 {
        crate::ansi::char_len(self.prompt)
    }

    fn append(&self, out: &mut String) {
        out.push_str(crate::ansi::LIME);
        out.push_str(self.prompt);
        out.push_str(crate::ansi::RESET);
    }

    fn display_text(&self) -> String {
        format!("[lime]{}[/]", self.prompt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segments::Segment;

    #[test]
    fn renders_lime_prompt() {
        let segment = PromptSegment::new(" \u{276F} ");
        let mut out = String::new();
        segment.append(&mut out);
        assert_eq!(out, "\x1b[38;5;10m \u{276F} \x1b[0m");
        assert_eq!(segment.unformatted_length(), 3);
    }
}
