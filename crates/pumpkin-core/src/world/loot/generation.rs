use std::collections::HashSet;

use super::{
    DynamicLootCondition, DynamicLootEntry, DynamicLootEntryKind, DynamicLootFunction,
    DynamicLootPool, DynamicLootTable, Item, ItemStack, LootCondition, LootContextParameters,
    LootEntry, LootEntryKind, LootFunction, LootFunctionKind, LootNumberProvider, LootPool,
    LootTable, LootTableHandle, RandomImpl, Taggable, Xoroshiro, apply_bonus_formula,
    check_condition, check_dynamic_condition, get_loot_table, smelt,
};

#[derive(Clone, Copy)]
pub(super) enum Table<'a> {
    Static(&'a LootTable),
    Dynamic(&'a DynamicLootTable),
}

#[derive(Clone, Copy)]
enum Pool<'a> {
    Static(&'a LootPool),
    Dynamic(&'a DynamicLootPool),
}
#[derive(Clone, Copy)]
enum Entries<'a> {
    Static(&'a [LootEntry]),
    Dynamic(&'a [DynamicLootEntry]),
}
#[derive(Clone, Copy)]
enum Entry<'a> {
    Static(&'a LootEntry),
    Dynamic(&'a DynamicLootEntry),
}
#[derive(Clone, Copy)]
enum Functions<'a> {
    Static(&'a [LootFunction]),
    Dynamic(&'a [DynamicLootFunction]),
}
#[derive(Clone, Copy)]
enum Condition<'a> {
    Static(LootCondition),
    Dynamic(&'a DynamicLootCondition),
}

enum Kind<'a> {
    Item(&'a str),
    Dynamic(&'a str),
    Empty,
    Tag(&'a str, bool),
    Alternatives(Entries<'a>),
    Group(Entries<'a>),
    Sequence(Entries<'a>),
    Table(&'a str),
    InlineTable(Table<'a>),
}

impl<'a> Table<'a> {
    fn pools(self) -> impl Iterator<Item = Pool<'a>> {
        let (s, d): (&[_], &[_]) = match self {
            Self::Static(table) => (table.pools, &[]),
            Self::Dynamic(table) => (&[], &table.pools),
        };
        s.iter()
            .map(Pool::Static)
            .chain(d.iter().map(Pool::Dynamic))
    }
    fn functions(self) -> Functions<'a> {
        match self {
            Self::Static(table) => Functions::Static(table.functions),
            Self::Dynamic(table) => Functions::Dynamic(&table.functions),
        }
    }
    const fn identity(self) -> *const () {
        match self {
            Self::Static(table) => std::ptr::from_ref(table).cast(),
            Self::Dynamic(table) => std::ptr::from_ref(table).cast(),
        }
    }
}
impl<'a> Pool<'a> {
    fn entries(self) -> Entries<'a> {
        match self {
            Self::Static(pool) => Entries::Static(pool.entries),
            Self::Dynamic(pool) => Entries::Dynamic(&pool.entries),
        }
    }
    const fn condition(self) -> Condition<'a> {
        match self {
            Self::Static(pool) => Condition::Static(pool.condition),
            Self::Dynamic(pool) => Condition::Dynamic(&pool.condition),
        }
    }
    fn functions(self) -> Functions<'a> {
        match self {
            Self::Static(pool) => Functions::Static(pool.functions),
            Self::Dynamic(pool) => Functions::Dynamic(&pool.functions),
        }
    }
    const fn rolls(self) -> (LootNumberProvider, LootNumberProvider) {
        match self {
            Self::Static(pool) => (pool.rolls, pool.bonus_rolls),
            Self::Dynamic(pool) => (pool.rolls, pool.bonus_rolls),
        }
    }
}
impl<'a> Entries<'a> {
    fn iter(self) -> impl Iterator<Item = Entry<'a>> {
        let (s, d): (&[_], &[_]) = match self {
            Self::Static(entries) => (entries, &[]),
            Self::Dynamic(entries) => (&[], entries),
        };
        s.iter()
            .map(Entry::Static)
            .chain(d.iter().map(Entry::Dynamic))
    }
}
impl<'a> Entry<'a> {
    fn kind(self) -> Kind<'a> {
        match self {
            Self::Static(entry) => match &entry.kind {
                LootEntryKind::Item(name) => Kind::Item(name),
                LootEntryKind::Dynamic(name) => Kind::Dynamic(name),
                LootEntryKind::Empty => Kind::Empty,
                LootEntryKind::Tag { name, expand } => Kind::Tag(name, *expand),
                LootEntryKind::Alternatives(entries) => {
                    Kind::Alternatives(Entries::Static(entries))
                }
                LootEntryKind::Group(children) => Kind::Group(Entries::Static(children)),
                LootEntryKind::Sequence(children) => Kind::Sequence(Entries::Static(children)),
                LootEntryKind::Table(key) => Kind::Table(key),
                LootEntryKind::InlineTable(table) => Kind::InlineTable(Table::Static(table)),
            },
            Self::Dynamic(entry) => match &entry.kind {
                DynamicLootEntryKind::Item(name) => Kind::Item(name),
                DynamicLootEntryKind::Dynamic(name) => Kind::Dynamic(name),
                DynamicLootEntryKind::Empty => Kind::Empty,
                DynamicLootEntryKind::Tag { name, expand } => Kind::Tag(name, *expand),
                DynamicLootEntryKind::Alternatives(entries) => {
                    Kind::Alternatives(Entries::Dynamic(entries))
                }
                DynamicLootEntryKind::Group(children) => Kind::Group(Entries::Dynamic(children)),
                DynamicLootEntryKind::Sequence(children) => {
                    Kind::Sequence(Entries::Dynamic(children))
                }
                DynamicLootEntryKind::Table(key) => Kind::Table(key),
                DynamicLootEntryKind::InlineTable(table) => {
                    Kind::InlineTable(Table::Dynamic(table))
                }
            },
        }
    }
    const fn condition(self) -> Condition<'a> {
        match self {
            Self::Static(entry) => Condition::Static(entry.condition),
            Self::Dynamic(entry) => Condition::Dynamic(&entry.condition),
        }
    }
    fn functions(self) -> Functions<'a> {
        match self {
            Self::Static(entry) => Functions::Static(entry.functions),
            Self::Dynamic(entry) => Functions::Dynamic(&entry.functions),
        }
    }
    fn weight(self, luck: f32) -> i32 {
        let (weight, quality) = match self {
            Self::Static(entry) => (entry.weight, entry.quality),
            Self::Dynamic(entry) => (entry.weight, entry.quality),
        };
        (weight as f32 + quality as f32 * luck).floor().max(0.0) as i32
    }
}
impl<'a> Functions<'a> {
    fn iter(self) -> impl Iterator<Item = (LootFunctionKind, Condition<'a>, Option<&'a str>)> {
        let (s, d): (&[_], &[_]) = match self {
            Self::Static(functions) => (functions, &[]),
            Self::Dynamic(functions) => (&[], functions),
        };
        s.iter()
            .map(|function: &LootFunction| {
                (
                    function.kind,
                    Condition::Static(function.condition),
                    function.enchantment,
                )
            })
            .chain(d.iter().map(|function: &DynamicLootFunction| {
                (
                    function.kind,
                    Condition::Dynamic(&function.condition),
                    function.enchantment.as_deref(),
                )
            }))
    }
    const fn is_empty(self) -> bool {
        match self {
            Self::Static(functions) => functions.is_empty(),
            Self::Dynamic(functions) => functions.is_empty(),
        }
    }
}

struct Candidate<'a> {
    entry: Entry<'a>,
    item: Option<&'static Item>,
    parents: Vec<Functions<'a>>,
}
struct Stack {
    item: &'static Item,
    count: i32,
}

pub(super) fn generate(
    table: Table<'_>,
    seed: i64,
    params: &LootContextParameters<'_>,
) -> Vec<ItemStack> {
    let has_silk_touch = params
        .tool
        .as_ref()
        .is_some_and(|tool| tool.get_enchantment_level(&pumpkin_data::Enchantment::SILK_TOUCH) > 0);
    let has_shears = params
        .tool
        .as_ref()
        .is_some_and(|tool| tool.item == &Item::SHEARS);
    let mut generator = Generator {
        params,
        rng: Xoroshiro::from_seed(seed as u64),
        visited: HashSet::new(),
        has_silk_touch,
        has_shears,
    };
    let mut stacks = Vec::new();
    generator.table(table, &[], &mut stacks);
    let mut result = Vec::new();
    for stack in stacks {
        let max_count = i32::from(ItemStack::new(1, stack.item).get_max_stack_size());
        let mut count = stack.count;
        while count > 0 {
            let split = count.min(max_count);
            result.push(ItemStack::new(split as u8, stack.item));
            count -= split;
        }
    }
    result
}

struct Generator<'a, 'b> {
    params: &'a LootContextParameters<'b>,
    rng: Xoroshiro,
    visited: HashSet<*const ()>,
    has_silk_touch: bool,
    has_shears: bool,
}

fn chain<'a>(functions: Functions<'a>, outer: &[Functions<'a>]) -> Vec<Functions<'a>> {
    let mut chain = Vec::with_capacity(outer.len() + usize::from(!functions.is_empty()));
    if !functions.is_empty() {
        chain.push(functions);
    }
    chain.extend_from_slice(outer);
    chain
}

impl Generator<'_, '_> {
    fn test(&mut self, condition: Condition<'_>) -> bool {
        match condition {
            Condition::Static(condition) => check_condition(
                condition,
                self.has_silk_touch,
                self.has_shears,
                self.params,
                &mut self.rng,
            ),
            Condition::Dynamic(condition) => check_dynamic_condition(
                condition,
                self.has_silk_touch,
                self.has_shears,
                self.params,
                &mut self.rng,
            ),
        }
    }
    fn between(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        min + self.rng.next_bounded_i32(max - min + 1)
    }
    fn number_int(&mut self, provider: LootNumberProvider) -> i32 {
        match provider {
            LootNumberProvider::Constant(value) => value as i32,
            LootNumberProvider::Uniform { min, max } => self.between(min as i32, max as i32),
            LootNumberProvider::Binomial { n, probability } => {
                (0..n).filter(|_| self.rng.next_f32() < probability).count() as i32
            }
        }
    }
    fn number_float(&mut self, provider: LootNumberProvider) -> f32 {
        match provider {
            LootNumberProvider::Constant(value) => value as f32,
            LootNumberProvider::Uniform { min, max } => {
                let (min, max) = (min as f32, max as f32);
                if min >= max {
                    min
                } else {
                    min + self.rng.next_f32() * (max - min)
                }
            }
            LootNumberProvider::Binomial { .. } => self.number_int(provider) as f32,
        }
    }
    fn table(&mut self, table: Table<'_>, outer: &[Functions<'_>], output: &mut Vec<Stack>) {
        let identity = table.identity();
        if !self.visited.insert(identity) {
            tracing::warn!("Detected recursive loot table");
            return;
        }
        let chain = chain(table.functions(), outer);
        for pool in table.pools() {
            self.pool(pool, &chain, output);
        }
        self.visited.remove(&identity);
    }
    fn pool(&mut self, pool: Pool<'_>, outer: &[Functions<'_>], output: &mut Vec<Stack>) {
        if !self.test(pool.condition()) {
            return;
        }
        let functions = chain(pool.functions(), outer);
        let (rolls, bonus) = pool.rolls();
        let rolls = self
            .number_int(rolls)
            .saturating_add((self.number_float(bonus) * self.params.luck).floor() as i32);
        for _ in 0..rolls {
            let mut candidates = Vec::new();
            for entry in pool.entries().iter() {
                self.expand(entry, &mut Vec::new(), &mut candidates);
            }
            candidates.retain(|candidate| candidate.entry.weight(self.params.luck) > 0);
            let Some(total) = candidates.iter().try_fold(0i32, |sum, candidate| {
                sum.checked_add(candidate.entry.weight(self.params.luck))
            }) else {
                tracing::warn!("Loot pool weights overflow");
                continue;
            };
            if candidates.is_empty() || total <= 0 {
                continue;
            }
            let selected = if candidates.len() == 1 {
                0
            } else {
                let mut pick = self.rng.next_bounded_i32(total);
                let Some(index) = candidates.iter().position(|candidate| {
                    pick -= candidate.entry.weight(self.params.luck);
                    pick < 0
                }) else {
                    continue;
                };
                index
            };
            let candidate = &candidates[selected];
            let mut entry_functions = chain(candidate.entry.functions(), &[]);
            entry_functions.extend(candidate.parents.iter().rev().copied());
            entry_functions.extend_from_slice(&functions);
            self.create(candidate, &entry_functions, output);
        }
    }
    fn expand<'a>(
        &mut self,
        entry: Entry<'a>,
        parents: &mut Vec<Functions<'a>>,
        output: &mut Vec<Candidate<'a>>,
    ) -> bool {
        if !self.test(entry.condition()) {
            return false;
        }
        if let Kind::Alternatives(children) | Kind::Group(children) | Kind::Sequence(children) =
            entry.kind()
        {
            let has_functions = !entry.functions().is_empty();
            if has_functions {
                parents.push(entry.functions());
            }
            let result = match entry.kind() {
                Kind::Alternatives(_) => children
                    .iter()
                    .any(|child| self.expand(child, parents, output)),
                Kind::Sequence(_) => children
                    .iter()
                    .all(|child| self.expand(child, parents, output)),
                Kind::Group(_) => {
                    let mut children = children.iter();
                    match (children.next(), children.next()) {
                        (None, _) => true,
                        (Some(first), None) => self.expand(first, parents, output),
                        (Some(first), Some(second)) => {
                            self.expand(first, parents, output);
                            self.expand(second, parents, output);
                            for child in children {
                                self.expand(child, parents, output);
                            }
                            true
                        }
                    }
                }
                _ => false,
            };
            if has_functions {
                parents.pop();
            }
            return result;
        }
        if let Kind::Tag(name, true) = entry.kind() {
            for name in Item::get_tag_values(name).unwrap_or_default() {
                if let Some(item) = Item::from_registry_key(name.trim_start_matches("minecraft:")) {
                    output.push(Candidate {
                        entry,
                        item: Some(item),
                        parents: parents.clone(),
                    });
                }
            }
        } else {
            output.push(Candidate {
                entry,
                item: None,
                parents: parents.clone(),
            });
        }
        true
    }
    fn create(
        &mut self,
        candidate: &Candidate<'_>,
        functions: &[Functions<'_>],
        output: &mut Vec<Stack>,
    ) {
        if let Some(item) = candidate.item {
            self.emit(Stack { item, count: 1 }, functions, output);
            return;
        }
        match candidate.entry.kind() {
            Kind::Item(name) => {
                if let Some(item) = Item::from_registry_key(name.trim_start_matches("minecraft:")) {
                    self.emit(Stack { item, count: 1 }, functions, output);
                }
            }
            // Dynamic drops emit nothing without a block-registered callback.
            Kind::Dynamic(_name) => {}
            Kind::Tag(name, _) => {
                for name in Item::get_tag_values(name).unwrap_or_default() {
                    if let Some(item) =
                        Item::from_registry_key(name.trim_start_matches("minecraft:"))
                    {
                        self.emit(Stack { item, count: 1 }, functions, output);
                    }
                }
            }
            Kind::Table(key) => {
                let handle = self
                    .params
                    .world
                    .map_or_else(|| get_loot_table(key), |world| world.get_loot_table(key));
                if let Some(handle) = handle {
                    match &handle {
                        LootTableHandle::Static(table) => {
                            self.table(Table::Static(table), functions, output);
                        }
                        LootTableHandle::Dynamic(table) => {
                            self.table(Table::Dynamic(table), functions, output);
                        }
                    }
                } else {
                    tracing::warn!("Unknown loot table {key}");
                }
            }
            Kind::InlineTable(table) => self.table(table, functions, output),
            Kind::Empty | Kind::Alternatives(_) | Kind::Group(_) | Kind::Sequence(_) => {}
        }
    }
    fn emit(&mut self, mut stack: Stack, functions: &[Functions<'_>], output: &mut Vec<Stack>) {
        for functions in functions {
            for (kind, condition, enchantment) in functions.iter() {
                if !self.test(condition) {
                    continue;
                }
                match kind {
                    LootFunctionKind::SetCount { count, add } => {
                        let count = self.number_int(count);
                        stack.count = if add {
                            stack.count.max(0).saturating_add(count)
                        } else {
                            count
                        };
                    }
                    LootFunctionKind::ApplyBonus(formula) => {
                        if self.params.tool.is_none() {
                            continue;
                        }
                        stack.count = apply_bonus_formula(
                            stack.count.max(0),
                            formula,
                            enchantment
                                .map_or(0, |name| super::tool_enchantment_level(self.params, name)),
                            &mut self.rng,
                        );
                    }
                    LootFunctionKind::EnchantedCountIncrease { count, limit } => {
                        let level = enchantment.map_or(0, |name| {
                            super::attacker_enchantment_level(self.params, name)
                        });
                        if level > 0 {
                            let addition = (f64::from(level as f32 * self.number_float(count))
                                + 0.5)
                                .floor() as i32;
                            stack.count = stack.count.max(0).saturating_add(addition);
                            if limit > 0 {
                                stack.count = stack.count.min(limit);
                            }
                        }
                    }
                    LootFunctionKind::LimitCount { min, max } => {
                        let count = stack.count.max(0);
                        let min = min.map(|value| self.number_int(value));
                        let max = max.map(|value| self.number_int(value));
                        stack.count = match (min, max) {
                            (Some(min), Some(max)) => {
                                if count < min {
                                    min
                                } else {
                                    count.min(max)
                                }
                            }
                            (Some(min), None) => count.max(min),
                            (None, Some(max)) => count.min(max),
                            (None, None) => count,
                        };
                    }
                    LootFunctionKind::ExplosionDecay => {
                        if let Some(radius) = self.params.explosion_radius
                            && radius > 1.0
                        {
                            stack.count = (0..stack.count.max(0))
                                .filter(|_| self.rng.next_f32() <= 1.0 / radius)
                                .count() as i32;
                        }
                    }
                    LootFunctionKind::FurnaceSmelt => {
                        if stack.count <= 0 {
                            continue;
                        }
                        let (item, count) = smelt(stack.item, stack.count);
                        stack.item = item;
                        stack.count = count;
                    }
                }
            }
        }
        output.push(stack);
    }
}
