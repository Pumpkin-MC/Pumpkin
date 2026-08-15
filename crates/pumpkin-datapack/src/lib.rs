pub mod advancement;
pub mod command;
pub mod damage_type;
pub mod function;
pub mod loot;
pub mod pack;
pub mod predicate;
pub mod recipe;
pub mod registry;
pub mod reload;
pub mod resource;
pub mod tag;

pub use pumpkin_util::identifier::Identifier;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock as SyncRwLock;
use tokio::sync::RwLock;

use function::manager::FunctionManager;
use pack::repository::PackRepository;
use reload::manager::ReloadManager;
use tag::registry::TagRegistry;

pub use loot::LootTable as DynamicLootTable;

use crate::registry::DatapackRegistries;

/// Error type for datapack operations.
#[derive(Debug, thiserror::Error)]
pub enum DatapackError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Pack not found: {0}")]
    PackNotFound(String),
    #[error("Pack has no metadata")]
    PackNoMetadata,
    #[error("Invalid pack format (current={current}, pack={pack})")]
    IncompatiblePack { current: u32, pack: u32 },
    #[error("Validation errors: {0:?}")]
    Validation(Vec<String>),
    #[error("Tag resolution error: {0}")]
    TagResolution(String),
    #[error("Function error: {0}")]
    Function(String),
    #[error("Invalid identifier: {0}")]
    Identifier(#[from] pumpkin_util::identifier::IdentifierError),
    #[error("Registry error: {0}")]
    Registry(#[from] pumpkin_registry::error::BootstrapError),
}

/// Top-level orchestrator for all datapack lifecycle operations.
pub struct DataPackManager {
    pub world_path: PathBuf,
    pub repository: RwLock<PackRepository>,
    pub tags: SyncRwLock<TagRegistry>,
    pub functions: RwLock<FunctionManager>,
    pub recipes: RwLock<Vec<pumpkin_protocol::codec::recipe::DynamicRecipe>>,
    pub loot_tables: RwLock<HashMap<Identifier, loot::LootTable>>,
    pub predicates: RwLock<HashMap<Identifier, predicate::Predicate>>,
    pub item_modifiers: RwLock<HashMap<Identifier, predicate::ItemModifier>>,
    pub advancements: RwLock<HashMap<Identifier, advancement::AdvancementFile>>,
    pub reload_manager: ReloadManager,
    pub registries: DatapackRegistries,
}

impl DataPackManager {
    /// Create a new manager bound to the given world path.
    #[must_use]
    pub fn new(world_path: PathBuf) -> Self {
        let repository = PackRepository::new(&world_path);
        Self {
            world_path,
            repository: RwLock::new(repository),
            tags: SyncRwLock::new(TagRegistry::new()),
            functions: RwLock::new(FunctionManager::new()),
            recipes: RwLock::new(Vec::new()),
            loot_tables: RwLock::new(HashMap::new()),
            predicates: RwLock::new(HashMap::new()),
            item_modifiers: RwLock::new(HashMap::new()),
            advancements: RwLock::new(HashMap::new()),
            reload_manager: ReloadManager::new(),
            registries: DatapackRegistries::new(),
        }
    }

    /// Perform a full reload: open resources for selected packs and reload all data.
    /// Caller must ensure `repository.write().await.reload()` has been called first.
    pub async fn reload(&self) -> Result<(), DatapackError> {
        let packs = {
            let repo = self.repository.write().await;
            repo.open_all_selected()
        };

        let manager = resource::manager::MultiPackResourceManager::new(&packs);

        let tags = tag::loader::load_tags(&manager)?;
        let functions = function::loader::load_functions(&manager)?;
        let recipes = recipe::loader::load_recipes(&manager)?;
        let loot_tables = loot::load_loot_tables(&manager)?;
        let predicates = predicate::load_predicates(&manager)?;
        let item_modifiers = predicate::load_item_modifiers(&manager)?;
        let advancements = advancement::load_advancements(&manager)?;
        let damage_types = damage_type::load_damage_types(&manager)?;

        self.registries.reload_damage_types(damage_types).await?;

        // TODO(datapack parity): Merge DP enchantments, entity types, biomes, etc.
        // into their respective static registries here once loaders are added.

        // Populate tick/load functions from tag registry
        let tick_tag = Identifier::parse("minecraft:tick")?;
        let load_tag = Identifier::parse("minecraft:load")?;
        let tick_funcs = tags
            .get_tag_values("function", &tick_tag)
            .map(<[_]>::to_vec)
            .unwrap_or_default();
        let load_funcs = tags
            .get_tag_values("function", &load_tag)
            .map(<[_]>::to_vec)
            .unwrap_or_default();

        self.tags.write().unwrap().replace_with(tags);
        self.functions.write().await.replace_with(functions);
        *self.recipes.write().await = recipes;
        *self.loot_tables.write().await = loot_tables;
        *self.predicates.write().await = predicates;
        *self.item_modifiers.write().await = item_modifiers;
        *self.advancements.write().await = advancements;

        // Set tick/load functions based on resolved tags
        self.functions
            .write()
            .await
            .set_special_functions(tick_funcs, load_funcs);

        Ok(())
    }

    /// Enable a pack by ID, then reload.
    pub async fn enable_pack(&self, id: &str) -> Result<(), DatapackError> {
        let changed = {
            let mut repo = self.repository.write().await;
            repo.add_pack(id)
        };
        if !changed {
            return Err(DatapackError::PackNotFound(id.to_string()));
        }
        self.reload().await
    }

    /// Enable a pack at a specific position, then reload.
    pub async fn enable_pack_at(&self, id: &str, index: usize) -> Result<(), DatapackError> {
        let changed = {
            let mut repo = self.repository.write().await;
            repo.add_pack_at(id, index)
        };
        if !changed {
            return Err(DatapackError::PackNotFound(id.to_string()));
        }
        self.reload().await
    }

    /// Disable a pack by ID, then reload.
    pub async fn disable_pack(&self, id: &str) -> Result<(), DatapackError> {
        {
            let mut repo = self.repository.write().await;
            repo.remove_pack(id);
        };
        self.reload().await
    }

    /// Check whether an element has a tag, consulting both static (compile-time)
    /// and dynamic (datapack) tag registries.
    ///
    /// # Arguments
    /// * `registry` - Tag registry key (e.g. `"block"`, `"item"`, `"entity_type"`)
    /// * `element_key` - Element identifier (e.g. `"minecraft:stone"`)
    /// * `tag_name` - Tag identifier (e.g. `"minecraft:stone_ore_replaceables"`)
    /// * `static_check` - Closure for compile-time tag check, e.g. `\|tag\| item.is_tagged_with(tag)`
    pub fn is_tagged(
        &self,
        registry: &str,
        element_key: &str,
        tag_name: &str,
        static_check: impl FnOnce(&str) -> Option<bool>,
    ) -> bool {
        self.tags
            .read()
            .unwrap()
            .is_tagged_bridge(registry, element_key, tag_name, static_check)
    }
}
