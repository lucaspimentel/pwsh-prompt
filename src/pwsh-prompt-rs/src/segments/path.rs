// Port of PathSegment: the first segment, showing the current directory with
// a git or default icon, repo-relative or home-relative display, and
// separator-based truncation.
//
// All index arithmetic operates on UTF-16 code units to match the C#
// StringSegment/String.Length semantics exactly.

use crate::ansi::{self, DebugSink};
use crate::git_info;
use crate::path_utils;

pub struct PathSegment {
    current_directory_display: String,
    is_in_user_home: bool,
    is_truncated: bool,
    is_git_repo: bool,
}

impl PathSegment {
    const DEFAULT_PREFIX: &'static str = " \u{F07C}  "; // folder glyph
    const GIT_PREFIX: &'static str = " \u{E5FB}  "; // git glyph

    pub fn new(
        current_directory: &str,
        is_file_system: bool,
        max_path_length: i32,
        simple_mode: bool,
        sink: &DebugSink,
    ) -> Self {
        let mut segment = PathSegment {
            current_directory_display: current_directory.to_string(),
            is_in_user_home: false,
            is_truncated: false,
            is_git_repo: false,
        };

        if !is_file_system {
            return segment;
        }

        let process_directory = std::env::current_exe().ok().and_then(|exe| {
            let dir = path_utils::get_directory_name(&exe.to_string_lossy());
            if dir.is_empty() { None } else { Some(dir) }
        });

        let Some(_) = process_directory else {
            return segment;
        };

        if let Some(git_directory) = git_info::try_find_git_folder(current_directory, sink) {
            segment.is_git_repo = true;

            // In simple mode, keep the full path instead of shortening to
            // repo root
            if !simple_mode {
                // currentDirectory = /path/to/repo[/child]
                // gitDirectory = /path/to/repo/.git or /path/to/repo/.git/worktrees/name
                // --------------------------------------------------------------
                // repositoryDirectory = /path/to/repo or /path/to/repo/.git/worktrees
                // repositoryParentDirectory = /path/to or /path/to/repo/.git

                let mut repository_directory = path_utils::get_directory_name(&git_directory);

                // For worktrees, gitDirectory is .git/worktrees/<name>, so we
                // need to go up twice more
                if repository_directory.ends_with("worktrees") {
                    let parent = path_utils::get_directory_name(&repository_directory);
                    repository_directory = path_utils::get_directory_name(&parent);
                }

                if !repository_directory.is_empty() {
                    let repository_parent_directory =
                        path_utils::get_directory_name(&repository_directory);
                    if !repository_parent_directory.is_empty() {
                        let display_path = path_utils::get_relative_path(
                            &repository_parent_directory,
                            current_directory,
                        );

                        if !display_path.is_empty() {
                            segment.current_directory_display = display_path;
                        }
                    }
                }
            }
        } else {
            let user_profile_directory = user_profile_directory();

            sink.yellow_line(&format!("userProfileDirectory: {user_profile_directory}"));

            if starts_with_ignore_case_units(current_directory, &user_profile_directory) {
                // remove user home from path, prepend "~" later
                segment.is_in_user_home = true;
                segment.current_directory_display =
                    substring_after_units(current_directory, &user_profile_directory);
            }
        }

        sink.yellow_line(&format!(
            "displayPath before truncating: {}",
            segment.current_directory_display
        ));

        if let Some(truncated) =
            try_shorten_path(&segment.current_directory_display, max_path_length)
        {
            segment.is_truncated = true;
            segment.current_directory_display = truncated;
        }

        sink.yellow_line(&format!(
            "displayPath after truncating: {}",
            segment.current_directory_display
        ));

        segment
    }

    fn prefix(&self) -> &'static str {
        if self.is_git_repo {
            Self::GIT_PREFIX
        } else {
            Self::DEFAULT_PREFIX
        }
    }
}

impl super::Segment for PathSegment {
    fn name(&self) -> &'static str {
        "PathSegment"
    }

    fn unformatted_length(&self) -> i32 {
        let length =
            ansi::utf16_len(&self.current_directory_display) + ansi::utf16_len(self.prefix());

        match (self.is_in_user_home, self.is_truncated) {
            (true, true) => length + 5,  // "~/..."
            (true, false) => length + 1, // "~"
            (false, true) => length + 3, // "..."
            (false, false) => length,
        }
    }

    fn append(&self, out: &mut String) {
        // Skip Path.Exists check for performance: rare edge case not worth
        // the I/O cost
        out.push_str(ansi::AQUA);

        out.push_str(self.prefix());

        if self.is_in_user_home {
            out.push('~');
        }

        if self.is_in_user_home && self.is_truncated {
            out.push(path_separator()); // Path.PathSeparator
        }

        if self.is_truncated {
            out.push_str("...");
        }

        out.push_str(&self.current_directory_display);
        out.push_str(ansi::RESET);
    }

    fn display_text(&self) -> String {
        // The C# ToString renders the markup tags as literal text.
        let mut text = String::from("[aqua]");
        text.push_str(self.prefix());

        if self.is_in_user_home {
            text.push('~');
        }

        if self.is_in_user_home && self.is_truncated {
            text.push(path_separator());
        }

        if self.is_truncated {
            text.push_str("...");
        }

        text.push_str(&self.current_directory_display);
        text.push_str("[/]");
        text
    }
}

