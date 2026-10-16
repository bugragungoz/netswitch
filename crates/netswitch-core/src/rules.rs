//! Firewall rule names. They follow AppIntBlocker (the PowerShell script and its window) exactly,
//! so netSwitch, the script and the window see and remove each other's rules:
//! `AppBlocker Rule -{application} - {file} (Inbound|Outbound)`.

pub const PREFIX: &str = "AppBlocker Rule -";

// AppIntBlocker cuts the part before the direction at 220 characters and appends "...".
const MAX_BASE_LEN: usize = 220;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleDirection {
    Inbound,
    Outbound,
}

impl RuleDirection {
    pub const BOTH: [RuleDirection; 2] = [RuleDirection::Inbound, RuleDirection::Outbound];

    fn label(self) -> &'static str {
        match self {
            Self::Inbound => "Inbound",
            Self::Outbound => "Outbound",
        }
    }
}

/// The rule name for one file of an application.
pub fn rule_name(application: &str, file: &str, direction: RuleDirection) -> String {
    let mut base = format!("{PREFIX}{application} - {file}");
    if base.chars().count() > MAX_BASE_LEN {
        base = base.chars().take(MAX_BASE_LEN).collect::<String>() + "...";
    }
    format!("{base} ({})", direction.label())
}

/// The application part of a rule name, or None for a name AppIntBlocker or netSwitch did not write.
/// Like the script, it ends at the first " -" after the prefix.
pub fn application_of(rule_name: &str) -> Option<&str> {
    let rest = rule_name.strip_prefix(PREFIX)?;
    let end = rest.find(" -")?;
    let app = rest[..end].trim_end();
    (!app.is_empty()).then_some(app)
}

/// The application name netSwitch writes for a program: its name, without the " -" that would end
/// the application part early, and without characters a rule name should not hold.
pub fn application_name(display_name: &str) -> String {
    let cleaned: String = display_name
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect::<String>()
        .replace(" -", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let cleaned = cleaned.trim_matches('-').trim().to_string();
    if cleaned.is_empty() {
        "App".to_string()
    } else {
        cleaned.chars().take(80).collect()
    }
}

/// The file part netSwitch writes: the program's file name.
pub fn file_name(program_path: &str) -> &str {
    program_path.rsplit(['\\', '/']).next().unwrap_or(program_path)
}

/// True for programs inside the Windows folder, such as svchost.exe, which runs the DNS client and
/// Windows Update among many services: blocking one can cut off the whole computer. netSwitch
/// does not block them.
pub fn is_protected(program_path: &str, system_root: &str) -> bool {
    let root = system_root.trim_end_matches(['\\', '/']);
    if root.is_empty() {
        return false;
    }
    let path = program_path.to_lowercase().replace('/', "\\");
    let root = root.to_lowercase().replace('/', "\\");
    path.strip_prefix(&root).is_some_and(|rest| rest.starts_with('\\'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_match_appintblocker() {
        assert_eq!(
            rule_name("Example", "bin\\example.exe", RuleDirection::Outbound),
            "AppBlocker Rule -Example - bin\\example.exe (Outbound)"
        );
        assert_eq!(
            rule_name("Example", "example.exe", RuleDirection::Inbound),
            "AppBlocker Rule -Example - example.exe (Inbound)"
        );
    }

    #[test]
    fn long_names_are_cut_like_the_script_does() {
        let name = rule_name("A", &"x".repeat(300), RuleDirection::Inbound);
        assert!(name.ends_with("... (Inbound)"));
        assert_eq!(name.chars().count(), 220 + "... (Inbound)".len());
    }

    #[test]
    fn application_ends_at_the_first_dash() {
        assert_eq!(
            application_of("AppBlocker Rule -Example - bin\\a.exe (Inbound)"),
            Some("Example")
        );
        assert_eq!(
            application_of("AppBlocker Rule -My App - a.exe (Outbound)"),
            Some("My App")
        );
        assert_eq!(application_of("Some other rule"), None);
        assert_eq!(application_of("AppBlocker Rule - - a.exe (Outbound)"), None);
    }

    #[test]
    fn application_names_cannot_end_the_application_part_early() {
        assert_eq!(application_name("Foo - Bar"), "Foo Bar");
        assert_eq!(application_name("  Google Chrome  "), "Google Chrome");
        assert_eq!(application_name("-"), "App");
        assert_eq!(application_name("a\tb"), "a b");
        let written = rule_name(&application_name("Foo - Bar"), "foo.exe", RuleDirection::Inbound);
        assert_eq!(application_of(&written), Some("Foo Bar"));
    }

    #[test]
    fn programs_in_the_windows_folder_are_protected() {
        assert!(is_protected("C:\\Windows\\System32\\svchost.exe", "C:\\Windows"));
        assert!(is_protected("c:\\windows\\explorer.exe", "C:\\WINDOWS\\"));
        assert!(!is_protected("C:\\WindowsApps\\app.exe", "C:\\Windows"));
        assert!(!is_protected("C:\\Program Files\\App\\app.exe", "C:\\Windows"));
        assert!(!is_protected("C:\\Windows\\System32\\svchost.exe", ""));
    }

    #[test]
    fn file_name_is_the_last_part_of_the_path() {
        assert_eq!(file_name("C:\\Program Files\\App\\app.exe"), "app.exe");
        assert_eq!(file_name("app.exe"), "app.exe");
    }
}
