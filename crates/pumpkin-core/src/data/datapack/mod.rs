pub mod advancement_loader;
pub mod context_provider_loader;
pub mod damage_type_loader;
mod dimension_type;
pub mod dynamic_registry_loader;
pub mod dynamic_registry_manager;
pub mod function_loader;
pub mod item_modifier_loader;
pub mod loot_table_loader;
pub mod predicate_loader;
pub mod recipe_loader;
pub mod slot_source_loader;
pub mod structure_loader;
pub mod tag_loader;
pub mod test_loader;
pub mod trade_loader;

use std::cell::Cell;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use pumpkin_world::world_info::{DEFAULT_ENABLED_DATA_PACK, LevelData, feature_flags};
use std::sync::{Arc, RwLock};
use tracing::{info, warn};

use pumpkin_data::damage::DamageType;
use pumpkin_data::registry::RegistryEntryData;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_nbt::{NbtCompound, nbt_compress::read_gzip_compound_tag};
use pumpkin_protocol::codec::recipe::DynamicRecipe;
use pumpkin_util::loot_table::DynamicLootTable;

use crate::command::context::command_source::CommandSource;
use crate::server::Server;
use crate::server::recipe::RecipeManager;
use crate::world::loot::LootTableHandle;

use self::context_provider_loader::ContextProviderRegistry;
use self::damage_type_loader::{DamageTypeDefinition, DamageTypeEntry, DamageTypeRegistry};
use self::test_loader::{
    TestInstance, TestInstanceRegistry, load_test_instances_from_dir, to_registry_entry,
};
use self::trade_loader::{DynamicVillagerTradeSet, TradeRegistry};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownPackData {
    pub namespace: String,
    pub id: String,
    pub version: String,
}

#[derive(Clone, Debug)]
pub struct LoadedDatapack {
    pub id: String,
    pub name: String,
    pub description: String,
    pub pack_format: u32,
    pub root_path: PathBuf,
    pub recipe_count: usize,
    pub function_count: usize,
    pub known_packs: Vec<KnownPackData>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatapackInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub pack_format: u32,
    pub is_enabled: bool,
    pub recipe_count: usize,
    pub function_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DatapackEnablePosition {
    First,
    Last,
    Before(String),
    After(String),
}

pub struct DatapackManager {
    loaded_packs: RwLock<Vec<LoadedDatapack>>,
    functions: RwLock<HashMap<String, Arc<[String]>>>,
    function_tags: RwLock<HashMap<String, Vec<String>>>,
    test_instances: RwLock<TestInstanceRegistry>,
    context_int_providers: RwLock<ContextProviderRegistry>,
    context_float_providers: RwLock<ContextProviderRegistry>,
    damage_types: RwLock<DamageTypeRegistry>,
    loot_tables: RwLock<HashMap<String, Arc<DynamicLootTable>>>,
    dynamic_registries: RwLock<HashMap<String, HashMap<String, RegistryEntryData>>>,
    pub dynamic_registry_manager: Arc<dynamic_registry_manager::DynamicRegistryManager>,
    trade_registry: RwLock<TradeRegistry>,
    advancements: RwLock<HashMap<String, advancement_loader::DynamicAdvancementDefinition>>,
    predicates: RwLock<HashMap<String, serde_json::Value>>,
    item_modifiers: RwLock<HashMap<String, serde_json::Value>>,
    slot_sources: RwLock<HashMap<String, serde_json::Value>>,
    tags: RwLock<HashMap<String, HashMap<String, tag_loader::DynamicTagEntry>>>,
}

fn share_function_bodies(
    functions: HashMap<String, Vec<String>>,
) -> HashMap<String, Arc<[String]>> {
    functions
        .into_iter()
        .map(|(name, lines)| (name, lines.into()))
        .collect()
}

/// Maximum nested functions per command chain. Far below vanilla's `maxCommandChainLength`
/// (65536): each level costs stack frames and a Rust stack overflow aborts the process.
pub const MAX_FUNCTION_CHAIN_DEPTH: usize = 256;

/// Maximum dispatched commands per chain (vanilla `maxCommandChainLength` default).
/// Depth alone cannot bound a fanning chain, which grows exponentially without nesting deep.
pub const MAX_FUNCTION_CHAIN_COMMANDS: usize = 65536;

thread_local! {
    /// How many functions the current thread is currently executing, nested.
    static FUNCTION_CHAIN_DEPTH: Cell<usize> = const { Cell::new(0) };
    /// How many command lines the current chain has dispatched so far.
    static FUNCTION_CHAIN_COMMANDS: Cell<usize> = const { Cell::new(0) };
    /// Depth limit tripped somewhere in the chain; tells every level to stop.
    static FUNCTION_CHAIN_HIT_DEPTH_LIMIT: Cell<bool> = const { Cell::new(false) };
    /// Command budget tripped somewhere in the chain; tells every level to stop.
    static FUNCTION_CHAIN_HIT_COMMAND_LIMIT: Cell<bool> = const { Cell::new(false) };
}

/// Why a function could not be executed.
#[derive(Debug, PartialEq, Eq)]
pub enum FunctionRunError {
    /// The function or function tag does not exist, or could not be read.
    Unknown(String),
    /// The command chain already nested [`MAX_FUNCTION_CHAIN_DEPTH`] functions.
    ChainLimitExceeded,
    /// The command chain dispatched [`MAX_FUNCTION_CHAIN_COMMANDS`] commands.
    CommandChainLimitExceeded,
}

impl fmt::Display for FunctionRunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown(reason) => f.write_str(reason),
            Self::ChainLimitExceeded => write!(
                f,
                "Maximum number of nested function calls reached ({MAX_FUNCTION_CHAIN_DEPTH})"
            ),
            Self::CommandChainLimitExceeded => write!(
                f,
                "Maximum number of commands in a function chain reached ({MAX_FUNCTION_CHAIN_COMMANDS})"
            ),
        }
    }
}

/// Counts nested function entries on this thread; the guard releases its level on drop.
struct FunctionChainGuard;

impl FunctionChainGuard {
    fn enter() -> Result<Self, FunctionRunError> {
        FUNCTION_CHAIN_DEPTH.with(|depth| {
            if depth.get() >= MAX_FUNCTION_CHAIN_DEPTH {
                FUNCTION_CHAIN_HIT_DEPTH_LIMIT.with(|hit| hit.set(true));
                return Err(FunctionRunError::ChainLimitExceeded);
            }
            depth.set(depth.get() + 1);
            Ok(Self)
        })
    }

    /// Records one dispatched line against the chain budget; false means stop the chain.
    fn count_command() -> bool {
        FUNCTION_CHAIN_COMMANDS.with(|commands| {
            let count = commands.get() + 1;
            commands.set(count);
            if count > MAX_FUNCTION_CHAIN_COMMANDS {
                FUNCTION_CHAIN_HIT_COMMAND_LIMIT.with(|hit| hit.set(true));
                return false;
            }
            true
        })
    }
}

impl Drop for FunctionChainGuard {
    fn drop(&mut self) {
        // Only ever dropped after a successful `enter`, so the depth cannot underflow.
        let outermost = FUNCTION_CHAIN_DEPTH.with(|depth| {
            let outer = depth.get() - 1;
            depth.set(outer);
            outer
        });
        if outermost == 0 {
            // Outermost chain ended; reset the budget and fail flags for the next chain.
            FUNCTION_CHAIN_COMMANDS.with(|commands| commands.set(0));
            FUNCTION_CHAIN_HIT_DEPTH_LIMIT.with(|hit| hit.set(false));
            FUNCTION_CHAIN_HIT_COMMAND_LIMIT.with(|hit| hit.set(false));
        }
    }
}

/// True while inside a nested function call; nested calls must not report their own result.
#[must_use]
pub fn is_nested_function_call() -> bool {
    FUNCTION_CHAIN_DEPTH.with(Cell::get) > 0
}

