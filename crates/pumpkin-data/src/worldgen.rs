use pumpkin_registry::{BoxedRegistry, MutableRegistry, Registry, RootRegistryReference, error::{RegistryInsertError, RegistryTreeError}};
use pumpkin_util::identifier::Identifier;
use std::sync::{Arc, LazyLock};

pub async fn initialize(root: RootRegistryReference) -> Result<(), RegistryTreeError> {
    let worldgen = MutableRegistry::<BoxedRegistry>::new(&[], &[])?;
    root.register(Identifier::vanilla_static("worldgen"), Box::new(worldgen)).await?;
    Ok(())
}