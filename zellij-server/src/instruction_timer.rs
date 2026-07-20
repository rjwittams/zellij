//! Standing latency instrumentation for the per-thread instruction loops.
//!
//! A wedge on the pty or screen thread previously left no log evidence; these
//! guards make any slow instruction (and the backlog it created) visible at
//! WARN by default.
use std::time::{Duration, Instant};

const SLOW_INSTRUCTION_WARN_THRESHOLD: Duration = Duration::from_millis(200);

/// Times the handling of one thread-loop instruction; construct after `recv`,
/// drops (and reports) on every exit path from the handling block.
pub struct InstructionTimer {
    thread: &'static str,
    instruction: String,
    queued_behind: usize,
    started: Instant,
}

impl InstructionTimer {
    pub fn new(
        thread: &'static str,
        instruction: impl std::fmt::Debug,
        queued_behind: usize,
    ) -> Self {
        InstructionTimer {
            thread,
            instruction: format!("{:?}", instruction),
            queued_behind,
            started: Instant::now(),
        }
    }
}

impl Drop for InstructionTimer {
    fn drop(&mut self) {
        let elapsed = self.started.elapsed();
        if elapsed > SLOW_INSTRUCTION_WARN_THRESHOLD {
            log::warn!(
                "{} thread: {} took {:?} ({} instructions queued behind it)",
                self.thread,
                self.instruction,
                elapsed,
                self.queued_behind,
            );
        }
    }
}
