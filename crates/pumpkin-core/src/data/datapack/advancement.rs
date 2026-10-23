//! Advancements defined by datapacks.
//!
//! Vanilla advancements are generated into `pumpkin-data` at build time. Datapack
//! advancements are loaded at runtime and are mostly used as event hooks: a trigger
//! fires, the criterion is awarded, and the reward function runs. Only the triggers
//! below are evaluated; criteria with any other trigger can still be granted by command.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::BuildHasher;
use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError, RwLock};

use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use tracing::{debug, warn};
use uuid::Uuid;

use crate::command::CommandSender;
use crate::data::predicate::{json_to_nbt, test_condition, test_entity, test_item};
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DatapackTrigger {
    Tick,
    UsingItem,
    PlayerInteractedWithEntity,
    InventoryChanged,
    /// `minecraft:impossible` and every trigger Pumpkin doesn't evaluate yet.
    Manual,
}

impl DatapackTrigger {
    fn from_id(id: &str) -> Self {
        match id.strip_prefix("minecraft:").unwrap_or(id) {
            "tick" => Self::Tick,
            "using_item" => Self::UsingItem,
            "player_interacted_with_entity" => Self::PlayerInteractedWithEntity,
            "inventory_changed" => Self::InventoryChanged,
            "impossible" => Self::Manual,
            other => {
                debug!("Datapack advancement trigger minecraft:{other} is not evaluated");
                Self::Manual
            }
        }
    }
}

#[derive(Debug)]
pub struct DatapackCriterion {
    pub name: String,
    pub trigger: DatapackTrigger,
    pub conditions: NbtCompound,
}

#[derive(Debug)]
pub struct DatapackAdvancement {
    pub id: String,
    pub criteria: Vec<DatapackCriterion>,
    /// Each inner list needs at least one granted criterion; all lists must be satisfied.
    pub requirements: Vec<Vec<String>>,
    pub reward_function: Option<String>,
    pub reward_experience: i32,
}

impl DatapackAdvancement {
    fn is_done(&self, granted: &HashSet<String>) -> bool {
        self.requirements
            .iter()
            .all(|group| group.iter().any(|name| granted.contains(name)))
    }
}

/// What the trigger site knows about the event.
pub enum TriggerEvent<'a> {
    Tick,
    UsingItem(&'a ItemStack),
    InteractedWithEntity {
        item: &'a ItemStack,
        target: &'a dyn EntityBase,
    },
    InventoryChanged,
}

impl TriggerEvent<'_> {
    const fn trigger(&self) -> DatapackTrigger {
        match self {
            Self::Tick => DatapackTrigger::Tick,
            Self::UsingItem(_) => DatapackTrigger::UsingItem,
            Self::InteractedWithEntity { .. } => DatapackTrigger::PlayerInteractedWithEntity,
            Self::InventoryChanged => DatapackTrigger::InventoryChanged,
        }
    }
}

#[derive(Default)]
pub struct DatapackAdvancements {
    advancements: RwLock<HashMap<String, Arc<DatapackAdvancement>>>,
    /// Granted criteria per player and advancement. Kept in memory only.
    progress: Mutex<HashMap<Uuid, HashMap<String, HashSet<String>>>>,
}

