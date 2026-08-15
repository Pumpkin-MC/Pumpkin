use std::sync::Arc;

use super::listener::PreparableReloadListener;
use crate::pack::resource::PackResources;
use crate::resource::manager::MultiPackResourceManager;

/// Orchestrates a full datapack reload.
pub struct ReloadManager {
    listeners: Vec<Arc<dyn PreparableReloadListener>>,
}

impl ReloadManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    /// Register a reload listener.
    pub fn register(&mut self, listener: Arc<dyn PreparableReloadListener>) {
        self.listeners.push(listener);
    }

    /// Get all registered listeners.
    #[must_use]
    pub fn listeners(&self) -> &[Arc<dyn PreparableReloadListener>] {
        &self.listeners
    }

    /// Perform a full reload: open packs, run all listeners.
    ///
    /// Returns errors from any listener.
    pub async fn reload(&self, packs: Vec<Arc<dyn PackResources>>) -> Result<(), Vec<String>> {
        let manager = MultiPackResourceManager::new(&packs);
        let mut errors = Vec::new();

        // Phase 1: Prepare all listeners in parallel
        let futures: Vec<_> = self
            .listeners
            .iter()
            .map(|listener| {
                let name = listener.name().to_string();
                let prep = listener.prepare(&manager);
                async move {
                    let result = prep.await;
                    (name, result)
                }
            })
            .collect();

        let results: Vec<(String, Result<(), Vec<String>>)> =
            futures::future::join_all(futures).await;

        // Phase 2: Apply sequentially on "main thread"
        for (listener, (_, prepare_result)) in self.listeners.iter().zip(&results) {
            match prepare_result {
                Ok(()) => {
                    if let Err(apply_errs) = listener.apply(&manager) {
                        errors.extend(apply_errs);
                    }
                }
                Err(prep_errs) => {
                    errors.extend(prep_errs.iter().cloned());
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for ReloadManager {
    fn default() -> Self {
        Self::new()
    }
}
