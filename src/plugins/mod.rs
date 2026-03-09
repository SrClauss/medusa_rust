//! Core plugin system for extensibility.
//!
//! This module defines a minimal trait-based infrastructure that allows the
//! application to register and enumerate "plugins" (payment providers,
//! fulfillment adapters, notification transports, etc.).  For now the loader is
//! static, but the abstractions permit loading external crates or dynamic
//! libraries later.

use crate::state::AppState;
use std::collections::HashMap;
use std::fmt;

/// General category of a plugin.  Used mainly for administrative UI and for
/// routing lookups from the core business logic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginKind {
    Payment,
    Fulfillment,
    Notification,
    Search,
    /// Any other kind; the `String` can hold a custom identifier.
    Other(String),
}

impl fmt::Display for PluginKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginKind::Payment => write!(f, "payment"),
            PluginKind::Fulfillment => write!(f, "fulfillment"),
            PluginKind::Notification => write!(f, "notification"),
            PluginKind::Search => write!(f, "search"),
            PluginKind::Other(s) => write!(f, "{s}"),
        }
    }
}

/// A dynamically loadable component that can extend the application.  Plugins
/// are expected to be cheap to construct; the heavy lifting (clients, database
/// queries, etc.) should be performed during `init`.  Once registered the
/// plugin is stored in the global [`PluginManager`].
pub trait Plugin: Send + Sync {
    /// Unique handle for the plugin ("stripe", "manual", "asaas").
    fn id(&self) -> &str;

    /// Broad category used for grouping and filtering.
    fn kind(&self) -> PluginKind;

    /// Called once when the plugin is registered during application startup.
    /// Plugins receive a reference to the shared `AppState` so they can register
    /// callbacks, construct clients, populate in‑memory caches, etc.
    fn init(&self, _state: &AppState) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Manages the set of plugins that have been loaded into the running process.
///
/// The manager itself is stored in [`AppState`] behind a `Mutex` and is
/// intended to be manipulated only from the main thread during startup; the
/// handlers will usually read from it only.
#[derive(Default)]
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
}

impl PluginManager {
    /// Create an empty manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a plugin.  The caller is responsible for holding a mutable
    /// reference to the manager (typically by locking the containing mutex).
    ///
    /// The plugin is stored as‑is; initialization (calling
    /// [`Plugin::init`] with a real `AppState`) must be performed by the caller
    /// before or after registration, depending on the use case. This separation
    /// keeps the manager simple and avoids borrowing issues.
    pub fn register(&mut self, plugin: impl Plugin + 'static) {
        let id = plugin.id().to_string();
        self.plugins.insert(id, Box::new(plugin));
    }

    /// Returns an immutable reference to a plugin by its id.
    pub fn get(&self, id: &str) -> Option<&dyn Plugin> {
        self.plugins.get(id).map(|b| &**b)
    }

    /// Return a list of all registered plugins.
    pub fn list(&self) -> Vec<&dyn Plugin> {
        self.plugins.values().map(|b| &**b).collect()
    }
}

// A tiny example plugin that does nothing useful.  It illustrates how a
// consumer crate would implement the trait.  This struct is only compiled when
// the `manual_plugin` feature is enabled so that the core binary doesn't
// depend on it by default.

/// Built‑in "manual" payment provider used for development and tests.
#[cfg(feature = "manual_plugin")]
pub struct ManualPlugin;

#[cfg(feature = "manual_plugin")]
impl Plugin for ManualPlugin {
    fn id(&self) -> &str { "manual" }
    fn kind(&self) -> PluginKind { PluginKind::Payment }
    fn init(&self, _state: &AppState) -> anyhow::Result<()> { Ok(()) }
}

#[cfg(feature = "manual_plugin")]
pub fn register_builtin_plugins(pm: &mut PluginManager, state: &AppState) -> anyhow::Result<()> {
    let p = ManualPlugin;
    p.init(state)?;
    pm.plugins.insert(p.id().to_string(), Box::new(p));
    Ok(())
}
