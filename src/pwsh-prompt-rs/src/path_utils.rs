// .NET System.IO.Path semantics needed by the port: GetDirectoryName,
// GetRelativePath, Join, and the Directory/File existence helpers. These are
// hand-rolled because .NET path rules differ from std::path (notably
// GetDirectoryName("C:\") returning an empty string rather than None, and
// case-insensitive comparisons on Windows).

/// Path.GetDirectoryName equivalent. Returns an empty string when there is
/// no parent directory (root paths, single relative components). The trailing
/// separator is trimmed except when the result is a root ("C:\", "/"),
/// matching .NET.
pub fn get_directory_name(path: &str) -> String {
    let trimmed = path.trim_end_matches(['/', '\\']);
    if trimmed.is_empty() {
        // The path consists only of separators.
        return String::new();
    }

    let Some(index) = trimmed.rfind(['/', '\\']) else {
        return String::new();
    };

    // Trim trailing separators unless the result would be the root.
    let mut end = index + 1;
    let root_len = root_length(trimmed);
    while end > root_len {
        let byte = trimmed.as_bytes()[end - 1];
        if byte != b'/' && byte != b'\\' {
            break;
        }
        end -= 1;
    }

    trimmed[..end].to_string()
}

/// Length of the path's root prefix: 3 for "C:\", 2 for "C:", 1 for "/" or
/// "\\", or 0 for relative paths. (UNC roots are not truncated here; their
/// leading separators make root_length 1, which only affects trailing-
/// separator trimming at the share boundary.)
fn root_length(path: &str) -> usize {
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' {
        return if bytes.len() >= 3 && (bytes[2] == b'/' || bytes[2] == b'\\') {
            3
        } else {
            2
        };
    }
    if bytes.first().is_some_and(|b| *b == b'/' || *b == b'\\') {
        1
    } else {
        0
    }
}

/// Path.Join(path, ".git") equivalent: concatenates directly when the path
/// already ends with a separator.
pub fn join_git(path: &str) -> String {
    let separator = std::path::MAIN_SEPARATOR;
    if path.ends_with('/') || path.ends_with('\\') || path.ends_with(separator) {
        format!("{path}.git")
    } else {
        format!("{path}{separator}.git")
    }
}

/// Path.Combine(parent, "config") equivalent (parent never empty in practice).
pub fn join_config(parent: &str) -> String {
    let separator = std::path::MAIN_SEPARATOR;
    if parent.ends_with('/') || parent.ends_with('\\') || parent.ends_with(separator) {
        format!("{parent}config")
    } else {
        format!("{parent}{separator}config")
    }
}

pub fn directory_exists(path: &str) -> bool {
    std::fs::metadata(path).map(|m| m.is_dir()).unwrap_or(false)
}

pub fn file_exists(path: &str) -> bool {
    std::fs::metadata(path)
        .map(|m| m.is_file())
        .unwrap_or(false)
}

/// Path.GetRelativePath(relativeTo, path) equivalent.
///
/// Returns "." when the paths are equal; joins the path's trailing components
/// onto relativeTo when relativeTo is a prefix (case-insensitively on
/// Windows); prefixes with ".." per leftover component otherwise. When the
/// roots differ (different drives on Windows), returns the path unchanged,
/// matching the .NET implementation.
pub fn get_relative_path(relative_to: &str, path: &str) -> String {
    let separator = std::path::MAIN_SEPARATOR;

    let (relative_root, relative_rest) = split_root(relative_to);
    let (path_root, path_rest) = split_root(path);

    let ignore_case = cfg!(windows);
    let roots_equal = if ignore_case {
        relative_root.eq_ignore_ascii_case(path_root)
    } else {
        relative_root == path_root
    };
    if !roots_equal {
        return path.to_string();
    }

    let relative_components: Vec<&str> = relative_rest
        .split(['/', '\\'])
        .filter(|s| !s.is_empty())
        .collect();
    let path_components: Vec<&str> = path_rest
        .split(['/', '\\'])
        .filter(|s| !s.is_empty())
        .collect();

    let mut common = 0;
    while common < relative_components.len()
        && common < path_components.len()
        && components_equal(
            relative_components[common],
            path_components[common],
            ignore_case,
        )
    {
        common += 1;
    }

    let remaining_relative = &relative_components[common..];
    let remaining_path = &path_components[common..];

    if remaining_path.is_empty() {
        return ".".to_string();
    }

    let mut result = String::new();
    for _ in 0..remaining_relative.len() {
        result.push_str("..");
        result.push(separator);
    }
    result.push_str(&remaining_path.join(&separator.to_string()));
    result
}

fn components_equal(a: &str, b: &str, ignore_case: bool) -> bool {
    if ignore_case {
        a.eq_ignore_ascii_case(b)
    } else {
        a == b
    }
}

