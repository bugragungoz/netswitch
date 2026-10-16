//! Which app a process belongs to. Reading a program path and its description costs system calls,
//! so answers are kept: per process for a minute (process ids are reused, but rarely that fast),
//! per program for the whole run.

use crate::model::AppIdentity;
use std::collections::HashMap;

/// How long a process id keeps its answer.
pub const PROCESS_TTL_MS: u64 = 60_000;

pub struct AppResolver<P, D>
where
    P: FnMut(u32) -> Option<String>,
    D: FnMut(&str) -> Option<String>,
{
    program_path: P,
    description: D,
    processes: HashMap<u32, (AppIdentity, u64)>,
    descriptions: HashMap<String, Option<String>>,
}

impl<P, D> AppResolver<P, D>
where
    P: FnMut(u32) -> Option<String>,
    D: FnMut(&str) -> Option<String>,
{
    /// `program_path`: the full path of a process's program, if readable.
    /// `description`: the file description of a program, if it has one.
    pub fn new(program_path: P, description: D) -> Self {
        Self {
            program_path,
            description,
            processes: HashMap::new(),
            descriptions: HashMap::new(),
        }
    }

    pub fn identity(&mut self, pid: u32, now: u64) -> AppIdentity {
        if let Some((identity, at)) = self.processes.get(&pid) {
            if now.saturating_sub(*at) < PROCESS_TTL_MS {
                return identity.clone();
            }
        }
        let identity = match (self.program_path)(pid) {
            Some(path) => {
                let key = path.to_lowercase();
                let description = match self.descriptions.get(&key) {
                    Some(known) => known.clone(),
                    None => {
                        let found = (self.description)(&path);
                        self.descriptions.insert(key, found.clone());
                        found
                    }
                };
                AppIdentity::from_path(&path, description)
            }
            None => AppIdentity::unknown(pid),
        };
        self.processes.insert(pid, (identity.clone(), now));
        identity
    }

    /// Drops answers older than the time to live, so the map does not grow with every process.
    pub fn prune(&mut self, now: u64) {
        self.processes
            .retain(|_, (_, at)| now.saturating_sub(*at) < PROCESS_TTL_MS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn answers_are_kept_and_renewed() {
        let path_calls = Cell::new(0);
        let description_calls = Cell::new(0);
        let mut resolver = AppResolver::new(
            |pid| {
                path_calls.set(path_calls.get() + 1);
                (pid == 10 || pid == 11).then(|| "C:\\Chrome\\chrome.exe".to_string())
            },
            |_| {
                description_calls.set(description_calls.get() + 1);
                Some("Google Chrome".to_string())
            },
        );
        assert_eq!(resolver.identity(10, 0).name, "Google Chrome");
        assert_eq!(resolver.identity(10, 1_000).name, "Google Chrome");
        assert_eq!(path_calls.get(), 1, "the process is asked once within the time to live");
        resolver.identity(11, 1_000);
        assert_eq!(description_calls.get(), 1, "the program's description is read once");
        resolver.identity(10, PROCESS_TTL_MS + 1);
        assert_eq!(path_calls.get(), 3, "an old answer is asked again");
    }

    #[test]
    fn unreadable_processes_get_their_own_app() {
        let mut resolver = AppResolver::new(|_| None, |_| None);
        let system = resolver.identity(4, 0);
        assert_eq!(system.key, "pid:4");
        assert_eq!(system.name, "System");
        assert_eq!(resolver.identity(1234, 0).name, "Process 1234");
    }

    #[test]
    fn prune_forgets_old_processes() {
        let mut resolver = AppResolver::new(|_| Some("C:\\a.exe".to_string()), |_| None);
        resolver.identity(1, 0);
        resolver.prune(PROCESS_TTL_MS + 1);
        assert!(resolver.processes.is_empty());
    }
}