impl Default for DatapackManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DatapackManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            loaded_packs: RwLock::new(Vec::new()),
            functions: RwLock::new(HashMap::new()),
            function_tags: RwLock::new(HashMap::new()),
            test_instances: RwLock::new(HashMap::new()),
            context_int_providers: RwLock::new(HashMap::new()),
            context_float_providers: RwLock::new(HashMap::new()),
            damage_types: RwLock::new(HashMap::new()),
            loot_tables: RwLock::new(HashMap::new()),
            dynamic_registries: RwLock::new(HashMap::new()),
            dynamic_registry_manager: Arc::new(
                dynamic_registry_manager::DynamicRegistryManager::new(
                    pumpkin_data::packet::CURRENT_MC_VERSION,
                ),
            ),
            trade_registry: RwLock::new(TradeRegistry::new()),
            advancements: RwLock::new(HashMap::new()),
            predicates: RwLock::new(HashMap::new()),
            item_modifiers: RwLock::new(HashMap::new()),
            slot_sources: RwLock::new(HashMap::new()),
            tags: RwLock::new(HashMap::new()),
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn load_all(
        &self,
        world_path: &Path,
        enabled_packs: &[String],
        recipe_manager: &RecipeManager,
    ) {
        let datapacks_dir = world_path.join("datapacks");
        structure_loader::clear_dynamic_worldgen_data();
        let mut loaded_packs_vec = Vec::new();
        let mut all_recipes: Vec<DynamicRecipe> = Vec::new();
        let mut all_functions: HashMap<String, Vec<String>> = HashMap::new();
        let function_count = function_loader::load_test_functions(&mut all_functions);
        if function_count > 0 {
            info!("Loaded {function_count} function(s)");
        }
        let mut all_function_tags: HashMap<String, Vec<String>> = HashMap::new();
        let mut all_test_instances: TestInstanceRegistry = HashMap::new();
        let mut all_context_int_providers: ContextProviderRegistry = HashMap::new();
        let mut all_context_float_providers: ContextProviderRegistry = HashMap::new();
        let mut all_damage_type_defs: HashMap<String, DamageTypeDefinition> = HashMap::new();
        let mut all_loot_tables: HashMap<String, Arc<DynamicLootTable>> = HashMap::new();

        // Embedded test instances are compile-time constants and always load,
        // so on-disk packs can override them by id.
        let embedded_count = test_loader::load_embedded_test_instances(&mut all_test_instances);
        if embedded_count > 0 {
            info!("Loaded {embedded_count} embedded test instance(s)");
        }
        let (embedded_ints, embedded_floats) =
            context_provider_loader::load_embedded_context_providers(
                &mut all_context_int_providers,
                &mut all_context_float_providers,
            );
        if embedded_ints > 0 || embedded_floats > 0 {
            info!(
                "Loaded {embedded_ints} int and {embedded_floats} float embedded context provider(s)"
            );
        }
        let mut all_dynamic_registries: HashMap<String, HashMap<String, RegistryEntryData>> =
            HashMap::new();
        let environment_count = test_loader::load_test_environments(
            all_dynamic_registries
                .entry("minecraft:test_environment".to_string())
                .or_default(),
        );
        if environment_count > 0 {
            info!("Loaded {environment_count} environment(s)");
        }
        let mut all_trade_registry = TradeRegistry::new();
        let mut all_advancements: HashMap<
            String,
            advancement_loader::DynamicAdvancementDefinition,
        > = HashMap::new();
        let mut all_predicates: HashMap<String, serde_json::Value> = HashMap::new();
        let mut all_item_modifiers: HashMap<String, serde_json::Value> = HashMap::new();
        let mut all_slot_sources: HashMap<String, serde_json::Value> = HashMap::new();
        let mut all_tags: HashMap<String, HashMap<String, tag_loader::DynamicTagEntry>> =
            HashMap::new();

        let mut acc = PackContentAccumulators {
            recipes: &mut all_recipes,
            functions: &mut all_functions,
            function_tags: &mut all_function_tags,
            test_instances: &mut all_test_instances,
            damage_types: &mut all_damage_type_defs,
            context_int_providers: &mut all_context_int_providers,
            context_float_providers: &mut all_context_float_providers,
            loot_tables: &mut all_loot_tables,
            dynamic_registries: &mut all_dynamic_registries,
            trade_registry: &mut all_trade_registry,
            advancements: &mut all_advancements,
            predicates: &mut all_predicates,
            item_modifiers: &mut all_item_modifiers,
            slot_sources: &mut all_slot_sources,
            tags: &mut all_tags,
        };

        load_enabled_packs(
            &datapacks_dir,
            enabled_packs,
            &mut acc,
            &mut loaded_packs_vec,
        );

        let damage_type_registry = build_damage_type_registry(all_damage_type_defs);

        recipe_manager.set_recipes(all_recipes);
        *self
            .loaded_packs
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = loaded_packs_vec;
        let all_functions = share_function_bodies(all_functions);
        *self
            .functions
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = all_functions;
        *self
            .function_tags
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = all_function_tags;
        *self
            .test_instances
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = all_test_instances;
        *self
            .context_int_providers
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = all_context_int_providers;
        *self
            .context_float_providers
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = all_context_float_providers;
        for (id, entry) in &damage_type_registry {
            all_dynamic_registries
                .entry("minecraft:damage_type".to_string())
                .or_default()
                .insert(id.clone(), entry.to_registry_entry());
        }
        self.dynamic_registry_manager
            .apply_custom_registries(&all_dynamic_registries);

        *self
            .damage_types
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = damage_type_registry;
        *self
            .loot_tables
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = all_loot_tables;
        *self
            .dynamic_registries
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = all_dynamic_registries;

        *self
            .trade_registry
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = all_trade_registry;
        *self
            .advancements
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = all_advancements;
        *self
            .predicates
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = all_predicates;
        *self
            .item_modifiers
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = all_item_modifiers;
        *self
            .slot_sources
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = all_slot_sources;
        *self
            .tags
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = all_tags;
    }

    #[must_use]
    pub fn get_predicate(&self, id: &str) -> Option<serde_json::Value> {
        let guard = self
            .predicates
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let full_id = if id.contains(':') {
            id.to_string()
        } else {
            format!("minecraft:{id}")
        };
        guard
            .get(&full_id)
            .or_else(|| guard.get(id))
            .cloned()
            .or_else(|| {
                pumpkin_data::loot_table::get_loot_predicate(&full_id)
                    .or_else(|| pumpkin_data::loot_table::get_loot_predicate(id))
                    .cloned()
            })
    }

    #[must_use]
    pub fn get_predicate_ids(&self) -> Vec<String> {
        let guard = self
            .predicates
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut names: Vec<_> = guard.keys().cloned().collect();
        names.sort_unstable();
        names
    }

    #[must_use]
    pub fn get_item_modifier(&self, id: &str) -> Option<serde_json::Value> {
        let guard = self
            .item_modifiers
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let full_id = if id.contains(':') {
            id.to_string()
        } else {
            format!("minecraft:{id}")
        };
        guard.get(&full_id).or_else(|| guard.get(id)).cloned()
    }

    #[must_use]
    pub fn get_item_modifier_ids(&self) -> Vec<String> {
        let guard = self
            .item_modifiers
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut names: Vec<_> = guard.keys().cloned().collect();
        names.sort_unstable();
        names
    }

    #[must_use]
    pub fn get_slot_source(&self, id: &str) -> Option<serde_json::Value> {
        let guard = self
            .slot_sources
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let full_id = if id.contains(':') {
            id.to_string()
        } else {
            format!("minecraft:{id}")
        };
        guard.get(&full_id).or_else(|| guard.get(id)).cloned()
    }

    #[must_use]
    pub fn get_slot_source_ids(&self) -> Vec<String> {
        let guard = self
            .slot_sources
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut names: Vec<_> = guard.keys().cloned().collect();
        names.sort_unstable();
        names
    }

    #[must_use]
    pub fn get_tag_values(&self, category: &str, tag_id: &str) -> Option<Vec<String>> {
        let guard = self
            .tags
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let cat_map = guard.get(category)?;
        let full_id = if tag_id.contains(':') {
            tag_id.to_string()
        } else {
            format!("minecraft:{tag_id}")
        };
        cat_map
            .get(&full_id)
            .or_else(|| cat_map.get(tag_id))
            .map(|e| e.values.clone())
    }

    #[must_use]
    pub fn get_tags_for_category(&self, category: &str) -> HashMap<String, Vec<String>> {
        let guard = self
            .tags
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.get(category).map_or_else(HashMap::new, |cat_map| {
            cat_map
                .iter()
                .map(|(k, v)| (k.clone(), v.values.clone()))
                .collect()
        })
    }

    pub fn get_advancement(
        &self,
        id: &str,
    ) -> Option<advancement_loader::DynamicAdvancementDefinition> {
        self.advancements
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(id)
            .cloned()
    }

    pub fn get_advancement_ids(&self) -> Vec<String> {
        self.advancements
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .keys()
            .cloned()
            .collect()
    }

    pub fn get_loaded_packs(&self) -> Vec<LoadedDatapack> {
        self.loaded_packs
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn get_functions(&self) -> HashMap<String, Vec<String>> {
        self.functions
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .map(|(name, lines)| (name.clone(), lines.to_vec()))
            .collect()
    }

    pub fn get_test_instance(&self, name: &str) -> Option<TestInstance> {
        self.test_instances
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(name)
            .cloned()
    }

    pub fn get_test_instance_names(&self) -> Vec<String> {
        let test_instances = self
            .test_instances
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut names: Vec<_> = test_instances.keys().cloned().collect();
        names.sort_unstable();
        names
    }

    #[must_use]
    pub fn get_context_int_provider(&self, id: &str) -> Option<NbtTag> {
        self.context_int_providers
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(id)
            .cloned()
    }

    #[must_use]
    pub fn get_context_float_provider(&self, id: &str) -> Option<NbtTag> {
        self.context_float_providers
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(id)
            .cloned()
    }

    #[must_use]
    pub fn get_context_int_provider_names(&self) -> Vec<String> {
        let providers = self
            .context_int_providers
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut names: Vec<_> = providers.keys().cloned().collect();
        names.sort_unstable();
        names
    }

    #[must_use]
    pub fn get_context_float_provider_names(&self) -> Vec<String> {
        let providers = self
            .context_float_providers
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut names: Vec<_> = providers.keys().cloned().collect();
        names.sort_unstable();
        names
    }

    pub fn insert_context_int_provider(&self, id: String, tag: NbtTag) {
        self.context_int_providers
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(id, tag);
    }

    pub fn insert_context_float_provider(&self, id: String, tag: NbtTag) {
        self.context_float_providers
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(id, tag);
    }

    #[must_use]
    pub fn get_loot_table(&self, key: &str) -> Option<LootTableHandle> {
        let full_key = if key.contains(':') {
            key.to_string()
        } else {
            format!("minecraft:{key}")
        };

        let guard = self
            .loot_tables
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if let Some(table) = guard.get(&full_key).or_else(|| guard.get(key)) {
            return Some(LootTableHandle::Dynamic(table.clone()));
        }

        pumpkin_data::loot_table::get_loot_table(key)
            .or_else(|| pumpkin_data::loot_table::get_loot_table(&full_key))
            .map(LootTableHandle::Static)
    }

    #[must_use]
    pub fn get_loot_table_names(&self) -> Vec<String> {
        let guard = self
            .loot_tables
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut names: Vec<_> = guard.keys().cloned().collect();
        names.sort_unstable();
        names
    }

    pub fn insert_loot_table(&self, key: String, table: Arc<DynamicLootTable>) {
        self.loot_tables
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(key, table);
    }

    pub fn remove_loot_table(&self, key: &str) {
        self.loot_tables
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(key);
    }

    #[must_use]
    pub fn get_damage_type(&self, id: &str) -> Option<DamageType> {
        let guard = self
            .damage_types
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let full_id = if id.contains(':') {
            id.to_string()
        } else {
            format!("minecraft:{id}")
        };
        guard
            .get(&full_id)
            .or_else(|| guard.get(id))
            .map(|e| e.damage_type)
    }

    #[must_use]
    pub fn get_damage_type_names(&self) -> Vec<String> {
        let guard = self
            .damage_types
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut names: Vec<_> = guard.keys().cloned().collect();
        names.sort_unstable();
        names
    }

    #[must_use]
    pub fn get_damage_type_registry_map(&self) -> HashMap<String, DamageTypeEntry> {
        self.damage_types
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    #[must_use]
    pub fn get_all_damage_type_registry_entries(&self) -> Vec<RegistryEntryData> {
        let guard = self
            .damage_types
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut entries: Vec<RegistryEntryData> = guard
            .values()
            .map(DamageTypeEntry::to_registry_entry)
            .collect();
        entries.sort_by(|a, b| a.entry_id.cmp(&b.entry_id));
        entries
    }

    #[must_use]
    pub fn merge_damage_type_entries(
        &self,
        vanilla_entries: &[RegistryEntryData],
    ) -> Vec<RegistryEntryData> {
        let guard = self
            .damage_types
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        merge_damage_type_entries(vanilla_entries, &guard)
    }

    #[must_use]
    pub fn merge_registry_entries(
        &self,
        registry_id: &str,
        vanilla_entries: &[RegistryEntryData],
    ) -> Vec<RegistryEntryData> {
        let guard = self
            .dynamic_registries
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let custom = guard
            .get(registry_id)
            .or_else(|| guard.get(&format!("minecraft:{registry_id}")))
            .or_else(|| {
                registry_id
                    .strip_prefix("minecraft:")
                    .and_then(|r| guard.get(r))
            });
        custom.map_or_else(
            || vanilla_entries.iter().map(clone_registry_entry).collect(),
            |custom| {
                dynamic_registry_loader::merge_dynamic_registry_entries(vanilla_entries, custom)
            },
        )
    }

    #[must_use]
    pub fn get_custom_registry_entry(
        &self,
        registry_id: &str,
        entry_id: &str,
    ) -> Option<RegistryEntryData> {
        let guard = self
            .dynamic_registries
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let custom_map = guard
            .get(registry_id)
            .or_else(|| guard.get(&format!("minecraft:{registry_id}")))
            .or_else(|| {
                registry_id
                    .strip_prefix("minecraft:")
                    .and_then(|r| guard.get(r))
            })?;
        custom_map.get(entry_id).map(clone_registry_entry)
    }

    #[must_use]
    pub fn get_custom_registry_entries(&self, registry_id: &str) -> Vec<RegistryEntryData> {
        let guard = self
            .dynamic_registries
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let custom_map = guard
            .get(registry_id)
            .or_else(|| guard.get(&format!("minecraft:{registry_id}")))
            .or_else(|| {
                registry_id
                    .strip_prefix("minecraft:")
                    .and_then(|r| guard.get(r))
            });
        custom_map.map_or_else(Vec::new, |entries| {
            let mut list: Vec<_> = entries.values().map(clone_registry_entry).collect();
            list.sort_by(|a, b| a.entry_id.cmp(&b.entry_id));
            list
        })
    }

    #[must_use]
    pub fn get_dynamic_registry_ids(&self) -> Vec<String> {
        let guard = self
            .dynamic_registries
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut ids: Vec<_> = guard.keys().cloned().collect();
        ids.sort();
        ids
    }

    #[must_use]
    pub fn get_dynamic_registry_raw_id(&self, registry_id: &str, entry_id: &str) -> Option<u32> {
        self.dynamic_registry_manager
            .get_raw_id(registry_id, entry_id)
    }

    #[must_use]
    pub fn get_dynamic_registry_entry_id(&self, registry_id: &str, raw_id: u32) -> Option<String> {
        self.dynamic_registry_manager
            .get_entry_id(registry_id, raw_id)
    }

    #[must_use]
    pub fn get_dynamic_registry_entry_data(
        &self,
        registry_id: &str,
        entry_id: &str,
    ) -> Option<Box<[u8]>> {
        self.dynamic_registry_manager
            .get_entry_data(registry_id, entry_id)
    }

    #[must_use]
    pub fn get_dynamic_registry_entries(&self, registry_id: &str) -> Vec<RegistryEntryData> {
        self.dynamic_registry_manager
            .get_registry_entries(registry_id)
    }

    #[must_use]
    pub fn has_dynamic_registry_entry(&self, registry_id: &str, entry_id: &str) -> bool {
        self.dynamic_registry_manager
            .contains_entry(registry_id, entry_id)
    }

    #[must_use]
    pub fn get_all_synced_registries(&self) -> Vec<pumpkin_data::registry::Registry> {
        let mut registries = self.dynamic_registry_manager.get_synced_registries();
        if let Some(test_instances) = registries
            .iter_mut()
            .find(|registry| registry.registry_id == "minecraft:test_instance")
        {
            test_instances.registry_entries = self.get_test_instance_registry_entries();
        }
        registries
    }

    #[must_use]
    pub fn is_in_tag(&self, category: &str, tag_id: &str, entry_id: &str) -> bool {
        let guard = self
            .tags
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let category = category.strip_prefix("minecraft:").unwrap_or(category);
        if let Some(cat_tags) = guard.get(category) {
            let tag_id = tag_id.strip_prefix('#').unwrap_or(tag_id);
            let full_tag = if tag_id.contains(':') {
                tag_id.to_string()
            } else {
                format!("minecraft:{tag_id}")
            };
            if let Some(tag_entry) = cat_tags.get(&full_tag).or_else(|| cat_tags.get(tag_id)) {
                let full_entry = if entry_id.contains(':') {
                    entry_id.to_string()
                } else {
                    format!("minecraft:{entry_id}")
                };
                return tag_entry
                    .values
                    .iter()
                    .any(|v| v == &full_entry || v == entry_id);
            }
        }
        false
    }

    #[must_use]
    pub fn get_tag_entries(&self, category: &str, tag_id: &str) -> Option<Vec<String>> {
        let guard = self
            .tags
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let category = category.strip_prefix("minecraft:").unwrap_or(category);
        let cat_tags = guard.get(category)?;
        let tag_id = tag_id.strip_prefix('#').unwrap_or(tag_id);
        let full_tag = if tag_id.contains(':') {
            tag_id.to_string()
        } else {
            format!("minecraft:{tag_id}")
        };
        let tag_entry = cat_tags.get(&full_tag).or_else(|| cat_tags.get(tag_id))?;
        Some(tag_entry.values.clone())
    }

    #[must_use]
    pub fn get_villager_trade_set(
        &self,
        profession: &str,
        level: i32,
    ) -> Option<DynamicVillagerTradeSet> {
        let registry = self
            .trade_registry
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        registry.get_villager_trade_set(profession, level)
    }

    #[must_use]
    pub fn get_wandering_trader_trade_set(&self, tier: &str) -> Option<DynamicVillagerTradeSet> {
        let registry = self
            .trade_registry
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        registry.get_wandering_trader_trade_set(tier)
    }

    #[must_use]
    pub fn get_trade_set(&self, key: &str) -> Option<DynamicVillagerTradeSet> {
        let registry = self
            .trade_registry
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        registry.resolve_trade_set(key)
    }

    /// Returns datapack test instances in the protocol's synced-registry entry format.
    /// The vanilla Test Instance Block renderer resolves required/padding/base rotation
    /// through this registry using the controller's `data.test` resource key.
    pub fn get_test_instance_registry_entries(&self) -> Vec<RegistryEntryData> {
        let test_instances = self
            .test_instances
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut entries: Vec<_> = test_instances
            .iter()
            .map(|(id, instance)| to_registry_entry(id.clone(), instance))
            .collect();
        entries.sort_unstable_by(|left, right| left.entry_id.cmp(&right.entry_id));
        entries
    }

    /// Returns the list of known packs for client configuration / pack synchronization,
    /// matching vanilla's `server.getResourceManager().listPacks().flatMap(p -> p.location().knownPackInfo().stream())`.
    pub fn get_known_packs<'a>(
        &self,
        server: &Server,
        server_version: &'a str,
        loaded_packs: &'a [LoadedDatapack],
    ) -> Vec<pumpkin_protocol::KnownPack<'a>> {
        use pumpkin_protocol::KnownPack;

        let mut known_packs = Vec::new();

        // 1. Primary vanilla core pack
        known_packs.push(KnownPack {
            namespace: "minecraft",
            id: "core",
            version: server_version,
        });

        // 2. Built-in feature packs that are enabled, vanilla `KnownPack.vanilla(id)`
        let enabled_packs = Self::get_enabled_packs(server);
        for pack_name in &enabled_packs {
            let Some(feature_pack) = loaded_packs
                .iter()
                .find(|pack| pack.id == *pack_name && !pack.id.starts_with("file/"))
            else {
                continue;
            };
            let pack = KnownPack {
                namespace: "minecraft",
                id: &feature_pack.id,
                version: server_version,
            };
            if !known_packs
                .iter()
                .any(|p| p.namespace == pack.namespace && p.id == pack.id)
            {
                known_packs.push(pack);
            }
        }

        // 3. Loaded packs with known_pack info from pack.mcmeta
        for pack in loaded_packs {
            for kp in &pack.known_packs {
                let p = KnownPack {
                    namespace: &kp.namespace,
                    id: &kp.id,
                    version: &kp.version,
                };
                if !known_packs.iter().any(|existing| {
                    existing.namespace == p.namespace
                        && existing.id == p.id
                        && existing.version == p.version
                }) {
                    known_packs.push(p);
                }
            }
        }

        known_packs
    }

    /// Vanilla `WorldDataConfiguration.enabledFeatures`: fixed when the world loads.
    #[must_use]
    pub fn get_enabled_features(server: &Server) -> Vec<&'static str> {
        server
            .level_info
            .load()
            .enabled_features
            .iter()
            .filter_map(|feature| feature_flags::ALL.iter().find(|flag| **flag == feature))
            .copied()
            .collect()
    }

    #[must_use]
    pub fn is_feature_enabled(server: &Server, feature: &str) -> bool {
        server.level_info.load().is_feature_enabled(feature)
    }

    /// The flags `pack` requests that this world doesn't have; vanilla won't enable it then.
    /// `None` when vanilla can't read the pack at all.
    #[must_use]
    pub fn missing_features(server: &Server, pack: &str) -> Option<Vec<&'static str>> {
        if Self::is_embedded_pack(pack) {
            return Some(Vec::new());
        }
        let available = available_packs(&server.basic_config.get_world_path(), false);
        let pack = available.iter().find(|available| available.id == pack)?;
        Some(missing_flags(
            &pack.requested_features,
            &server.level_info.load().enabled_features,
        ))
    }

    /// Vanilla `DataPackCommand.getPack`: a feature pack that requests flags can't be disabled.
    #[must_use]
    pub fn is_feature_pack(pack: &str) -> bool {
        feature_packs().iter().any(|feature_pack| {
            feature_pack.id == pack && !feature_pack.requested_features.is_empty()
        })
    }

    /// Vanilla `MinecraftServer.configurePackRepository`: the packs and features `level` runs
    /// with. A new world allows every feature, an existing one only the features it was saved
    /// with, which it also keeps.
    pub fn configure_world_packs(world_path: &Path, level: &mut LevelData, new_world: bool) {
        let available = available_packs(world_path, true);
        let legacy = !new_world && level.enabled_features_missing;
        if legacy {
            enable_legacy_pack_features(level, &available);
        }
        let saved: Vec<&'static str> = level
            .enabled_features
            .iter()
            .filter_map(|feature| {
                let flag = feature_flags::ALL.iter().find(|flag| **flag == feature);
                if flag.is_none() {
                    warn!("Unknown feature flag: {feature}");
                }
                flag.copied()
            })
            .collect();
        let allowed: Vec<&'static str> = if new_world {
            feature_flags::ALL.to_vec()
        } else {
            saved.clone()
        };
        let forced = if new_world { Vec::new() } else { saved };

        let features_of = |id: &str| {
            available
                .iter()
                .find(|pack| pack.id == id)
                .map(|pack| pack.requested_features.as_slice())
        };
        let mut selected: Vec<String> = Vec::new();
        for id in &level.data_packs.enabled {
            if selected.contains(id) {
                continue;
            }
            if features_of(id).is_some() {
                selected.push(id.clone());
            } else if let Some(pack) = legacy_feature_pack(id, &available).filter(|_| legacy) {
                // Takes the alias's place, so the pack keeps its load order.
                if !selected.contains(&pack.id) {
                    info!("Using feature pack {} for data pack {id}", pack.id);
                    selected.push(pack.id.clone());
                }
            } else if is_world_archive_pack(world_path, id) {
                warn!("Can't load data pack {id}, Pumpkin only loads folder packs");
                selected.push(id.clone());
            } else if !Self::is_embedded_pack(id) {
                warn!("Missing data pack {id}");
            }
        }
        for pack in &available {
            let id = &pack.id;
            if level.data_packs.disabled.contains(id) {
                continue;
            }
            let is_selected = selected.contains(id);
            let missing = missing_flags(&pack.requested_features, &allowed);
            if !is_selected && pack.source.should_add_automatically() {
                if missing.is_empty() {
                    info!("Found new data pack {id}, loading it automatically");
                    selected.push(id.clone());
                } else {
                    info!(
                        "Found new data pack {id}, but can't load it due to missing features {}",
                        missing.join(", ")
                    );
                }
            }
            if is_selected && !missing.is_empty() {
                warn!(
                    "Pack {id} requires features {} that are not enabled for this world, disabling pack.",
                    missing.join(", ")
                );
                selected.retain(|pack| pack != id);
            }
        }
        if selected.is_empty() {
            info!("No datapacks selected, forcing vanilla");
            selected.push(DEFAULT_ENABLED_DATA_PACK.to_owned());
        }

        enable_forced_feature_packs(&mut selected, &forced, &available);

        let mut features = vec![feature_flags::VANILLA];
        for feature in selected
            .iter()
            .filter_map(|id| features_of(id))
            .flatten()
            .chain(&forced)
        {
            if !features.contains(feature) {
                features.push(feature);
            }
        }
        level.data_packs.disabled = disabled_packs(
            world_path,
            &available,
            &selected,
            &level.data_packs.disabled,
        );
        level.data_packs.enabled = selected;
        level.enabled_features = features.into_iter().map(str::to_owned).collect();
    }

    /// Loads a Java Edition structure NBT from generated data or enabled datapacks.
    ///
    /// Structure identifiers are resource locations such as
    /// `minecraft:village/plains/houses/plains_small_house_1`. Both the current
    /// `structure` directory and the legacy `structures` directory are checked.
    pub async fn load_structure(
        &self,
        world_path: &Path,
        resource_location: &str,
    ) -> Result<NbtCompound, String> {
        let (namespace, path) = parse_structure_resource_location(resource_location)?;
        let generated_root = world_path.join("generated");
        let pack_roots = {
            let loaded_packs = self
                .loaded_packs
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            loaded_packs
                .iter()
                .rev()
                .map(|pack| pack.root_path.clone())
                .collect::<Vec<_>>()
        };
        let namespace = namespace.to_string();
        let path = path.to_string();
        let resource_location = resource_location.to_string();

        tokio::task::spawn_blocking(move || {
            let candidate = generated_root
                .join(&namespace)
                .join("structure")
                .join(format!("{path}.nbt"));
            if candidate.is_file() {
                let display_path = candidate.display();
                let file = fs::File::open(&candidate).map_err(|error| {
                    format!("Failed to open structure '{display_path}': {error}")
                })?;
                return read_gzip_compound_tag(file).map_err(|error| {
                    format!("Failed to parse structure '{display_path}': {error}")
                });
            }
            // Runtime datapacks override embedded resources.
            for root in pack_roots {
                for structure_dir in ["structure", "structures"] {
                    let candidate = root
                        .join("data")
                        .join(&namespace)
                        .join(structure_dir)
                        .join(format!("{path}.nbt"));

                    if candidate.is_file() {
                        let display_path = candidate.display();
                        let file = fs::File::open(&candidate).map_err(|error| {
                            format!("Failed to open structure '{display_path}': {error}")
                        })?;

                        return read_gzip_compound_tag(file).map_err(|error| {
                            format!("Failed to parse structure '{display_path}': {error}")
                        });
                    }
                }
            }

            // Fall back to compile-time embedded structures.
            let structure_id = format!("{namespace}:{path}");
            if let Some(template) =
                pumpkin_world::generation::structure::template::get_template(&structure_id)
            {
                return Ok(template.save());
            }

            Err(format!(
                "Structure '{resource_location}' was not found in any enabled datapack or embedded resources"
            ))
        })
        .await
        .map_err(|error| format!("Structure loader task failed: {error}"))?
    }

    /// Saves a structure NBT under the world's generated data directory.
    pub async fn save_generated_structure(
        &self,
        world_path: &Path,
        resource_location: &str,
        structure: NbtCompound,
    ) -> Result<PathBuf, String> {
        let (namespace, path) = parse_structure_resource_location(resource_location)?;
        let file_path = world_path
            .join("generated")
            .join(namespace)
            .join("structure")
            .join(format!("{path}.nbt"));
        tokio::task::spawn_blocking(move || {
            let parent = file_path
                .parent()
                .ok_or_else(|| "Structure has no parent directory".to_string())?;
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            let file = fs::File::create(&file_path).map_err(|error| error.to_string())?;
            pumpkin_nbt::nbt_compress::write_gzip_compound_tag(
                structure,
                std::io::BufWriter::new(file),
            )
            .map_err(|error| error.to_string())?;
            Ok(file_path)
        })
        .await
        .map_err(|error| error.to_string())?
    }

    pub fn get_function_names(&self) -> Vec<String> {
        let fns = self
            .functions
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let tags = self
            .function_tags
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut names = Vec::with_capacity(fns.len() + tags.len());
        names.extend(fns.keys().cloned());
        for tag in tags.keys() {
            names.push(format!("#{tag}"));
        }
        names
    }

    /// Runs every line of a function or function tag; also the single place chains are bounded.
    pub fn execute_function(
        &self,
        server: &Arc<Server>,
        source: &CommandSource,
        name: &str,
    ) -> Result<usize, FunctionRunError> {
        let Ok(_guard) = FunctionChainGuard::enter() else {
            return Err(FunctionRunError::ChainLimitExceeded);
        };

        self.visit_function_lines(name, |line| {
            server
                .command_dispatcher
                .load()
                .handle_command(source, line);
        })
    }

    fn visit_function_lines(
        &self,
        name: &str,
        mut visit: impl FnMut(&str),
    ) -> Result<usize, FunctionRunError> {
        let (functions_to_run, is_tag) = if let Some(tag_name) = name.strip_prefix('#') {
            let tags = self
                .function_tags
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(fns) = tags.get(tag_name) else {
                return Err(FunctionRunError::Unknown(format!(
                    "Unknown function tag: #{tag_name}"
                )));
            };
            (fns.clone(), true)
        } else {
            (vec![name.to_string()], false)
        };

        let functions = {
            let all_fns = self
                .functions
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut functions = Vec::with_capacity(functions_to_run.len());
            for fn_id in functions_to_run {
                let Some(lines) = all_fns.get(&fn_id) else {
                    if !is_tag {
                        return Err(FunctionRunError::Unknown(format!(
                            "Unknown function: {fn_id}"
                        )));
                    }
                    continue;
                };
                functions.push(Arc::clone(lines));
            }
            functions
        };

        let mut total_executed = 0;
        for lines in functions {
            if let Some(error) = Self::boundary_limit() {
                return Err(error);
            }
            for line in lines.iter() {
                if let Some(error) = Self::boundary_limit() {
                    return Err(error);
                }
                if !FunctionChainGuard::count_command() {
                    return Err(FunctionRunError::CommandChainLimitExceeded);
                }
                visit(line);
                total_executed += 1;
                if let Some(error) = Self::boundary_limit() {
                    return Err(error);
                }
            }
        }

        Ok(total_executed)
    }

    fn boundary_limit() -> Option<FunctionRunError> {
        if FUNCTION_CHAIN_HIT_COMMAND_LIMIT.with(Cell::get) {
            return Some(FunctionRunError::CommandChainLimitExceeded);
        }
        if FUNCTION_CHAIN_HIT_DEPTH_LIMIT.with(Cell::get) {
            return Some(FunctionRunError::ChainLimitExceeded);
        }
        None
    }

    #[must_use]
    pub fn is_embedded_pack(name: &str) -> bool {
        pumpkin_world::generation::structure::template::all_embedded_datapack_names()
            .contains(&name)
    }

    pub fn get_all_known_packs(server: &Server) -> Vec<String> {
        let mut packs: Vec<String> =
            pumpkin_world::generation::structure::template::all_embedded_datapack_names()
                .iter()
                .map(|name| (*name).to_owned())
                .collect();

        for pack in available_packs(&server.basic_config.get_world_path(), false) {
            if !packs.contains(&pack.id) {
                packs.push(pack.id);
            }
        }
        packs
    }

    pub fn get_enabled_packs(server: &Server) -> Vec<String> {
        // Compile-time embedded datapacks cannot be disabled at runtime, so
        // expose them as permanently enabled.
        let mut packs: Vec<String> =
            pumpkin_world::generation::structure::template::all_embedded_datapack_names()
                .iter()
                .map(|name| (*name).to_owned())
                .collect();

        // Zip packs stay in `level.dat` for vanilla, but Pumpkin can't load them.
        let world_path = server.basic_config.get_world_path();
        for pack in &server.level_info.load().data_packs.enabled {
            if !is_world_archive_pack(&world_path, pack)
                && !packs.iter().any(|existing| existing == pack)
            {
                packs.push(pack.clone());
            }
        }

        packs
    }

    pub fn get_available_packs(server: &Server) -> Vec<String> {
        let enabled = Self::get_enabled_packs(server);
        let level_info = server.level_info.load();
        // Vanilla lists only packs this world's features allow.
        available_packs(&server.basic_config.get_world_path(), false)
            .into_iter()
            .filter(|pack| {
                !enabled.contains(&pack.id)
                    && missing_flags(&pack.requested_features, &level_info.enabled_features)
                        .is_empty()
            })
            .map(|pack| pack.id)
            .collect()
    }

    pub fn find_pack_name(server: &Server, input: &str) -> Option<String> {
        Self::resolve_pack_name(&Self::get_all_known_packs(server), input)
    }

    /// The pack `/datapack disable` targets. Vanilla drops a pack that became unreadable from the
    /// selection on reload, but Pumpkin keeps it in `DataPacks.Enabled`, so it stays removable.
    pub fn find_pack_to_disable(server: &Server, input: &str) -> Option<String> {
        Self::find_pack_name(server, input)
            .or_else(|| Self::resolve_pack_name(&Self::get_enabled_packs(server), input))
    }

    /// Resolves `input` to one of `known`, also matching a world pack's name without `file/`.
    fn resolve_pack_name(known: &[String], input: &str) -> Option<String> {
        if let Some(p) = known.iter().find(|p| *p == input) {
            return Some(p.clone());
        }
        let file_input = format!("file/{input}");
        if let Some(p) = known.iter().find(|p| **p == file_input) {
            return Some(p.clone());
        }
        if let Some(p) = known
            .iter()
            .find(|p| p.strip_prefix("file/") == Some(input))
        {
            return Some(p.clone());
        }
        None
    }

    pub fn get_pack_info(server: &Server, name_or_id: &str) -> Option<DatapackInfo> {
        let resolved_name = Self::find_pack_name(server, name_or_id)?;
        let enabled_packs = Self::get_enabled_packs(server);
        let loaded = server.datapack_manager.get_loaded_packs();
        Some(Self::pack_info(
            server,
            &resolved_name,
            &enabled_packs,
            &loaded,
        ))
    }

    /// Details of the known pack `resolved_name`.
    fn pack_info(
        server: &Server,
        resolved_name: &str,
        enabled_packs: &[String],
        loaded: &[LoadedDatapack],
    ) -> DatapackInfo {
        let is_enabled = enabled_packs.iter().any(|pack| pack == resolved_name);
        if let Some(pack) = loaded.iter().find(|p| p.id == resolved_name) {
            return DatapackInfo {
                id: pack.id.clone(),
                name: pack.name.clone(),
                description: pack.description.clone(),
                pack_format: pack.pack_format,
                is_enabled,
                recipe_count: pack.recipe_count,
                function_count: pack.function_count,
            };
        }

        let (id, name, description, pack_format) = if resolved_name == "vanilla" {
            (
                "vanilla".to_string(),
                "vanilla".to_string(),
                "The default data pack".to_string(),
                61,
            )
        } else if Self::is_embedded_pack(resolved_name) {
            (
                resolved_name.to_owned(),
                resolved_name.to_owned(),
                format!("Embedded datapack: {resolved_name}"),
                61,
            )
        } else if let Some(stripped) = resolved_name.strip_prefix("file/") {
            let pack_path = server
                .basic_config
                .get_world_path()
                .join("datapacks")
                .join(stripped);
            let (desc, format, _) = read_pack_mcmeta(&pack_path);
            (resolved_name.to_owned(), stripped.to_string(), desc, format)
        } else {
            (
                resolved_name.to_owned(),
                resolved_name.to_owned(),
                format!("Bundled datapack: {resolved_name}"),
                61,
            )
        };

        DatapackInfo {
            id,
            name,
            description,
            pack_format,
            is_enabled,
            recipe_count: 0,
            function_count: 0,
        }
    }

    /// Details of `packs`, already resolved ids, without rescanning the packs per entry.
    fn pack_infos(server: &Server, packs: &[String]) -> Vec<DatapackInfo> {
        let enabled_packs = Self::get_enabled_packs(server);
        let loaded = server.datapack_manager.get_loaded_packs();
        packs
            .iter()
            .map(|pack| Self::pack_info(server, pack, &enabled_packs, &loaded))
            .collect()
    }

    pub fn list_all_packs(server: &Server) -> Vec<DatapackInfo> {
        Self::pack_infos(server, &Self::get_all_known_packs(server))
    }

    pub fn list_enabled_packs(server: &Server) -> Vec<DatapackInfo> {
        Self::pack_infos(server, &Self::get_enabled_packs(server))
    }

    pub fn list_available_packs(server: &Server) -> Vec<DatapackInfo> {
        Self::pack_infos(server, &Self::get_available_packs(server))
    }

    pub fn is_pack_enabled(server: &Server, name: &str) -> bool {
        let Some(resolved) = Self::find_pack_name(server, name) else {
            return false;
        };
        Self::get_enabled_packs(server).contains(&resolved)
    }

    pub fn enable_pack(
        server: &Arc<Server>,
        name: &str,
        position: DatapackEnablePosition,
    ) -> Result<(), String> {
        let Some(resolved_name) = Self::find_pack_name(server, name) else {
            return Err(format!("Unknown datapack '{name}'"));
        };

        let enabled = Self::get_enabled_packs(server);
        if enabled.contains(&resolved_name) {
            return Err(format!("Datapack '{resolved_name}' is already enabled"));
        }
        let Some(missing) = Self::missing_features(server, &resolved_name) else {
            return Err(format!("Unknown datapack '{resolved_name}'"));
        };
        if !missing.is_empty() {
            return Err(no_flags_error(&resolved_name, &missing));
        }

        let target = resolved_name;
        match position {
            DatapackEnablePosition::First => {
                server.level_info.rcu(|level_info| {
                    let mut new_info = (**level_info).clone();
                    new_info.data_packs.disabled.retain(|p| p != &target);
                    new_info.data_packs.enabled.retain(|p| p != &target);
                    new_info.data_packs.enabled.insert(0, target.clone());
                    new_info
                });
            }
            DatapackEnablePosition::Last => {
                server.level_info.rcu(|level_info| {
                    let mut new_info = (**level_info).clone();
                    new_info.data_packs.disabled.retain(|p| p != &target);
                    new_info.data_packs.enabled.retain(|p| p != &target);
                    new_info.data_packs.enabled.push(target.clone());
                    new_info
                });
            }
            DatapackEnablePosition::Before(existing_name) => {
                let Some(existing_pack) = Self::find_pack_name(server, &existing_name) else {
                    return Err(format!("Unknown existing datapack '{existing_name}'"));
                };
                if !enabled.contains(&existing_pack) {
                    return Err(format!("Datapack '{existing_pack}' is not enabled"));
                }
                server.level_info.rcu(|level_info| {
                    let mut new_info = (**level_info).clone();
                    new_info.data_packs.disabled.retain(|p| p != &target);
                    new_info.data_packs.enabled.retain(|p| p != &target);
                    if let Some(idx) = new_info
                        .data_packs
                        .enabled
                        .iter()
                        .position(|p| p == &existing_pack)
                    {
                        new_info.data_packs.enabled.insert(idx, target.clone());
                    } else {
                        new_info.data_packs.enabled.push(target.clone());
                    }
                    new_info
                });
            }
            DatapackEnablePosition::After(existing_name) => {
                let Some(existing_pack) = Self::find_pack_name(server, &existing_name) else {
                    return Err(format!("Unknown existing datapack '{existing_name}'"));
                };
                if !enabled.contains(&existing_pack) {
                    return Err(format!("Datapack '{existing_pack}' is not enabled"));
                }
                server.level_info.rcu(|level_info| {
                    let mut new_info = (**level_info).clone();
                    new_info.data_packs.disabled.retain(|p| p != &target);
                    new_info.data_packs.enabled.retain(|p| p != &target);
                    if let Some(idx) = new_info
                        .data_packs
                        .enabled
                        .iter()
                        .position(|p| p == &existing_pack)
                    {
                        new_info.data_packs.enabled.insert(idx + 1, target.clone());
                    } else {
                        new_info.data_packs.enabled.push(target.clone());
                    }
                    new_info
                });
            }
        }

        if let Err(err) = server.save_world_info() {
            tracing::error!("Failed to save world info: {err}");
        }

        server.reload_datapacks(server);
        Ok(())
    }

    pub fn disable_pack(server: &Arc<Server>, name: &str) -> Result<(), String> {
        let Some(target_pack) = Self::find_pack_to_disable(server, name) else {
            return Err(format!("Unknown datapack '{name}'"));
        };

        let enabled = Self::get_enabled_packs(server);
        if !enabled.contains(&target_pack) {
            return Err(format!("Datapack '{target_pack}' is not enabled"));
        }

        if Self::is_embedded_pack(&target_pack) {
            return Err(format!(
                "Cannot disable embedded datapack '{target_pack}' because it is compiled into the server"
            ));
        }
        if Self::is_feature_pack(&target_pack) {
            return Err(format!(
                "Pack '{target_pack}' cannot be disabled, since it is part of an enabled flag!"
            ));
        }
        // Vanilla `DataPackCommand.getPack` checks the flags when disabling too.
        if let Some(missing) = Self::missing_features(server, &target_pack)
            && !missing.is_empty()
        {
            return Err(no_flags_error(&target_pack, &missing));
        }

        let target = target_pack;
        server.level_info.rcu(|level_info| {
            let mut new_info = (**level_info).clone();
            new_info.data_packs.enabled.retain(|p| p != &target);
            if !new_info.data_packs.disabled.contains(&target) {
                new_info.data_packs.disabled.push(target.clone());
            }
            new_info
        });

        if let Err(err) = server.save_world_info() {
            tracing::error!("Failed to save world info: {err}");
        }

        server.reload_datapacks(server);
        Ok(())
    }

    pub fn reload(server: &Arc<Server>) -> Result<(), String> {
        server.reload_datapacks(server);
        Ok(())
    }

    pub fn execute_function_from_console(
        server: &Arc<Server>,
        name: &str,
    ) -> Result<usize, String> {
        let source = crate::command::CommandSender::Console.into_source(server);
        server
            .datapack_manager
            .execute_function(server, &source, name)
            .map_err(|error| error.to_string())
    }
}

