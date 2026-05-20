//! Translation between plugin-visible "virtual" paths (`/host/...`, `/data/...`,
//! `/cache/...`, `/tmp/...`) and real filesystem paths.
//!
//! On the WASM target the WASI runtime handles this transparently via preopens;
//! [`translate_plugin_path`] is the identity function. On native targets there
//! is no WASI in between the plugin and the kernel, so direct `std::fs` calls
//! with WASI-style paths would hit the literal `/host/...` directory on the
//! real filesystem and fail. Plugins that want to be native-eligible should
//! pipe paths through this function before handing them to `std::fs`.
//!
//! Caveat: this is a half-VFS — it only translates paths. Anything that
//! depended on WASI's broader sandbox semantics (capability checks, virtual
//! files) is not modelled.

use std::path::{Path, PathBuf};

/// Expand `~` and `$VAR`/`${VAR}` against the plugin's environment. Zellij
/// seeds the WASI ctx via `inherit_env()`, so this works on both native and
/// wasm. On expansion failure the original string is returned unchanged.
///
/// This is the right thing to call on user-supplied path strings (e.g. plugin
/// configuration values) at the point you read them out of `configuration`.
/// Translation through [`translate_plugin_path`] is a separate step, applied
/// at the point you actually hand the path to `std::fs`, because the host
/// folder mount can change at runtime.
pub fn expand_env(s: &str) -> String {
    match shellexpand::full(s) {
        Ok(cow) => cow.into_owned(),
        Err(_) => s.to_owned(),
    }
}

/// Convenience for one-shot use: `expand_env` followed by `translate_plugin_path`.
pub fn resolve_path(s: &str) -> PathBuf {
    translate_plugin_path(expand_env(s))
}

/// Convert a plugin-visible path to the real path used by `std::fs`.
///
/// - On wasm: identity (WASI handles translation downstream).
/// - On native: rewrites paths that start with `/host/`, `/data/`, etc. to the
///   corresponding host-side directories. Paths that don't match any virtual
///   prefix are returned unchanged.
pub fn translate_plugin_path(path: impl AsRef<Path>) -> PathBuf {
    #[cfg(target_family = "wasm")]
    {
        path.as_ref().to_path_buf()
    }
    #[cfg(not(target_family = "wasm"))]
    {
        native::translate(path.as_ref())
    }
}

#[cfg(not(target_family = "wasm"))]
pub mod native {
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};

    thread_local! {
        /// Mirror of the host-side mount the plugin set via `change_host_folder`.
        /// Defaults to `None`; if unset when a `/host/...` path is translated,
        /// the prefix is stripped and the remainder used as-is (i.e. as if the
        /// host root was mounted at `/`).
        static HOST_FOLDER: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    }

    /// Called by the `change_host_folder` shim to record the plugin's current
    /// host mount. Pure cache: the host-side instruction still runs through the
    /// normal dispatch path so the wasm-side runtime stays in sync.
    pub fn set_host_folder(folder: PathBuf) {
        HOST_FOLDER.with(|cell| *cell.borrow_mut() = Some(folder));
    }

    pub fn translate(path: &Path) -> PathBuf {
        if let Ok(stripped) = path.strip_prefix("/host") {
            HOST_FOLDER.with(|cell| {
                let folder = cell
                    .borrow()
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("/"));
                folder.join(stripped)
            })
        } else {
            // /data, /cache, /tmp aren't supported on native yet — would need
            // the host to communicate the per-plugin directories at load time.
            // Until then, pass through and let the syscall fail loudly.
            path.to_path_buf()
        }
    }
}
