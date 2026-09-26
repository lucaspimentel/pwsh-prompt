// Port of GitSegment: branch name plus optional PR info in magenta.

use crate::env::EnvSource;

pub struct GitSegment {
    branch_name: String,
    pr_number: Option<String>,
    pr_icon: &'static str,
}

impl GitSegment {
    const BRANCH_PREFIX: &'static str = "  \u{E725} "; // branch glyph
    const PR_ICON_OPEN: &'static str = "  \u{EA64} "; // open PR icon
    const PR_ICON_CLOSED: &'static str = "  \u{EBDA} "; // closed PR icon
    const PR_ICON_DRAFT: &'static str = "  \u{EBDB} "; // draft PR icon

    pub fn new(path: &str, env: &dyn EnvSource, sink: &crate::ansi::DebugSink) -> Self {
        if crate::path_utils::file_exists(path) || dir_exists(path) {
            let branch_name = crate::git_info::get_branch_name(path, env, sink);

            let pr_number = env.var("PROMPT_PR_NUMBER_CACHED").unwrap_or_default();
            if !pr_number.is_empty() {
                let pr_state = env.var("PROMPT_PR_STATE_CACHED").unwrap_or_default();
                let pr_icon = match pr_state.as_str() {
                    "closed" => Self::PR_ICON_CLOSED,
                    "draft" => Self::PR_ICON_DRAFT,
                    _ => Self::PR_ICON_OPEN,
                };
                GitSegment {
                    branch_name,
                    pr_number: Some(pr_number),
                    pr_icon,
                }
            } else {
                GitSegment {
                    branch_name,
                    pr_number: None,
                    pr_icon: Self::PR_ICON_OPEN,
                }
            }
        } else {
            GitSegment {
                branch_name: String::new(),
                pr_number: None,
                pr_icon: Self::PR_ICON_OPEN,
            }
        }
    }
}

fn dir_exists(path: &str) -> bool {
    std::fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false)
}

impl super::Segment for GitSegment {
    fn name(&self) -> &'static str {
        "GitSegment"
    }

    fn unformatted_length(&self) -> i32 {
        if self.branch_name.is_empty() {
            0
        } else {
            let pr_prefix_length = self
                .pr_number
                .as_ref()
                .map(|n| crate::ansi::utf16_len(self.pr_icon) + crate::ansi::utf16_len(n))
                .unwrap_or(0);
            crate::ansi::utf16_len(Self::BRANCH_PREFIX)
                + crate::ansi::utf16_len(&self.branch_name)
                + pr_prefix_length
        }
    }

    fn append(&self, out: &mut String) {
        if self.branch_name.is_empty() {
            return;
        }

        out.push_str(crate::ansi::MAGENTA_FF7FFF);
        out.push_str(Self::BRANCH_PREFIX);
        out.push_str(&self.branch_name);

        if let Some(pr_number) = &self.pr_number {
            out.push_str(self.pr_icon);
            out.push_str(pr_number);
        }

        out.push_str(crate::ansi::RESET);
    }

    fn display_text(&self) -> String {
        if self.branch_name.is_empty() {
            return String::new();
        }
        let mut text = String::from("[#ff7fff]");
        text.push_str(Self::BRANCH_PREFIX);
        text.push_str(&self.branch_name);
        if let Some(pr_number) = &self.pr_number {
            text.push_str(self.pr_icon);
            text.push_str(pr_number);
        }
        text.push_str("[/]");
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::FakeEnv;
    use crate::segments::Segment;

    #[test]
    fn git_segment_scenarios() {
        let sink = crate::ansi::DebugSink::disabled();
        let empty = FakeEnv::empty();

        let dir = std::env::temp_dir().join("pwsh-prompt-tests-git-segment");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.to_str().unwrap();

        // Nonexistent path: nothing rendered.
        let segment = GitSegment::new(r"C:\definitely\not\here", &empty, &sink);
        let mut out = String::new();
        segment.append(&mut out);
        assert_eq!(out, "");
        assert_eq!(segment.unformatted_length(), 0);

        // Existing directory without a repo: nothing rendered (empty
        // branch), PR icon defaults to open but is not rendered.
        let segment = GitSegment::new(path, &empty, &sink);
        let mut out = String::new();
        segment.append(&mut out);
        assert_eq!(out, "");

        // Env-var PR states only affect rendering when a branch exists, so
        // use the env cache to force one.
        let branch_env = FakeEnv::empty()
            .with("PROMPT_GIT_DIR_CACHED", "D:\\fake\\.git")
            .with("PROMPT_GIT_BRANCH_CACHED", "main");

        let segment = GitSegment::new(path, &branch_env, &sink);
        assert_eq!(segment.unformatted_length(), 8); // prefix 4 + "main" 4
        let mut out = String::new();
        segment.append(&mut out);
        assert_eq!(out, "\x1b[38;2;255;127;255m  \u{E725} main\x1b[0m");

        let closed_pr_env = branch_env
            .clone()
            .with("PROMPT_PR_NUMBER_CACHED", "#12")
            .with("PROMPT_PR_STATE_CACHED", "closed");
        let segment = GitSegment::new(path, &closed_pr_env, &sink);
        let mut out = String::new();
        segment.append(&mut out);
        assert_eq!(
            out,
            "\x1b[38;2;255;127;255m  \u{E725} main  \u{EBDA} #12\x1b[0m"
        );
        assert_eq!(segment.unformatted_length(), 15);

        let draft_pr_env = branch_env
            .clone()
            .with("PROMPT_PR_NUMBER_CACHED", "#12")
            .with("PROMPT_PR_STATE_CACHED", "draft");
        let segment = GitSegment::new(path, &draft_pr_env, &sink);
        let mut out = String::new();
        segment.append(&mut out);
        assert!(out.contains("\u{EBDB}"));

        let open_pr_env = branch_env
            .clone()
            .with("PROMPT_PR_NUMBER_CACHED", "#12")
            .with("PROMPT_PR_STATE_CACHED", "");
        let segment = GitSegment::new(path, &open_pr_env, &sink);
        let mut out = String::new();
        segment.append(&mut out);
        assert!(out.contains("\u{EA64}"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
