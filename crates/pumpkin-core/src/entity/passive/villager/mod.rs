use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, Ordering};
use std::sync::{Arc, Weak};
use uuid::Uuid;

use pumpkin_data::Enchantment;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::item::{Item, JavaToBedrockItemMapping};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Effect;
use pumpkin_data::tag::{Enchantment as EnchantmentTag, Taggable};
use pumpkin_data::tracked_data;
use pumpkin_inventory::SimpleInventory;
use pumpkin_inventory::merchant::merchant_screen_handler::MerchantScreenHandler;
use pumpkin_inventory::screen_handler::{
    InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::bedrock::{
    client::set_actor_data::{MetadataValue, SyncedActorDataList, entity_data_key},
    server::actor_event::ActorEventID,
};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::CMerchantOffers;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;

use crate::entity::ageable::{AgeableData, AgeableMob};
use crate::entity::ai::brain::Brain;
use crate::entity::ai::brain::memory::PackedMemories;
use crate::entity::mob::conversion::{ConversionParams, convert_to};
use crate::entity::mob::spawn::EntitySpawnReason;
use crate::entity::player::Player;
use crate::entity::{
    Entity, EntityBase,
    experience_orb::ExperienceOrbEntity,
    mob::{Mob, MobEntity},
};
use crate::world::World;
use rand::RngExt;

pub mod ai;
pub mod breeding;
pub mod data;
pub mod golem;
pub mod gossip;
pub mod reputation;
pub mod sleep;
pub use data::{
    BREEDING_FOOD_THRESHOLD, GossipType, VillagerData, VillagerProfession, VillagerType,
    can_level_up, get_food_points, get_max_xp_per_level,
};
pub use gossip::GossipContainer;
pub use reputation::ReputationEventType;

pub(crate) fn trigger_trade_advancement(player: &Player) {
    player.trigger_advancement(
        crate::entity::player::advancement::trigger::AdvancementTrigger::TradedWithVillager,
    );
}

fn enchanted_book_offer_items(
    rng: &mut impl rand::Rng,
) -> Option<(ItemStack, ItemStack, Option<ItemStack>)> {
    use pumpkin_data::data_component::DataComponent;
    use pumpkin_data::data_component_impl::StoredEnchantmentsImpl;
    use rand::{RngExt, seq::IndexedRandom};
    use std::borrow::Cow;

    let enchantment_id = *EnchantmentTag::MINECRAFT_TRADEABLE.1.choose(rng)? as u8;
    let enchantment = Enchantment::from_id(enchantment_id)?;
    let level = rng.random_range(1..=enchantment.max_level);
    let mut emeralds = 2 + rng.random_range(0..5 + level * 10) + 3 * level;
    if enchantment.has_tag(&EnchantmentTag::MINECRAFT_DOUBLE_TRADE_PRICE) {
        emeralds *= 2;
    }

    let output = ItemStack::new_with_component(
        1,
        &Item::ENCHANTED_BOOK,
        vec![(
            DataComponent::StoredEnchantments,
            Some(Box::new(StoredEnchantmentsImpl {
                enchantment: Cow::Owned(vec![(enchantment, level)]),
            })),
        )],
    );

    Some((
        ItemStack::new(emeralds.min(64) as u8, &Item::EMERALD),
        output,
        Some(ItemStack::new(1, &Item::BOOK)),
    ))
}

fn enchant_trade_item(
    rng: &mut impl rand::Rng,
    item: &'static Item,
    min_level: i32,
    max_level: i32,
) -> Option<(ItemStack, i32)> {
    use pumpkin_data::data_component_impl::EnchantableImpl;
    use rand::RngExt;

    let mut stack = ItemStack::new(1, item);
    let enchantability = stack
        .get_data_component::<EnchantableImpl>()
        .map_or(1, |value| value.value);
    let additional_cost = rng.random_range(min_level..=max_level);
    let mut level = additional_cost
        + 1
        + rng.random_range(0..=enchantability / 4)
        + rng.random_range(0..=enchantability / 4);
    let bonus = (rng.random::<f32>() + rng.random::<f32>() - 1.0) * 0.15;
    level = ((level as f32 + level as f32 * bonus).round() as i32).max(1);

    let mut available = EnchantmentTag::MINECRAFT_ON_TRADED_EQUIPMENT
        .1
        .iter()
        .filter_map(|id| Enchantment::from_id(*id as u8))
        .filter(|enchantment| enchantment.can_enchant(item))
        .filter_map(|enchantment| {
            (1..=enchantment.max_level)
                .rev()
                .find(|candidate_level| {
                    (enchantment.min_cost.calculate(*candidate_level)
                        ..=enchantment.max_cost.calculate(*candidate_level))
                        .contains(&level)
                })
                .map(|candidate_level| (enchantment, candidate_level))
        })
        .collect::<Vec<_>>();

    if available.is_empty() {
        return None;
    }

    while !available.is_empty() {
        let total_weight: i32 = available
            .iter()
            .map(|(enchantment, _)| enchantment.weight)
            .sum();
        let mut choice = rng.random_range(0..total_weight);
        let chosen_index = available
            .iter()
            .position(|(enchantment, _)| {
                choice -= enchantment.weight;
                choice < 0
            })
            .unwrap_or(0);
        let (enchantment, enchantment_level) = available[chosen_index];
        stack.enchant(enchantment, enchantment_level);

        if rng.random_range(0..50) > level {
            break;
        }
        available.retain(|(candidate, _)| candidate.are_compatible(enchantment));
        level /= 2;
    }
    Some((stack, additional_cost))
}

pub(crate) fn apply_random_dye(rng: &mut impl rand::Rng, stack: &mut ItemStack) {
    use pumpkin_data::data_component::DataComponent;
    use pumpkin_data::data_component_impl::{DataComponentImpl, DyedColorImpl};
    use rand::RngExt;

    const COLORS: [i32; 16] = [
        0xF9FFFE, 0xF9801D, 0xC74EBD, 0x3AB3DA, 0xFED83D, 0x80C71F, 0xF38BAA, 0x474F52, 0x9D9D97,
        0x169C9C, 0x8932B8, 0x3C44AA, 0x835432, 0x5E7C16, 0xB02E26, 0x1D1D21,
    ];
    let dye_count = 1 + i32::from(rng.random_bool(0.75)) + i32::from(rng.random_bool(0.75));
    let mut channels = [0; 3];
    let mut brightness = 0;
    for _ in 0..dye_count {
        let color = COLORS[rng.random_range(0..COLORS.len())];
        let rgb = [(color >> 16) & 255, (color >> 8) & 255, color & 255];
        brightness += rgb[0].max(rgb[1]).max(rgb[2]);
        for (total, value) in channels.iter_mut().zip(rgb) {
            *total += value;
        }
    }
    let mut rgb = channels.map(|channel| channel / dye_count);
    let average_brightness = brightness / dye_count;
    let max_channel = rgb[0].max(rgb[1]).max(rgb[2]);
    for channel in &mut rgb {
        *channel = average_brightness * *channel / max_channel;
    }
    let color = (rgb[0] << 16) | (rgb[1] << 8) | rgb[2];
    stack.patch.push((
        DataComponent::DyedColor,
        Some(DyedColorImpl { rgb: color }.to_dyn()),
    ));
}

pub(crate) fn apply_stew_effect(
    rng: &mut impl rand::Rng,
    stack: &mut ItemStack,
    effects: &[(String, i32)],
) {
    use rand::RngExt;

    if effects.is_empty() {
        return;
    }
    let (effect, duration) = &effects[rng.random_range(0..effects.len())];
    if let Some(effect) = pumpkin_data::effect::StatusEffect::from_minecraft_name(effect) {
        crate::world::loot::add_stew_effect(stack, effect, *duration);
    }
}

pub(crate) fn apply_potion(stack: &mut ItemStack, potion_name: &str) {
    use pumpkin_data::data_component::DataComponent;
    use pumpkin_data::data_component_impl::{DataComponentImpl, PotionContentsImpl};

    let Some(potion) = pumpkin_data::potion::Potion::from_name(
        potion_name
            .strip_prefix("minecraft:")
            .unwrap_or(potion_name),
    ) else {
        return;
    };
    stack.patch.push((
        DataComponent::PotionContents,
        Some(
            PotionContentsImpl {
                potion_id: Some(i32::from(potion.id)),
                custom_color: None,
                custom_effects: Vec::new(),
                custom_name: None,
            }
            .to_dyn(),
        ),
    ));
}

/// Resolves a trade destination to its structure set, structure, fallback map
/// name and decoration. 26.3 prefixes these with `#` and renamed several, so
/// both spellings are accepted.
fn explorer_map_target(
    destination: &str,
) -> Option<(
    &'static str,
    pumpkin_data::structures::StructureKeys,
    &'static str,
    i32,
)> {
    use pumpkin_data::structures::StructureKeys;

    let destination = destination.strip_prefix('#').unwrap_or(destination);
    Some(match destination {
        "minecraft:on_jungle_pyramid_maps" | "minecraft:on_jungle_explorer_maps" => (
            "jungle_temples",
            StructureKeys::JunglePyramid,
            "filled_map.explorer_jungle",
            32,
        ),
        "minecraft:on_swamp_hut_maps" | "minecraft:on_swamp_explorer_maps" => (
            "swamp_huts",
            StructureKeys::SwampHut,
            "filled_map.explorer_swamp",
            33,
        ),
        "minecraft:on_desert_village_maps" => (
            "villages",
            StructureKeys::VillageDesert,
            "filled_map.village_desert",
            27,
        ),
        "minecraft:on_plains_village_maps" => (
            "villages",
            StructureKeys::VillagePlains,
            "filled_map.village_plains",
            28,
        ),
        "minecraft:on_savanna_village_maps" => (
            "villages",
            StructureKeys::VillageSavanna,
            "filled_map.village_savanna",
            29,
        ),
        "minecraft:on_snowy_village_maps" => (
            "villages",
            StructureKeys::VillageSnowy,
            "filled_map.village_snowy",
            30,
        ),
        "minecraft:on_taiga_village_maps" => (
            "villages",
            StructureKeys::VillageTaiga,
            "filled_map.village_taiga",
            31,
        ),
        "minecraft:on_ocean_monument_maps" | "minecraft:on_ocean_explorer_maps" => (
            "ocean_monuments",
            StructureKeys::Monument,
            "filled_map.monument",
            9,
        ),
        "minecraft:on_buried_trial_chambers_maps" | "minecraft:on_trial_chambers_maps" => (
            "trial_chambers",
            StructureKeys::TrialChambers,
            "filled_map.trial_chambers",
            34,
        ),
        "minecraft:on_woodland_mansion_maps" | "minecraft:on_woodland_explorer_maps" => (
            "woodland_mansions",
            StructureKeys::Mansion,
            "filled_map.mansion",
            8,
        ),
        _ => return None,
    })
}

const INVENTORY_SIZE: usize = 8;

pub struct VillagerEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    villager_data_finalized: AtomicBool,
    pub villager_data: std::sync::Mutex<VillagerData>,
    pub food_level: AtomicI32,
    pub xp: AtomicI32,
    pub last_restock_time: AtomicI64,
    pub last_restock_check_day: AtomicI64,
    pub restocks_today: AtomicI32,
    pub last_gossip_decay_time: AtomicI64,
    pub last_gossip_share_time: AtomicI64,
    pub gossips: std::sync::Mutex<GossipContainer>,
    pub inventory: std::sync::Mutex<Vec<ItemStack>>,
    pub merchant_inventory: Arc<SimpleInventory>,
    pub offers: std::sync::Mutex<Vec<pumpkin_protocol::java::client::play::MerchantOffer>>,
    pub unhappy_counter: AtomicI32,
    pub trade_sound_cooldown: AtomicI32,
    pub increase_profession_level_on_update: AtomicBool,
    pub last_traded_player: std::sync::Mutex<Option<Uuid>>,
    pub trading_player: std::sync::Mutex<Option<(Uuid, u8)>>,
    pub is_trading: AtomicBool,
    pub(crate) sleeping_pos: std::sync::Mutex<Option<BlockPos>>,
    pub self_weak: std::sync::Mutex<Option<Weak<Self>>>,
}

