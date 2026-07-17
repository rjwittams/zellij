//! A background scanner keeps a shared snapshot of the process table so that
//! pane cwd/command discovery on the pty thread is a map lookup instead of OS
//! work. Enumeration goes through sysinfo (syscalls only — no `ps` exec, which
//! can cost >1s per spawn under macOS syspolicyd assessment).
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

const SCAN_INTERVAL: Duration = Duration::from_millis(1000);
const SLOW_SCAN_WARN_THRESHOLD: Duration = Duration::from_millis(250);
// A scan that took T is followed by a pause of at least PACING_FACTOR * T so
// a degraded system is never saturated by its own monitoring.
const PACING_FACTOR: u32 = 3;

#[derive(Debug, Clone, Default)]
pub struct ProcessInfo {
    pub ppid: Option<u32>,
    pub cmd: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub start_time: u64,
}

#[derive(Debug, Default)]
pub struct ProcessSnapshot {
    processes: HashMap<u32, ProcessInfo>,
    newest_child_by_ppid: HashMap<u32, u32>,
}

impl ProcessSnapshot {
    pub fn new(processes: HashMap<u32, ProcessInfo>) -> Self {
        let mut newest: HashMap<u32, (u64, u32)> = HashMap::new(); // ppid -> (start_time, pid)
        for (pid, info) in &processes {
            let Some(ppid) = info.ppid else { continue };
            if info.cmd.is_empty() {
                continue;
            }
            let candidate = (info.start_time, *pid);
            let entry = newest.entry(ppid).or_insert(candidate);
            if candidate > *entry {
                *entry = candidate;
            }
        }
        let newest_child_by_ppid = newest.into_iter().map(|(ppid, (_, pid))| (ppid, pid)).collect();
        ProcessSnapshot {
            processes,
            newest_child_by_ppid,
        }
    }
    /// Whether the snapshot saw this process at all (directly or through a
    /// child). A pane whose pid is absent was spawned after the scan — absence
    /// of data, not absence of a foreground command.
    pub fn knows(&self, pid: u32) -> bool {
        self.processes.contains_key(&pid) || self.newest_child_by_ppid.contains_key(&pid)
    }
    pub fn cwd(&self, pid: u32) -> Option<&PathBuf> {
        self.processes.get(&pid).and_then(|info| info.cwd.as_ref())
    }
    pub fn cmd(&self, pid: u32) -> Option<&Vec<String>> {
        self.processes
            .get(&pid)
            .filter(|info| !info.cmd.is_empty())
            .map(|info| &info.cmd)
    }
    /// The command of the most recently started child — the pane's foreground
    /// process when `pid` is a shell.
    pub fn newest_child_cmd(&self, pid: u32) -> Option<&Vec<String>> {
        self.newest_child_by_ppid
            .get(&pid)
            .and_then(|child_pid| self.cmd(*child_pid))
    }
    pub fn len(&self) -> usize {
        self.processes.len()
    }
}

static STORE: OnceLock<Mutex<Arc<ProcessSnapshot>>> = OnceLock::new();

/// The most recent snapshot. The first call starts the scanner thread; until
/// its first scan completes this returns an empty snapshot.
pub fn latest() -> Arc<ProcessSnapshot> {
    let store = STORE.get_or_init(|| {
        spawn_scanner();
        Mutex::new(Arc::new(ProcessSnapshot::default()))
    });
    store.lock().unwrap().clone()
}

fn spawn_scanner() {
    let spawned = std::thread::Builder::new()
        .name("process_scanner".into())
        .spawn(|| {
            let mut system = System::new();
            loop {
                let scan_started = Instant::now();
                let snapshot = scan(&mut system);
                let elapsed = scan_started.elapsed();
                if elapsed > SLOW_SCAN_WARN_THRESHOLD {
                    log::warn!(
                        "process table scan took {:?} ({} processes)",
                        elapsed,
                        snapshot.len()
                    );
                }
                if let Some(store) = STORE.get() {
                    *store.lock().unwrap() = Arc::new(snapshot);
                }
                std::thread::sleep(SCAN_INTERVAL.max(elapsed * PACING_FACTOR));
            }
        });
    if let Err(e) = spawned {
        log::error!("failed to spawn process scanner thread: {}", e);
    }
}

fn scan(system: &mut System) -> ProcessSnapshot {
    let refresh_kind = ProcessRefreshKind::nothing()
        .with_cwd(UpdateKind::Always)
        .with_cmd(UpdateKind::Always);
    system.refresh_processes_specifics(ProcessesToUpdate::All, true, refresh_kind);
    let mut processes = HashMap::new();
    for (pid, process) in system.processes() {
        processes.insert(
            pid.as_u32(),
            ProcessInfo {
                ppid: process.parent().map(|ppid| ppid.as_u32()),
                cmd: process
                    .cmd()
                    .iter()
                    .map(|s| s.to_string_lossy().into_owned())
                    .collect(),
                cwd: process.cwd().map(|cwd| cwd.to_path_buf()),
                start_time: process.start_time(),
            },
        );
    }
    ProcessSnapshot::new(processes)
}