fn parse_structure_resource_location(resource_location: &str) -> Result<(&str, &str), String> {
    let (namespace, raw_path) = resource_location
        .split_once(':')
        .unwrap_or(("minecraft", resource_location));

    if namespace.is_empty()
        || namespace.bytes().all(|byte| byte == b'.')
        || !namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
    {
        return Err(format!(
            "Invalid structure namespace in '{resource_location}'"
        ));
    }

    let path = raw_path.strip_suffix(".nbt").unwrap_or(raw_path);
    if path.is_empty()
        || path
            .split('/')
            .any(|segment| segment.is_empty() || segment.bytes().all(|byte| byte == b'.'))
        || !path.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'-' | b'.' | b'/')
        })
    {
        return Err(format!("Invalid structure path in '{resource_location}'"));
    }

    Ok((namespace, path))
}

fn build_damage_type_registry(
    mut all_damage_type_defs: HashMap<String, DamageTypeDefinition>,
) -> DamageTypeRegistry {
    let mut damage_type_registry: DamageTypeRegistry = HashMap::new();
    let mut next_custom_id = 51u8;
    let mut static_map: HashMap<String, &'static DamageType> = HashMap::new();
    let mut names = Vec::new();

    let mut sorted_ids: Vec<String> = all_damage_type_defs.keys().cloned().collect();
    sorted_ids.sort();

    for id in sorted_ids {
        let Some(def) = all_damage_type_defs.remove(&id) else {
            continue;
        };
        let path = id.split_once(':').map_or(id.as_str(), |(_, p)| p);
        let numeric_id = pumpkin_data::damage::DamageType::from_name(path)
            .or_else(|| pumpkin_data::damage::DamageType::from_name(&id))
            .map_or_else(
                || {
                    let nid = next_custom_id;
                    next_custom_id = next_custom_id.saturating_add(1);
                    nid
                },
                |vanilla_dt| vanilla_dt.id,
            );

        let dt = damage_type_loader::to_damage_type(numeric_id, &def);
        let nbt_data = damage_type_loader::to_registry_nbt_bytes(&def);
        let leaked_dt = Box::leak(Box::new(dt));
        static_map.insert(id.clone(), leaked_dt);
        static_map.insert(path.to_string(), leaked_dt);
        names.push(id.clone());

        damage_type_registry.insert(
            id.clone(),
            DamageTypeEntry {
                id,
                numeric_id,
                definition: def,
                damage_type: dt,
                nbt_data,
            },
        );
    }

    pumpkin_command::argument_types::resource::register_dynamic_damage_types(
        Box::new(move |req_id: &str| static_map.get(req_id).copied()),
        names,
    );

    damage_type_registry
}

