//! Which programs the firewall blocks, from the rules netSwitch and AppIntBlocker write.

use crate::rules::{application_of, RuleDirection};
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};

/// The parts of a firewall rule netSwitch reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FirewallRule {
    pub name: String,
    pub program: Option<String>,
    pub direction: RuleDirection,
    pub block: bool,
    pub enabled: bool,
}

/// An application with block rules, as the Blocked page lists it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockedApp {
    pub name: String,
    pub programs: Vec<String>,
    pub rules: u32,
    /// True when every program has both an inbound and an outbound rule.
    pub complete: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Blocklist {
    /// Lowercased program paths with an enabled block rule.
    pub programs: HashSet<String>,
    pub apps: Vec<BlockedApp>,
}

impl Blocklist {
    pub fn is_blocked(&self, lowercase_path: &str) -> bool {
        self.programs.contains(lowercase_path)
    }
}

#[derive(Default)]
struct Group {
    name: String,
    /// Program path -> (has an inbound rule, has an outbound rule).
    programs: BTreeMap<String, (bool, bool)>,
    rules: u32,
}

pub fn blocklist(rules: &[FirewallRule]) -> Blocklist {
    let mut programs = HashSet::new();
    let mut groups: BTreeMap<String, Group> = BTreeMap::new();
    for rule in rules.iter().filter(|r| r.block && r.enabled) {
        let Some(app) = application_of(&rule.name) else {
            continue;
        };
        let group = groups.entry(app.to_lowercase()).or_insert_with(|| Group {
            name: app.to_string(),
            ..Group::default()
        });
        group.rules += 1;
        if let Some(program) = rule.program.as_deref().filter(|p| !p.is_empty()) {
            programs.insert(program.to_lowercase());
            let dirs = group.programs.entry(program.to_string()).or_default();
            match rule.direction {
                RuleDirection::Inbound => dirs.0 = true,
                RuleDirection::Outbound => dirs.1 = true,
            }
        }
    }
    let apps = groups
        .into_values()
        .map(|group| BlockedApp {
            name: group.name,
            complete: !group.programs.is_empty() && group.programs.values().all(|(i, o)| *i && *o),
            programs: group.programs.into_keys().collect(),
            rules: group.rules,
        })
        .collect();
    Blocklist { programs, apps }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::rule_name;

    fn rule(app: &str, program: &str, direction: RuleDirection) -> FirewallRule {
        FirewallRule {
            name: rule_name(app, "x.exe", direction),
            program: Some(program.into()),
            direction,
            block: true,
            enabled: true,
        }
    }

    #[test]
    fn groups_rules_by_application() {
        let rules = vec![
            rule("Game", "C:\\Game\\game.exe", RuleDirection::Inbound),
            rule("Game", "C:\\Game\\game.exe", RuleDirection::Outbound),
            rule("Game", "C:\\Game\\updater.exe", RuleDirection::Outbound),
            rule("Chat", "C:\\Chat\\chat.exe", RuleDirection::Inbound),
            rule("Chat", "C:\\Chat\\chat.exe", RuleDirection::Outbound),
        ];
        let list = blocklist(&rules);
        assert_eq!(
            list.apps.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(),
            vec!["Chat", "Game"]
        );
        let game = &list.apps[1];
        assert_eq!(game.programs, vec!["C:\\Game\\game.exe", "C:\\Game\\updater.exe"]);
        assert_eq!(game.rules, 3);
        assert!(!game.complete, "updater.exe has no inbound rule");
        assert!(list.apps[0].complete);
        assert!(list.is_blocked("c:\\game\\updater.exe"));
    }

    #[test]
    fn ignores_other_rules_and_disabled_or_allowing_ones() {
        let mut allow = rule("Game", "C:\\a.exe", RuleDirection::Inbound);
        allow.block = false;
        let mut disabled = rule("Game", "C:\\b.exe", RuleDirection::Inbound);
        disabled.enabled = false;
        let other = FirewallRule {
            name: "Core Networking - DNS (UDP-Out)".into(),
            ..rule("x", "C:\\c.exe", RuleDirection::Outbound)
        };
        let list = blocklist(&[allow, disabled, other]);
        assert!(list.apps.is_empty());
        assert!(list.programs.is_empty());
    }

    #[test]
    fn names_differing_only_in_case_are_one_application() {
        let list = blocklist(&[
            rule("game", "C:\\g.exe", RuleDirection::Inbound),
            rule("Game", "C:\\g.exe", RuleDirection::Outbound),
        ]);
        assert_eq!(list.apps.len(), 1);
        assert!(list.apps[0].complete);
    }
}
