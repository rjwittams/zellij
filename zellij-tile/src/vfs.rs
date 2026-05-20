//! Translation between plugin-visible "virtual" paths (`/host/...`, `/data/...`,
//! `/cache/...`, `/tmp/...`) and real filesystem paths.
//!
//! Two abstractions live here:
//!
//! - [`translate_plugin_path`] is the low-level primitive: input is a
//!   plugin-visible path with explicit WASI-style prefix (`/host/...`). On
//!   wasm it's the identity (WASI handles translation downstream); on native
//!   it rewrites `/host/...` to the cached host-folder mount. Use this when
//!   you already have a WASI-namespace path.
//!
//! - [`resolve_host_path`] is the higher-level convenience: input is a clean
//!   host-filesystem path the plugin's user typed in plugin config. It
//!   strips an optional `file:` prefix, expands env vars, and uses the
//!   cached host-folder mount to produce a path `std::fs` can read on either
//!   target. Use this for plugin-config string values that name a file on
//!   the host.
//!
//! Both rely on the host folder cache populated by `change_host_folder` —
//! plugins should call that once at startup with the mount root they want.
//!
//! Caveat: this is a half-VFS — it only translates paths. Anything that
//! depended on WASI's broader sandbox semantics (capability checks, virtual
//! files) is not modelled.

use std::cell::RefCell;
use std::path::{Path, PathBuf};

thread_local! {
    /// Mirror of the host-side mount the plugin most recently set via
    /// `change_host_folder`. `None` means the plugin has not configured a
    /// mount, in which case helpers behave as if the mount were `/`.
    static HOST_FOLDER: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// Called by the `change_host_folder` shim to record the plugin's current
/// host mount. Pure cache: the host-side instruction still runs through the
/// normal dispatch path so the runtime stays in sync.
pub fn set_host_folder(folder: PathBuf) {
    HOST_FOLDER.with(|cell| *cell.borrow_mut() = Some(folder));
}

/// The currently-mounted host folder, or `/` if `change_host_folder` was
/// never called.
pub fn host_folder() -> PathBuf {
    HOST_FOLDER.with(|cell| {
        cell.borrow()
            .clone()
            .unwrap_or_else(|| PathBuf::from("/"))
    })
}

/// Expand `~` and `$VAR`/`${VAR}` against the plugin's environment. Zellij
/// seeds the WASI ctx via `inherit_env()`, so this works on both native and
/// wasm. On expansion failure the original string is returned unchanged.
///
/// This is the right thing to call on user-supplied path strings (e.g. plugin
/// configuration values) at the point you read them out of `configuration`.
/// Translation through [`translate_plugin_path`] or [`resolve_host_path`] is
/// a separate step, applied at the point you actually hand the path to
/// `std::fs`, because the host folder mount can change at runtime.
pub fn expand_env(s: &str) -> String {
    match shellexpand::full(s) {
        Ok(cow) => cow.into_owned(),
        Err(_) => s.to_owned(),
    }
}

/// Legacy one-shot convenience: `expand_env` followed by `translate_plugin_path`.
/// New code should prefer [`resolve_host_path`], which handles the optional
/// `file:` prefix and works regardless of where `change_host_folder` mounted
/// the host root.
pub fn resolve_path(s: &str) -> PathBuf {
    translate_plugin_path(expand_env(s))
}

/// Resolve a user-supplied host-filesystem path into a path that can be
/// handed to `std::fs` on the current target.
///
/// Accepts:
/// - Optional leading `file:` prefix.
/// - `~` and `$VAR`/`${VAR}` references (expanded via the plugin's
///   environment).
/// - Absolute paths on the host filesystem.
///
/// Behaviour:
/// - On native: returns the (expanded) path as-is; native `std::fs` reads it
///   directly from the host filesystem.
/// - On wasm: rewrites the path so WASI can map it through whatever the
///   plugin most recently set via `change_host_folder`. If the path lies
///   outside the mounted host folder it is unreachable from the plugin
///   sandbox and this returns [`Err(PathOutsideHostFolder)`].
///
/// Plugins should call `change_host_folder` once at startup so the cache
/// reflects the mount they want. Path resolution will fall back to a `/`
/// mount if you don't.
pub fn resolve_host_path(s: &str) -> Result<PathBuf, PathOutsideHostFolder> {
    let stripped = s.strip_prefix("file:").unwrap_or(s);
    let expanded = expand_env(stripped);
    let path = PathBuf::from(expanded);

    #[cfg(target_family = "wasm")]
    {
        let mount = host_folder();
        match path.strip_prefix(&mount) {
            Ok(rel) => Ok(PathBuf::from("/host").join(rel)),
            Err(_) => Err(PathOutsideHostFolder {
                path,
                host_folder: mount,
            }),
        }
    }
    #[cfg(not(target_family = "wasm"))]
    {
        Ok(path)
    }
}

/// Returned by [`resolve_host_path`] when the user-supplied path is not
/// reachable through the plugin's current host-folder mount. Only ever
/// produced on wasm.
#[derive(Debug, Clone)]
pub struct PathOutsideHostFolder {
    pub path: PathBuf,
    pub host_folder: PathBuf,
}

impl std::fmt::Display for PathOutsideHostFolder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "path {} is outside the plugin's host folder mount ({})",
            self.path.display(),
            self.host_folder.display()
        )
    }
}

impl std::error::Error for PathOutsideHostFolder {}

/// Convert a plugin-visible WASI-style path to the real path used by `std::fs`.
///
/// - On wasm: identity (WASI handles translation downstream).
/// - On native: rewrites paths that start with `/host/...` to the
///   corresponding host-side path using the cached host-folder mount. Paths
///   that don't start with `/host` are returned unchanged (and may fail
///   loudly at `std::fs` time if they target `/data`, `/cache`, `/tmp`,
///   etc. — those aren't modelled yet).
pub fn translate_plugin_path(path: impl AsRef<Path>) -> PathBuf {
    #[cfg(target_family = "wasm")]
    {
        path.as_ref().to_path_buf()
    }
    #[cfg(not(target_family = "wasm"))]
    {
        let path = path.as_ref();
        if let Ok(stripped) = path.strip_prefix("/host") {
            host_folder().join(stripped)
        } else {
            path.to_path_buf()
        }
    }
}