impl VillagerEntity {
    fn bedrock_metadata(data: VillagerData, xp: i32) -> SyncedActorDataList {
        const PROFESSIONS: [i32; 15] = [0, 8, 11, 6, 7, 1, 2, 4, 12, 5, 13, 14, 3, 10, 9];
        const REGIONS: [i32; 7] = [1, 2, 0, 3, 4, 5, 6];

        let mut metadata = SyncedActorDataList::new();
        metadata.set(
            entity_data_key::VARIANT,
            MetadataValue::Int(
                usize::try_from(data.profession.0)
                    .ok()
                    .and_then(|id| PROFESSIONS.get(id))
                    .copied()
                    .unwrap_or_default(),
            ),
        );
        metadata.set(
            entity_data_key::MARK_VARIANT,
            MetadataValue::Int(
                usize::try_from(data.r#type.0)
                    .ok()
                    .and_then(|id| REGIONS.get(id))
                    .copied()
                    .unwrap_or_default(),
            ),
        );
        metadata.set(
            entity_data_key::TRADE_TIER,
            MetadataValue::Int(data.level.0.saturating_sub(1)),
        );
        metadata.set(entity_data_key::MAX_TRADE_TIER, MetadataValue::Int(4));
        metadata.set(entity_data_key::TRADE_EXPERIENCE, MetadataValue::Int(xp));
        metadata
    }

    #[allow(clippy::too_many_lines)]
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        let navigator = mob_entity
            .navigator
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        navigator.set_can_open_doors(true);
        navigator.set_can_float(true);
        navigator.set_required_path_length(48.0);
        mob_entity.set_can_pick_up_loot(true);
        let villager_data = VillagerData::new(VillagerType::Plains, VillagerProfession::None, 1);
        let inventory = std::sync::Mutex::new((0..8).map(|_| ItemStack::EMPTY.clone()).collect());

        let villager = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            villager_data_finalized: AtomicBool::new(false),
            villager_data: std::sync::Mutex::new(villager_data),
            food_level: AtomicI32::new(0),
            xp: AtomicI32::new(0),
            last_restock_time: AtomicI64::new(0),
            last_restock_check_day: AtomicI64::new(0),
            restocks_today: AtomicI32::new(0),
            last_gossip_decay_time: AtomicI64::new(0),
            last_gossip_share_time: AtomicI64::new(0),
            gossips: std::sync::Mutex::new(GossipContainer::new()),
            inventory,
            merchant_inventory: Arc::new(SimpleInventory::new(3)),
            offers: std::sync::Mutex::new(Vec::new()),
            unhappy_counter: AtomicI32::new(0),
            trade_sound_cooldown: AtomicI32::new(0),
            increase_profession_level_on_update: AtomicBool::new(false),
            last_traded_player: std::sync::Mutex::new(None),
            trading_player: std::sync::Mutex::new(None),
            is_trading: AtomicBool::new(false),
            sleeping_pos: std::sync::Mutex::new(None),
            self_weak: std::sync::Mutex::new(None),
        };
        let mob_arc = Arc::new(villager);
        *mob_arc
            .self_weak
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::downgrade(&mob_arc));
        // Send initial metadata
        let bedrock_metadata = Self::bedrock_metadata(villager_data, 0);
        mob_arc
            .get_entity()
            .set_synced_data(tracked_data::villager::VILLAGER_DATA, villager_data);
        mob_arc
            .get_entity()
            .send_bedrock_actor_data(&bedrock_metadata);

        mob_arc
    }

    fn can_add_item(&self, stack: &ItemStack) -> bool {
        let inventory = self
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inventory.len() < INVENTORY_SIZE
            || inventory.iter().any(|slot| {
                slot.is_empty()
                    || (slot.are_items_and_components_equal(stack)
                        && slot.item_count < slot.get_max_stack_size())
            })
    }

    pub fn count_food_points_in_inventory(&self) -> i32 {
        use pumpkin_data::data_component_impl::VillagerFoodImpl;

        let inventory = self
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut total = 0;
        for stack in inventory.iter() {
            if !stack.is_empty()
                && let Some(food) = stack.get_data_component::<VillagerFoodImpl>()
            {
                total += food.nutrition * i32::from(stack.item_count);
            }
        }
        total
    }

    pub fn eat_until_full(&self) {
        use pumpkin_data::data_component_impl::VillagerFoodImpl;

        if self.food_level.load(Ordering::Relaxed) >= BREEDING_FOOD_THRESHOLD {
            return;
        }
        let mut inventory = self
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for stack in inventory.iter_mut() {
            if !stack.is_empty() {
                let points = stack
                    .get_data_component::<VillagerFoodImpl>()
                    .map_or(0, |food| food.nutrition);
                if points > 0 {
                    while stack.item_count > 0
                        && self.food_level.load(Ordering::Relaxed) < BREEDING_FOOD_THRESHOLD
                    {
                        self.food_level.fetch_add(points, Ordering::Relaxed);
                        stack.item_count -= 1;
                    }
                    if stack.item_count == 0 {
                        *stack = ItemStack::EMPTY.clone();
                    }
                    if self.food_level.load(Ordering::Relaxed) >= BREEDING_FOOD_THRESHOLD {
                        break;
                    }
                }
            }
        }
    }

    #[must_use]
    pub fn get_villager_data_finalized(&self) -> bool {
        self.villager_data_finalized.load(Ordering::Relaxed)
    }

    pub fn set_villager_data_finalized(&self, finalized: bool) {
        self.villager_data_finalized
            .store(finalized, Ordering::Relaxed);
        self.get_entity()
            .set_synced_data(tracked_data::villager::VILLAGER_DATA_FINALIZED, finalized);
    }

    pub fn finalize_villager_type(&self, world: &World, pos: &BlockPos) {
        self.finalize_villager_type_in_biome(world.get_biome(pos));
    }

    pub fn finalize_villager_type_in_biome(&self, biome: &pumpkin_data::chunk::Biome) {
        if self.get_villager_data_finalized() {
            return;
        }
        let biome_type = breeding::villager_type_for_biome(biome.registry_id);
        let data = *self
            .villager_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.set_villager_data(VillagerData::new(
            biome_type,
            data.profession_enum(),
            data.level.0,
        ));
        self.set_villager_data_finalized(true);
    }

    pub fn set_villager_data(&self, data: VillagerData) {
        let old_profession = {
            let mut villager_data = self
                .villager_data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let old_profession = villager_data.profession;
            *villager_data = data;
            old_profession
        };
        let bedrock_metadata = Self::bedrock_metadata(data, self.xp.load(Ordering::Relaxed));
        self.get_entity()
            .set_synced_data(tracked_data::villager::VILLAGER_DATA, data);
        self.get_entity().send_bedrock_actor_data(&bedrock_metadata);

        if old_profession != data.profession {
            self.offers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clear();
        }
    }

    fn create_explorer_map(&self, destination: &str, given: &ItemStack) -> Option<ItemStack> {
        use pumpkin_data::data_component::DataComponent;
        use pumpkin_data::data_component_impl::{DataComponentImpl, ItemNameImpl, MapIdImpl};
        use pumpkin_data::structures::StructureSet;
        use pumpkin_world::generation::generator::structure_finder::find_nearest_structure_start;

        let (structure_set, structure, name, icon_type) = explorer_map_target(destination)?;

        let world = self.get_entity().world.load().clone();
        let generator = world.level.world_gen();
        let (target, _) = find_nearest_structure_start(
            self.get_entity().block_pos.load(),
            StructureSet::get(structure_set)?,
            &[structure],
            100,
            &generator,
        )?;
        let server = world.server.upgrade()?;
        let map_id = server.next_map_id();
        let map = server.map_manager.create_map(
            map_id,
            world.dimension.clone(),
            target.0.x,
            target.0.z,
            2,
        );
        map.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .decorations
            .push(crate::world::map::MapDecoration {
                icon_type,
                x: 0,
                z: 0,
                direction: 8,
                display_name: None,
            });

        // 26.3 trades hand out a dedicated map item that already carries its own
        // name; older data trades a plain filled map that has to be named here.
        let mut stack = if given.is_empty() || given.item.id == Item::FILLED_MAP.id {
            let mut stack = ItemStack::new(1, &Item::FILLED_MAP);
            stack.patch.push((
                DataComponent::ItemName,
                Some(ItemNameImpl { name: name.into() }.to_dyn()),
            ));
            stack
        } else {
            given.clone()
        };
        stack.patch.push((
            DataComponent::MapId,
            Some(MapIdImpl { id: map_id }.to_dyn()),
        ));
        Some(stack)
    }

    #[allow(clippy::too_many_lines)]
    pub fn add_trades(&self, profession: VillagerProfession, level: i32) {
        use crate::data::datapack::trade_loader::{DynamicTradeModifier, DynamicVillagerTradeSet};
        use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
        use rand::seq::IndexedRandom;
        use rand::{RngExt, SeedableRng, rngs::StdRng};
        use std::borrow::Cow;

        let villager_type = self
            .villager_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .type_enum();
        let mut new_offers = Vec::new();

        let trade_set = self
            .get_entity()
            .world
            .load()
            .server
            .upgrade()
            .and_then(|server| {
                server
                    .datapack_manager
                    .get_villager_trade_set(profession.to_name(), level)
            })
            .or_else(|| {
                profession
                    .trade_set(level)
                    .map(DynamicVillagerTradeSet::from)
            });

        if let Some(trade_set) = trade_set {
            let mut rng = StdRng::from_rng(&mut rand::rng());
            let mut remaining_trades = trade_set.trades.clone();
            let mut added = 0;
            while added < trade_set.amount && !remaining_trades.is_empty() {
                let index = rng.random_range(0..remaining_trades.len());
                let trade = remaining_trades.remove(index);
                if !trade.allowed_types.is_empty() && !trade.allowed_types.contains(&villager_type)
                {
                    continue;
                }
                let mut base_cost_a = ItemStack::new(trade.wants.count as u8, trade.wants.item);
                let mut output = ItemStack::new(trade.gives.count as u8, trade.gives.item);
                let mut cost_b = trade
                    .wants_b
                    .as_ref()
                    .map(|b| ItemStack::new(b.count as u8, b.item));

                match &trade.modifier {
                    DynamicTradeModifier::None => {}
                    DynamicTradeModifier::EnchantRandomly => {
                        let Some(items) = enchanted_book_offer_items(&mut rng) else {
                            continue;
                        };
                        (base_cost_a, output, cost_b) = items;
                    }
                    DynamicTradeModifier::EnchantWithLevels { min, max } => {
                        let Some((enchanted, additional_cost)) =
                            enchant_trade_item(&mut rng, trade.gives.item, *min, *max)
                        else {
                            continue;
                        };
                        output = enchanted;
                        let count = i32::from(base_cost_a.item_count)
                            .saturating_add(additional_cost)
                            .clamp(0, i32::from(base_cost_a.get_max_stack_size()));
                        if count == 0 {
                            continue;
                        }
                        base_cost_a.set_count(count as u8);
                    }
                    DynamicTradeModifier::ExplorationMap { destination } => {
                        let Some(map) = self.create_explorer_map(destination, &output) else {
                            continue;
                        };
                        output = map;
                    }
                    DynamicTradeModifier::RandomDyes => apply_random_dye(&mut rng, &mut output),
                    DynamicTradeModifier::RandomPotion => {
                        let Some(potion_name) = pumpkin_data::tag::Potion::MINECRAFT_TRADEABLE
                            .0
                            .choose(&mut rng)
                        else {
                            continue;
                        };
                        apply_potion(&mut output, potion_name);
                    }
                    DynamicTradeModifier::SuspiciousStew(effects) => {
                        apply_stew_effect(&mut rng, &mut output, effects);
                    }
                    DynamicTradeModifier::Potion(potion) => apply_potion(&mut output, potion),
                }
                new_offers.push(pumpkin_protocol::java::client::play::MerchantOffer {
                    base_cost_a: ItemStackSerializer(Cow::Owned(base_cost_a)),
                    output: ItemStackSerializer(Cow::Owned(output)),
                    cost_b: cost_b.map(|stack| ItemStackSerializer(Cow::Owned(stack))),
                    reward_exp: true,
                    uses: 0,
                    max_uses: trade.max_uses,
                    xp: trade.xp,
                    special_price: 0,
                    price_multiplier: trade.price_multiplier,
                    demand: 0,
                });
                added += 1;
            }
        }

        self.offers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .extend(new_offers);
    }

    pub fn generate_trades(&self, profession: VillagerProfession, level: i32) {
        self.offers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        self.add_trades(profession, level);
    }

    #[must_use]
    pub fn get_player_reputation(&self, player: &Player) -> i32 {
        self.gossips
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_reputation(player.get_entity().entity_uuid, |_| true)
    }

    fn update_special_prices(&self, player: &Player) {
        let reputation = self.get_player_reputation(player);
        let hero_amplifier = player
            .living_entity
            .get_effect(&StatusEffect::HERO_OF_THE_VILLAGE)
            .map(|effect| i32::from(effect.amplifier));

        let mut offers = self
            .offers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for offer in offers.iter_mut() {
            offer.special_price = -((reputation as f32 * offer.price_multiplier).floor() as i32);
            if let Some(amplifier) = hero_amplifier {
                let discount = ((0.3 + 0.0625 * f64::from(amplifier))
                    * f64::from(offer.base_cost_a.0.item_count))
                .floor() as i32;
                offer.special_price -= discount.max(1);
            }
        }
    }

    fn reset_special_prices(&self) {
        for offer in self
            .offers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter_mut()
        {
            offer.special_price = 0;
        }
    }

    fn can_continue_trading(
        &self,
        inventory_player: &dyn InventoryPlayer,
        player_uuid: Uuid,
        sync_id: u8,
    ) -> bool {
        let Some(player) = inventory_player.as_any().downcast_ref::<Player>() else {
            return false;
        };
        let entity = self.get_entity();
        let range = player
            .living_entity
            .get_attribute_value(&Attributes::ENTITY_INTERACTION_RANGE)
            + 4.0;
        entity.is_alive()
            && self.mob_entity.living_entity.health.load() > 0.0
            && self
                .trading_player
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .as_ref()
                .is_some_and(|(uuid, id)| *uuid == player_uuid && *id == sync_id)
            && entity
                .bounding_box
                .load()
                .squared_magnitude(player.eye_position())
                < range * range
    }

    fn complete_trade(&self, offer_index: usize, world: &Arc<World>, player_uuid: Uuid) {
        let (xp_gain, reward_exp) = {
            let mut offers = self
                .offers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(offer) = offers.get_mut(offer_index) else {
                return;
            };
            offer.uses += 1;
            (offer.xp, offer.reward_exp)
        };

        let mut pop_xp = 3 + self.get_random().random_range(0..4);
        let current_xp = self.xp.fetch_add(xp_gain, Ordering::Relaxed) + xp_gain;
        let villager_data = *self
            .villager_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let bedrock_metadata = Self::bedrock_metadata(villager_data, current_xp);
        self.get_entity()
            .set_synced_data(tracked_data::villager::VILLAGER_DATA, villager_data);
        self.get_entity().send_bedrock_actor_data(&bedrock_metadata);

        *self
            .last_traded_player
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(player_uuid);

        if can_level_up(villager_data.level.0)
            && current_xp >= get_max_xp_per_level(villager_data.level.0)
        {
            self.increase_profession_level_on_update
                .store(true, Ordering::Relaxed);
            pop_xp += 5;
        }

        if reward_exp {
            ExperienceOrbEntity::spawn(
                world,
                self.get_entity().pos.load().add_raw(0.0, 0.5, 0.0),
                pop_xp as u32,
            );
        }

        if let Some(player) = world.get_player_by_uuid(player_uuid) {
            self.resend_offers_to_player(&player);
        }
    }

    fn resend_offers_to_player(&self, player: &Arc<Player>) {
        let trading_player = *self
            .trading_player
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some((player_uuid, sync_id)) = trading_player else {
            return;
        };
        if player.get_entity().entity_uuid != player_uuid {
            return;
        }
        let offers = self
            .offers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let villager_data = *self
            .villager_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.send_trade_offers(player, sync_id, &offers, villager_data);
    }

    fn resend_offers_to_trading_player(&self) {
        let trading_player = *self
            .trading_player
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some((player_uuid, sync_id)) = trading_player else {
            return;
        };
        let world = self.get_entity().world.load();
        let Some(player) = world.get_player_by_uuid(player_uuid) else {
            return;
        };
        let offers = self
            .offers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let villager_data = *self
            .villager_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let ok = {
            let screen = player
                .current_screen_handler
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            let mut screen = screen
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if screen.sync_id() != sync_id {
                false
            } else if let Some(handler) =
                screen.as_any_mut().downcast_mut::<MerchantScreenHandler>()
            {
                handler.offers.clone_from(&offers);
                handler.update_result_slot();
                true
            } else {
                false
            }
        };
        if !ok {
            return;
        }
        self.send_trade_offers(&player, sync_id, &offers, villager_data);
    }

    fn notify_trading_player_offers_updated(&self) {
        if self
            .trading_player
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
            && let Some(villager) = self
                .self_weak
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .as_ref()
                .and_then(Weak::upgrade)
        {
            villager.resend_offers_to_trading_player();
        }
    }

    pub fn should_restock(&self, game_time: i64, day: i64) -> bool {
        let half_day_passed_time = self.last_restock_time.load(Ordering::Relaxed) + 12_000;
        let mut is_new_day = game_time > half_day_passed_time;
        let last_check_day = self.last_restock_check_day.swap(day, Ordering::Relaxed);
        is_new_day |= last_check_day > 0 && day > last_check_day;
        if is_new_day {
            self.last_restock_time.store(game_time, Ordering::Relaxed);
            self.reset_number_of_restocks();
        }
        self.allowed_to_restock(game_time) && self.needs_to_restock()
    }

    pub fn restock(&self, game_time: i64) {
        self.update_demand();
        {
            let mut offers = self
                .offers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for offer in offers.iter_mut() {
                offer.reset_uses();
            }
        }
        self.notify_trading_player_offers_updated();
        self.last_restock_time.store(game_time, Ordering::Relaxed);
        self.restocks_today.fetch_add(1, Ordering::Relaxed);
    }

    pub fn play_celebrate_sound(&self) {
        self.get_entity()
            .play_sound(pumpkin_data::sound::Sound::EntityVillagerCelebrate);
    }

    pub fn play_work_sound(&self) {
        let profession = self
            .villager_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .profession_enum();
        if let Some(sound) = profession.work_sound() {
            self.get_entity().play_sound(sound);
        }
    }

    pub fn has_farm_seeds(&self) -> bool {
        let inventory = self
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inventory.iter().any(|stack| {
            stack
                .item
                .has_tag(&pumpkin_data::tag::Item::MINECRAFT_VILLAGER_PLANTABLE_SEEDS)
        })
    }

    pub fn inventory_count(&self, item: &'static Item) -> i32 {
        let inventory = self
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        inventory
            .iter()
            .filter(|stack| stack.item == item)
            .map(|stack| i32::from(stack.item_count))
            .sum()
    }

    fn reset_number_of_restocks(&self) {
        self.catch_up_demand();
        self.restocks_today.store(0, Ordering::Relaxed);
    }

    fn catch_up_demand(&self) {
        let missed_updates = 2 - self.restocks_today.load(Ordering::Relaxed);
        if missed_updates > 0 {
            let mut offers = self
                .offers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for offer in offers.iter_mut() {
                offer.reset_uses();
            }
        }
        for _ in 0..missed_updates {
            self.update_demand();
        }
        self.notify_trading_player_offers_updated();
    }

    fn update_demand(&self) {
        let mut offers = self
            .offers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for offer in offers.iter_mut() {
            offer.update_demand();
        }
    }

    fn needs_to_restock(&self) -> bool {
        self.offers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .any(pumpkin_protocol::java::client::play::MerchantOffer::needs_restock)
    }

    fn allowed_to_restock(&self, game_time: i64) -> bool {
        let restocks_today = self.restocks_today.load(Ordering::Relaxed);
        restocks_today == 0
            || (restocks_today < 2
                && game_time > self.last_restock_time.load(Ordering::Relaxed) + 2_400)
    }

    fn increase_merchant_career(&self) {
        let data = *self
            .villager_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.set_villager_data(VillagerData::new(
            data.type_enum(),
            data.profession_enum(),
            data.level.0 + 1,
        ));
        self.update_trades();
    }

    fn update_trades(&self) {
        let data = *self
            .villager_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.add_trades(data.profession_enum(), data.level.0);
        self.refresh_trading_player_prices(None);
    }

    pub fn set_unhappy(&self) {
        let entity = self.get_entity();
        self.unhappy_counter.store(40, Ordering::Relaxed);
        entity.set_synced_data(tracked_data::villager::UNHAPPY_COUNTER, VarInt(40));
        entity.play_sound(pumpkin_data::sound::Sound::EntityVillagerNo);
    }

    pub fn open_trading_screen(&self, player: &Arc<Player>) {
        // Open the merchant screen and then send the current offers packet
        if let Some(sync_id) = player.open_handled_screen(self, None) {
            let offers = self
                .offers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            let villager_data = *self
                .villager_data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            self.send_trade_offers(player, sync_id, &offers, villager_data);
        }
    }

    fn bedrock_trade_item(stack: &ItemStack, count: u8) -> NbtCompound {
        let mut item = NbtCompound::new();
        if stack.is_empty() {
            return item;
        }
        let Some(mapping) = JavaToBedrockItemMapping::from_java_item_id(stack.item.id) else {
            return item;
        };
        item.put_byte("Count", count as i8);
        item.put_short("Damage", mapping.bedrock_data as i16);
        item.put_string("Name", mapping.bedrock_item.registry_key.to_owned());
        item
    }

    fn bedrock_trade_data(
        offers: &[pumpkin_protocol::java::client::play::MerchantOffer],
        level: i32,
    ) -> NbtCompound {
        use pumpkin_nbt::tag::NbtTag;

        let tier = level.saturating_sub(1).max(0);
        let mut recipes = Vec::with_capacity(offers.len() + usize::from(level < 5));
        for (index, offer) in offers.iter().enumerate() {
            let base_cost = &offer.base_cost_a.0;
            let demand_bonus = (i32::from(base_cost.item_count).saturating_mul(offer.demand) as f32
                * offer.price_multiplier)
                .floor()
                .max(0.0) as i32;
            let adjusted_count = i32::from(base_cost.item_count)
                .saturating_add(demand_bonus)
                .saturating_add(offer.special_price)
                .clamp(1, i32::from(base_cost.get_max_stack_size()))
                as u8;

            let mut recipe = NbtCompound::new();
            recipe.put_int("netId", index as i32 + 1);
            recipe.put_int(
                "maxUses",
                if offer.is_out_of_stock() {
                    0
                } else {
                    offer.max_uses
                },
            );
            recipe.put_int("traderExp", offer.xp);
            recipe.put_float("priceMultiplierA", offer.price_multiplier);
            recipe.put_float("priceMultiplierB", 0.0);
            recipe.put_compound(
                "sell",
                Self::bedrock_trade_item(&offer.output.0, offer.output.0.item_count),
            );
            recipe.put_int("buyCountA", i32::from(base_cost.item_count));
            recipe.put_int(
                "buyCountB",
                offer
                    .cost_b
                    .as_ref()
                    .map_or(0, |cost| i32::from(cost.0.item_count)),
            );
            recipe.put_int("demand", offer.demand);
            recipe.put_int("tier", (index as i32 / 2).min(tier));
            recipe.put_compound("buyA", Self::bedrock_trade_item(base_cost, adjusted_count));
            recipe.put_compound(
                "buyB",
                offer.cost_b.as_ref().map_or_else(NbtCompound::new, |cost| {
                    Self::bedrock_trade_item(&cost.0, cost.0.item_count)
                }),
            );
            recipe.put_int("uses", offer.uses);
            recipe.put_byte("rewardExp", i8::from(offer.reward_exp));
            recipes.push(NbtTag::Compound(recipe));
        }

        // Bedrock uses this hidden next-tier entry to render the villager XP bar.
        if level < 5 {
            let mut recipe = NbtCompound::new();
            recipe.put_int("maxUses", 0);
            recipe.put_int("traderExp", 0);
            recipe.put_float("priceMultiplierA", 0.0);
            recipe.put_float("priceMultiplierB", 0.0);
            recipe.put_int("buyCountA", 0);
            recipe.put_int("buyCountB", 0);
            recipe.put_int("demand", 0);
            recipe.put_int("tier", 5);
            recipe.put_int("uses", 0);
            recipe.put_byte("rewardExp", 0);
            recipes.push(NbtTag::Compound(recipe));
        }

        let mut data = NbtCompound::new();
        data.put_list("Recipes", recipes);
        data.put_list(
            "TierExpRequirements",
            [0, 10, 70, 150, 250]
                .into_iter()
                .enumerate()
                .map(|(tier, xp)| {
                    let mut requirement = NbtCompound::new();
                    requirement.put_int(&tier.to_string(), xp);
                    NbtTag::Compound(requirement)
                })
                .collect(),
        );
        data
    }

    fn send_trade_offers(
        &self,
        player: &Player,
        sync_id: u8,
        offers: &[pumpkin_protocol::java::client::play::MerchantOffer],
        villager_data: VillagerData,
    ) {
        use pumpkin_protocol::{bedrock::client::CUpdateTrade, codec::var_long::VarLong};

        let java = CMerchantOffers::new(
            VarInt(i32::from(sync_id)),
            offers.to_owned(),
            villager_data.level,
            VarInt(self.xp.load(Ordering::Relaxed)),
            true,
            true,
        );
        let bedrock = CUpdateTrade {
            container_id: sync_id,
            r#type: 15,
            size: VarInt(0),
            trader_tier: VarInt(villager_data.level.0.saturating_sub(1)),
            entity_unique_id: VarLong(i64::from(self.get_entity().entity_id)),
            last_trading_player: VarLong(i64::from(player.entity_id())),
            display_name: ScreenHandlerFactory::get_display_name(self).to_pretty_console(),
            use_new_trade_screen: true,
            using_economy_trade: true,
            data: Self::bedrock_trade_data(offers, villager_data.level.0),
        };
        player.client.try_enqueue_packet_editioned(&java, &bedrock);
    }
}

