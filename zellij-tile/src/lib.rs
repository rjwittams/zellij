//! The zellij-tile crate acts as the Rust API for developing plugins for Zellij.
//!
//! To read more about Zellij plugins:
//! [https://zellij.dev/documentation/plugins](https://zellij.dev/documentation/plugins)
//!
//! ### Interesting things in this libary:
//! - The [`ZellijPlugin`] trait for implementing plugins combined with the
//! [`register_plugin!`](register_plugin) macro to register them.
//! - The list of [commands](shim) representing what a plugin can do.
//! - The list of [`Events`](prelude::Event) a plugin can subscribe to
//! - The [`ZellijWorker`] trait for implementing background workers combined with the
//! [`register_worker!`](register_worker) macro to register them
//!
//! ### Full Example and Development Environment
//! For a working plugin example as well as a development environment, please see:
//! [https://github.com/zellij-org/rust-plugin-example](https://github.com/zellij-org/rust-plugin-example)
//!
pub mod prelude;
pub mod shim;
pub mod ui_components;
pub mod vfs;

/// Cross-target render-output macros.
///
/// Plugin code that wants to compile on both WASM and native targets should
/// `use zellij_tile::output::{print, println};` and then use the bare names.
/// These shadow `std`'s `print!`/`println!` in the importing module's scope:
///
/// - On **WASM**, they pass through to `std::print!` / `std::println!`, which
///   write to the plugin's stdio — the historical channel the host reads as
///   render output.
///
/// - On **native** (plugin linked into the zellij binary), they write into a
///   per-thread buffer installed by the host around each `render()` call.
///   Standard `std::println!` would dump to the server's real stdout, so
///   plugins that want native compat must opt in via this import.
///
/// Outside a `render()` call on native, output is silently dropped.
pub mod output {
    #[cfg(target_family = "wasm")]
    pub use ::std::{print, println};

    #[cfg(not(target_family = "wasm"))]
    pub use crate::{print, println};
}

#[cfg(not(target_family = "wasm"))]
#[doc(hidden)]
pub use shim::__native_print;

#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! println {
    () => { $crate::__native_print(::std::format_args!("\n")) };
    ($($arg:tt)*) => {
        $crate::__native_print(::std::format_args!("{}\n", ::std::format_args!($($arg)*)))
    };
}

#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::__native_print(::std::format_args!($($arg)*))
    };
}
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use zellij_utils::data::{Event, PipeMessage};

// use zellij_tile::shim::plugin_api::event::ProtobufEvent;

/// This trait should be implemented - once per plugin - on a struct (normally representing the
/// plugin state). This struct should then be registered with the
/// [`register_plugin!`](register_plugin) macro.
#[allow(unused_variables)]
pub trait ZellijPlugin: Default {
    /// Will be called when the plugin is loaded, this is a good place to [`subscribe`](shim::subscribe) to events that are interesting for this plugin.
    fn load(&mut self, configuration: BTreeMap<String, String>) {}
    /// Will be called with an [`Event`](prelude::Event) if the plugin is subscribed to said event.
    /// If the plugin returns `true` from this function, Zellij will know it should be rendered and call its `render` function.
    fn update(&mut self, event: Event) -> bool {
        false
    } // return true if it should render
    /// Will be called when data is being piped to the plugin, a PipeMessage.payload of None signifies the pipe
    /// has ended
    /// If the plugin returns `true` from this function, Zellij will know it should be rendered and call its `render` function.
    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        false
    } // return true if it should render
    /// Will be called either after an `update` that requested it, or when the plugin otherwise needs to be re-rendered (eg. on startup, or when the plugin is resized).
    /// The `rows` and `cols` values represent the "content size" of the plugin (this will not include its surrounding frame if the user has pane frames enabled).
    fn render(&mut self, rows: usize, cols: usize) {}
}

