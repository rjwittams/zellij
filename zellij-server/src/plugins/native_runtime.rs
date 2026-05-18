//! Runtime support for natively-linked plugins.
//!
//! Native plugins use the same `ZellijPlugin` trait as WASM plugins, but their
//! `render()` method writes ANSI output via the `zellij_tile::println!` macro,
//! which on non-WASM targets routes to a per-thread buffer set up here.
//!
//! Host calls (`subscribe`, `set_selectable`, …) are routed through a
//! `NativeBridge` installed in `zellij-tile` around each plugin entry-point
//! call. The bridge shares its stdin/stdout `VecDeque`s with the plugin's
//! `PluginEnv`, so the existing `wasi_read_bytes`/`wasi_write_object` helpers
//! work unchanged on the host side.

use std::cell::{Cell, RefCell};
use std::fmt;

use crate::plugins::plugin_map::PluginEnv;
use crate::plugins::zellij_exports::dispatch_plugin_command_from_pipe;

thread_local! {
    /// Per-thread render buffer. `Some` only during a `call_render` invocation;
    /// `None` outside of one, in which case `zellij_tile::println!` on native
    /// falls back to no-op to avoid leaking to the server's real stdout.
    static RENDER_BUFFER: RefCell<Option<String>> = const { RefCell::new(None) };

    /// Raw pointer to the current PluginEnv during a native plugin call.
    /// Set by `with_native_call`, used by the dispatcher when the plugin calls a
    /// host function via `host_run_plugin_command`.
    ///
    /// Safety: the pointer is valid for the *duration of the closure passed to
    /// `with_native_call`* — i.e., for the entire `load`/`update`/`render`/`pipe`
    /// call. The plugin can only call host functions synchronously from within
    /// these calls, so the pointer is always live when dereferenced.
    static CURRENT_ENV: Cell<*mut PluginEnv> = const { Cell::new(std::ptr::null_mut()) };
}

/// Run `f` with a freshly-installed thread-local render buffer; return whatever
/// `f` wrote via `zellij_tile::println!` / `print!`.
pub fn with_render_buffer(f: impl FnOnce()) -> String {
    RENDER_BUFFER.with(|cell| *cell.borrow_mut() = Some(String::new()));
    f();
    RENDER_BUFFER.with(|cell| cell.borrow_mut().take().unwrap_or_default())
}

/// Used by `zellij_tile::println!` / `print!` on the native target. Writes
/// formatted output into the current render buffer; silently drops if no buffer
/// is installed (i.e., outside of a `render()` call).
pub fn write_to_render_buffer(args: fmt::Arguments<'_>) {
    use std::fmt::Write as _;
    RENDER_BUFFER.with(|cell| {
        if let Some(buf) = cell.borrow_mut().as_mut() {
            let _ = buf.write_fmt(args);
        }
    });
}

/// Run `f` with the NativeBridge installed and CURRENT_ENV set, so the plugin
/// can issue host calls during the closure. Restores both on the way out, even
/// on panic.
pub fn with_native_call<R>(env: &mut PluginEnv, f: impl FnOnce() -> R) -> R {
    use zellij_tile::shim::{install_native_bridge, NativeBridge};

    struct Guard {
        prev_bridge: Option<NativeBridge>,
        prev_env: *mut PluginEnv,
    }
    impl Drop for Guard {
        fn drop(&mut self) {
            install_native_bridge(self.prev_bridge.take());
            CURRENT_ENV.with(|c| c.set(self.prev_env));
        }
    }

    let prev_bridge = install_native_bridge(Some(NativeBridge {
        stdin: env.stdin_pipe.clone(),
        stdout: env.stdout_pipe.clone(),
    }));
    let prev_env = CURRENT_ENV.with(|c| c.replace(env as *mut PluginEnv));
    let _guard = Guard { prev_bridge, prev_env };

    f()
}

/// Dispatcher function registered into zellij-tile at startup. When the plugin
/// invokes `host_run_plugin_command` from within a `with_native_call` scope,
/// this fires: it reads the encoded command bytes from the plugin's stdout
/// pipe (= NativeBridge stdout) and runs the same dispatch path as the wasmi
/// shim does.
pub fn dispatch_from_current_env() {
    CURRENT_ENV.with(|cell| {
        let env_ptr = cell.get();
        if env_ptr.is_null() {
            log::error!(
                "native dispatcher fired with no CURRENT_ENV set — \
                 plugin called a host function outside a plugin entry-point call"
            );
            return;
        }
        // Safety: with_native_call guarantees env_ptr is valid for the duration
        // of the plugin call, and host calls only happen synchronously from there.
        let env = unsafe { &mut *env_ptr };
        dispatch_plugin_command_from_pipe(env);
    });
}