impl ScreenHandlerFactory for VillagerEntity {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<pumpkin_inventory::player::player_inventory::PlayerInventory>,
        player: &dyn InventoryPlayer,
    ) -> Option<SharedScreenHandler> {
        let self_weak = self
            .self_weak
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()?;
        let server_player = player.as_any().downcast_ref::<Player>();
        let player_uuid =
            server_player.map_or_else(uuid::Uuid::nil, |p| p.get_entity().entity_uuid);
        if let Some(player) = server_player {
            self.update_special_prices(player);
        }
        let offers = self
            .offers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let world = self.get_entity().world.load().clone();

        let mut handler = MerchantScreenHandler::new(
            sync_id,
            player_inventory,
            self.merchant_inventory.clone(),
            offers,
        );

        self.is_trading.store(true, Ordering::Relaxed);
        *self
            .trading_player
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some((player_uuid, sync_id));
        let validity_weak = self_weak.clone();
        handler.validity_check = Some(Box::new(move |inventory_player| {
            validity_weak.upgrade().is_some_and(|villager| {
                villager.can_continue_trading(inventory_player, player_uuid, sync_id)
            })
        }));
        let update_weak = self_weak.clone();
        handler.on_trade_updated = Some(Box::new(move |has_result| {
            let Some(villager) = update_weak.upgrade() else {
                return;
            };
            if villager
                .trade_sound_cooldown
                .compare_exchange(0, 20, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                villager.get_entity().play_sound(if has_result {
                    pumpkin_data::sound::Sound::EntityVillagerYes
                } else {
                    pumpkin_data::sound::Sound::EntityVillagerNo
                });
            }
        }));
        let close_weak = self_weak.clone();
        handler.on_close = Some(Box::new(move || {
            if let Some(villager) = close_weak.upgrade() {
                villager.is_trading.store(false, Ordering::Relaxed);
                *villager
                    .trading_player
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
                villager.reset_special_prices();
            }
        }));

        handler.on_trade = Some(Box::new(move |offer_index| {
            if let Some(villager) = self_weak.upgrade() {
                villager.complete_trade(offer_index, &world, player_uuid);
            }
        }));

        Some(Arc::new(std::sync::Mutex::new(handler)) as SharedScreenHandler)
    }

    fn get_display_name(&self) -> TextComponent {
        let profession = self
            .villager_data
            .try_lock()
            .map_or(VillagerProfession::None, |data| data.profession_enum());
        TextComponent::translate(profession.translation_key(), [])
    }
}

