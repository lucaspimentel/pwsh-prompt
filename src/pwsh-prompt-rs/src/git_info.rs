// Port of the C# GitInfo class: discovers the .git directory (with the
// PowerShell-provided env cache short-circuit) and resolves the branch name
// from HEAD plus the git config branch section.

use crate::ansi;
use crate::path_utils;
use crate::settings;

const GIT_DIR_CACHE_ENV: &str = "PROMPT_GIT_DIR_CACHED";
const GIT_BRANCH_CACHE_ENV: &str = "PROMPT_GIT_BRANCH_CACHED";

fn env_var(name: &str) -> String {
    std::env::var(name).unwrap_or_default()
}

/// GitInfo.GetBranchName equivalent. Returns an empty string when the path is
/// not in a git repository or the HEAD is detached.
pub fn get_branch_name(path: &str) -> String {
    // Check environment variable cache first. PowerShell sets
    // PROMPT_GIT_DIR_CACHED and PROMPT_GIT_BRANCH_CACHED together; if the git
    // dir cache is empty, the branch cache (if any) is stale and must not be
    // used.
    let cached_git_dir = env_var(GIT_DIR_CACHE_ENV);
    if !cached_git_dir.is_empty() {
        let cached_branch = env_var(GIT_BRANCH_CACHE_ENV);
        if !cached_branch.is_empty() {
            return cached_branch;
        }
    }

    let Some(git_folder) = try_find_git_folder(path) else {
        return String::new();
    };

    let mut branch = String::new();

    // Get Git commit
    let head_path = format!("{git_folder}{}HEAD", std::path::MAIN_SEPARATOR);
    if path_utils::file_exists(&head_path)
        && let Ok(head) = std::fs::read_to_string(&head_path)
    {
        // .NET File.ReadAllText strips a leading UTF-8 BOM; match that.
        let head = head.trim_start_matches('\u{FEFF}').trim();

        // Symbolic Reference
        if let Some(rest) = head.strip_prefix("ref:") {
            branch = rest.to_string();
        }
    }

    // Process Git Config
    // For worktrees, gitFolder points to .git/worktrees/<name>, so we need to
    // go up to find config
    let mut config_path = path_utils::join_config(&git_folder);

    if !path_utils::file_exists(&config_path) {
        // Try parent directory for worktrees (.git/worktrees/<name>/../..)
        let parent_git_folder =
            path_utils::get_directory_name(&path_utils::get_directory_name(&git_folder));
        if !parent_git_folder.is_empty() {
            config_path = path_utils::join_config(&parent_git_folder);
        }
    }

    for item in read_config_items(&config_path) {
        if item.kind == "branch" && branch == item.merge.unwrap_or_default() {
            branch = item.name.unwrap_or_default();
            break;
        }
    }

    if !branch.is_empty() {
        branch = branch.trim().to_string();
    }

    if let Some(rest) = branch.strip_prefix("refs/heads/") {
        branch = rest.to_string();
    }

    branch
}

/// GitInfo.TryFindGitFolder equivalent. Checks the environment cache first,
/// then walks up the directory tree looking for a .git directory or a .git
/// file pointing at a worktree directory.
pub fn try_find_git_folder(path: &str) -> Option<String> {
    // Check environment variable cache first
    let cached_git_dir = env_var(GIT_DIR_CACHE_ENV);
    if !cached_git_dir.is_empty() {
        return Some(cached_git_dir);
    }

    if settings::debug() {
        ansi::write_plain_line("");
    }

    let mut current = path.to_string();
    loop {
        let git_path = path_utils::join_git(&current);

        if path_utils::directory_exists(&git_path) {
            if settings::debug() {
                ansi::write_yellow_line(&format!("Git: found in {git_path}"));
            }
            return Some(git_path);
        }

        // Check if .git is a file (git worktree)
        if path_utils::file_exists(&git_path)
            && let Ok(git_file_content) = std::fs::read_to_string(&git_path)
        {
            let git_file_content = git_file_content.trim();

            // Git worktree .git file format: "gitdir: /path/to/worktree"
            if let Some(rest) = git_file_content.strip_prefix("gitdir: ") {
                let worktree_git_dir = rest.trim();

                if path_utils::directory_exists(worktree_git_dir) {
                    if settings::debug() {
                        ansi::write_yellow_line(&format!(
                            "Git: found worktree in {worktree_git_dir}"
                        ));
                    }
                    return Some(worktree_git_dir.to_string());
                }
            }
        }

        if settings::debug() {
            ansi::write_yellow_line(&format!("Git: not found in {git_path}"));
        }

        let parent = path_utils::get_directory_name(&current);
        if parent.is_empty() {
            return None;
        }
        current = parent;
    }
}