/// System.IO.Path.PathSeparator: ';' on Windows, ':' on Unix.
fn path_separator() -> char {
    if cfg!(windows) { ';' } else { ':' }
}

/// Environment.GetFolderPath(SpecialFolder.UserProfile) equivalent. The
/// USERPROFILE environment variable is what Windows resolves for the user
/// profile; on Unix, $HOME (falling back to getpwuid via HOME being unset).
fn user_profile_directory() -> String {
    std::env::var(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).unwrap_or_default()
}

/// PathSegment.TryShortenPath equivalent. When the path exceeds
/// max_path_length (in UTF-16 units), walks the string and cuts at the first
/// separator whose suffix plus "..." fits. Returns Some(suffix) when
/// truncated.
fn try_shorten_path(path: &str, max_path_length: i32) -> Option<String> {
    let units: Vec<u16> = path.encode_utf16().collect();

    if units.len() as i64 <= i64::from(max_path_length) {
        return None;
    }

    for (i, &unit) in units.iter().enumerate() {
        if unit == b'\\' as u16 || unit == b'/' as u16 {
            let new_length = units.len() as i64 - i as i64 + 3;

            if new_length > 0 && new_length <= i64::from(max_path_length) {
                return Some(String::from_utf16_lossy(&units[i..]));
            }
        }
    }

    // couldn't find a separator to truncate at
    None
}

/// Case-insensitive prefix check on UTF-16 code units, mirroring
/// StringComparison.OrdinalIgnoreCase for ASCII and exact comparison for
/// non-ASCII units.
fn starts_with_ignore_case_units(path: &str, prefix: &str) -> bool {
    let path_units: Vec<u16> = path.encode_utf16().collect();
    let prefix_units: Vec<u16> = prefix.encode_utf16().collect();
    path_units.len() >= prefix_units.len()
        && path_units[..prefix_units.len()]
            .iter()
            .zip(prefix_units.iter())
            .all(|(a, b)| {
                let (a, b) = (*a, *b);
                let ascii_lower = |u: u16| {
                    if (b'A' as u16..=b'Z' as u16).contains(&u) {
                        u + 32
                    } else {
                        u
                    }
                };
                a == b || (a < 0x80 && b < 0x80 && ascii_lower(a) == ascii_lower(b))
            })
}