struct PackContentAccumulators<'a> {
    recipes: &'a mut Vec<DynamicRecipe>,
    functions: &'a mut HashMap<String, Vec<String>>,
    function_tags: &'a mut HashMap<String, Vec<String>>,
    test_instances: &'a mut TestInstanceRegistry,
    damage_types: &'a mut HashMap<String, DamageTypeDefinition>,
    context_int_providers: &'a mut ContextProviderRegistry,
    context_float_providers: &'a mut ContextProviderRegistry,
    loot_tables: &'a mut HashMap<String, Arc<DynamicLootTable>>,
    dynamic_registries: &'a mut HashMap<String, HashMap<String, RegistryEntryData>>,
    trade_registry: &'a mut TradeRegistry,
    advancements: &'a mut HashMap<String, advancement_loader::DynamicAdvancementDefinition>,
    predicates: &'a mut HashMap<String, serde_json::Value>,
    item_modifiers: &'a mut HashMap<String, serde_json::Value>,
    slot_sources: &'a mut HashMap<String, serde_json::Value>,
    tags: &'a mut HashMap<String, HashMap<String, tag_loader::DynamicTagEntry>>,
}

#[derive(Default, Clone, Copy)]
struct PackContentCounts {
    recipes: usize,
    functions: usize,
    test_instances: usize,
    damage_types: usize,
    int_providers: usize,
    float_providers: usize,
    loot_tables: usize,
    structures: usize,
    template_pools: usize,
    processor_lists: usize,
    dynamic_registries: usize,
    trade_sets: usize,
    trades: usize,
    trade_tags: usize,
    advancements: usize,
    predicates: usize,
    item_modifiers: usize,
    slot_sources: usize,
    tags: usize,
}