impl VillagerEntity {
    pub fn villager_mob_tick(&self) {
        let world = self.get_entity().world.load();

        let unhappy_counter = self.unhappy_counter.load(Ordering::Relaxed);
        if unhappy_counter > 0 {
            let unhappy_counter = unhappy_counter - 1;
            self.unhappy_counter
                .store(unhappy_counter, Ordering::Relaxed);
            self.get_entity().set_synced_data(
                tracked_data::villager::UNHAPPY_COUNTER,
                VarInt(unhappy_counter),
            );
        }
        self.trade_sound_cooldown
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |cooldown| {
                (cooldown > 0).then_some(cooldown - 1)
            })
            .ok();

        let game_time = world
            .level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .world_age;
        self.maybe_decay_gossip(game_time);

        if self.is_sleeping() && !self.check_bed_exists() {
            self.wake_up();
        }
    }

    fn villager_ai_step(&self) {
        let world = self.get_entity().world.load();

        if self
            .increase_profession_level_on_update
            .swap(false, Ordering::Relaxed)
        {
            self.increase_merchant_career();
            self.mob_entity.living_entity.add_effect(Effect {
                effect_type: &StatusEffect::REGENERATION,
                duration: 200,
                amplifier: 0,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: false,
            });
        }

        let last_traded_player = self
            .last_traded_player
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(player_uuid) = last_traded_player {
            self.on_reputation_event_from(ReputationEventType::Trade, player_uuid);
            world.send_entity_status(
                self.get_entity(),
                pumpkin_data::entity::EntityStatus::VillagerHappy,
                ActorEventID::VillagerHappy,
            );
        }

        if !self.mob_entity.is_no_ai() && self.get_random().random_range(0..100) == 0 {
            let pos = self.get_entity().block_pos.load();
            let raid_in_progress = world
                .raids
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get_raid_at(&pos)
                .is_some_and(|raid| raid.is_active() && !raid.is_over());
            if raid_in_progress {
                world.send_entity_status(
                    self.get_entity(),
                    pumpkin_data::entity::EntityStatus::VillagerSweat,
                    ActorEventID::None,
                );
            }
        }
    }
}

