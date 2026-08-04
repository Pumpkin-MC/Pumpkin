use pumpkin_registry::{Registry, RegistryAccess, error::RegistryInsertError};
use pumpkin_util::identifier::Identifier;
use std::sync::{Arc, LazyLock};

pub static WORLD_GEN: LazyLock<Arc<Registry<dyn RegistryAccess + Send + Sync>>> = LazyLock::new(|| {
    let registry = Arc::new(Registry::new());
    pumpkin_registry::ROOT
        .register_arc(
            Identifier::vanilla_static("worldgen"),
            registry.clone(),
        )
        .unwrap();
    return registry;
});