/// The bundled copies of the vanilla jar's `data/minecraft/datapacks`, which hold the feature
/// packs, in lookup order.
fn bundled_datapacks_dirs() -> Vec<PathBuf> {
    let sub = "assets/datapack/data/minecraft/datapacks";
    let mut dirs = vec![
        PathBuf::from(sub),
        PathBuf::from(format!("../{sub}")),
        PathBuf::from(format!("../../{sub}")),
    ];
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        dirs.push(Path::new(&manifest_dir).join(format!("../../{sub}")));
    }
    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        dirs.extend(
            exe_dir
                .ancestors()
                .take(5)
                .map(|ancestor| ancestor.join(sub)),
        );
    }
    dirs.retain(|dir| dir.is_dir());
    dirs
}

/// The folder packs in a world's `datapacks` directory, by folder name.
fn world_pack_dirs(datapacks_dir: &Path) -> Vec<(String, PathBuf)> {
    if !datapacks_dir.is_dir() {
        return Vec::new();
    }
    let entries = match fs::read_dir(datapacks_dir) {
        Ok(entries) => entries,
        Err(error) => {
            warn!(
                "Failed to read datapacks directory '{}': {error}",
                datapacks_dir.display()
            );
            return Vec::new();
        }
    };
    entries
        .flatten()
        .map(|entry| {
            (
                entry.file_name().to_string_lossy().into_owned(),
                entry.path(),
            )
        })
        .filter(|(name, path)| !name.starts_with('.') && path.is_dir())
        .collect()
}

/// Loads the enabled world and feature packs in `DataPacks.Enabled` order, so a later pack
/// overrides an earlier one like in vanilla `MultiPackResourceManager`.
// TODO: recipes don't follow that yet: every pack's recipes are appended and lookups take the
// first match, so an earlier pack wins a duplicate recipe id.
fn load_enabled_packs(
    datapacks_dir: &Path,
    enabled_packs: &[String],
    acc: &mut PackContentAccumulators<'_>,
    loaded_packs_vec: &mut Vec<LoadedDatapack>,
) {
    let world_packs = world_pack_dirs(datapacks_dir);
    let feature_packs = feature_packs();
    for id in enabled_packs {
        // Vanilla ids: `minecart_improvements` is a feature pack, `file/minecart_improvements`
        // a world pack.
        let (name, pack_path, kind) = if let Some(name) = id.strip_prefix("file/") {
            let Some((_, path)) = world_packs.iter().find(|(dir, _)| dir == name) else {
                continue;
            };
            if read_pack_features(id, path).is_none() {
                continue;
            }
            (name, path.clone(), "datapack")
        } else if let Some(path) = feature_packs
            .iter()
            .find(|pack| pack.id == *id)
            .and_then(|pack| pack.path.clone())
        {
            (id.as_str(), path, "bundled datapack")
        } else {
            continue;
        };

        let (description, pack_format, known_packs) = read_pack_mcmeta(&pack_path);
        let counts = load_pack_contents(&pack_path, acc);
        info!(
            "Loaded {kind} '{name}': {} recipe(s), {} function(s), {} test instance(s), {} damage type(s), {} int provider(s), {} float provider(s), {} loot table(s), {} structure(s), {} template pool(s), {} processor list(s), {} dynamic registry entry/entries, {} trade set(s), {} trade(s), {} advancement(s), {} predicate(s), {} tag(s)",
            counts.recipes,
            counts.functions,
            counts.test_instances,
            counts.damage_types,
            counts.int_providers,
            counts.float_providers,
            counts.loot_tables,
            counts.structures,
            counts.template_pools,
            counts.processor_lists,
            counts.dynamic_registries,
            counts.trade_sets,
            counts.trades,
            counts.advancements,
            counts.predicates,
            counts.tags,
        );
        loaded_packs_vec.push(LoadedDatapack {
            id: id.clone(),
            name: name.to_owned(),
            description,
            pack_format,
            root_path: pack_path,
            recipe_count: counts.recipes,
            function_count: counts.functions,
            known_packs,
        });
    }
}

/// Loads recipes, functions, function tags, test instances, damage types, context number providers,
/// and loot tables from a single datapack directory, returning per-pack counts as [`PackContentCounts`].
#[allow(clippy::too_many_lines)]
fn load_pack_contents(
    pack_path: &Path,
    acc: &mut PackContentAccumulators<'_>,
) -> PackContentCounts {
    let data_dir = pack_path.join("data");
    let mut counts = PackContentCounts::default();

    if data_dir.is_dir()
        && let Ok(ns_entries) = fs::read_dir(&data_dir)
    {
        for ns_entry in ns_entries.flatten() {
            let ns_path = ns_entry.path();
            if !ns_path.is_dir() {
                continue;
            }
            let namespace = ns_entry.file_name().to_string_lossy().to_string();

            // Load recipes
            for recipe_sub in ["recipe", "recipes"] {
                let recipe_dir = ns_path.join(recipe_sub);
                if recipe_dir.is_dir() {
                    load_recipes_from_dir(
                        &namespace,
                        &recipe_dir,
                        acc.recipes,
                        &mut counts.recipes,
                    );
                }
            }

            // Load functions
            for fn_sub in ["function", "functions"] {
                let fn_dir = ns_path.join(fn_sub);
                if fn_dir.is_dir() {
                    let before = acc.functions.len();
                    function_loader::load_functions_from_dir(&namespace, &fn_dir, acc.functions);
                    counts.functions += acc.functions.len() - before;
                }
            }

            // Load tags
            let tags_dir = ns_path.join("tags");
            if tags_dir.is_dir() {
                function_loader::load_function_tags_from_dir(
                    &namespace,
                    &tags_dir,
                    acc.function_tags,
                );
                counts.tags += tag_loader::load_tags_from_dir(&namespace, &tags_dir, acc.tags);
            }
            // Load game test instances
            let test_instance_dir = ns_path.join("test_instance");
            if test_instance_dir.is_dir() {
                counts.test_instances += load_test_instances_from_dir(
                    &namespace,
                    &test_instance_dir,
                    acc.test_instances,
                );
            }

            // Load damage types
            for dt_sub in ["damage_type", "damage_types"] {
                let dt_dir = ns_path.join(dt_sub);
                if dt_dir.is_dir() {
                    counts.damage_types += damage_type_loader::load_damage_types_from_dir(
                        &namespace,
                        &dt_dir,
                        acc.damage_types,
                    );
                }
            }

            // Load context int providers
            for int_sub in ["context_int_provider", "context_int_providers"] {
                let provider_dir = ns_path.join(int_sub);
                if provider_dir.is_dir() {
                    counts.int_providers +=
                        context_provider_loader::load_context_providers_from_dir(
                            &namespace,
                            &provider_dir,
                            acc.context_int_providers,
                        );
                }
            }

            // Load context float providers
            for float_sub in ["context_float_provider", "context_float_providers"] {
                let provider_dir = ns_path.join(float_sub);
                if provider_dir.is_dir() {
                    counts.float_providers +=
                        context_provider_loader::load_context_providers_from_dir(
                            &namespace,
                            &provider_dir,
                            acc.context_float_providers,
                        );
                }
            }

            // Load loot tables
            for lt_sub in ["loot_table", "loot_tables"] {
                let lt_dir = ns_path.join(lt_sub);
                if lt_dir.is_dir() {
                    counts.loot_tables += loot_table_loader::load_loot_tables_from_dir(
                        &namespace,
                        &lt_dir,
                        acc.loot_tables,
                    );
                }
            }

            // Load advancements
            for adv_sub in ["advancement", "advancements"] {
                let adv_dir = ns_path.join(adv_sub);
                if adv_dir.is_dir() {
                    counts.advancements += advancement_loader::load_advancements_from_dir(
                        &namespace,
                        &adv_dir,
                        acc.advancements,
                    );
                }
            }

            // Load predicates
            for pred_sub in ["predicate", "predicates"] {
                let pred_dir = ns_path.join(pred_sub);
                if pred_dir.is_dir() {
                    counts.predicates += predicate_loader::load_predicates_from_dir(
                        &namespace,
                        &pred_dir,
                        acc.predicates,
                    );
                }
            }

            // Load item modifiers
            for im_sub in ["item_modifier", "item_modifiers"] {
                let im_dir = ns_path.join(im_sub);
                if im_dir.is_dir() {
                    counts.item_modifiers += item_modifier_loader::load_item_modifiers_from_dir(
                        &namespace,
                        &im_dir,
                        acc.item_modifiers,
                    );
                }
            }

            // Load slot sources
            for ss_sub in ["slot_source", "slot_sources"] {
                let ss_dir = ns_path.join(ss_sub);
                if ss_dir.is_dir() {
                    counts.slot_sources += slot_source_loader::load_slot_sources_from_dir(
                        &namespace,
                        &ss_dir,
                        acc.slot_sources,
                    );
                }
            }

            // Load structure templates
            for st_sub in ["structure", "structures"] {
                let st_dir = ns_path.join(st_sub);
                if st_dir.is_dir() {
                    counts.structures +=
                        structure_loader::load_structures_from_dir(&namespace, &st_dir);
                }
            }

            // Load worldgen template pools and processor lists
            let worldgen_dir = ns_path.join("worldgen");
            if worldgen_dir.is_dir() {
                let pool_dir = worldgen_dir.join("template_pool");
                if pool_dir.is_dir() {
                    counts.template_pools +=
                        structure_loader::load_template_pools_from_dir(&namespace, &pool_dir);
                }
                let proc_dir = worldgen_dir.join("processor_list");
                if proc_dir.is_dir() {
                    counts.processor_lists +=
                        structure_loader::load_processor_lists_from_dir(&namespace, &proc_dir);
                }
            }

            // Load dynamic registry entries (wolf_variant, cat_variant, frog_variant, instrument,
            // trim_material, trim_pattern, banner_pattern, decorated_pot_pattern, chat_type, etc.)
            counts.dynamic_registries += dynamic_registry_loader::load_dynamic_registries_from_ns(
                &namespace,
                &ns_path,
                acc.dynamic_registries,
            );

            // Load trade sets
            for ts_sub in ["trade_set", "trade_sets"] {
                let ts_dir = ns_path.join(ts_sub);
                if ts_dir.is_dir() {
                    counts.trade_sets += trade_loader::load_trade_sets_from_dir(
                        &namespace,
                        &ts_dir,
                        &mut acc.trade_registry.trade_sets,
                    );
                }
            }

            // Load villager trades
            for vt_sub in ["villager_trade", "villager_trades"] {
                let vt_dir = ns_path.join(vt_sub);
                if vt_dir.is_dir() {
                    counts.trades += trade_loader::load_trades_from_dir(
                        &namespace,
                        &vt_dir,
                        &mut acc.trade_registry.trades,
                    );
                }
            }

            // Load villager trade tags
            let tags_dir = ns_path.join("tags");
            if tags_dir.is_dir() {
                for vt_tag_sub in ["villager_trade", "villager_trades"] {
                    let vt_tag_dir = tags_dir.join(vt_tag_sub);
                    if vt_tag_dir.is_dir() {
                        counts.trade_tags += trade_loader::load_trade_tags_from_dir(
                            &namespace,
                            &vt_tag_dir,
                            &mut acc.trade_registry.trade_tags,
                        );
                    }
                }
            }
        }
    }

    counts
}

