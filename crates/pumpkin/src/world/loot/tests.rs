use super::*;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{
    CustomNameImpl, DataComponentImpl, EnchantmentsImpl, LoreImpl, RepairCostImpl,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::loot_table::LootPool;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use std::sync::atomic::{AtomicU32, Ordering};

struct ComponentSource {
    collections: AtomicU32,
}

impl BlockEntity for ComponentSource {
    /// This source has no persistent block fields.
    fn write_nbt(&self, _nbt: &mut NbtCompound) {}

    /// Creates a source whose successive snapshots make function ordering observable.
    fn from_nbt(_nbt: &NbtCompound, _position: BlockPos) -> Self {
        Self {
            collections: AtomicU32::new(0),
        }
    }

    /// Identifies the source as a chest without depending on a loaded world.
    fn resource_location(&self) -> &'static str {
        "minecraft:chest"
    }

    /// Supplies a fixed position because component selection does not query the world.
    fn get_position(&self) -> BlockPos {
        BlockPos::new(0, 64, 0)
    }

    /// Produces a new name per collection and a stable second component for filter checks.
    fn collect_components(&self) -> Vec<(DataComponent, Box<dyn DataComponentImpl>)> {
        let sequence = self.collections.fetch_add(1, Ordering::Relaxed);
        vec![
            (
                DataComponent::CustomName,
                CustomNameImpl {
                    name: TextComponent::text(format!("Snapshot {sequence}")).bold(),
                }
                .to_dyn(),
            ),
            (
                DataComponent::RepairCost,
                RepairCostImpl { cost: 7 }.to_dyn(),
            ),
        ]
    }

    /// Allows the standard block-entity trait downcast.
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Builds a fresh source without allocating a world or consuming loot RNG.
fn source() -> Arc<ComponentSource> {
    Arc::new(ComponentSource::from_nbt(
        &NbtCompound::new(),
        BlockPos::new(0, 64, 0),
    ))
}

/// Supplies the same count and selection parameters for the function and control tables.
const fn entry(functions: &'static [LootFunction]) -> LootEntry {
    LootEntry {
        item: "minecraft:chest",
        weight: 3,
        min_count: 1,
        max_count: 4,
        condition: LootCondition::None,
        bonus_formula: Some(LootBonusFormula::UniformBonusCount(2)),
        functions,
    }
}

/// Makes repeated rolls exercise RNG advancement after a copied stack.
const fn pool(entries: &'static [LootEntry]) -> LootPool {
    LootPool {
        entries,
        min_rolls: 4,
        max_rolls: 8,
        empty_weight: 2,
        condition: LootCondition::None,
    }
}

const COPY_ALL: LootFunction = LootFunction::CopyComponents {
    source: "block_entity",
    include: None,
    exclude: None,
    condition: LootCondition::None,
};

const COPY_TABLE: LootTable = LootTable {
    pools: &[pool(&[entry(&[COPY_ALL])])],
};

/// Omitted inclusion permits all, empty inclusion permits none, and exclusions win.
#[test]
fn component_filters_distinguish_absent_and_empty_lists() {
    let source = source();
    for (include, exclude, name, cost) in [
        (None, None, true, true),
        (Some(&[][..]), None, false, false),
        (Some(&["custom_name"][..]), None, true, false),
        (
            Some(&["minecraft:custom_name", "minecraft:repair_cost"][..]),
            Some(&["custom_name"][..]),
            false,
            true,
        ),
        (None, Some(&["minecraft:repair_cost"][..]), true, false),
    ] {
        let mut stack = ItemStack::new(1, &Item::CHEST);
        copy_components(&mut stack, source.as_ref(), include, exclude);
        assert_eq!(stack.get_data_component::<CustomNameImpl>().is_some(), name);
        assert_eq!(
            stack
                .get_data_component::<RepairCostImpl>()
                .is_some_and(|value| value.cost == 7),
            cost
        );
    }
}

/// Source values overwrite destination values; missing values never remove destination data.
#[test]
fn copy_overwrites_only_present_selected_values() {
    let source = source();
    let mut stack = ItemStack::new(1, &Item::CHEST);
    stack.set_data_component(CustomNameImpl {
        name: TextComponent::text("Previous"),
    });
    stack.set_data_component(RepairCostImpl { cost: 99 });
    stack.set_data_component(LoreImpl {
        lines: vec![TextComponent::text("Keep this lore")],
    });
    copy_components(
        &mut stack,
        source.as_ref(),
        Some(&["custom_name", "lore"]),
        None,
    );
    assert_eq!(
        stack
            .get_data_component::<CustomNameImpl>()
            .map(|v| &v.name),
        Some(&TextComponent::text("Snapshot 0").bold())
    );
    assert_eq!(
        stack.get_data_component::<RepairCostImpl>().map(|v| v.cost),
        Some(99)
    );
    assert_eq!(
        stack.get_data_component::<LoreImpl>().map(|v| &v.lines),
        Some(&vec![TextComponent::text("Keep this lore")])
    );
    assert_eq!(stack.patch.len(), 3);
}

/// Missing source entities leave otherwise generated items unchanged.
#[test]
fn missing_block_entity_does_not_change_loot() {
    let drops = generate_loot_with_context(&COPY_TABLE, 42, &LootContextParameters::default());
    assert!(!drops.is_empty());
    assert!(drops.iter().all(|stack| stack.patch.is_empty()));
}

/// Functions run in order and the later selected value replaces the earlier snapshot.
#[test]
fn ordered_functions_keep_the_last_copied_value() {
    const TABLE: LootTable = LootTable {
        pools: &[LootPool {
            entries: &[entry(&[COPY_ALL, COPY_ALL])],
            min_rolls: 1,
            max_rolls: 1,
            empty_weight: 0,
            condition: LootCondition::None,
        }],
    };
    let source = source();
    let drops = generate_loot_with_context(
        &TABLE,
        42,
        &LootContextParameters {
            block_entity: Some(source.clone()),
            ..Default::default()
        },
    );
    assert_eq!(source.collections.load(Ordering::Relaxed), 2);
    assert_eq!(drops.len(), 1);
    assert_eq!(
        drops
            .first()
            .and_then(|stack| stack.get_data_component::<CustomNameImpl>())
            .map(|v| &v.name),
        Some(&TextComponent::text("Snapshot 1").bold())
    );
}

/// A failed condition must not read the block entity or mutate the generated item.
#[test]
fn failed_function_condition_skips_collection() {
    const TABLE: LootTable = LootTable {
        pools: &[pool(&[entry(&[LootFunction::CopyComponents {
            source: "block_entity",
            include: None,
            exclude: None,
            condition: LootCondition::SilkTouch,
        }])])],
    };
    let source = source();
    let drops = generate_loot_with_context(
        &TABLE,
        42,
        &LootContextParameters {
            block_entity: Some(source.clone()),
            ..Default::default()
        },
    );
    assert!(!drops.is_empty());
    assert!(drops.iter().all(|stack| stack.patch.is_empty()));
    assert_eq!(source.collections.load(Ordering::Relaxed), 0);
}

/// Unconditional copying preserves count, selection, and subsequent random draws.
#[test]
fn copying_does_not_change_loot_randomness() {
    const CONTROL: LootTable = LootTable {
        pools: &[pool(&[entry(&[])])],
    };
    let mut tool = ItemStack::new(1, &Item::DIAMOND_PICKAXE);
    tool.set_data_component(EnchantmentsImpl {
        enchantment: std::borrow::Cow::Owned(vec![(&pumpkin_data::Enchantment::FORTUNE, 3)]),
    });
    let params = LootContextParameters {
        block_entity: Some(source()),
        tool: Some(tool),
        ..Default::default()
    };
    for seed in 0..64 {
        let copied = generate_loot_with_context(&COPY_TABLE, seed, &params);
        let control = generate_loot_with_context(&CONTROL, seed, &params);
        let counts = |stacks: Vec<ItemStack>| {
            stacks
                .into_iter()
                .map(|stack| (stack.item.id, stack.item_count))
                .collect::<Vec<_>>()
        };
        assert_eq!(counts(copied), counts(control));
    }
}