/// Object-safe wrapper around [`ZellijPlugin`]. Needed because `ZellijPlugin: Default`
/// is not dyn-compatible (constructor in trait bound), so the host cannot hold
/// `Box<dyn ZellijPlugin>` directly. A blanket impl makes any `T: ZellijPlugin + Send + 'static`
/// usable through this trait, so plugin authors keep implementing [`ZellijPlugin`] as
/// before and don't see this type unless they're writing a `create_plugin` factory.
pub trait BoxableZellijPlugin: Send {
    fn load(&mut self, configuration: BTreeMap<String, String>);
    fn update(&mut self, event: Event) -> bool;
    fn pipe(&mut self, pipe_message: PipeMessage) -> bool;
    fn render(&mut self, rows: usize, cols: usize);
}

impl<T: ZellijPlugin + Send + 'static> BoxableZellijPlugin for T {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        <Self as ZellijPlugin>::load(self, configuration)
    }
    fn update(&mut self, event: Event) -> bool {
        <Self as ZellijPlugin>::update(self, event)
    }
    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        <Self as ZellijPlugin>::pipe(self, pipe_message)
    }
    fn render(&mut self, rows: usize, cols: usize) {
        <Self as ZellijPlugin>::render(self, rows, cols)
    }
}

/// This trait is used to create workers. Workers can be used by plugins to run longer running
/// background tasks without blocking their own rendering (eg. and showing some sort of loading
/// indication in part of the UI as needed while waiting for the task to complete).
///
/// ## Starting workers on plugin load
/// Implement this trait on a struct (typically representing the worker state) and register it with
/// the [`register_worker!`](register_worker) macro.
///
/// ## Sending messages to workers and back to the plugin
/// Send messages to workers with the [`post_message_to`](shim::post_message_to) method.
/// Send messages from workers back to plugins with the
/// [`post_message_to_plugin`](shim::post_message_to_plugin) method (but be sure the plugin has
/// [`subscribe`](shim::subscribe)d to the [`CustomMessage`](prelude::Event::CustomMessage)) event
/// first!
#[allow(unused_variables)]
pub trait ZellijWorker<'de>: Default + Serialize + Deserialize<'de> {
    /// Triggered whenever the plugin sends the worker a message using the
    /// [`post_message_to`](shim::post_message_to) method.
    fn on_message(&mut self, message: String, payload: String) {}
}

pub const PLUGIN_MISMATCH: &str =
    "An error occured in a plugin while receiving an Event from zellij. This means
that the plugins aren't compatible with the current zellij version.

The most likely explanation for this is that you're running either a
self-compiled zellij or plugin version. Please make sure that, while developing,
you also rebuild the plugins in order to pick up changes to the plugin code.

Please refer to the documentation for further information:
    https://github.com/zellij-org/zellij/blob/main/CONTRIBUTING.md#building
";