/// Splits a path into (root, remaining). Roots handled: drive ("C:" /
/// "C:\"), UNC ("\\server\share"), single leading separator ("/" or "\"),
/// and none (relative path).
fn split_root(path: &str) -> (&str, &str) {
    let bytes = path.as_bytes();

    // UNC: starts with two separators
    if bytes.len() >= 2 && bytes[0] == b'\\' && bytes[1] == b'\\' {
        // \\server\share[...]
        let mut separators = 0;
        let mut index = 2;
        while index < bytes.len() {
            if bytes[index] == b'\\' || bytes[index] == b'/' {
                separators += 1;
                if separators == 2 {
                    return (&path[..=index], &path[index + 1..]);
                }
            }
            index += 1;
        }
        return (path, "");
    }

    // Drive: letter + ':'
    if bytes.len() >= 2 && bytes[1] == b':' {
        if bytes.len() >= 3 && (bytes[2] == b'\\' || bytes[2] == b'/') {
            return (&path[..3], &path[3..]);
        }
        return (&path[..2], &path[2..]);
    }

    if bytes.first().is_some_and(|b| *b == b'/' || *b == b'\\') {
        return (&path[..1], &path[1..]);
    }

    ("", path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn directory_name_windows() {
        assert_eq!(get_directory_name(r"C:\Users\lucas"), r"C:\Users");
        assert_eq!(get_directory_name(r"C:\Users"), r"C:\");
        assert_eq!(get_directory_name(r"C:\"), "");
        assert_eq!(get_directory_name(r"C:"), "");
        assert_eq!(
            get_directory_name(r"D:\source\repo\.git"),
            r"D:\source\repo"
        );
        assert_eq!(get_directory_name("foo"), "");
        assert_eq!(get_directory_name(r"C:\foo\"), r"C:\");
    }

    #[cfg(not(windows))]
    #[test]
    fn directory_name_unix() {
        assert_eq!(get_directory_name("/home/lucas"), "/home");
        assert_eq!(get_directory_name("/home"), "/");
        assert_eq!(get_directory_name("/"), "");
        assert_eq!(get_directory_name("foo"), "");
    }

    #[cfg(windows)]
    #[test]
    fn relative_path_windows() {
        assert_eq!(
            get_relative_path(
                r"D:\source\lucaspimentel",
                r"D:\source\lucaspimentel\pwsh-prompt\src"
            ),
            r"pwsh-prompt\src"
        );
        assert_eq!(
            get_relative_path(r"D:\source\lucaspimentel", r"D:\source\lucaspimentel"),
            "."
        );
        // Different roots: .NET returns the target path unchanged.
        assert_eq!(
            get_relative_path(
                r"D:\source\lucaspimentel",
                r"C:\Users\lucas\.nuget\packages"
            ),
            r"C:\Users\lucas\.nuget\packages"
        );
        // Case-insensitive prefix.
        assert_eq!(
            get_relative_path(r"D:\Source", r"d:\source\repo\x"),
            r"repo\x"
        );
        // Mid-component mismatch (src vs src2).
        assert_eq!(
            get_relative_path(r"D:\src", r"D:\src2\file.txt"),
            r"..\src2\file.txt"
        );
        // Root as base.
        assert_eq!(get_relative_path(r"C:\", r"C:\repo\src"), r"repo\src");
    }

    #[cfg(not(windows))]
    #[test]
    fn relative_path_unix() {
        assert_eq!(
            get_relative_path("/home/lucas", "/home/lucas/repo/src"),
            "repo/src"
        );
        assert_eq!(get_relative_path("/home/lucas", "/home/lucas"), ".");
        assert_eq!(get_relative_path("/home/lucas", "/etc"), "../../etc");
    }

    #[test]
    fn join_respects_trailing_separator() {
        #[cfg(windows)]
        {
            assert_eq!(join_git(r"C:\"), r"C:\.git");
            assert_eq!(join_git(r"C:\Users\lucas"), r"C:\Users\lucas\.git");
            assert_eq!(join_config(r"D:\repo\.git"), r"D:\repo\.git\config");
        }
        #[cfg(not(windows))]
        {
            assert_eq!(join_git("/"), "/.git");
            assert_eq!(join_git("/home/lucas"), "/home/lucas/.git");
        }
    }

    #[test]
    fn split_root_cases() {
        #[cfg(windows)]
        {
            assert_eq!(split_root(r"C:\Users\x"), (r"C:\", r"Users\x"));
            assert_eq!(split_root(r"C:"), ("C:", ""));
            assert_eq!(split_root(r"relative\path"), ("", r"relative\path"));
        }
        assert_eq!(split_root("/usr/bin"), ("/", "usr/bin"));
        assert_eq!(split_root("usr/bin"), ("", "usr/bin"));
    }
}