#[must_use]
pub fn clone_registry_entry(entry: &RegistryEntryData) -> RegistryEntryData {
    RegistryEntryData {
        entry_id: entry.entry_id.clone(),
        data: entry.data.clone(),
    }
}

#[must_use]
pub fn merge_damage_type_entries<S: std::hash::BuildHasher>(
    vanilla_entries: &[RegistryEntryData],
    custom_entries: &HashMap<String, DamageTypeEntry, S>,
) -> Vec<RegistryEntryData> {
    if custom_entries.is_empty() {
        return vanilla_entries.iter().map(clone_registry_entry).collect();
    }

    let mut merged = Vec::with_capacity(vanilla_entries.len() + custom_entries.len());
    let mut overridden = std::collections::HashSet::new();

    for entry in vanilla_entries {
        let full_id = if entry.entry_id.contains(':') {
            entry.entry_id.clone()
        } else {
            format!("minecraft:{}", entry.entry_id)
        };

        if let Some(custom) = custom_entries
            .get(&full_id)
            .or_else(|| custom_entries.get(&entry.entry_id))
        {
            let mut reg_entry = custom.to_registry_entry();
            reg_entry.entry_id.clone_from(&entry.entry_id);
            merged.push(reg_entry);
            overridden.insert(full_id);
            overridden.insert(entry.entry_id.clone());
        } else {
            merged.push(clone_registry_entry(entry));
        }
    }

    let mut sorted_custom: Vec<&DamageTypeEntry> = custom_entries
        .values()
        .filter(|e| !overridden.contains(&e.id))
        .collect();
    sorted_custom.sort_by_key(|e| e.numeric_id);

    for custom in sorted_custom {
        merged.push(custom.to_registry_entry());
    }

    merged
}

/// Vanilla `PackSource` of the packs a world can select.
#[derive(Clone, Copy, PartialEq, Eq)]
enum PackSource {
    BuiltIn,
    Feature,
    World,
}

impl PackSource {
    const fn should_add_automatically(self) -> bool {
        !matches!(self, Self::Feature)
    }
}

/// A vanilla `Pack` in `PackRepository.available`.
struct AvailablePack {
    id: String,
    source: PackSource,
    requested_features: Vec<&'static str>,
    /// Where the pack's files are, `None` for the built-in `vanilla` pack.
    path: Option<PathBuf>,
}

/// The flags of `requested` that `allowed` lacks, vanilla `FeatureFlags.printMissingFlags`.
fn missing_flags<S: AsRef<str>>(requested: &[&'static str], allowed: &[S]) -> Vec<&'static str> {
    requested
        .iter()
        .filter(|feature| !allowed.iter().any(|allowed| allowed.as_ref() == **feature))
        .copied()
        .collect()
}

/// Vanilla zip packs are `file/<name>.zip`. Pumpkin keeps them in `level.dat` but doesn't load
/// them.
// TODO: read and load zip data packs like vanilla `FilePackResources`.
#[expect(
    clippy::case_sensitive_file_extension_comparisons,
    reason = "vanilla `PackDetector` matches `.zip` case-sensitively"
)]
fn is_archive_pack(id: &str) -> bool {
    id.strip_prefix("file/")
        .is_some_and(|name| name.ends_with(".zip"))
}

fn is_world_archive_pack(world_path: &Path, id: &str) -> bool {
    is_archive_pack(id)
        && id
            .strip_prefix("file/")
            .is_some_and(|name| world_path.join("datapacks").join(name).is_file())
}

/// Vanilla `MinecraftServer.getSelectedPacks` with `disableInactive`: every available pack that
/// isn't selected, plus the zip packs `previous` disabled.
fn disabled_packs(
    world_path: &Path,
    available: &[AvailablePack],
    selected: &[String],
    previous: &[String],
) -> Vec<String> {
    let mut disabled: Vec<String> = available
        .iter()
        .map(|pack| &pack.id)
        .filter(|id| !selected.contains(id))
        .cloned()
        .collect();
    for id in previous {
        if !disabled.contains(id) && !selected.contains(id) && is_world_archive_pack(world_path, id)
        {
            disabled.push(id.clone());
        }
    }
    disabled
}

/// Older Pumpkin took flags from these pack names without saving `enabled_features`, so a
/// feature pack id, or its `file/` or `minecraft:` alias, in `DataPacks.Enabled` keeps its flags.
fn enable_legacy_pack_features(level: &mut LevelData, available: &[AvailablePack]) {
    for id in &level.data_packs.enabled {
        let name = legacy_pack_name(id);
        // Without the bundled pack, the name maps to its flag like it did in those versions, so
        // the saved flag brings the pack back once it's installed.
        let flags: Vec<&'static str> = legacy_feature_pack(id, available).map_or_else(
            || {
                feature_flags::ALL
                    .iter()
                    .copied()
                    .filter(|flag| flag.strip_prefix("minecraft:") == Some(name))
                    .collect()
            },
            |pack| pack.requested_features.clone(),
        );
        for flag in missing_flags(&flags, &level.enabled_features) {
            info!("Keeping feature {flag} that data pack {id} enabled in an older Pumpkin version");
            level.enabled_features.push(flag.to_owned());
        }
    }
}

/// The feature pack name in an older Pumpkin pack id.
fn legacy_pack_name(id: &str) -> &str {
    id.strip_prefix("file/")
        .or_else(|| id.strip_prefix("minecraft:"))
        .unwrap_or(id)
}

/// The feature pack an older Pumpkin pack id stood for.
fn legacy_feature_pack<'a>(id: &str, available: &'a [AvailablePack]) -> Option<&'a AvailablePack> {
    let name = legacy_pack_name(id);
    available
        .iter()
        .find(|pack| pack.source == PackSource::Feature && pack.id == name)
}

/// Vanilla `MinecraftServer.enableForcedFeaturePacks`: a saved flag no selected pack provides
/// selects the feature pack for it.
fn enable_forced_feature_packs(
    selected: &mut Vec<String>,
    forced: &[&'static str],
    available: &[AvailablePack],
) {
    let provided: Vec<&'static str> = available
        .iter()
        .filter(|pack| selected.contains(&pack.id))
        .flat_map(|pack| pack.requested_features.iter().copied())
        .collect();
    let mut missing = missing_flags(forced, &provided);
    for pack in available
        .iter()
        .filter(|pack| pack.source == PackSource::Feature)
    {
        if missing.is_empty() {
            break;
        }
        let flags = &pack.requested_features;
        if !flags.is_empty()
            && flags.iter().any(|flag| missing.contains(flag))
            && missing_flags(flags, forced).is_empty()
            && !selected.contains(&pack.id)
        {
            info!(
                "Found feature pack ('{}') for requested feature, forcing to enabled",
                pack.id
            );
            selected.push(pack.id.clone());
            missing.retain(|feature| !flags.contains(feature));
        }
    }
}

/// Vanilla `ServerPacksSource` feature packs: the bundled `data/minecraft/datapacks/*`. None
/// when the bundled assets are missing.
fn feature_packs() -> Vec<AvailablePack> {
    // The first copy of each pack wins, so an incomplete copy doesn't hide a later one.
    let mut found: Vec<(String, PathBuf)> = Vec::new();
    for dir in bundled_datapacks_dirs() {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let id = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            if path.join("pack.mcmeta").is_file() && !found.iter().any(|(known, _)| *known == id) {
                found.push((id, path));
            }
        }
    }
    let mut packs: Vec<AvailablePack> = found
        .into_iter()
        .filter_map(|(id, path)| {
            let requested_features = read_pack_features(&id, &path)?;
            Some(AvailablePack {
                id,
                source: PackSource::Feature,
                requested_features,
                path: Some(path),
            })
        })
        .collect();
    packs.sort_by(|a, b| a.id.cmp(&b.id));
    packs
}

/// Every pack a world can select, vanilla `PackRepository.available`, sorted by id like its
/// `TreeMap`. Unreadable ones are left out, with a warning when `warn_unreadable`.
fn available_packs(world_path: &Path, warn_unreadable: bool) -> Vec<AvailablePack> {
    let mut packs = vec![AvailablePack {
        id: DEFAULT_ENABLED_DATA_PACK.to_owned(),
        source: PackSource::BuiltIn,
        requested_features: Vec::new(),
        path: None,
    }];
    packs.extend(feature_packs());
    if let Ok(entries) = fs::read_dir(world_path.join("datapacks")) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            // `PackDetector`: a folder is a pack only with a `pack.mcmeta`.
            if name.starts_with('.') || !path.join("pack.mcmeta").is_file() {
                continue;
            }
            let id = format!("file/{name}");
            let requested_features = if warn_unreadable {
                read_pack_features(&id, &path)
            } else {
                mcmeta_features(&path).ok()
            };
            if let Some(requested_features) = requested_features {
                packs.push(AvailablePack {
                    id,
                    source: PackSource::World,
                    requested_features,
                    path: Some(path),
                });
            }
        }
    }
    packs.sort_by(|a, b| a.id.cmp(&b.id));
    packs
}

/// Vanilla `commands.datapack.enable.no_flags`, which it uses for disabling as well.
fn no_flags_error(pack: &str, missing: &[&str]) -> String {
    format!(
        "Pack '{pack}' cannot be enabled, since required flags are not enabled in this world: {}!",
        missing.join(", ")
    )
}

/// Why vanilla `Pack.readPackMetadata` refuses a pack.
enum PackMetaError {
    /// No `pack.mcmeta` or no `pack` section in it.
    Missing,
    /// Unreadable or malformed metadata.
    Invalid(String),
    UnknownFeatures(Vec<String>),
}

/// Vanilla `FeatureFlagsMetadataSection` of a `pack.mcmeta`, `None` when vanilla can't load the
/// pack.
fn read_pack_features(pack_id: &str, pack_path: &Path) -> Option<Vec<&'static str>> {
    match mcmeta_features(pack_path) {
        Ok(features) => Some(features),
        Err(PackMetaError::Missing) => {
            warn!("Missing metadata in pack {pack_id}");
            None
        }
        Err(PackMetaError::Invalid(error)) => {
            warn!("Failed to read pack {pack_id} metadata: {error}");
            None
        }
        Err(PackMetaError::UnknownFeatures(unknown)) => {
            warn!("Failed to read pack {pack_id} metadata: unknown feature ids {unknown:?}");
            None
        }
    }
}

/// Bounds reading a `pack.mcmeta`, far above any real one.
const MAX_PACK_MCMETA_BYTES: u64 = 1024 * 1024;

/// The flags a `pack.mcmeta` requests; no `features` section requests none.
fn mcmeta_features(pack_path: &Path) -> Result<Vec<&'static str>, PackMetaError> {
    let file = match fs::File::open(pack_path.join("pack.mcmeta")) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(PackMetaError::Missing);
        }
        Err(error) => return Err(PackMetaError::Invalid(error.to_string())),
    };
    let mut content = String::new();
    file.take(MAX_PACK_MCMETA_BYTES + 1)
        .read_to_string(&mut content)
        .map_err(|error| PackMetaError::Invalid(error.to_string()))?;
    if content.len() as u64 > MAX_PACK_MCMETA_BYTES {
        return Err(PackMetaError::Invalid(format!(
            "pack.mcmeta is larger than {MAX_PACK_MCMETA_BYTES} bytes"
        )));
    }
    // Gson's `JsonReader` skips a leading byte order mark.
    let content = content.strip_prefix('\u{feff}').unwrap_or(&content);
    let meta: serde_json::Value =
        serde_json::from_str(content).map_err(|error| PackMetaError::Invalid(error.to_string()))?;
    let meta = meta
        .as_object()
        .ok_or_else(|| PackMetaError::Invalid("pack.mcmeta is not a JSON object".to_owned()))?;
    let Some(pack) = meta.get("pack") else {
        return Err(PackMetaError::Missing);
    };
    // Vanilla `PackMetadataSection.FALLBACK_CODEC` needs only a `description` component, so a
    // bad format range alone doesn't reject the pack.
    // TODO: check the whole component, not just that it is a string, list or object.
    let has_description = pack
        .get("description")
        .is_some_and(|description| match description {
            serde_json::Value::String(_) | serde_json::Value::Object(_) => true,
            serde_json::Value::Array(parts) => !parts.is_empty(),
            _ => false,
        });
    if !has_description {
        return Err(PackMetaError::Invalid(
            "pack.description is missing or not a text component".to_owned(),
        ));
    }
    let Some(section) = meta.get("features") else {
        return Ok(Vec::new());
    };
    let requested = section
        .get("enabled")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| PackMetaError::Invalid("features.enabled is not a list".to_owned()))?;
    let mut features = Vec::new();
    let mut unknown = Vec::new();
    for id in requested {
        let id = id
            .as_str()
            .ok_or_else(|| PackMetaError::Invalid("feature id is not a string".to_owned()))?;
        let full = if id.contains(':') {
            id.to_owned()
        } else {
            format!("minecraft:{id}")
        };
        match feature_flags::ALL.iter().find(|flag| **flag == full) {
            Some(flag) => features.push(*flag),
            None => unknown.push(id.to_owned()),
        }
    }
    if unknown.is_empty() {
        Ok(features)
    } else {
        Err(PackMetaError::UnknownFeatures(unknown))
    }
}