/// Used to register a plugin implementing the [`ZellijPlugin`] trait.
///
/// eg.
/// ```rust
/// use zellij_tile::prelude::*;
///
/// #[derive(Default)]
/// pub struct MyPlugin {}
///
/// impl ZellijPlugin for MyPlugin {
///    // ...
/// }
///
/// register_plugin!(MyPlugin);
/// ```
#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! register_plugin {
    ($t:ty) => {
        thread_local! {
            static STATE: std::cell::RefCell<$t> = std::cell::RefCell::new(Default::default());
        }

        fn main() {
            // Register custom panic handler
            std::panic::set_hook(Box::new(|info| {
                report_panic(info);
            }));
        }

        #[no_mangle]
        fn load() {
            STATE.with(|state| {
                use std::collections::BTreeMap;
                use std::convert::TryFrom;
                use std::convert::TryInto;
                use zellij_tile::shim::plugin_api::action::ProtobufPluginConfiguration;
                use zellij_tile::shim::prost::Message;
                let protobuf_bytes: Vec<u8> = $crate::shim::object_from_stdin().unwrap();
                let protobuf_configuration: ProtobufPluginConfiguration =
                    ProtobufPluginConfiguration::decode(protobuf_bytes.as_slice()).unwrap();
                let plugin_configuration: BTreeMap<String, String> =
                    BTreeMap::try_from(&protobuf_configuration).unwrap();
                // UFCS so it picks ZellijPlugin::load specifically, not the
                // blanket BoxableZellijPlugin::load (which exists for native dispatch).
                <$t as $crate::ZellijPlugin>::load(
                    &mut *state.borrow_mut(),
                    plugin_configuration,
                );
            });
        }

        #[no_mangle]
        pub fn update() -> bool {
            let err_context = "Failed to deserialize event";
            use std::convert::TryInto;
            use zellij_tile::shim::plugin_api::event::ProtobufEvent;
            use zellij_tile::shim::prost::Message;
            STATE.with(|state| {
                let protobuf_bytes: Vec<u8> = $crate::shim::object_from_stdin().unwrap();
                let protobuf_event: ProtobufEvent =
                    ProtobufEvent::decode(protobuf_bytes.as_slice()).unwrap();
                let event = protobuf_event.try_into().unwrap();
                <$t as $crate::ZellijPlugin>::update(&mut *state.borrow_mut(), event)
            })
        }

        #[no_mangle]
        pub fn pipe() -> bool {
            let err_context = "Failed to deserialize pipe message";
            use std::convert::TryInto;
            use zellij_tile::shim::plugin_api::pipe_message::ProtobufPipeMessage;
            use zellij_tile::shim::prost::Message;
            STATE.with(|state| {
                let protobuf_bytes: Vec<u8> = $crate::shim::object_from_stdin().unwrap();
                let protobuf_pipe_message: ProtobufPipeMessage =
                    ProtobufPipeMessage::decode(protobuf_bytes.as_slice()).unwrap();
                let pipe_message = protobuf_pipe_message.try_into().unwrap();
                <$t as $crate::ZellijPlugin>::pipe(&mut *state.borrow_mut(), pipe_message)
            })
        }

        #[no_mangle]
        pub fn render(rows: i32, cols: i32) {
            STATE.with(|state| {
                <$t as $crate::ZellijPlugin>::render(
                    &mut *state.borrow_mut(),
                    rows as usize,
                    cols as usize,
                );
            });
        }

        #[no_mangle]
        pub fn plugin_version() {
            println!("{}", $crate::prelude::VERSION);
        }
    };
}

/// On non-wasm targets the WASM exports aren't needed — the host invokes the
/// `ZellijPlugin` trait methods directly via the native registry. Instead we
/// emit a `create_plugin` factory function that the registry can call to
/// construct a fresh boxed instance of `$t`. Plugin source compiles unchanged
/// for both targets.
#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! register_plugin {
    ($t:ty) => {
        pub fn create_plugin() -> std::boxed::Box<dyn $crate::BoxableZellijPlugin> {
            std::boxed::Box::new(<$t as ::std::default::Default>::default())
        }
    };
}

/// Used to register a plugin worker implementing the [`ZellijWorker`] trait.
///
/// eg.
/// ```rust
/// use zellij_tile::prelude::*;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Default, Serialize, Deserialize)]
/// pub struct FileSearchWorker {}
///
/// impl ZellijWorker<'_> for FileSearchWorker {
///     fn on_message(&mut self, message: String, payload: String) {
///         // ...
///     }
/// }
///
/// register_worker!(
///     FileSearchWorker,
///     file_search_worker, // registers the worker as the namespace "file_search"
///     FILE_SEARCH_WORKER  // expanded to a static variable in which the worker state it held
/// );
/// ```
#[macro_export]
macro_rules! register_worker {
    ($worker:ty, $worker_name:ident, $worker_static_name:ident) => {
        // persist worker state in memory in a static variable
        thread_local! {
            static $worker_static_name: std::cell::RefCell<$worker> = std::cell::RefCell::new(Default::default());
        }
        #[no_mangle]
        pub fn $worker_name() {
            use zellij_tile::shim::plugin_api::message::ProtobufMessage;
            use zellij_tile::shim::prost::Message;
            let worker_display_name = std::stringify!($worker_name);
            let protobuf_bytes: Vec<u8> = $crate::shim::object_from_stdin()
                .unwrap();
            let protobuf_message: ProtobufMessage = ProtobufMessage::decode(protobuf_bytes.as_slice())
                .unwrap();
            let message = protobuf_message.name;
            let payload = protobuf_message.payload;
            $worker_static_name.with(|worker_instance| {
                let mut worker_instance = worker_instance.borrow_mut();
                worker_instance.on_message(message, payload);
            });
         }
    };
}
