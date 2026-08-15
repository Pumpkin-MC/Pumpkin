use crate::damage_type::DamageTypeFile;
use pumpkin_registry::{
    Registry, RegistryBuilder, ReloadableRegistry,
    bootstrap::RegistryEntry,
    bootstrap_provider,
    error::BootstrapError,
};
use pumpkin_util::identifier::Identifier;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

static DAMAGE_TYPE_REGISTRY: OnceLock<Arc<ReloadableRegistry<DamageTypeFile>>> = OnceLock::new();

fn damage_type_registry() -> Result<Arc<ReloadableRegistry<DamageTypeFile>>, BootstrapError> {
    if let Some(registry) = DAMAGE_TYPE_REGISTRY.get() {
        return Ok(Arc::clone(registry));
    }

    let registry = Arc::new(RegistryBuilder::reloadable(&Identifier::vanilla_static(
        "damage_type",
    ))?);

    if DAMAGE_TYPE_REGISTRY.set(Arc::clone(&registry)).is_ok() {
        return Ok(registry);
    }

    DAMAGE_TYPE_REGISTRY
        .get()
        .map(Arc::clone)
        .ok_or(BootstrapError::Uninitialized)
}

bootstrap_provider! {
    DATAPACK_DAMAGE_TYPE_REGISTRY: Arc<dyn Registry> => "minecraft:root",
    || {
        let registry: Arc<dyn Registry> = damage_type_registry()
            .unwrap_or_else(|error| panic!("failed to bootstrap damage type registry: {error}"));

        vec![RegistryEntry::new(
            Identifier::vanilla_static("damage_type"),
            registry,
        )]
    }
}

pub struct DatapackRegistries {
    pending_damage_types: RwLock<Vec<(Identifier, DamageTypeFile)>>,
}

impl Default for DatapackRegistries {
    fn default() -> Self {
        Self::new()
    }
}

impl DatapackRegistries {
    #[must_use]
    pub fn new() -> Self {
        Self {
            pending_damage_types: RwLock::new(Vec::new()),
        }
    }

    pub async fn reload_damage_types<I>(&self, entries: I) -> Result<(), BootstrapError>
    where
        I: IntoIterator<Item = (Identifier, DamageTypeFile)>,
    {
        let mut entries: Vec<_> = entries.into_iter().collect();
        entries.sort_unstable_by(|(left_id, _), (right_id, _)| left_id.cmp(right_id));

        *self.pending_damage_types.write().await = entries.clone();

        if let Some(registry) = DAMAGE_TYPE_REGISTRY.get() {
            registry.replace_entries(entries).await?;
        }

        Ok(())
    }

    /// Apply registry data loaded before the root registry was bootstrapped.
    ///
    /// The initial datapack reload happens before plugin registry providers are finalized,
    /// so registry data is staged until the root registry creates its reloadable children.
    pub async fn apply_pending(&self) -> Result<(), BootstrapError> {
        let entries = self.pending_damage_types.read().await.clone();
        damage_type_registry()?.replace_entries(entries).await
    }

    #[must_use]
    pub fn damage_types() -> Option<Arc<ReloadableRegistry<DamageTypeFile>>> {
        DAMAGE_TYPE_REGISTRY.get().map(Arc::clone)
    }
}

#[cfg(test)]
mod tests {
    use super::DatapackRegistries;
    use crate::damage_type::DamageTypeFile;
    use pumpkin_registry::{AsyncTypedRegistry, Registry, error::BootstrapError};
    use pumpkin_util::identifier::Identifier;

    fn damage_type(id: Identifier, message_id: &str) -> DamageTypeFile {
        DamageTypeFile {
            id,
            data: serde_json::json!({ "message_id": message_id }),
        }
    }

    #[tokio::test]
    async fn reload_damage_types_sorts_and_populates_registry() -> Result<(), BootstrapError> {
        let registries = DatapackRegistries::new();
        let alpha = Identifier::parse_static("test:alpha");
        let zeta = Identifier::parse_static("test:zeta");

        registries
            .reload_damage_types([
                (zeta.clone(), damage_type(zeta.clone(), "zeta")),
                (alpha.clone(), damage_type(alpha.clone(), "alpha")),
            ])
            .await?;

        registries.apply_pending().await?;
        let damage_types = registries.damage_types().ok_or(BootstrapError::Uninitialized)?;

        assert_eq!(
            Registry::get_id_async(damage_types.as_ref(), &alpha).await,
            Some(0),
        );
        assert_eq!(
            Registry::get_id_async(damage_types.as_ref(), &zeta).await,
            Some(1),
        );

        let alpha_entry = AsyncTypedRegistry::get(damage_types.as_ref(), &alpha).await;
        assert_eq!(
            alpha_entry.as_deref().map(DamageTypeFile::message_id),
            Some("alpha"),
        );

        Ok(())
    }
}
