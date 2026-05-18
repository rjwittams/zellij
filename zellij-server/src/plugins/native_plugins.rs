//! Registry of natively-linked Zellij plugins. Each entry is a `(name, factory)`
//! pair; the factory constructs a fresh boxed plugin instance.
//!
//! Adding a new native plugin: implement `ZellijPlugin` on a `Default`able type,
//! then add a registry entry below (optionally gated behind a Cargo feature).

use crate::plugins::plugin_map::BoxableZellijPlugin;

/// Constructor for a native plugin instance.
pub type NativePluginFactory = fn() -> Box<dyn BoxableZellijPlugin>;

/// All native plugins available in this build. Lookup happens by name from
/// `RunPluginLocation::Native(name)` at load time.
pub static NATIVE_PLUGIN_REGISTRY: &[(&str, NativePluginFactory)] = &[
    ("native-hello", || Box::new(hello::NativeHello::default())),
];

pub fn factory_for(name: &str) -> Option<NativePluginFactory> {
    NATIVE_PLUGIN_REGISTRY
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, f)| *f)
}

/// The spike's trivial proof-of-life native plugin. Renders a static banner.
mod hello {
    use std::collections::BTreeMap;
    use zellij_tile::ZellijPlugin;
    use zellij_utils::data::{Event, PipeMessage};

    #[derive(Default)]
    pub struct NativeHello {
        renders: usize,
    }

    impl ZellijPlugin for NativeHello {
        fn load(&mut self, _configuration: BTreeMap<String, String>) {
            log::info!("[native-hello] loaded");
        }

        fn update(&mut self, _event: Event) -> bool {
            // Not subscribed to anything yet, so this is unreachable in practice.
            false
        }

        fn pipe(&mut self, _pipe_message: PipeMessage) -> bool {
            false
        }

        fn render(&mut self, rows: usize, cols: usize) {
            self.renders += 1;
            zellij_tile::println!("\u{1b}[1;36mHello from native!\u{1b}[0m");
            zellij_tile::println!("  rows={rows} cols={cols} renders={}", self.renders);
            zellij_tile::println!("  (running as Rust code linked into the zellij binary)");
        }
    }
}