fn read_pack_mcmeta(pack_path: &Path) -> (String, u32, Vec<KnownPackData>) {
    let mcmeta_path = pack_path.join("pack.mcmeta");
    if let Ok(content) = fs::read_to_string(mcmeta_path)
        && let Ok(val) = serde_json::from_str::<serde_json::Value>(&content)
    {
        let pack = val.get("pack");
        let description = pack
            .and_then(|p| p.get("description"))
            .map(|d| {
                d.as_str()
                    .map_or_else(|| d.to_string(), ToString::to_string)
            })
            .unwrap_or_default();
        let pack_format = pack
            .and_then(|p| {
                p.get("pack_format")
                    .or_else(|| p.get("max_format"))
                    .or_else(|| p.get("min_format"))
            })
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(61) as u32;

        let mut known_packs = Vec::new();
        let parse_item = |item: &serde_json::Value| -> Option<KnownPackData> {
            let ns = item.get("namespace").and_then(serde_json::Value::as_str)?;
            let id = item.get("id").and_then(serde_json::Value::as_str)?;
            let ver = item.get("version").and_then(serde_json::Value::as_str)?;
            Some(KnownPackData {
                namespace: ns.to_string(),
                id: id.to_string(),
                version: ver.to_string(),
            })
        };

        if let Some(packs_array) = val.get("known_packs").and_then(serde_json::Value::as_array) {
            for item in packs_array {
                if let Some(kp) = parse_item(item) {
                    known_packs.push(kp);
                }
            }
        } else if let Some(item) = val.get("known_pack") {
            if let Some(kp) = parse_item(item) {
                known_packs.push(kp);
            }
        } else if let Some(pack_obj) = pack {
            if let Some(packs_array) = pack_obj
                .get("known_packs")
                .and_then(serde_json::Value::as_array)
            {
                for item in packs_array {
                    if let Some(kp) = parse_item(item) {
                        known_packs.push(kp);
                    }
                }
            } else if let Some(item) = pack_obj.get("known_pack")
                && let Some(kp) = parse_item(item)
            {
                known_packs.push(kp);
            }
        }

        return (description, pack_format, known_packs);
    }
    (String::new(), 61, Vec::new())
}

fn load_recipes_recursive(
    namespace: &str,
    base_dir: &Path,
    current_dir: &Path,
    all_recipes: &mut Vec<DynamicRecipe>,
    count: &mut usize,
) {
    let Ok(entries) = fs::read_dir(current_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            load_recipes_recursive(namespace, base_dir, &path, all_recipes, count);
        } else if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            let Ok(relative_path) = path.strip_prefix(base_dir) else {
                continue;
            };

            let recipe_name = relative_path
                .with_extension("")
                .to_string_lossy()
                .replace('\\', "/");

            if let Ok(content) = fs::read_to_string(&path)
                && let Some(recipe) = recipe_loader::parse_recipe(namespace, &recipe_name, &content)
            {
                all_recipes.push(recipe);
                *count += 1;
            }
        }
    }
}

fn load_recipes_from_dir(
    namespace: &str,
    dir: &Path,
    all_recipes: &mut Vec<DynamicRecipe>,
    count: &mut usize,
) {
    load_recipes_recursive(namespace, dir, dir, all_recipes, count);
}
#[cfg(test)]
mod tests {
    use super::{
        DatapackManager, FunctionChainGuard, FunctionRunError, MAX_FUNCTION_CHAIN_COMMANDS,
        MAX_FUNCTION_CHAIN_DEPTH, enable_legacy_pack_features,
    };

    #[test]
    fn function_chain_depth_is_bounded_instead_of_recursing_forever() {
        // Unbounded entry is refused instead of overflowing the stack.
        let mut guards = Vec::new();
        let error = loop {
            match FunctionChainGuard::enter() {
                Ok(guard) => guards.push(guard),
                Err(error) => break error,
            }
        };

        assert_eq!(error, FunctionRunError::ChainLimitExceeded);
        assert_eq!(guards.len(), MAX_FUNCTION_CHAIN_DEPTH);
    }

    #[test]
    fn function_chain_depth_is_released_after_a_bounded_chain() {
        // Bounded recursion still completes and releases the depth.
        fn recurse(remaining: usize) -> Result<(), FunctionRunError> {
            if remaining == 0 {
                return Ok(());
            }
            let _guard = FunctionChainGuard::enter()?;
            recurse(remaining - 1)
        }

        recurse(100).expect("bounded recursion must succeed");
        // The depth has to be released again, or later chains would inherit it.
        assert!(FunctionChainGuard::enter().is_ok());
    }

    #[test]
    fn function_chain_commands_are_budgeted_per_chain() {
        // An exhausted budget stops the chain; the next chain starts fresh.
        let manager = DatapackManager::new();
        manager
            .functions
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(
                "test:wide".to_string(),
                vec!["say hi".to_string(); MAX_FUNCTION_CHAIN_COMMANDS + 10].into(),
            );

        {
            let _guard = FunctionChainGuard::enter().expect("chain must start");
            let mut visited = 0;
            let error = manager
                .visit_function_lines("test:wide", |_| visited += 1)
                .expect_err("the budget must stop the chain with an error, not ok");
            assert_eq!(error, FunctionRunError::CommandChainLimitExceeded);
            assert_eq!(visited, MAX_FUNCTION_CHAIN_COMMANDS);
        };
        // Guard dropped above, so the budget was reset.
        {
            let _guard = FunctionChainGuard::enter().expect("next chain must start");
            let mut visited = 0;
            let error = manager
                .visit_function_lines("test:wide", |_| visited += 1)
                .expect_err("the budget must stop the chain with an error, not ok");
            assert_eq!(error, FunctionRunError::CommandChainLimitExceeded);
            assert_eq!(visited, MAX_FUNCTION_CHAIN_COMMANDS);
        };
    }

    #[test]
    fn structure_resource_locations_reject_dot_only_components() {
        for name in [
            "..:foo",
            ".:foo",
            "...:foo",
            "minecraft:../foo",
            "minecraft:foo/../bar",
            "minecraft:foo/.../bar",
        ] {
            assert!(
                super::parse_structure_resource_location(name).is_err(),
                "accepted {name}"
            );
        }
        assert_eq!(
            super::parse_structure_resource_location("namespace.with.dots:foo.bar/baz"),
            Ok(("namespace.with.dots", "foo.bar/baz"))
        );
    }

    #[test]
    fn function_dispatch_releases_the_functions_lock() {
        let manager = DatapackManager::new();
        manager
            .functions
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(
                "test:reentrant".to_string(),
                vec!["first command".to_string(), "second command".to_string()].into(),
            );

        let mut visited = Vec::new();
        let executed = manager
            .visit_function_lines("test:reentrant", |line| {
                let functions = manager
                    .functions
                    .try_write()
                    .expect("function dispatch must not hold the functions read lock");
                drop(functions);
                visited.push(line.to_string());
            })
            .expect("execute function lines");

        assert_eq!(executed, 2);
        assert_eq!(visited, ["first command", "second command"]);
    }

    #[test]
    fn function_resolution_preserves_unknown_errors_and_skips_missing_tag_entries() {
        let manager = DatapackManager::new();
        manager
            .functions
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(
                "test:known".to_string(),
                vec!["known command".to_string()].into(),
            );
        manager
            .function_tags
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(
                "test:mixed".to_string(),
                vec!["test:missing".to_string(), "test:known".to_string()],
            );

        let mut visited = Vec::new();
        let executed = manager
            .visit_function_lines("#test:mixed", |line| visited.push(line.to_string()))
            .expect("execute function tag");
        assert_eq!(executed, 1);
        assert_eq!(visited, ["known command"]);

        assert_eq!(
            manager
                .visit_function_lines("test:missing", |_| {})
                .expect_err("unknown function must fail"),
            FunctionRunError::Unknown("Unknown function: test:missing".to_string())
        );
        assert_eq!(
            manager
                .visit_function_lines("#test:missing", |_| {})
                .expect_err("unknown function tag must fail"),
            FunctionRunError::Unknown("Unknown function tag: #test:missing".to_string())
        );
    }

    async fn test_server(world_dir: &std::path::Path) -> std::sync::Arc<crate::server::Server> {
        use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
        use pumpkin_util::world_seed::Seed;

        let basic_config = BasicConfiguration {
            seed: Seed(123),
            default_level_name: world_dir.to_string_lossy().into_owned(),
            allow_nether: false,
            allow_end: false,
            ..BasicConfiguration::default()
        };
        let advanced_config = {
            let mut advanced_config = AdvancedConfiguration::default();
            // Keep the test offline; the defaults would fetch keys and telemetry.
            advanced_config.networking.bedrock.online_mode = false;
            advanced_config.networking.bedrock.authentication.enabled = false;
            advanced_config
        };
        let vanilla_data = crate::data::VanillaData {
            banned_ip_list: std::sync::RwLock::new(crate::data::banned_ip::BannedIpList::default()),
            banned_player_list: std::sync::RwLock::new(
                crate::data::banned_player::BannedPlayerList::default(),
            ),
            operator_config: std::sync::RwLock::new(crate::data::op::OperatorConfig::default()),
            user_cache: std::sync::RwLock::new(crate::data::usercache::UserCache::default()),
            whitelist_config: std::sync::RwLock::new(
                crate::data::whitelist::WhitelistConfig::default(),
            ),
        };
        let telemetry_config = TelemetryConfig {
            enabled: false,
            ..TelemetryConfig::default()
        };
        crate::server::Server::new(
            basic_config,
            advanced_config,
            telemetry_config,
            vanilla_data,
            Vec::new(),
        )
        .await
        .expect("the test server must start")
    }

    #[tokio::test]
    async fn fans_out_through_dispatcher_and_hits_the_command_chain_limit() {
        let world_dir = tempfile::tempdir().expect("tempdir");
        let server = test_server(world_dir.path()).await;

        // Fan-out under the depth cap: each level spends its budget on leaf lines
        // before the recursive call, so the command budget trips first.
        let mut lines = vec!["say leaf".to_string(); 1024];
        lines.push("function test:fanout".to_string());
        server
            .datapack_manager
            .functions
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert("test:fanout".to_string(), lines.into());

        let source = crate::command::CommandSender::Console.into_source(&server);
        let error = server
            .datapack_manager
            .execute_function(&server, &source, "test:fanout")
            .expect_err("a chain this wide must be stopped by the command budget");
        assert_eq!(error, FunctionRunError::CommandChainLimitExceeded);
    }

    #[tokio::test]
    async fn recurse_through_dispatcher_and_hit_the_depth_limit() {
        let world_dir = tempfile::tempdir().expect("tempdir");
        let server = test_server(world_dir.path()).await;

        server
            .datapack_manager
            .functions
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(
                "test:self".to_string(),
                vec!["function test:self".to_string()].into(),
            );

        let source = crate::command::CommandSender::Console.into_source(&server);
        let error = server
            .datapack_manager
            .execute_function(&server, &source, "test:self")
            .expect_err("self-recursion must be stopped by the depth limit");
        assert_eq!(error, FunctionRunError::ChainLimitExceeded);
    }

    #[test]
    fn read_pack_mcmeta_parses_known_packs() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let pack_path = temp_dir.path();

