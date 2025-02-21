use crate::{init, registry, Callback, Mechanism, MECHANISMS, SASL};
use std::cmp::Ordering;
use std::sync::Arc;

impl SASL {
    pub fn build() -> Builder {
        Builder::new()
    }

    pub fn new() -> Self {
        Self {
            callback: None,

            #[cfg(feature = "registry_dynamic")]
            dynamic_mechs: Vec::new(),

            #[cfg(feature = "registry_static")]
            static_mechs: &registry::MECHANISMS,

            sort_fn: |a, b| a.priority.cmp(&b.priority),
        }
    }

    /// Initialize this SASL with the builtin Mechanisms
    ///
    /// Calling this function is usually not necessary if you're using the `registry_static`
    /// feature since the builtin mechanisms are registered at compile time then. However the
    /// optimizer may strip modules that it deems are unused so a call may still be necessary but
    /// it then extremely cheap.
    pub fn init(&mut self) {
        init::register_builtin(self);
    }

    pub fn install_callback(&mut self, callback: Arc<dyn Callback + Send + Sync>) {
        self.callback = Some(callback);
    }
}

pub struct Builder {
    callback: Option<Arc<dyn Callback + Send + Sync>>,
    dynamic_mechs: Option<Vec<&'static Mechanism>>,
    static_mechs: Option<&'static [Mechanism]>,
    sort_fn: Option<fn(a: &&Mechanism, b: &&Mechanism) -> Ordering>,
}
impl Builder {
    pub fn new() -> Self {
        Self {
            callback: None,
            dynamic_mechs: None,
            static_mechs: None,
            sort_fn: None,
        }
    }
    pub fn finish(self) -> SASL {
        let callback = self.callback;
        let dynamic_mechs = self.dynamic_mechs.unwrap_or_else(Vec::new);
        let static_mechs = self.static_mechs.unwrap_or(&MECHANISMS);
        let sort_fn = self.sort_fn.unwrap_or(|a, b| a.priority.cmp(&b.priority));

        SASL {
            callback,
            dynamic_mechs,
            static_mechs,
            sort_fn,
        }
    }

    pub fn with_static_mechs(mut self, static_mechs: &'static [Mechanism]) -> Self {
        self.static_mechs = Some(static_mechs);
        self
    }
}