impl DatapackAdvancements {
    pub fn set(&self, advancements: HashMap<String, Arc<DatapackAdvancement>>) {
        *self
            .advancements
            .write()
            .unwrap_or_else(PoisonError::into_inner) = advancements;
        // Criteria of advancements that no longer exist would never be cleared.
        let known = self
            .advancements
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        for player in self
            .progress
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .values_mut()
        {
            player.retain(|id, _| known.contains_key(id));
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.advancements
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .is_empty()
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<Arc<DatapackAdvancement>> {
        let id = normalize_id(id);
        self.advancements
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(id.as_str())
            .cloned()
    }

    #[must_use]
    pub fn ids(&self) -> Vec<String> {
        self.advancements
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .keys()
            .cloned()
            .collect()
    }

    /// Awards the given criteria (all of them when `None`). Returns whether anything changed.
    pub fn grant(
        &self,
        server: &Arc<Server>,
        player: &Arc<Player>,
        advancement: &Arc<DatapackAdvancement>,
        criterion: Option<&str>,
    ) -> bool {
        let names: Vec<&str> = criterion.map_or_else(
            || {
                advancement
                    .criteria
                    .iter()
                    .map(|c| c.name.as_str())
                    .collect()
            },
            |name| vec![name],
        );
        let (changed, completed) = {
            let mut progress = self.progress.lock().unwrap_or_else(PoisonError::into_inner);
            let granted = progress
                .entry(player.gameprofile.id)
                .or_default()
                .entry(advancement.id.clone())
                .or_default();
            let was_done = advancement.is_done(granted);
            let mut changed = false;
            for name in names {
                changed |= granted.insert(name.to_owned());
            }
            (changed, !was_done && advancement.is_done(granted))
        };
        // Rewards run after the lock is released: reward functions usually revoke the
        // advancement again so it can fire on the next event.
        if completed {
            run_rewards(server, player, advancement);
        }
        changed
    }

    /// Removes the given criteria (all of them when `None`). Returns whether anything changed.
    pub fn revoke(
        &self,
        player: &Player,
        advancement: &DatapackAdvancement,
        criterion: Option<&str>,
    ) -> bool {
        let mut progress = self.progress.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(granted) = progress
            .get_mut(&player.gameprofile.id)
            .and_then(|p| p.get_mut(&advancement.id))
        else {
            return false;
        };
        if let Some(name) = criterion {
            granted.remove(name)
        } else {
            let changed = !granted.is_empty();
            granted.clear();
            changed
        }
    }

    /// Mirrors `SimpleCriterionTrigger.trigger`: every criterion of an unfinished
    /// advancement listening to this trigger is tested and awarded when it matches.
    pub fn trigger(&self, server: &Arc<Server>, player: &Arc<Player>, event: &TriggerEvent) {
        let trigger = event.trigger();
        let matched: Vec<(Arc<DatapackAdvancement>, String)> = {
            let advancements = self
                .advancements
                .read()
                .unwrap_or_else(PoisonError::into_inner);
            if advancements.is_empty() {
                return;
            }
            let progress = self.progress.lock().unwrap_or_else(PoisonError::into_inner);
            let player_progress = progress.get(&player.gameprofile.id);
            let mut matched = Vec::new();
            for advancement in advancements.values() {
                let granted = player_progress.and_then(|p| p.get(&advancement.id));
                if granted.is_some_and(|g| advancement.is_done(g)) {
                    continue;
                }
                for criterion in &advancement.criteria {
                    if criterion.trigger != trigger
                        || granted.is_some_and(|g| g.contains(&criterion.name))
                    {
                        continue;
                    }
                    matched.push((advancement.clone(), criterion.name.clone()));
                }
            }
            matched
        };
        for (advancement, name) in matched {
            let Some(criterion) = advancement.criteria.iter().find(|c| c.name == name) else {
                continue;
            };
            if test_criterion(&criterion.conditions, player, event) {
                self.grant(server, player, &advancement, Some(&name));
            }
        }
    }
}

fn run_rewards(server: &Arc<Server>, player: &Arc<Player>, advancement: &DatapackAdvancement) {
    if advancement.reward_experience > 0 {
        player.add_experience_points(advancement.reward_experience);
    }
    if let Some(function) = &advancement.reward_function {
        // Vanilla runs reward functions as the player with suppressed output and
        // gamemaster permissions, whatever the player's own op level is.
        let source = player
            .get_command_source(server)
            .with_output(CommandSender::Console)
            .with_silent();
        if let Err(error) = server
            .datapack_manager
            .execute_function(server, &source, function)
        {
            warn!(
                "Failed to run reward function {function} of advancement {}: {error}",
                advancement.id
            );
        }
    }
}

fn write_item(stack: &ItemStack) -> NbtCompound {
    let mut nbt = NbtCompound::new();
    stack.write_item_stack(&mut nbt);
    nbt
}

/// Tests a `ContextAwarePredicate`: a list of loot conditions, or the legacy
/// shorthand of a bare entity predicate.
fn test_context_predicate(predicate: Option<&NbtTag>, entity: &dyn EntityBase) -> bool {
    match predicate {
        None => true,
        Some(NbtTag::List(conditions)) => conditions.iter().all(|condition| match condition {
            NbtTag::Compound(condition) => test_condition(condition, Some(entity)),
            _ => false,
        }),
        Some(NbtTag::Compound(compound)) if compound.get_string("condition").is_some() => {
            test_condition(compound, Some(entity))
        }
        Some(NbtTag::Compound(compound)) => test_entity(compound, entity),
        Some(_) => false,
    }
}

fn test_criterion(conditions: &NbtCompound, player: &Arc<Player>, event: &TriggerEvent) -> bool {
    if !test_context_predicate(conditions.get("player"), player.as_ref()) {
        return false;
    }
    match event {
        TriggerEvent::Tick => true,
        TriggerEvent::UsingItem(stack) => conditions
            .get_compound("item")
            .is_none_or(|item| test_item(item, &write_item(stack))),
        TriggerEvent::InteractedWithEntity { item, target } => {
            conditions
                .get_compound("item")
                .is_none_or(|predicate| test_item(predicate, &write_item(item)))
                && test_context_predicate(conditions.get("entity"), *target)
        }
        TriggerEvent::InventoryChanged => {
            let Some(NbtTag::List(predicates)) = conditions.get("items") else {
                return true;
            };
            let items: Vec<NbtCompound> = player
                .inventory()
                .main_inventory
                .read()
                .unwrap_or_else(PoisonError::into_inner)
                .iter()
                .filter(|stack| !stack.is_empty())
                .map(write_item)
                .collect();
            predicates.iter().all(|predicate| match predicate {
                NbtTag::Compound(predicate) => items.iter().any(|item| test_item(predicate, item)),
                _ => false,
            })
        }
    }
}

fn normalize_id(id: &str) -> String {
    if id.contains(':') {
        id.to_owned()
    } else {
        format!("minecraft:{id}")
    }
}

/// Loads every `data/<namespace>/advancement/**.json` of a pack into `out`.
pub fn load_advancements_from_pack<S: BuildHasher>(
    pack_root: &Path,
    out: &mut HashMap<String, Arc<DatapackAdvancement>, S>,
) -> usize {
    let Ok(namespaces) = fs::read_dir(pack_root.join("data")) else {
        return 0;
    };
    let mut count = 0;
    for ns_entry in namespaces.flatten() {
        let namespace = ns_entry.file_name().to_string_lossy().into_owned();
        for dir_name in ["advancement", "advancements"] {
            let dir = ns_entry.path().join(dir_name);
            if dir.is_dir() {
                count += load_dir(&namespace, &dir, &dir, out);
            }
        }
    }
    count
}

fn load_dir<S: BuildHasher>(
    namespace: &str,
    base: &Path,
    dir: &Path,
    out: &mut HashMap<String, Arc<DatapackAdvancement>, S>,
) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    let mut count = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            count += load_dir(namespace, base, &path, out);
            continue;
        }
        if !path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            continue;
        }
        let Ok(relative) = path
            .with_extension("")
            .strip_prefix(base)
            .map(Path::to_path_buf)
        else {
            continue;
        };
        let id = format!(
            "{namespace}:{}",
            relative.to_string_lossy().replace('\\', "/")
        );
        let parsed = fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|content| {
                serde_json::from_str::<serde_json::Value>(&content).map_err(|e| e.to_string())
            })
            .and_then(|json| parse_advancement(&id, &json));
        match parsed {
            Ok(advancement) => {
                out.insert(id, Arc::new(advancement));
                count += 1;
            }
            Err(error) => warn!("Couldn't load advancement {id}: {error}"),
        }
    }
    count
}

