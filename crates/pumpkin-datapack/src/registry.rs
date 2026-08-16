use crate::{DatapackError, damage_type::DamageTypeFile, dimension_type::DimensionType};
use pumpkin_registry::{
    Registry, RegistryBuilder, ReloadableRegistry, bootstrap::RegistryEntry, bootstrap_provider,
    error::BootstrapError,
};
use pumpkin_util::identifier::Identifier;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

static DAMAGE_TYPE_REGISTRY: OnceLock<Arc<ReloadableRegistry<DamageTypeFile>>> = OnceLock::new();
static DIMENSION_TYPE_REGISTRY: OnceLock<Arc<ReloadableRegistry<DimensionType>>> = OnceLock::new();

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

fn dimension_type_registry() -> Result<Arc<ReloadableRegistry<DimensionType>>, BootstrapError> {
    if let Some(registry) = DIMENSION_TYPE_REGISTRY.get() {
        return Ok(Arc::clone(registry));
    }

    let registry = Arc::new(RegistryBuilder::reloadable(&Identifier::vanilla_static(
        "dimension_type",
    ))?);
    if DIMENSION_TYPE_REGISTRY.set(Arc::clone(&registry)).is_ok() {
        return Ok(registry);
    }

    DIMENSION_TYPE_REGISTRY
        .get()
        .map(Arc::clone)
        .ok_or(BootstrapError::Uninitialized)
}

fn init_register() -> Vec<RegistryEntry<Arc<dyn Registry>>> {
    #![allow(clippy::panic)]
    let damage_type: Arc<dyn Registry> = damage_type_registry()
        .unwrap_or_else(|error| panic!("failed to bootstrap damage type registry: {error}"));
    let dimension_type: Arc<dyn Registry> = dimension_type_registry()
        .unwrap_or_else(|error| panic!("failed to bootstrap dimension type registry: {error}"));

    vec![
        RegistryEntry::new(Identifier::vanilla_static("damage_type"), damage_type),
        RegistryEntry::new(Identifier::vanilla_static("dimension_type"), dimension_type),
    ]
}

bootstrap_provider! {
    DATAPACK_DAMAGE_TYPE_REGISTRY: Arc<dyn Registry> => "minecraft:root",
    init_register
}

pub struct DatapackRegistries {
    pending_damage_types: RwLock<Vec<(Identifier, DamageTypeFile)>>,
    pending_dimension_types: RwLock<Vec<(Identifier, DimensionType)>>,
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
            pending_dimension_types: RwLock::new(Vec::new()),
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
            registry.overlay_entries(entries)?;
        }

        Ok(())
    }

    pub async fn reload_dimension_types<I>(&self, entries: I) -> Result<(), DatapackError>
    where
        I: IntoIterator<Item = (Identifier, DimensionType)>,
    {
        let mut entries: Vec<_> = entries.into_iter().collect();
        entries.sort_unstable_by(|(left_id, _), (right_id, _)| left_id.cmp(right_id));
        *self.pending_dimension_types.write().await = entries.clone();

        if let Some(registry) = DIMENSION_TYPE_REGISTRY.get() {
            registry.overlay_entries(entries)?;
        }

        Ok(())
    }

    /// Apply registry data loaded before the root registry was bootstrapped.
    ///
    /// The initial datapack reload happens before plugin registry providers are finalized,
    /// so registry data is staged until the root registry creates its reloadable children.
    pub async fn apply_pending(&self) -> Result<(), DatapackError> {
        let damage_types = self.pending_damage_types.read().await.clone();
        damage_type_registry()?.overlay_entries(damage_types)?;

        let dimension_types = self.pending_dimension_types.read().await.clone();
        dimension_type_registry()?.overlay_entries(dimension_types)?;
        Ok(())
    }

    #[must_use]
    pub fn damage_types(&self) -> Option<Arc<ReloadableRegistry<DamageTypeFile>>> {
        DAMAGE_TYPE_REGISTRY.get().map(Arc::clone)
    }

    #[must_use]
    pub fn dimension_types(&self) -> Option<Arc<ReloadableRegistry<DimensionType>>> {
        DIMENSION_TYPE_REGISTRY.get().map(Arc::clone)
    }
}

#[cfg(test)]
mod tests {
    use super::DatapackRegistries;
    use crate::{DatapackError, damage_type::DamageTypeFile};
    use pumpkin_registry::{
        BOOTSTRAP, Registry, TypedRegistry, bootstrap::BootstrapManager, error::BootstrapError,
    };
    use pumpkin_util::identifier::Identifier;

    fn damage_type(id: Identifier, message_id: &str) -> DamageTypeFile {
        DamageTypeFile {
            id,
            data: serde_json::json!({ "message_id": message_id }),
        }
    }

    #[tokio::test]
    async fn reload_damage_types_sorts_and_populates_registry() -> Result<(), DatapackError> {
        BOOTSTRAP.get_or_init(BootstrapManager::new);

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
        let damage_types = registries
            .damage_types()
            .ok_or(BootstrapError::Uninitialized)?;

        assert_eq!(Registry::get_id(damage_types.as_ref(), &alpha), Some(0),);
        assert_eq!(Registry::get_id(damage_types.as_ref(), &zeta), Some(1),);

        let alpha_entry = TypedRegistry::get(damage_types.as_ref(), &alpha);
        assert_eq!(
            alpha_entry.as_deref().map(DamageTypeFile::message_id),
            Some("alpha"),
        );

        Ok(())
    }
}