/// The remainder of path after the prefix, in UTF-16 units.
fn substring_after_units(path: &str, prefix: &str) -> String {
    let prefix_units: Vec<u16> = prefix.encode_utf16().collect();
    let units: Vec<u16> = path.encode_utf16().collect();
    String::from_utf16_lossy(&units[prefix_units.len().min(units.len())..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segments::Segment;

    const MAIN_SEPARATOR: char = std::path::MAIN_SEPARATOR;
    const SEP: &str = if cfg!(windows) { "\\" } else { "/" };

    fn build(
        current_directory: &str,
        max_path_length: i32,
        simple_mode: bool,
    ) -> (PathSegment, String) {
        let segment = PathSegment::new(
            current_directory,
            true,
            max_path_length,
            simple_mode,
            &DebugSink::disabled(),
        );
        let mut out = String::new();
        segment.append(&mut out);
        (segment, out)
    }

    fn strip_ansi(text: &str) -> String {
        // Remove the leading color and trailing reset for plain assertions.
        text.strip_prefix(ansi::AQUA)
            .unwrap_or(text)
            .strip_suffix(ansi::RESET)
            .unwrap_or(text)
            .to_string()
    }

    // Env vars are process-global, so all scenarios run in one test.
    #[test]
    fn path_segment_scenarios() {
        let _lock = crate::test_support::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _guard = crate::test_support::PromptEnvGuard::clear();

        // Redirect the user profile to a fake home so the temp dir used
        // below sits outside it.
        let fake_home = std::env::temp_dir().join("pwsh-prompt-tests-path/fake-home");
        let _ = std::fs::remove_dir_all(&fake_home);
        std::fs::create_dir_all(&fake_home).unwrap();
        let home_var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
        let saved_home = std::env::var(home_var).ok();
        crate::test_support::set_var(home_var, fake_home.to_str().unwrap());

        // Outside any repo: default prefix, verbatim path.
        let dir = std::env::temp_dir().join("pwsh-prompt-tests-path/nonrepo");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let (segment, out) = build(dir.to_str().unwrap(), 500, false);
        assert!(!segment.is_git_repo);
        assert_eq!(
            strip_ansi(&out),
            format!(" \u{F07C}  {}", dir.to_str().unwrap())
        );
        assert_eq!(
            segment.unformatted_length(),
            ansi::utf16_len(&dir.to_string_lossy()) + 4
        );

        // In a repo, non-simple: display relative to the repo's parent.
        let repo = std::env::temp_dir().join("pwsh-prompt-tests-path/repo");
        let _ = std::fs::remove_dir_all(&repo);
        std::fs::create_dir_all(repo.join(".git")).unwrap();
        std::fs::write(repo.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
        std::fs::create_dir_all(repo.join("src/deep")).unwrap();
        let (segment, out) = build(repo.join("src/deep").to_str().unwrap(), 500, false);
        assert!(segment.is_git_repo);
        assert_eq!(
            strip_ansi(&out),
            format!(" \u{E5FB}  repo{SEP}src{SEP}deep")
        );
        assert_eq!(
            segment.unformatted_length(),
            4 + ansi::utf16_len(&format!("repo{SEP}src{SEP}deep"))
        );

        // Simple mode keeps the full path (with the git prefix).
        let (segment, out) = build(repo.join("src/deep").to_str().unwrap(), 500, true);
        assert!(segment.is_git_repo);
        assert_eq!(
            strip_ansi(&out),
            format!(" \u{E5FB}  {}", repo.join("src/deep").to_str().unwrap())
        );

        // Inside the user's home, not in a repo: "~" display.
        let home = fake_home.to_string_lossy().into_owned();
        let home_sub = format!("{home}{MAIN_SEPARATOR}.cache");
        let _ = std::fs::create_dir_all(&home_sub);
        let (segment, out) = build(&home_sub, 500, false);
        assert!(segment.is_in_user_home);
        assert!(!segment.is_git_repo);
        assert_eq!(
            strip_ansi(&out),
            format!(" \u{F07C}  ~{MAIN_SEPARATOR}.cache")
        );
        assert_eq!(
            segment.unformatted_length(),
            4 + 1 + ansi::utf16_len(&format!("{MAIN_SEPARATOR}.cache"))
        );

        // Exactly at the home root: just "~".
        let (segment, out) = build(&home, 500, false);
        assert!(segment.is_in_user_home);
        assert_eq!(strip_ansi(&out), " \u{F07C}  ~");
        assert_eq!(segment.unformatted_length(), 5);

        let _ = std::fs::remove_dir_all(std::env::temp_dir().join("pwsh-prompt-tests-path"));
        let _ = std::fs::remove_dir_all(&home_sub);

        if let Some(home) = saved_home {
            crate::test_support::set_var(home_var, &home);
        }
    }

    // Truncation operates purely on strings; test the helper directly.
    #[test]
    fn try_shorten_path_scenarios() {
        // Fits: not truncated.
        assert_eq!(try_shorten_path(r"pwsh-prompt\src", 19), None);
        // Exactly fits.
        assert_eq!(try_shorten_path(r"pwsh-prompt\src", 15), None);
        // First separator whose suffix + "..." fits wins.
        let long =
            r"C:\Users\lucas\.nuget\packages\microsoft.extensions.primitives\10.0.11\lib\net10.0";
        let truncated = try_shorten_path(long, 19).unwrap();
        assert_eq!(truncated, r"\lib\net10.0");
        // No separator fits: not truncated.
        assert_eq!(
            try_shorten_path(r"single-component-with-no-separator", 5),
            None
        );
        // Zero/negative budget: no truncation possible.
        assert_eq!(try_shorten_path(r"a\b", 0), None);
        assert_eq!(try_shorten_path(r"a\b", -1), None);
        // Truncation keeps the separator itself.
        assert_eq!(try_shorten_path(r"a\b\c\d\e", 5).as_deref(), Some(r"\e"));
    }

    #[test]
    fn starts_with_ignore_case_units_matches_ordinal() {
        assert!(starts_with_ignore_case_units(
            r"C:\Users\lucas\x",
            r"c:\users\LUCAS"
        ));
        assert!(!starts_with_ignore_case_units(
            r"C:\Users",
            r"C:\Users\lucas"
        ));
        assert!(starts_with_ignore_case_units("", ""));
        assert!(!starts_with_ignore_case_units("abc", "abcd"));
    }

    #[test]
    fn substring_after_units_splits() {
        assert_eq!(
            substring_after_units(r"C:\Users\lucas\x", r"C:\Users\lucas"),
            r"\x"
        );
        assert_eq!(
            substring_after_units(r"C:\Users\lucas", r"C:\Users\lucas"),
            ""
        );
    }
}