fn parse_advancement(id: &str, json: &serde_json::Value) -> Result<DatapackAdvancement, String> {
    let criteria_json = json
        .get("criteria")
        .and_then(serde_json::Value::as_object)
        .filter(|criteria| !criteria.is_empty())
        .ok_or("Advancement criteria cannot be empty")?;
    let mut criteria = Vec::with_capacity(criteria_json.len());
    for (name, criterion) in criteria_json {
        let trigger = criterion
            .get("trigger")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("Criterion {name} has no trigger"))?;
        let conditions = match criterion.get("conditions").map(json_to_nbt) {
            Some(NbtTag::Compound(conditions)) => conditions,
            _ => NbtCompound::new(),
        };
        criteria.push(DatapackCriterion {
            name: name.clone(),
            trigger: DatapackTrigger::from_id(trigger),
            conditions,
        });
    }

    // Without explicit requirements every criterion is needed (AdvancementRequirements.allOf).
    let requirements: Vec<Vec<String>> = json
        .get("requirements")
        .and_then(serde_json::Value::as_array)
        .map_or_else(
            || criteria.iter().map(|c| vec![c.name.clone()]).collect(),
            |groups| {
                groups
                    .iter()
                    .map(|group| {
                        group
                            .as_array()
                            .map(|names| {
                                names
                                    .iter()
                                    .filter_map(serde_json::Value::as_str)
                                    .map(str::to_owned)
                                    .collect()
                            })
                            .unwrap_or_default()
                    })
                    .collect()
            },
        );
    for name in requirements.iter().flatten() {
        if !criteria_json.contains_key(name) {
            return Err(format!("Requirement references unknown criterion {name}"));
        }
    }

    let rewards = json.get("rewards");
    let reward_function = rewards
        .and_then(|r| r.get("function"))
        .and_then(serde_json::Value::as_str)
        .map(normalize_id);
    let reward_experience = rewards
        .and_then(|r| r.get("experience"))
        .and_then(serde_json::Value::as_i64)
        .and_then(|xp| i32::try_from(xp).ok())
        .unwrap_or(0);

    Ok(DatapackAdvancement {
        id: id.to_owned(),
        criteria,
        requirements,
        reward_function,
        reward_experience,
    })
}
