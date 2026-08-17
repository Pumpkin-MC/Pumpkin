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

fn overlay_entries<T>(
    registry: &OnceLock<Arc<ReloadableRegistry<T>>>,
    mut entries: Vec<(Identifier, T)>,
) -> Result<(), BootstrapError>
where
    T: Send + Sync + 'static,
{
    entries.sort_unstable_by(|(left_id, _), (right_id, _)| left_id.cmp(right_id));
    registry
        .get()
        .ok_or(BootstrapError::Uninitialized)?
        .overlay_sorted_entries(entries)
}

pub struct DatapackRegistries;

impl Default for DatapackRegistries {
    fn default() -> Self {
        Self::new()
    }
}

impl DatapackRegistries {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    pub fn reload_damage_types(
        &self,
        entries: Vec<(Identifier, DamageTypeFile)>,
    ) -> Result<(), BootstrapError> {
        overlay_entries(&DAMAGE_TYPE_REGISTRY, entries)
    }

    pub fn reload_dimension_types(
        &self,
        entries: Vec<(Identifier, DimensionType)>,
    ) -> Result<(), DatapackError> {
        overlay_entries(&DIMENSION_TYPE_REGISTRY, entries)?;
        Ok(())
    }

    pub fn reload_timelines(
        &self,
        entries: Vec<(Identifier, Timeline)>,
    ) -> Result<(), DatapackError> {
        overlay_entries(&TIMELINE_REGISTRY, entries)?;
        Ok(())
    }

    pub fn reload_world_clocks(
        &self,
        entries: Vec<(Identifier, WorldClock)>,
    ) -> Result<(), DatapackError> {
        overlay_entries(&WORLD_CLOCK_REGISTRY, entries)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), DatapackError> {
        let world_clock_registry = world_clock_registry()?;
        let timeline_registry = timeline_registry()?;

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
        BOOTSTRAP, ROOT, Registry, RegistryBuilder, TypedRegistry, bootstrap::BootstrapManager,
        error::BootstrapError,
    };
    use pumpkin_util::identifier::Identifier;
    use std::sync::Arc;

    fn damage_type(id: Identifier, message_id: &str) -> DamageTypeFile {
        DamageTypeFile {
            id,
            data: serde_json::json!({ "message_id": message_id }),
        }
    }

    #[tokio::test]
    async fn dimension_stem_resolves_registry_backed_generator() {
        use pumpkin_codecs::{Decode, json_ops::JsonOps};
        use pumpkin_nbt::nbt_ops::NbtOps;
        use pumpkin_util::world_seed::Seed;
        use pumpkin_world::generation::dimension_stem::DimensionStem;

        BOOTSTRAP.get_or_init(BootstrapManager::new);
        ROOT.get_or_init(|| {
            RegistryBuilder::<Arc<dyn Registry>>::frozen(&Identifier::vanilla_static("root"))
                .expect("test root registry must initialize")
        });

        let temp = tempfile::tempdir().expect("temporary world directory must be created");
        let manager = crate::DataPackManager::new(temp.path().to_path_buf());
        let packs = {
            let mut repository = manager.repository.write().await;
            repository.reload();
            repository.open_all_selected()
        };
        let resources = crate::resource::manager::MultiPackResourceManager::new(&packs);
        let dimension_types = crate::dimension_type::load_dimension_types(&resources)
            .expect("vanilla dimension types must load");
        manager
            .registries
            .reload_dimension_types(dimension_types)
            .expect("dimension type registry must reload");

        let stem = DimensionStem::parse(
            serde_json::json!({
                "type": "minecraft:overworld",
                "generator": {
                    "type": "minecraft:noise",
                    "settings": "minecraft:amplified",
                    "biome_source": {
                        "type": "minecraft:multi_noise",
                        "preset": "minecraft:overworld"
                    }
                }
            }),
            &JsonOps,
        )
        .into_result()
        .expect("dimension stem must decode");

        let root = ROOT.get().expect("root registry must be initialized");
        let dimension = stem
            .dimension_type
            .get(root)
            .expect("dimension type must resolve from datapack registry");
        let generator_type = stem
            .generator
            .generator_type
            .get(root)
            .expect("generator type must resolve from registry");
        let generator = generator_type
            .decode(
                stem.generator.input,
                &NbtOps,
                Seed(1234),
                (*dimension).clone(),
            )
            .into_result()
            .expect("generator must decode");

        assert_eq!(generator.seed(), 1234);
        assert!(generator.dimension().is_overworld_like());
    }

    #[test]
    fn reload_damage_types_sorts_and_populates_registry() -> Result<(), DatapackError> {
        BOOTSTRAP.get_or_init(BootstrapManager::new);
        ROOT.get_or_init(|| {
            RegistryBuilder::<Arc<dyn Registry>>::frozen(&Identifier::vanilla_static("root"))
                .expect("test root registry must initialize")
        });

        let registries = DatapackRegistries::new();
        let alpha = Identifier::parse_static("test:alpha");
        let zeta = Identifier::parse_static("test:zeta");

        registries.reload_damage_types(vec![
            (zeta.clone(), damage_type(zeta.clone(), "zeta")),
            (alpha.clone(), damage_type(alpha.clone(), "alpha")),
        ])?;

        registries.validate()?;
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
