//! Registry of natively-linked Zellij plugins. Each entry is a `(name, factory)`
//! pair; the factory constructs a fresh boxed plugin instance.
//!
//! Adding a new native plugin:
//!   1. Implement `ZellijPlugin` on a `Default`able type in a crate that exposes
//!      `register_plugin!(MyState);`. On native targets the macro generates a
//!      `pub fn create_plugin() -> Box<dyn BoxableZellijPlugin>` factory.
//!   2. Add the crate as an optional dep on zellij-server, gated behind a
//!      Cargo feature (e.g. `native-my-plugin = ["native-plugins", "dep:my-plugin"]`).
//!   3. Add a registry entry below referencing `my_plugin::create_plugin`.

use crate::plugins::plugin_map::BoxableZellijPlugin;

/// Constructor for a native plugin instance.
pub type NativePluginFactory = fn() -> Box<dyn BoxableZellijPlugin>;

/// All native plugins available in this build. Lookup happens by name from
/// `RunPluginLocation::Native(name)` at load time.
pub static NATIVE_PLUGIN_REGISTRY: &[(&str, NativePluginFactory)] = &[
    ("native-hello", hello::create_plugin),
    #[cfg(feature = "native-status-bar")]
    ("status-bar", status_bar::create_plugin),
    #[cfg(feature = "native-tab-bar")]
    ("tab-bar", tab_bar::create_plugin),
];

pub fn factory_for(name: &str) -> Option<NativePluginFactory> {
    NATIVE_PLUGIN_REGISTRY
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, f)| *f)
}

/// The spike's trivial proof-of-life native plugin. Renders a static banner.
mod hello {
    use zellij_tile::output::println;
    use zellij_tile::prelude::*;

    #[derive(Default)]
    struct NativeHello {
        renders: usize,
    }

    impl ZellijPlugin for NativeHello {
        fn load(&mut self, _configuration: std::collections::BTreeMap<String, String>) {
            log::info!("[native-hello] loaded");
        }

        fn render(&mut self, rows: usize, cols: usize) {
            self.renders += 1;
            println!("\u{1b}[1;36mHello from native!\u{1b}[0m");
            println!("  rows={rows} cols={cols} renders={}", self.renders);
            println!("  (running as Rust code linked into the zellij binary)");
        }
    }

    register_plugin!(NativeHello);
}
