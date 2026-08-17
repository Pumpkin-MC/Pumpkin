use crate::{
    DatapackError, damage_type::DamageTypeFile, dimension_type::DimensionType, timeline::Timeline,
    world_clock::WorldClock,
};
use pumpkin_registry::{
    ROOT, Registry, RegistryBuilder, ReloadableRegistry, TypedRegistry, bootstrap::RegistryEntry,
    bootstrap_provider, error::BootstrapError,
};
use pumpkin_util::identifier::Identifier;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

static DAMAGE_TYPE_REGISTRY: OnceLock<Arc<ReloadableRegistry<DamageTypeFile>>> = OnceLock::new();
static DIMENSION_TYPE_REGISTRY: OnceLock<Arc<ReloadableRegistry<DimensionType>>> = OnceLock::new();
static TIMELINE_REGISTRY: OnceLock<Arc<ReloadableRegistry<Timeline>>> = OnceLock::new();
static WORLD_CLOCK_REGISTRY: OnceLock<Arc<ReloadableRegistry<WorldClock>>> = OnceLock::new();

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

fn timeline_registry() -> Result<Arc<ReloadableRegistry<Timeline>>, BootstrapError> {
    if let Some(registry) = TIMELINE_REGISTRY.get() {
        return Ok(Arc::clone(registry));
    }

    let registry = Arc::new(RegistryBuilder::reloadable(&Identifier::vanilla_static(
        "timeline",
    ))?);
    if TIMELINE_REGISTRY.set(Arc::clone(&registry)).is_ok() {
        return Ok(registry);
    }

    TIMELINE_REGISTRY
        .get()
        .map(Arc::clone)
        .ok_or(BootstrapError::Uninitialized)
}

fn world_clock_registry() -> Result<Arc<ReloadableRegistry<WorldClock>>, BootstrapError> {
    if let Some(registry) = WORLD_CLOCK_REGISTRY.get() {
        return Ok(Arc::clone(registry));
    }

    let registry = Arc::new(RegistryBuilder::reloadable(&Identifier::vanilla_static(
        "world_clock",
    ))?);
    if WORLD_CLOCK_REGISTRY.set(Arc::clone(&registry)).is_ok() {
        return Ok(registry);
    }

    WORLD_CLOCK_REGISTRY
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
    let timeline: Arc<dyn Registry> = timeline_registry()
        .unwrap_or_else(|error| panic!("failed to bootstrap timeline registry: {error}"));
    let world_clock: Arc<dyn Registry> = world_clock_registry()
        .unwrap_or_else(|error| panic!("failed to bootstrap world clock registry: {error}"));

    vec![
        RegistryEntry::new(Identifier::vanilla_static("damage_type"), damage_type),
        RegistryEntry::new(Identifier::vanilla_static("dimension_type"), dimension_type),
        RegistryEntry::new(Identifier::vanilla_static("timeline"), timeline),
        RegistryEntry::new(Identifier::vanilla_static("world_clock"), world_clock),
    ]
}

bootstrap_provider! {
    DATAPACK_DAMAGE_TYPE_REGISTRY: Arc<dyn Registry> => "minecraft:root",
    init_register
}

pub struct DatapackRegistries {
    pending_damage_types: RwLock<Vec<(Identifier, DamageTypeFile)>>,
    pending_dimension_types: RwLock<Vec<(Identifier, DimensionType)>>,
    pending_timelines: RwLock<Vec<(Identifier, Timeline)>>,
    pending_world_clocks: RwLock<Vec<(Identifier, WorldClock)>>,
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
            pending_timelines: RwLock::new(Vec::new()),
            pending_world_clocks: RwLock::new(Vec::new()),
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

    pub async fn reload_timelines<I>(&self, entries: I) -> Result<(), DatapackError>
    where
        I: IntoIterator<Item = (Identifier, Timeline)>,
    {
        let mut entries: Vec<_> = entries.into_iter().collect();
        entries.sort_unstable_by(|(left_id, _), (right_id, _)| left_id.cmp(right_id));
        *self.pending_timelines.write().await = entries.clone();

        if let Some(registry) = TIMELINE_REGISTRY.get() {
            registry.overlay_entries(entries)?;
        }

        Ok(())
    }

    pub async fn reload_world_clocks<I>(&self, entries: I) -> Result<(), DatapackError>
    where
        I: IntoIterator<Item = (Identifier, WorldClock)>,
    {
        let mut entries: Vec<_> = entries.into_iter().collect();
        entries.sort_unstable_by(|(left_id, _), (right_id, _)| left_id.cmp(right_id));
        *self.pending_world_clocks.write().await = entries.clone();

        if let Some(registry) = WORLD_CLOCK_REGISTRY.get() {
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

        let world_clocks = self.pending_world_clocks.read().await.clone();
        let world_clock_registry = world_clock_registry()?;
        world_clock_registry.overlay_entries(world_clocks)?;

        let timelines = self.pending_timelines.read().await.clone();
        let timeline_registry = timeline_registry()?;
        timeline_registry.overlay_entries(timelines)?;

        let dimension_types = self.pending_dimension_types.read().await.clone();
        dimension_type_registry()?.overlay_entries(dimension_types)?;

        let root = ROOT.get().ok_or(BootstrapError::Uninitialized)?;
        let environment_attribute_id = Identifier::vanilla_static("environment_attribute");
        let Some(environment_attributes) = TypedRegistry::get(root, &environment_attribute_id)
        else {
            return Err(DatapackError::Validation(vec![
                "environment attribute registry is not initialized".to_string(),
            ]));
        };

        let mut errors = Vec::new();
        for (identifier, timeline) in timeline_registry.iter() {
            if let pumpkin_codecs::DataResult::Error { message, .. } = timeline.validate(
                world_clock_registry.as_ref(),
                environment_attributes.as_ref(),
            ) {
                errors.push(format!("timeline {identifier}: {message}"));
            }
        }
        if !errors.is_empty() {
            return Err(DatapackError::Validation(errors));
        }

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

    #[must_use]
    pub fn timelines(&self) -> Option<Arc<ReloadableRegistry<Timeline>>> {
        TIMELINE_REGISTRY.get().map(Arc::clone)
    }

    #[must_use]
    pub fn world_clocks(&self) -> Option<Arc<ReloadableRegistry<WorldClock>>> {
        WORLD_CLOCK_REGISTRY.get().map(Arc::clone)
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
