use crate::resource::ResourceManager;
use std::future::Future;
use std::pin::Pin;

/// A reload listener that can prepare data in parallel and apply it on the main thread.
pub trait PreparableReloadListener: Send + Sync {
    /// Name for logging.
    fn name(&self) -> &str;

    /// Prepare data on a background thread.
    fn prepare(
        &self,
        manager: &dyn ResourceManager,
    ) -> Pin<Box<dyn Future<Output = Result<(), Vec<String>>> + Send + '_>>;

    /// Apply prepared data on the main thread.
    fn apply(&self, _manager: &dyn ResourceManager) -> Result<(), Vec<String>> {
        Ok(())
    }
}
