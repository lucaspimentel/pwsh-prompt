// Build script: resolves the current git commit hash so --version can print
// "<version>+<sha>", matching the .NET SDK's SourceRevisionId behavior that
// produces the AssemblyInformationalVersion string in the C# build.

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    println!("cargo:rerun-if-changed=build.rs");

    let sha = find_commit_hash().unwrap_or_default();
    // Expose the hash to the crate; empty string means no git metadata found.
    println!("cargo:rustc-env=PROMPT_GIT_SHA={sha}");
    ExitCode::SUCCESS
}

/// Walks up from the crate directory to find a `.git` directory or worktree
/// file, then resolves HEAD to a full 40-character commit hash.
fn find_commit_hash() -> Option<String> {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let git_dir = loop {
        let candidate = dir.join(".git");
        if candidate.is_dir() {
            break candidate;
        }
        if candidate.is_file() {
            // Worktree: the file contains "gitdir: <path>"
            let content = std::fs::read_to_string(&candidate).ok()?;
            let worktree = content.trim().strip_prefix("gitdir: ")?.trim();
            let worktree = resolve_worktree_path(&dir, worktree)?;
            break worktree;
        }
        if !dir.pop() {
            return None;
        }
    };

    resolve_head(&git_dir)
}

fn resolve_worktree_path(base: &std::path::Path, worktree: &str) -> Option<PathBuf> {
    let path = PathBuf::from(worktree);
    if path.is_absolute() {
        Some(path)
    } else {
        Some(base.join(path))
    }
}

fn resolve_head(git_dir: &std::path::Path) -> Option<String> {
    let head = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let head = head.trim();

    if let Some(reference) = head.strip_prefix("ref: ") {
        let reference = reference.trim();
        // Try the ref in the git dir first, then via common dir (worktrees)
        let candidates = [git_dir.join(reference), common_dir(git_dir).join(reference)];
        for candidate in candidates {
            if let Ok(content) = std::fs::read_to_string(&candidate) {
                let content = content.trim().to_string();
                if is_commit_hash(&content) {
                    return Some(content);
                }
            }
        }
        return None;
    }

    // Detached HEAD: the file contains the hash itself
    if is_commit_hash(head) {
        return Some(head.to_string());
    }
    None
}

fn common_dir(git_dir: &std::path::Path) -> PathBuf {
    let commondir = git_dir.join("commondir");
    if let Ok(content) = std::fs::read_to_string(&commondir) {
        let relative = content.trim();
        let path = PathBuf::from(relative);
        if path.is_absolute() {
            return path;
        }
        return git_dir.join(path);
    }
    git_dir.to_path_buf()
}

fn is_commit_hash(value: &str) -> bool {
    value.len() == 40 && value.chars().all(|c| c.is_ascii_hexdigit())
}
