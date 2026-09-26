// Port of Init.GetPowerShell: emits the PowerShell init script with
// {{processName}} replaced by the running executable's path.
//
// The script text is checked in at init.ps1 (extracted from the C#
// implementation's output). Line endings are normalized to LF on output,
// matching the C# Init normalization, so the emitted script is byte-identical
// regardless of the working tree's checkout style. Console.WriteLine appends
// the platform's newline at the end.

const TEMPLATE: &str = include_str!("../init.ps1");

pub fn get_power_shell(process_name: &str) -> String {
    // CRLF checkouts must produce the same bytes as LF ones.
    TEMPLATE
        .replace("\r\n", "\n")
        .replace("{{processName}}", process_name)
}

/// Full init output: the script plus the trailing newline that
/// Console.WriteLine appends.
pub fn render() -> String {
    let process_name = std::env::current_exe()
        .map(|exe| exe.to_string_lossy().into_owned())
        .unwrap_or_default();
    let script = get_power_shell(&process_name);
    format!("{script}{}", crate::ansi::platform_newline())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_contains_placeholder_exactly_once() {
        assert_eq!(TEMPLATE.matches("{{processName}}").count(), 1);
    }

    #[test]
    fn replaces_process_name() {
        let script = get_power_shell("C:\\tools\\pwsh-prompt.exe");
        assert!(script.contains("Invoke-Native -Executable 'C:\\tools\\pwsh-prompt.exe'"));
        assert!(!script.contains("{{processName}}"));
    }

    #[test]
    fn script_starts_and_ends_like_the_csharp_output() {
        // The normalized script starts with a leading empty line and ends with
        // the module's closing brace followed by an LF.
        assert!(
            TEMPLATE.starts_with("\r\n# Create a new dynamic module")
                || TEMPLATE.starts_with("\n# Create a new dynamic module")
        );
        assert!(TEMPLATE.ends_with("}\r\n") || TEMPLATE.ends_with("}\n"));
        let script = get_power_shell("X");
        assert!(!script.contains("\r\n"));
        assert!(script.ends_with("}\n"));
    }
}
