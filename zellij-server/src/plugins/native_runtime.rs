//! Runtime support for natively-linked plugins.
//!
//! Native plugins use the same `ZellijPlugin` trait as WASM plugins, but their
//! `render()` method writes ANSI output via the `zellij_tile::println!` macro,
//! which on non-WASM targets routes to a per-thread buffer set up here.

use std::cell::RefCell;
use std::fmt;

thread_local! {
    /// Per-thread render buffer. `Some` only during a `call_render` invocation;
    /// `None` outside of one, in which case `zellij_tile::println!` on native
    /// falls back to no-op to avoid leaking to the server's real stdout.
    static RENDER_BUFFER: RefCell<Option<String>> = const { RefCell::new(None) };
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