struct ConfigItem {
    kind: String,
    name: Option<String>,
    merge: Option<String>,
}

/// GitInfo.GetConfigItemsIterator equivalent: parses section headers matching
/// ^\[(.*) "(.*)"\] (greedy groups) and tab-indented "merge = <value>" lines.
fn read_config_items(config_file: &str) -> Vec<ConfigItem> {
    let Ok(content) = std::fs::read_to_string(config_file) else {
        return Vec::new();
    };

    let mut items: Vec<ConfigItem> = Vec::new();

    for line in content.lines() {
        if line.is_empty() {
            continue;
        }

        if line.starts_with('\t') {
            let Some(current) = items.last_mut() else {
                continue;
            };
            if let Some(equals_index) = line.find(" = ")
                && equals_index > 1
                && &line[1..equals_index] == "merge"
            {
                current.merge = Some(line[equals_index + 3..].to_string());
            }
            continue;
        }

        if let Some((kind, name)) = parse_config_section(line) {
            items.push(ConfigItem {
                kind,
                name: Some(name),
                merge: None,
            });
        }
    }

    items
}

/// Emulates the C# regex ^\[(.*) "(.*)"\] with greedy groups: the section
/// closes at the last `"]` in the line, and the section name is quoted after
/// the last ` "` before it.
fn parse_config_section(line: &str) -> Option<(String, String)> {
    if !line.starts_with('[') {
        return None;
    }

    let closing = line.rfind("\"]")?;
    let separator = line[..closing].rfind(" \"")?;
    Some((
        line[1..separator].to_string(),
        line[separator + 2..closing].to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    struct TempDir(PathBuf);
    impl TempDir {
        fn new(name: &str) -> Self {
            let base = std::env::temp_dir().join(format!("pwsh-prompt-tests-{name}"));
            let _ = fs::remove_dir_all(&base);
            fs::create_dir_all(&base).unwrap();
            TempDir(base)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    /// Saves and clears the cache env vars, restoring them on drop.
    struct EnvGuard {
        saved: Vec<(String, String)>,
    }
    impl EnvGuard {
        fn new() -> Self {
            let names = [GIT_DIR_CACHE_ENV, GIT_BRANCH_CACHE_ENV];
            let saved: Vec<(String, String)> = names
                .iter()
                .filter_map(|n| std::env::var(n).ok().map(|v| (n.to_string(), v)))
                .collect();
            for n in names {
                crate::test_support::remove_var(n);
            }
            EnvGuard { saved }
        }
    }
    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (name, value) in &self.saved {
                crate::test_support::set_var(name, value);
            }
        }
    }

    // All scenarios run in one test because they mutate process-wide
    // environment variables, which are shared across parallel test threads.
    #[test]
    fn git_info_scenarios() {
        let _lock = crate::test_support::ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _guard = EnvGuard::new();

        // Walk up finds .git and resolves the branch.
        let dir = TempDir::new("walk-up");
        let repo = dir.path().join("repo");
        let nested = repo.join("a/b/c");
        write(&repo.join(".git/HEAD"), "ref: refs/heads/main\n");
        write(&repo.join(".git/config"), "[core]\n");

        let found = try_find_git_folder(nested.to_str().unwrap()).unwrap();
        assert_eq!(PathBuf::from(&found), repo.join(".git"));
        assert_eq!(get_branch_name(nested.to_str().unwrap()), "main");

        // No git anywhere.
        let dir = TempDir::new("no-git");
        assert!(try_find_git_folder(dir.path().to_str().unwrap()).is_none());
        assert_eq!(get_branch_name(dir.path().to_str().unwrap()), "");

        // Worktree .git file points at the worktree git dir.
        let dir = TempDir::new("worktree");
        let main_git = dir.path().join("main").join(".git");
        let wt = dir.path().join("wt");
        fs::create_dir_all(&wt).unwrap();
        write(
            &main_git.join("worktrees/wt/HEAD"),
            "ref: refs/heads/feature\n",
        );
        write(
            &main_git.join("config"),
            "[core]\n\trepositoryformatversion = 0\n",
        );
        write(
            &wt.join(".git"),
            &format!(
                "gitdir: {}",
                main_git.join("worktrees/wt").to_str().unwrap()
            ),
        );

        let found = try_find_git_folder(wt.to_str().unwrap()).unwrap();
        assert!(found.contains("worktrees"));
        assert_eq!(get_branch_name(wt.to_str().unwrap()), "feature");

        // Detached HEAD: empty branch.
        let dir = TempDir::new("detached");
        let git = dir.path().join(".git");
        write(
            &git.join("HEAD"),
            "a1b2c3d4e5f6a7b8c9d0a1b2c3d4e5f6a7b8c9d0\n",
        );
        write(&git.join("config"), "[core]\n");
        assert_eq!(get_branch_name(dir.path().to_str().unwrap()), "");

        // No branch section: falls back to the ref name.
        let dir = TempDir::new("no-branch-section");
        let git = dir.path().join(".git");
        write(&git.join("HEAD"), "ref: refs/heads/topic\n");
        write(&git.join("config"), "[core]\n\tbare = false\n");
        assert_eq!(get_branch_name(dir.path().to_str().unwrap()), "topic");

        // Matching branch section resolves the section name. HEAD uses a
        // "ref:<name>" form without a space so it equals the merge value,
        // mirroring the C# StringSegment offset behavior.
        let dir = TempDir::new("branch-section");
        let git = dir.path().join(".git");
        write(&git.join("HEAD"), "ref:develop\n");
        write(&git.join("config"), "[branch \"dev\"]\n\tmerge = develop\n");
        assert_eq!(get_branch_name(dir.path().to_str().unwrap()), "dev");

        // Env cache short-circuits both discovery and branch lookup.
        crate::test_support::set_var(GIT_DIR_CACHE_ENV, r"D:\fake\.git");
        crate::test_support::set_var(GIT_BRANCH_CACHE_ENV, "cached-branch");
        assert_eq!(get_branch_name(r"C:\nowhere\at\all"), "cached-branch");
        assert_eq!(
            try_find_git_folder(r"C:\nowhere\at\all").as_deref(),
            Some(r"D:\fake\.git")
        );

        // Branch cache is ignored when the dir cache is empty.
        crate::test_support::set_var(GIT_DIR_CACHE_ENV, "");
        crate::test_support::set_var(GIT_BRANCH_CACHE_ENV, "stale");
        let dir = TempDir::new("stale-cache");
        assert_eq!(get_branch_name(dir.path().to_str().unwrap()), "");
    }

    #[test]
    fn config_section_parsing() {
        assert_eq!(
            parse_config_section("[branch \"main\"]"),
            Some(("branch".to_string(), "main".to_string()))
        );
        assert_eq!(parse_config_section("[core]"), None);
        assert_eq!(parse_config_section("no brackets"), None);
        // Greedy groups: last quoted section wins.
        assert_eq!(
            parse_config_section("[a \"b\" c \"d\"]"),
            Some(("a \"b\" c".to_string(), "d".to_string()))
        );
    }

    #[test]
    fn config_merge_line_parsing() {
        let dir = TempDir::new("config-merge");
        let config = dir.path().join("config");
        write(
            &config,
            "[branch \"main\"]\n\tmerge = refs/heads/main\n\tremote = origin\n",
        );

        let items = read_config_items(config.to_str().unwrap());
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, "branch");
        assert_eq!(items[0].name.as_deref(), Some("main"));
        assert_eq!(items[0].merge.as_deref(), Some("refs/heads/main"));
    }

    #[test]
    fn missing_config_file_yields_no_items() {
        assert!(read_config_items(r"C:\definitely\not\here\config").is_empty());
    }
}
