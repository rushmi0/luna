use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use mlua::{HookTriggers, Lua, Result as LuaResult, VmState};

/// How many VM instructions elapse between checks of the instruction/time
/// budget. Lower catches runaway scripts sooner but adds per-instruction
/// overhead (see mlua's own warning on `HookTriggers::every_nth_instruction`).
const CHECK_INTERVAL: u32 = 10_000;

/// Raised by the resource-limit hook when a script exceeds its instruction or
/// time budget. Kept distinct from a plain Lua runtime error so the host can
/// tell "the script misbehaved" apart from "the sandbox force-killed it".
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub(crate) struct ResourceLimitError(pub String);

/// Per-call instruction/time budget, reset before each top-level call and
/// enforced by a hook installed once on the `Lua` state.
pub(crate) struct ExecGuard {
    max_instructions: Option<u64>,
    timeout: Option<Duration>,
    executed: AtomicU64,
    deadline: Mutex<Option<Instant>>,
}

impl ExecGuard {
    /// Resets the budget. Call once before entering a top-level `run`/`exec`/
    /// `run_file`; the hook enforces it for the duration of that call.
    pub(crate) fn reset(&self) {
        self.executed.store(0, Ordering::Relaxed);
        *self.deadline.lock().unwrap() = self.timeout.map(|d| Instant::now() + d);
    }

    fn check(&self) -> LuaResult<VmState> {
        if let Some(max) = self.max_instructions {
            let executed = self
                .executed
                .fetch_add(u64::from(CHECK_INTERVAL), Ordering::Relaxed)
                + u64::from(CHECK_INTERVAL);
            if executed >= max {
                return Err(mlua::Error::external(ResourceLimitError(format!(
                    "script exceeded the instruction limit ({max})"
                ))));
            }
        }
        if let Some(deadline) = *self.deadline.lock().unwrap() {
            if Instant::now() >= deadline {
                return Err(mlua::Error::external(ResourceLimitError(
                    "script exceeded its execution timeout".to_string(),
                )));
            }
        }
        Ok(VmState::Continue)
    }
}

/// Installs a global instruction-count hook enforcing `max_instructions`/
/// `timeout` on every thread `lua` creates from now on (mlua re-arms the
/// hook automatically for coroutines spawned during async execution).
/// Returns `None` without touching `lua` when both limits are disabled.
pub(crate) fn install(
    lua: &Lua,
    max_instructions: Option<u64>,
    timeout: Option<Duration>,
) -> mlua::Result<Option<Arc<ExecGuard>>> {
    if max_instructions.is_none() && timeout.is_none() {
        return Ok(None);
    }
    let guard = Arc::new(ExecGuard {
        max_instructions,
        timeout,
        executed: AtomicU64::new(0),
        deadline: Mutex::new(None),
    });
    let hook_guard = Arc::clone(&guard);
    lua.set_global_hook(
        HookTriggers::new().every_nth_instruction(CHECK_INTERVAL),
        move |_, _| hook_guard.check(),
    )?;
    Ok(Some(guard))
}