fn add_to_inventory(inventory: &mut Vec<ItemStack>, stack: &mut ItemStack) {
    for slot in inventory.iter_mut() {
        if stack.is_empty() {
            return;
        }
        if !slot.is_empty() && slot.are_items_and_components_equal(stack) {
            let count = stack
                .item_count
                .min(slot.get_max_stack_size().saturating_sub(slot.item_count));
            stack.decrement(count);
            slot.increment(count);
        }
    }
    for slot in inventory.iter_mut() {
        if stack.is_empty() {
            return;
        }
        if slot.is_empty() {
            *slot = std::mem::replace(stack, ItemStack::EMPTY.clone());
        }
    }
    if !stack.is_empty() && inventory.len() < INVENTORY_SIZE {
        inventory.push(std::mem::replace(stack, ItemStack::EMPTY.clone()));
    }
}

impl AgeableMob for VillagerEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }

    fn age_boundary_reached(&self) {
        self.mob_entity.request_refresh_brain();
    }
}

impl Mob for VillagerEntity {
    fn finalize_spawn_in_biome(&self, biome: &'static pumpkin_data::chunk::Biome) {
        self.finalize_villager_type_in_biome(biome);
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        reason: crate::entity::mob::spawn::EntitySpawnReason,
        group_data: Option<crate::entity::mob::spawn::SpawnGroupData>,
    ) -> Option<crate::entity::mob::spawn::SpawnGroupData> {
        if reason == crate::entity::mob::spawn::EntitySpawnReason::Breeding {
            let data = *self
                .villager_data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            self.set_villager_data(VillagerData::new(
                data.type_enum(),
                VillagerProfession::None,
                data.level.0,
            ));
        }
        self.finalize_villager_type(world, &self.get_entity().block_pos.load());
        self.default_finalize_spawn(world, group_data)
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn wants_to_pick_up(&self, stack: &ItemStack) -> bool {
        let profession = self
            .villager_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .profession_enum();
        (stack
            .item
            .has_tag(&pumpkin_data::tag::Item::MINECRAFT_VILLAGER_PICKS_UP)
            || stack
                .get_data_component::<pumpkin_data::data_component_impl::VillagerFoodImpl>()
                .is_some()
            || profession
                .requested_items()
                .iter()
                .any(|item| item.id == stack.item.id))
            && self.can_add_item(stack)
    }

    fn pick_up_item(&self, item: &crate::entity::item::ItemEntity) {
        let stack = item
            .get_item_stack()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        if !self.wants_to_pick_up(&stack) {
            return;
        }
        let mut remainder = stack.clone();
        add_to_inventory(
            &mut self
                .inventory
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
            &mut remainder,
        );
        let taken = stack.item_count - remainder.item_count;
        let living = &self.mob_entity.living_entity;
        if taken == 0 || !living.pickup(item.get_entity(), u32::from(taken)) {
            return;
        }
        let mut remainder = stack;
        add_to_inventory(
            &mut self
                .inventory
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            &mut remainder,
        );
        living.on_item_pickup(item);
        if remainder.is_empty() {
            item.get_entity().remove();
        } else {
            *item
                .get_item_stack()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = remainder;
            item.init_data_tracker();
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        {
            let data = self
                .villager_data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            nbt.put_compound("VillagerData", data.write_nbt());
        };

        nbt.put_bool("VillagerDataFinalized", self.get_villager_data_finalized());
        nbt.put_int("FoodLevel", self.food_level.load(Ordering::Relaxed));
        nbt.put_int("Xp", self.xp.load(Ordering::Relaxed));
        nbt.put_long(
            "LastRestock",
            self.last_restock_time.load(Ordering::Relaxed),
        );
        nbt.put_int("RestocksToday", self.restocks_today.load(Ordering::Relaxed));
        nbt.put_long(
            "LastGossipDecay",
            self.last_gossip_decay_time.load(Ordering::Relaxed),
        );

        let last_gossip_share = self.last_gossip_share_time.load(Ordering::Relaxed);
        if last_gossip_share > 0 {
            nbt.put_long("LastGossipShare", last_gossip_share);
        }

        let offers = self
            .offers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !offers.is_empty() {
            let mut recipes = Vec::new();
            for offer in offers.iter() {
                let mut recipe = NbtCompound::new();
                let mut buy = NbtCompound::new();
                let mut sell = NbtCompound::new();

                let item_stack: &ItemStack = offer.base_cost_a.0.as_ref();
                item_stack.write_item_stack(&mut buy);
                recipe.put_compound("buy", buy);

                let item_stack: &ItemStack = offer.output.0.as_ref();
                item_stack.write_item_stack(&mut sell);
                recipe.put_compound("sell", sell);

                if let Some(cost_b) = &offer.cost_b {
                    let mut buy_b = NbtCompound::new();
                    let item_stack: &ItemStack = cost_b.0.as_ref();
                    item_stack.write_item_stack(&mut buy_b);
                    recipe.put_compound("buyB", buy_b);
                }

                recipe.put_int("uses", offer.uses);
                recipe.put_int("maxUses", offer.max_uses);
                recipe.put_bool("rewardExp", offer.reward_exp);
                recipe.put_int("xp", offer.xp);
                recipe.put_float("priceMultiplier", offer.price_multiplier);
                recipe.put_int("specialPrice", offer.special_price);
                recipe.put_int("demand", offer.demand);

                recipes.push(NbtTag::Compound(recipe));
            }
            let mut offers_compound = NbtCompound::new();
            offers_compound.put("Recipes", NbtTag::List(recipes));
            nbt.put_compound("Offers", offers_compound);
        }

        let inventory = self
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !inventory.is_empty() {
            let mut inventory_list = Vec::new();
            for item in inventory.iter() {
                let mut item_compound = NbtCompound::new();
                item.write_item_stack(&mut item_compound);
                inventory_list.push(NbtTag::Compound(item_compound));
            }
            nbt.put("Inventory", NbtTag::List(inventory_list));
        }

        let gossip_entries = self
            .gossips
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entries();
        if !gossip_entries.is_empty() {
            let mut gossip_list = Vec::new();
            for (uuid, gossip_type, val) in gossip_entries {
                let mut gossip_nbt = NbtCompound::new();
                let (u1, u2) = uuid.as_u64_pair();
                let uuid_array = vec![(u1 >> 32) as i32, u1 as i32, (u2 >> 32) as i32, u2 as i32];
                gossip_nbt.put("Target", NbtTag::IntArray(uuid_array));
                gossip_nbt.put_string("Type", gossip_type.name().to_string());
                gossip_nbt.put_int("Value", val);
                gossip_list.push(NbtTag::Compound(gossip_nbt));
            }
            nbt.put("Gossips", NbtTag::List(gossip_list));
        }
    }

    #[allow(clippy::too_many_lines)]
    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if nbt.get_bool("VillagerDataFinalized").unwrap_or(false)
            || nbt.get_compound("VillagerData").is_some()
        {
            self.set_villager_data_finalized(true);
        }
        if let Some(villager_data_nbt) = nbt.get_compound("VillagerData") {
            let mut data = self
                .villager_data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *data = VillagerData::read_nbt(villager_data_nbt, *data);
        }

        if let Some(food) = nbt.get_int("FoodLevel") {
            self.food_level.store(food, Ordering::Relaxed);
        }
        if let Some(xp) = nbt.get_int("Xp") {
            self.xp.store(xp, Ordering::Relaxed);
        }
        if let Some(restock) = nbt.get_long("LastRestock") {
            self.last_restock_time.store(restock, Ordering::Relaxed);
        }
        if let Some(today) = nbt.get_int("RestocksToday") {
            self.restocks_today.store(today, Ordering::Relaxed);
        }
        if let Some(last_decay) = nbt.get_long("LastGossipDecay") {
            self.last_gossip_decay_time
                .store(last_decay, Ordering::Relaxed);
        }

        if let Some(offers_compound) = nbt.get_compound("Offers")
            && let Some(recipes) = offers_compound.get_list("Recipes")
        {
            let mut offers = self
                .offers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            offers.clear();
            for tag in recipes {
                if let Some(recipe) = tag.extract_compound() {
                    let buy = recipe
                        .get_compound("buy")
                        .and_then(ItemStack::read_item_stack);
                    let buy_b = recipe
                        .get_compound("buyB")
                        .and_then(ItemStack::read_item_stack);
                    let sell_item = recipe
                        .get_compound("sell")
                        .and_then(ItemStack::read_item_stack);

                    if let (Some(buy), Some(sell_item)) = (buy, sell_item)
                        && !buy.is_empty()
                        && !sell_item.is_empty()
                        && buy_b.as_ref().is_none_or(|stack| !stack.is_empty())
                    {
                        let uses = recipe.get_int("uses").unwrap_or(0);
                        let max_uses = recipe.get_int("maxUses").unwrap_or(12);
                        let reward_exp = recipe.get_bool("rewardExp").unwrap_or(true);
                        let xp = recipe.get_int("xp").unwrap_or(2);
                        let price_multiplier = recipe.get_float("priceMultiplier").unwrap_or(0.05);
                        let special_price = recipe.get_int("specialPrice").unwrap_or(0);
                        let demand = recipe.get_int("demand").unwrap_or(0);

                        offers.push(pumpkin_protocol::java::client::play::MerchantOffer {
                            base_cost_a: buy.into(),
                            output: sell_item.into(),
                            cost_b: buy_b.map(Into::into),
                            reward_exp,
                            uses,
                            max_uses,
                            xp,
                            special_price,
                            price_multiplier,
                            demand,
                        });
                    }
                }
            }
        }

        // Inventory
        if let Some(inventory_list) = nbt.get_list("Inventory") {
            let mut inventory = self
                .inventory
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            inventory.clear();
            for tag in inventory_list {
                if let Some(item_compound) = tag.extract_compound()
                    && let Some(stack) = ItemStack::read_item_stack(item_compound)
                {
                    inventory.push(stack);
                }
            }
        }

        if let Some(gossip_list) = nbt.get_list("Gossips") {
            let mut entries = Vec::new();
            for tag in gossip_list {
                if let Some(gossip_nbt) = tag.extract_compound() {
                    let uuid = gossip_nbt.get_int_array("Target").and_then(|uuid_array| {
                        let [a, b, c, d] = uuid_array else {
                            return None;
                        };
                        Some(Uuid::from_u128(
                            (*a as u32 as u128) << 96
                                | (*b as u32 as u128) << 64
                                | (*c as u32 as u128) << 32
                                | (*d as u32 as u128),
                        ))
                    });
                    let gossip_type = gossip_nbt
                        .get_string("Type")
                        .and_then(GossipType::from_name)
                        .or_else(|| {
                            gossip_nbt
                                .get_int("Type")
                                .and_then(GossipType::from_legacy_id)
                        });
                    if let (Some(uuid), Some(gossip_type), Some(val)) =
                        (uuid, gossip_type, gossip_nbt.get_int("Value"))
                    {
                        entries.push((uuid, gossip_type, val));
                    }
                }
            }
            *self
                .gossips
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                GossipContainer::from_entries(entries);
        }
        self.mob_entity.request_refresh_brain();
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_bedrock_identifier(&self) -> Option<&'static str> {
        Some("minecraft:villager_v2")
    }

    fn mob_metadata(&self) -> Option<crate::entity::mob::MobMetadata> {
        let data = *self
            .villager_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let xp = self.xp.load(Ordering::Relaxed);
        let mut meta = crate::entity::mob::MobMetadata::new().villager_data(data, xp);
        if self.get_entity().age.load(Ordering::Relaxed) < 0 {
            meta = meta.baby(true);
        }
        Some(meta)
    }

    fn clear_trading_player(&self) {
        *self
            .trading_player
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }

    fn get_trading_player(&self) -> Option<Arc<Player>> {
        let trading_player = *self
            .trading_player
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (player_uuid, _) = trading_player?;
        self.get_entity()
            .world
            .load()
            .get_player_by_uuid(player_uuid)
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let data = *self
            .villager_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let bedrock_metadata = Self::bedrock_metadata(data, self.xp.load(Ordering::Relaxed));
        entity.set_synced_data(tracked_data::villager::VILLAGER_DATA, data);
        entity.send_bedrock_actor_data(&bedrock_metadata);
        if entity.age.load(Ordering::Relaxed) < 0 {
            entity.set_synced_data(tracked_data::villager::BABY_ID, true);
        }
    }

    fn on_damage_with_context(
        &self,
        _damage_type: pumpkin_data::damage::DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        if self.is_sleeping() {
            self.wake_up();
        }
        if let Some(hurt_by) = cause
            .or(source)
            .filter(|entity| entity.get_living_entity().is_some())
        {
            self.on_reputation_event_from(
                ReputationEventType::VillagerHurt,
                hurt_by.get_entity().entity_uuid,
            );
            if !self.mob_entity.living_entity.is_dead_or_dying()
                && hurt_by.get_entity().entity_type == &pumpkin_data::entity::EntityType::PLAYER
            {
                self.get_entity().world.load().send_entity_status(
                    self.get_entity(),
                    pumpkin_data::entity::EntityStatus::VillagerAngry,
                    ActorEventID::VillagerAngry,
                );
            }
        }
        true
    }

    fn mob_on_lightning_strike(
        &self,
        caller: &dyn EntityBase,
        lightning: &crate::entity::lightning::LightningBoltEntity,
    ) {
        let world = self.get_entity().world.load_full();
        let converted = (world.level_info.load().difficulty != pumpkin_util::Difficulty::Peaceful)
            .then(|| {
                convert_to(
                    self,
                    &pumpkin_data::entity::EntityType::WITCH,
                    ConversionParams::single(false, false),
                    |witch| {
                        witch.finalize_spawn(&world, EntitySpawnReason::Conversion, None);
                        witch
                            .get_mob_entity()
                            .persistence_required
                            .store(true, Ordering::Relaxed);
                        self.release_all_pois();
                    },
                )
            })
            .flatten();
        if converted.is_none() {
            self.mob_entity
                .living_entity
                .on_lightning_strike(caller, lightning);
        }
    }

    fn get_ambient_sound(&self) -> Option<pumpkin_data::sound::Sound> {
        if self.is_sleeping() {
            None
        } else if self.is_trading.load(Ordering::Relaxed) {
            Some(pumpkin_data::sound::Sound::EntityVillagerTrade)
        } else {
            Some(pumpkin_data::sound::Sound::EntityVillagerAmbient)
        }
    }

    fn on_die(&self, source: Option<&dyn EntityBase>) {
        if let Some(murderer) = source {
            self.tell_witnesses_that_i_was_murdered(murderer);
        }
        self.release_all_pois();
        if self.is_sleeping() {
            self.stop_sleeping();
        }
    }

    fn make_brain(&self, packed: &PackedMemories) -> Brain {
        ai::make_brain(self, packed)
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        self.mob_entity.tick_brain(self);
        self.villager_ai_step();
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        self.villager_mob_tick();
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if item_stack.item == &Item::VILLAGER_SPAWN_EGG
            || self.mob_entity.living_entity.is_dead_or_dying()
            || self.is_trading.load(Ordering::Relaxed)
            || self.is_sleeping()
        {
            return false;
        }
        if self.get_entity().age.load(Ordering::Relaxed) < 0 {
            self.set_unhappy();
            return true;
        }

        let trade_params = {
            let offers = self
                .offers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if offers.is_empty() {
                let data = self
                    .villager_data
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                (data.profession_enum() != VillagerProfession::None
                    && data.profession_enum() != VillagerProfession::Nitwit)
                    .then(|| (data.profession_enum(), data.level.0))
            } else {
                None
            }
        };
        if let Some((prof, level)) = trade_params {
            self.generate_trades(prof, level);
        }

        let has_offers = !self
            .offers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty();
        if !has_offers {
            self.set_unhappy();
        }

        player.increment_stat(
            pumpkin_data::statistic::StatisticCategory::Custom,
            pumpkin_data::statistic::CustomStatistic::TalkedToVillager as i32,
            1,
        );
        if !has_offers {
            return true;
        }

        let villager = self
            .self_weak
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .and_then(Weak::upgrade);
        if let Some(villager) = villager {
            villager.open_trading_screen(player);
        }

        true
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use pumpkin_data::data_component_impl::{EnchantmentsImpl, StoredEnchantmentsImpl};
    use pumpkin_data::villager::VillagerTradeModifier;
    use pumpkin_protocol::java::client::play::Metadata;
    use pumpkin_util::version::JavaMinecraftVersion;

    use super::*;

    #[test]
    fn villager_data_metadata_uses_the_villager_tracker_slot() {
        let data = VillagerData::new(VillagerType::Plains, VillagerProfession::Librarian, 1);
        let metadata = Metadata::new(tracked_data::villager::VILLAGER_DATA, data);
        let mut bytes = Vec::new();

        metadata
            .write(&mut bytes, &JavaMinecraftVersion::V_26_3)
            .unwrap();

        assert_eq!(bytes, [19, 18, 2, 9, 1]);
    }

    #[test]
    fn villager_data_maps_to_bedrock_appearance_metadata() {
        let metadata = VillagerEntity::bedrock_metadata(
            VillagerData::new(VillagerType::Plains, VillagerProfession::Librarian, 3),
            75,
        );

        assert!(matches!(
            metadata.0.get(&entity_data_key::VARIANT),
            Some(MetadataValue::Int(5))
        ));
        assert!(matches!(
            metadata.0.get(&entity_data_key::MARK_VARIANT),
            Some(MetadataValue::Int(0))
        ));
        assert!(matches!(
            metadata.0.get(&entity_data_key::TRADE_TIER),
            Some(MetadataValue::Int(2))
        ));
        assert!(matches!(
            metadata.0.get(&entity_data_key::MAX_TRADE_TIER),
            Some(MetadataValue::Int(4))
        ));
        assert!(matches!(
            metadata.0.get(&entity_data_key::TRADE_EXPERIENCE),
            Some(MetadataValue::Int(75))
        ));
    }

    #[test]
    fn unhappy_counter_metadata_uses_the_abstract_villager_tracker_slot() {
        let metadata = Metadata::new(tracked_data::villager::UNHAPPY_COUNTER, VarInt(40));
        let mut bytes = Vec::new();

        metadata
            .write(&mut bytes, &JavaMinecraftVersion::V_26_3)
            .unwrap();

        assert_eq!(bytes, [18, 1, 40]);
    }

    #[test]
    fn enchanted_book_offer_has_vanilla_items_and_a_nonzero_price() {
        let (emeralds, enchanted_book, book) = enchanted_book_offer_items(&mut rand::rng())
            .expect("the generated tradeable-enchantment tag is populated");
        let stored = enchanted_book
            .get_data_component::<StoredEnchantmentsImpl>()
            .unwrap();

        assert_eq!(emeralds.item.id, Item::EMERALD.id);
        assert!((5..=64).contains(&emeralds.item_count));
        assert_eq!(book.unwrap().item.id, Item::BOOK.id);
        assert_eq!(enchanted_book.item.id, Item::ENCHANTED_BOOK.id);
        assert_eq!(stored.enchantment.len(), 1);
        assert!(
            stored.enchantment[0]
                .0
                .has_tag(&EnchantmentTag::MINECRAFT_TRADEABLE)
        );
        assert!((1..=stored.enchantment[0].0.max_level).contains(&stored.enchantment[0].1));
    }

    #[test]
    fn every_exploration_map_destination_resolves() {
        // An unresolved destination makes the trade get skipped silently, so the
        // generated data and the lookup have to stay in step.
        let mut checked = 0;
        for id in 0..=14 {
            let Some(profession) = VillagerProfession::from_i32(id) else {
                continue;
            };
            for level in 1..=5 {
                let Some(trade_set) = profession.trade_set(level) else {
                    continue;
                };
                for trade in trade_set.trades {
                    if let VillagerTradeModifier::ExplorationMap { destination } = trade.modifier {
                        assert!(
                            explorer_map_target(destination).is_some(),
                            "unresolved exploration map destination: {destination}"
                        );
                        checked += 1;
                    }
                }
            }
        }
        assert!(checked > 0, "no exploration map trades found");
    }

    #[test]
    fn generated_trades_keep_dynamic_modifiers_and_secondary_costs() {
        let librarian = VillagerProfession::Librarian.trade_set(1).unwrap();
        let enchanted_book = librarian
            .trades
            .iter()
            .find(|trade| trade.modifier == VillagerTradeModifier::EnchantRandomly)
            .unwrap();
        assert_eq!(enchanted_book.wants.item, &Item::EMERALD);
        assert_eq!(enchanted_book.wants_b.unwrap().item, &Item::BOOK);
        assert_eq!(enchanted_book.price_multiplier, 0.2);

        let cartographer = VillagerProfession::Cartographer.trade_set(2).unwrap();
        assert!(cartographer.trades.iter().any(|trade| {
            matches!(trade.modifier, VillagerTradeModifier::ExplorationMap { .. })
                && !trade.allowed_types.is_empty()
                && trade
                    .wants_b
                    .is_some_and(|cost| cost.item == &Item::COMPASS)
        }));

        let fletcher = VillagerProfession::Fletcher.trade_set(5).unwrap();
        assert!(fletcher.trades.iter().any(|trade| {
            trade.modifier == VillagerTradeModifier::RandomPotion
                && trade.wants_b.is_some_and(|cost| cost.item == &Item::ARROW)
        }));
    }

    #[test]
    fn smith_trade_sets_include_the_shared_vanilla_trades() {
        let armorer_novice = VillagerProfession::Armorer.trade_set(1).unwrap();
        assert!(armorer_novice.trades.iter().any(|trade| {
            trade.wants.item == &Item::COAL && trade.gives.item == &Item::EMERALD
        }));

        let armorer_apprentice = VillagerProfession::Armorer.trade_set(2).unwrap();
        assert!(armorer_apprentice.trades.iter().any(|trade| {
            trade.wants.item == &Item::EMERALD && trade.gives.item == &Item::BELL
        }));
        assert!(armorer_apprentice.trades.iter().any(|trade| {
            trade.wants.item == &Item::IRON_INGOT && trade.gives.item == &Item::EMERALD
        }));

        for profession in [
            VillagerProfession::Toolsmith,
            VillagerProfession::Weaponsmith,
        ] {
            assert!(profession.trade_set(1).unwrap().trades.iter().any(|trade| {
                trade.wants.item == &Item::COAL && trade.gives.item == &Item::EMERALD
            }));
        }
    }

    #[test]
    fn traded_equipment_is_enchanted_and_reports_its_additional_price() {
        for _ in 0..32 {
            let (stack, additional_price) =
                enchant_trade_item(&mut rand::rng(), &Item::DIAMOND_SWORD, 5, 19).unwrap();
            let enchantments = stack.get_data_component::<EnchantmentsImpl>().unwrap();

            assert!((5..=19).contains(&additional_price));
            assert!(!enchantments.enchantment.is_empty());
        }
    }
}