        // 1. Test root known_packs array
        let mcmeta_content = r#"{
            "pack": {
                "description": "Test Pack",
                "pack_format": 61
            },
            "known_packs": [
                {
                    "namespace": "example",
                    "id": "content",
                    "version": "1.0.0"
                }
            ]
        }"#;
        std::fs::write(pack_path.join("pack.mcmeta"), mcmeta_content).unwrap();

        let (desc, format, known_packs) = super::read_pack_mcmeta(pack_path);
        assert_eq!(desc, "Test Pack");
        assert_eq!(format, 61);
        assert_eq!(known_packs.len(), 1);
        assert_eq!(known_packs[0].namespace, "example");
        assert_eq!(known_packs[0].id, "content");
        assert_eq!(known_packs[0].version, "1.0.0");

        // 2. Test nested pack.known_packs
        let mcmeta_content_nested = r#"{
            "pack": {
                "description": "Nested Pack",
                "pack_format": 61,
                "known_packs": [
                    {
                        "namespace": "nested",
                        "id": "pack",
                        "version": "2.0.0"
                    }
                ]
            }
        }"#;
        std::fs::write(pack_path.join("pack.mcmeta"), mcmeta_content_nested).unwrap();

        let (_, _, known_packs) = super::read_pack_mcmeta(pack_path);
        assert_eq!(known_packs.len(), 1);
        assert_eq!(known_packs[0].namespace, "nested");
        assert_eq!(known_packs[0].id, "pack");
        assert_eq!(known_packs[0].version, "2.0.0");
    }

    #[test]
    fn world_packs_follow_the_saved_features() {
        let world = tempfile::tempdir().unwrap();
        let write_pack = |name: &str, features: &str| {
            let dir = world.path().join("datapacks").join(name);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                dir.join("pack.mcmeta"),
                format!(
                    r#"{{"features":{{"enabled":[{features}]}},"pack":{{"description":"","min_format":121,"max_format":121}}}}"#
                ),
            )
            .unwrap();
        };
        write_pack("carts", r#""minecraft:minecart_improvements""#);
        write_pack("broken", r#""minecraft:no_such_flag""#);
        let level =
            || pumpkin_world::world_info::LevelData::default(pumpkin_util::world_seed::Seed(0));
        let pack = |name: &str| name.to_owned();

        // A new world allows every flag and picks up its own packs.
        let mut created = level();
        DatapackManager::configure_world_packs(world.path(), &mut created, true);
        assert!(created.data_packs.enabled.contains(&pack("file/carts")));
        assert_eq!(
            created.enabled_features,
            ["minecraft:vanilla", "minecraft:minecart_improvements"]
        );
        // An unreadable pack is neither enabled nor disabled.
        assert!(!created.data_packs.enabled.contains(&pack("file/broken")));
        assert!(!created.data_packs.disabled.contains(&pack("file/broken")));
        // Packs compiled into Pumpkin aren't saved.
        assert!(
            !created
                .data_packs
                .enabled
                .contains(&pack("pumpkin-unit-test"))
        );

        // A world saved without the flag disables a pack that needs it, and keeps a zip pack
        // Pumpkin can't read.
        std::fs::write(world.path().join("datapacks").join("old.zip"), b"").unwrap();
        let mut plain = level();
        plain.data_packs.enabled.push(pack("file/carts"));
        plain.data_packs.enabled.push(pack("file/old.zip"));
        DatapackManager::configure_world_packs(world.path(), &mut plain, false);
        assert!(plain.data_packs.disabled.contains(&pack("file/carts")));
        assert!(plain.data_packs.enabled.contains(&pack("file/old.zip")));
        assert_eq!(plain.enabled_features, ["minecraft:vanilla"]);

        // Older Pumpkin enabled a feature from the pack name alone, but only saves without
        // `enabled_features` get it back.
        let mut saved = level();
        saved
            .data_packs
            .enabled
            .push(pack("file/minecart_improvements"));
        saved.data_packs.disabled.push(pack("file/carts"));
        let mut migrated = saved.clone();
        DatapackManager::configure_world_packs(world.path(), &mut saved, false);
        assert_eq!(saved.enabled_features, ["minecraft:vanilla"]);
        migrated.enabled_features_missing = true;
        DatapackManager::configure_world_packs(world.path(), &mut migrated, false);
        assert!(
            migrated
                .data_packs
                .enabled
                .contains(&pack("minecart_improvements"))
        );
        assert_eq!(
            migrated.enabled_features,
            ["minecraft:vanilla", "minecraft:minecart_improvements"]
        );
        // The alias keeps its place, even when another pack provides the same flag.
        let mut aliased = level();
        aliased.enabled_features_missing = true;
        aliased
            .data_packs
            .enabled
            .extend([pack("file/minecart_improvements"), pack("file/carts")]);
        DatapackManager::configure_world_packs(world.path(), &mut aliased, false);
        assert_eq!(
            aliased.data_packs.enabled,
            ["vanilla", "minecart_improvements", "file/carts"]
        );

        // A saved flag no pack provides brings its feature pack back.
        let mut kept = level();
        kept.data_packs.disabled.push(pack("file/carts"));
        kept.enabled_features
            .push("minecraft:minecart_improvements".to_owned());
        DatapackManager::configure_world_packs(world.path(), &mut kept, false);
        assert!(
            kept.data_packs
                .enabled
                .contains(&pack("minecart_improvements"))
        );
        assert_eq!(
            kept.enabled_features,
            ["minecraft:vanilla", "minecraft:minecart_improvements"]
        );
    }

    /// An older save keeps its feature flag even when the bundled feature packs are missing.
    #[test]
    fn legacy_features_survive_missing_bundled_packs() {
        let mut level =
            pumpkin_world::world_info::LevelData::default(pumpkin_util::world_seed::Seed(0));
        level.data_packs.enabled.extend([
            "file/minecart_improvements".to_owned(),
            "file/carts".to_owned(),
        ]);
        enable_legacy_pack_features(&mut level, &[]);
        assert_eq!(
            level.enabled_features,
            ["minecraft:vanilla", "minecraft:minecart_improvements"]
        );
    }

    #[test]
    fn pack_metadata_needs_a_described_pack_section() {
        let dir = tempfile::tempdir().unwrap();
        let features = |meta: &str| {
            std::fs::write(dir.path().join("pack.mcmeta"), meta).unwrap();
            super::mcmeta_features(dir.path())
        };
        // A bad format range falls back to the description alone, and no `features` is fine.
        assert!(matches!(
            features(r#"{"pack":{"description":"x","min_format":"bad"}}"#),
            Ok(requested) if requested.is_empty()
        ));
        assert!(matches!(
            features("\u{feff}{\"pack\":{\"description\":\"x\"}}"),
            Ok(requested) if requested.is_empty()
        ));
        assert!(matches!(
            features(r#"{"features":{"enabled":[]}}"#),
            Err(super::PackMetaError::Missing)
        ));
        for invalid in [
            r#"{"pack":null}"#,
            r#"{"pack":{}}"#,
            r#"{"pack":{"description":[]}}"#,
            "[]",
        ] {
            assert!(
                matches!(features(invalid), Err(super::PackMetaError::Invalid(_))),
                "{invalid}"
            );
        }
    }

    #[test]
    fn enabled_packs_load_in_saved_order() {
        let world = tempfile::tempdir().unwrap();
        let datapacks = world.path().join("datapacks");
        for name in ["a", "b"] {
            let dir = datapacks.join(name);
            std::fs::create_dir_all(dir.join("data/test/function")).unwrap();
            std::fs::write(dir.join("pack.mcmeta"), r#"{"pack":{"description":""}}"#).unwrap();
            std::fs::write(
                dir.join("data/test/function/f.mcfunction"),
                format!("say {name}"),
            )
            .unwrap();
        }

        let mut recipes = Vec::new();
        let mut functions = std::collections::HashMap::new();
        let mut function_tags = std::collections::HashMap::new();
        let mut test_instances = std::collections::HashMap::new();
        let mut damage_types = std::collections::HashMap::new();
        let mut context_int_providers = std::collections::HashMap::new();
        let mut context_float_providers = std::collections::HashMap::new();
        let mut loot_tables = std::collections::HashMap::new();
        let mut dynamic_registries = std::collections::HashMap::new();
        let mut trade_registry = super::TradeRegistry::new();
        let mut advancements = std::collections::HashMap::new();
        let mut predicates = std::collections::HashMap::new();
        let mut item_modifiers = std::collections::HashMap::new();
        let mut slot_sources = std::collections::HashMap::new();
        let mut tags = std::collections::HashMap::new();
        let mut acc = super::PackContentAccumulators {
            recipes: &mut recipes,
            functions: &mut functions,
            function_tags: &mut function_tags,
            test_instances: &mut test_instances,
            damage_types: &mut damage_types,
            context_int_providers: &mut context_int_providers,
            context_float_providers: &mut context_float_providers,
            loot_tables: &mut loot_tables,
            dynamic_registries: &mut dynamic_registries,
            trade_registry: &mut trade_registry,
            advancements: &mut advancements,
            predicates: &mut predicates,
            item_modifiers: &mut item_modifiers,
            slot_sources: &mut slot_sources,
            tags: &mut tags,
        };
        let mut loaded = Vec::new();
        let enabled = ["file/b".to_owned(), "file/a".to_owned()];
        super::load_enabled_packs(&datapacks, &enabled, &mut acc, &mut loaded);

        let ids: Vec<&str> = loaded.iter().map(|pack| pack.id.as_str()).collect();
        assert_eq!(ids, ["file/b", "file/a"]);
        assert_eq!(functions["test:f"], ["say a"]);
    }

    #[test]
    fn vanilla_datapack_coverage_test() {
        let datapack_path = std::env::current_exe()
            .unwrap()
            .with_file_name("assets")
            .join("datapack");
        let datapack_path = if datapack_path.is_dir() {
            datapack_path
        } else {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/datapack")
        };
        if !datapack_path.join("data/minecraft").is_dir() {
            return;
        }

        let mut recipes = Vec::new();
        let mut functions = std::collections::HashMap::new();
        let mut function_tags = std::collections::HashMap::new();
        let mut test_instances = std::collections::HashMap::new();
        let mut damage_types = std::collections::HashMap::new();
        let mut context_int_providers = std::collections::HashMap::new();
        let mut context_float_providers = std::collections::HashMap::new();
        let mut loot_tables = std::collections::HashMap::new();
        let mut dynamic_registries = std::collections::HashMap::new();
        let mut trade_registry = super::TradeRegistry::new();
        let mut advancements = std::collections::HashMap::new();
        let mut predicates = std::collections::HashMap::new();
        let mut item_modifiers = std::collections::HashMap::new();
        let mut slot_sources = std::collections::HashMap::new();
        let mut tags = std::collections::HashMap::new();

        let mut acc = super::PackContentAccumulators {
            recipes: &mut recipes,
            functions: &mut functions,
            function_tags: &mut function_tags,
            test_instances: &mut test_instances,
            damage_types: &mut damage_types,
            context_int_providers: &mut context_int_providers,
            context_float_providers: &mut context_float_providers,
            loot_tables: &mut loot_tables,
            dynamic_registries: &mut dynamic_registries,
            trade_registry: &mut trade_registry,
            advancements: &mut advancements,
            predicates: &mut predicates,
            item_modifiers: &mut item_modifiers,
            slot_sources: &mut slot_sources,
            tags: &mut tags,
        };

        let counts = super::load_pack_contents(&datapack_path, &mut acc);
        assert_eq!(counts.recipes, 2042, "all vanilla recipes must load");
        assert_eq!(
            counts.loot_tables, 1447,
            "all vanilla loot tables must load"
        );
        assert_eq!(
            counts.advancements, 1866,
            "all vanilla advancements must load"
        );
        assert_eq!(counts.structures, 1511, "all vanilla structures must load");
        assert_eq!(
            counts.template_pools, 245,
            "all vanilla template pools must load"
        );
        assert_eq!(
            counts.processor_lists, 40,
            "all vanilla processor lists must load"
        );
        assert_eq!(
            counts.damage_types, 51,
            "all vanilla damage types must load"
        );
        assert_eq!(
            counts.int_providers, 26,
            "all vanilla int providers must load"
        );
        assert_eq!(
            counts.float_providers, 4,
            "all vanilla float providers must load"
        );
        assert_eq!(
            counts.test_instances, 1,
            "all vanilla test instances must load"
        );
        assert!(
            counts.dynamic_registries >= 2080,
            "all vanilla dynamic registry entries must load"
        );
        assert!(counts.tags >= 1406, "all vanilla tags must load");
        assert_eq!(counts.predicates, 3, "all vanilla predicates must load");
    }

    #[test]
    fn bundled_trade_rebalance_coverage_test() {
        if let Some(pack_path) = super::feature_packs()
            .into_iter()
            .find(|pack| pack.id == "trade_rebalance")
            .and_then(|pack| pack.path)
        {
            let mut recipes = Vec::new();
            let mut functions = std::collections::HashMap::new();
            let mut function_tags = std::collections::HashMap::new();
            let mut test_instances = std::collections::HashMap::new();
            let mut damage_types = std::collections::HashMap::new();
            let mut context_int_providers = std::collections::HashMap::new();
            let mut context_float_providers = std::collections::HashMap::new();
            let mut loot_tables = std::collections::HashMap::new();
            let mut dynamic_registries = std::collections::HashMap::new();
            let mut trade_registry = super::TradeRegistry::new();
            let mut advancements = std::collections::HashMap::new();
            let mut predicates = std::collections::HashMap::new();
            let mut item_modifiers = std::collections::HashMap::new();
            let mut slot_sources = std::collections::HashMap::new();
            let mut tags = std::collections::HashMap::new();

            let mut acc = super::PackContentAccumulators {
                recipes: &mut recipes,
                functions: &mut functions,
                function_tags: &mut function_tags,
                test_instances: &mut test_instances,
                damage_types: &mut damage_types,
                context_int_providers: &mut context_int_providers,
                context_float_providers: &mut context_float_providers,
                loot_tables: &mut loot_tables,
                dynamic_registries: &mut dynamic_registries,
                trade_registry: &mut trade_registry,
                advancements: &mut advancements,
                predicates: &mut predicates,
                item_modifiers: &mut item_modifiers,
                slot_sources: &mut slot_sources,
                tags: &mut tags,
            };

            let counts = super::load_pack_contents(&pack_path, &mut acc);
            assert!(counts.trades > 0, "trade rebalance must load trades");
            assert!(
                counts.trade_tags > 0,
                "trade rebalance must load trade tags"
            );
            assert!(
                counts.loot_tables > 0,
                "trade rebalance must load loot tables"
            );
        }
    }

    #[test]
    fn datapack_manager_dynamic_registry_and_tag_test() {
        let manager = super::DatapackManager::new();
        assert!(
            manager
                .dynamic_registry_manager
                .get_synced_registries()
                .len()
                >= 30
        );
        let synced = manager.get_all_synced_registries();
        assert!(
            synced
                .iter()
                .any(|r| r.registry_id == "minecraft:painting_variant")
        );
        assert!(
            synced
                .iter()
                .any(|r| r.registry_id == "minecraft:wolf_variant")
        );
    }
}
