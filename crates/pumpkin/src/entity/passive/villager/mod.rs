use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, Ordering};
use std::sync::{Arc, Weak};
use uuid::Uuid;

use crate::block::blocks::bed::BedBlock;
use pumpkin_data::Enchantment;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::block_properties::{
    BedPart, BlockProperties, WhiteBedLikeProperties as BedProperties,
};
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::{EntityPose, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::potion::Effect;
use pumpkin_data::tag::{Enchantment as EnchantmentTag, Taggable};
use pumpkin_data::tracked_data::{TrackedData, TrackedId};
use pumpkin_inventory::merchant::merchant_screen_handler::MerchantScreenHandler;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{CMerchantOffers, Metadata};
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::SimpleInventory;
use tokio::sync::Mutex;

use crate::entity::player::Player;
use crate::entity::{
    Entity, EntityBase, NBTStorage,
    ai::{
        goal::{
            avoid_entity::AvoidEntityGoal, look_around::RandomLookAroundGoal,
            look_at_entity::LookAtEntityGoal, swim::SwimGoal,
            trade_with_player::TradeWithPlayerGoal, wander_around::WanderAroundGoal,
            work_at_job_site::WorkAtJobSiteGoal,
        },
        pathfinder::Navigator,
    },
    experience_orb::ExperienceOrbEntity,
    mob::{Mob, MobEntity},
};
use crate::world::World;
use crate::world::villager_poi::profession_for_block;

pub mod data;
pub use data::{
    BREEDING_FOOD_THRESHOLD, GossipType, VillagerData, VillagerProfession, VillagerType,
    get_food_points,
};

// `VILLAGER_DATA` is also used by zombie villagers, whose tracker slot is one higher.
// The flattened generated tracker data currently resolves that duplicate name to the zombie slot.
const VILLAGER_DATA_TRACKED_ID: TrackedId = TrackedId {
    v1_21: 18,
    v1_21_2: 18,
    v1_21_4: 18,
    v1_21_5: 18,
    v1_21_6: 18,
    v1_21_7: 18,
    v1_21_9: 18,
    v1_21_11: 18,
    v26_1: 19,
    v26_2: 19,
};

async fn trigger_trade_advancement(player: &Player) {
    player
        .trigger_advancement(
            crate::entity::player::advancement::trigger::AdvancementTrigger::TradedWithVillager,
        )
        .await;
}

fn enchanted_book_offer_items(
    rng: &mut impl rand::Rng,
) -> (ItemStack, ItemStack, Option<ItemStack>) {
    use pumpkin_data::data_component::DataComponent;
    use pumpkin_data::data_component_impl::StoredEnchantmentsImpl;
    use rand::{RngExt, seq::IndexedRandom};
    use std::borrow::Cow;

    let enchantment = Enchantment::from_id(
        *EnchantmentTag::MINECRAFT_TRADEABLE
            .1
            .choose(rng)
            .expect("tradeable enchantments must not be empty") as u8,
    )
    .expect("tradeable enchantment must exist");
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

    (
        ItemStack::new(emeralds.min(64) as u8, &Item::EMERALD),
        output,
        Some(ItemStack::new(1, &Item::BOOK)),
    )
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

fn apply_random_dye(rng: &mut impl rand::Rng, stack: &mut ItemStack) {
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
        brightness += *rgb.iter().max().unwrap();
        for (total, value) in channels.iter_mut().zip(rgb) {
            *total += value;
        }
    }
    let mut rgb = channels.map(|channel| channel / dye_count);
    let average_brightness = brightness / dye_count;
    let max_channel = *rgb.iter().max().unwrap();
    for channel in &mut rgb {
        *channel = average_brightness * *channel / max_channel;
    }
    let color = (rgb[0] << 16) | (rgb[1] << 8) | rgb[2];
    stack.patch.push((
        DataComponent::DyedColor,
        Some(DyedColorImpl { rgb: color }.to_dyn()),
    ));
}

fn apply_random_stew_effect(rng: &mut impl rand::Rng, stack: &mut ItemStack) {
    use pumpkin_data::data_component::DataComponent;
    use pumpkin_data::data_component_impl::{
        DataComponentImpl, SuspiciousStewEffect, SuspiciousStewEffectsImpl,
    };
    use rand::seq::IndexedRandom;
    use std::borrow::Cow;

    const EFFECTS: [(&str, i32); 6] = [
        ("minecraft:night_vision", 100),
        ("minecraft:jump_boost", 160),
        ("minecraft:weakness", 140),
        ("minecraft:blindness", 120),
        ("minecraft:poison", 280),
        ("minecraft:saturation", 7),
    ];
    let (effect, duration) = EFFECTS.choose(rng).unwrap();
    stack.patch.push((
        DataComponent::SuspiciousStewEffects,
        Some(
            SuspiciousStewEffectsImpl {
                effects: Cow::Owned(vec![SuspiciousStewEffect {
                    effect: Cow::Owned((*effect).to_owned()),
                    duration: *duration,
                }]),
            }
            .to_dyn(),
        ),
    ));
}

fn apply_potion(stack: &mut ItemStack, potion_name: &str) {
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

pub struct VillagerEntity {
    pub mob_entity: MobEntity,
    pub villager_data: Mutex<VillagerData>,
    pub food_level: AtomicI32,
    pub xp: AtomicI32,
    pub last_restock_time: AtomicI64,
    pub last_restock_check_day: AtomicI64,
    pub last_worked_at_poi: AtomicI64,
    pub restocks_today: AtomicI32,
    pub last_gossip_decay_time: AtomicI64,
    pub gossips: Mutex<HashMap<Uuid, HashMap<GossipType, i32>>>,
    pub inventory: Arc<Mutex<Vec<Arc<Mutex<ItemStack>>>>>,
    pub merchant_inventory: Arc<SimpleInventory>,
    pub offers: Mutex<Vec<pumpkin_protocol::java::client::play::MerchantOffer>>,
    pub merchant_update_timer: AtomicI32,
    pub unhappy_counter: AtomicI32,
    pub trade_sound_cooldown: AtomicI32,
    pub increase_profession_level_on_update: AtomicBool,
    pub last_traded_player: Mutex<Option<Uuid>>,
    pub trading_player: std::sync::Mutex<Option<(Uuid, u8)>>,
    pub is_trading: AtomicBool,
    pub job_site: std::sync::Mutex<Option<BlockPos>>,
    pub job_site_pending: AtomicBool,
    pub home_pos: std::sync::Mutex<Option<BlockPos>>,
    pub self_weak: std::sync::Mutex<Option<Weak<Self>>>,
}

impl VillagerEntity {
    #[allow(clippy::too_many_lines)]
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let villager_data = VillagerData::new(VillagerType::Plains, VillagerProfession::None, 1);
        let inventory = Arc::new(Mutex::new(
            (0..8)
                .map(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone())))
                .collect(),
        ));

        let villager = Self {
            mob_entity,
            villager_data: Mutex::new(villager_data),
            food_level: AtomicI32::new(0),
            xp: AtomicI32::new(0),
            last_restock_time: AtomicI64::new(0),
            last_restock_check_day: AtomicI64::new(0),
            last_worked_at_poi: AtomicI64::new(0),
            restocks_today: AtomicI32::new(0),
            last_gossip_decay_time: AtomicI64::new(0),
            gossips: Mutex::new(HashMap::new()),
            inventory,
            merchant_inventory: Arc::new(SimpleInventory::new(3)),
            offers: Mutex::new(Vec::new()),
            merchant_update_timer: AtomicI32::new(0),
            unhappy_counter: AtomicI32::new(0),
            trade_sound_cooldown: AtomicI32::new(0),
            increase_profession_level_on_update: AtomicBool::new(false),
            last_traded_player: Mutex::new(None),
            trading_player: std::sync::Mutex::new(None),
            is_trading: AtomicBool::new(false),
            job_site: std::sync::Mutex::new(None),
            job_site_pending: AtomicBool::new(false),
            home_pos: std::sync::Mutex::new(None),
            self_weak: std::sync::Mutex::new(None),
        };
        let mob_arc = Arc::new(villager);
        *mob_arc.self_weak.lock().unwrap() = Some(Arc::downgrade(&mob_arc));
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            // Villagers avoid threats
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(&EntityType::ZOMBIE, 8.0, 0.5, 0.5)),
            );
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(
                    &EntityType::ZOMBIE_VILLAGER,
                    8.0,
                    0.5,
                    0.5,
                )),
            );
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(&EntityType::HUSK, 8.0, 0.5, 0.5)),
            );
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(&EntityType::DROWNED, 8.0, 0.5, 0.5)),
            );
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(&EntityType::PILLAGER, 12.0, 0.5, 0.5)),
            );
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(
                    &EntityType::VINDICATOR,
                    12.0,
                    0.5,
                    0.5,
                )),
            );
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(&EntityType::EVOKER, 12.0, 0.5, 0.5)),
            );
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(&EntityType::RAVAGER, 12.0, 0.5, 0.5)),
            );
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(&EntityType::VEX, 12.0, 0.5, 0.5)),
            );

            goal_selector.add_goal(2, Box::new(TradeWithPlayerGoal::new(0.5)));
            // Basic movement and looking (Vanilla uses 0.5 speed)
            goal_selector.add_goal(3, Box::new(WorkAtJobSiteGoal::new(0.5)));
            goal_selector.add_goal(4, Box::new(WanderAroundGoal::new(0.5)));
            goal_selector.add_goal(
                5,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::VILLAGER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));
        };

        // Send initial metadata
        mob_arc.get_entity().send_meta_data(
            &[Metadata::new(
                VILLAGER_DATA_TRACKED_ID,
                MetaDataType::VILLAGER_DATA,
                villager_data,
            )],
            None,
        );

        mob_arc
    }

    pub async fn count_food_points_in_inventory(&self) -> i32 {
        let inventory = self.inventory.lock().await;
        let mut total = 0;
        for stack_mutex in inventory.iter() {
            let stack = stack_mutex.lock().await;
            if !stack.is_empty() {
                total += get_food_points(stack.get_item()) * stack.item_count as i32;
            }
        }
        total
    }

    fn poi_owner(&self) -> Weak<dyn EntityBase> {
        self.self_weak.lock().unwrap().clone().unwrap()
    }

    pub async fn eat_until_full(&self) {
        if self.food_level.load(Ordering::Relaxed) >= BREEDING_FOOD_THRESHOLD {
            return;
        }
        let inventory = self.inventory.lock().await;
        for stack_mutex in inventory.iter() {
            let mut stack = stack_mutex.lock().await;
            if !stack.is_empty() {
                let points = get_food_points(stack.get_item());
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

    pub async fn set_villager_data(&self, data: VillagerData) {
        let old_profession = {
            let mut villager_data = self.villager_data.lock().await;
            let old_profession = villager_data.profession;
            *villager_data = data;
            old_profession
        };
        self.get_entity().send_meta_data(
            &[Metadata::new(
                VILLAGER_DATA_TRACKED_ID,
                MetaDataType::VILLAGER_DATA,
                data,
            )],
            None,
        );

        if old_profession != data.profession {
            self.offers.lock().await.clear();
        }
    }

    #[expect(clippy::too_many_lines)]
    async fn create_explorer_map(&self, destination: &str) -> Option<ItemStack> {
        use pumpkin_data::data_component::DataComponent;
        use pumpkin_data::data_component_impl::{DataComponentImpl, ItemNameImpl, MapIdImpl};
        use pumpkin_data::structures::{StructureKeys, StructureSet};
        use pumpkin_world::generation::generator::structure_finder::find_nearest_structure_start;

        let (structure_set, structure, name, icon_type) = match destination {
            "minecraft:on_jungle_explorer_maps" => (
                "jungle_temples",
                StructureKeys::JunglePyramid,
                "filled_map.explorer_jungle",
                32,
            ),
            "minecraft:on_swamp_explorer_maps" => (
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
            "minecraft:on_ocean_explorer_maps" => (
                "ocean_monuments",
                StructureKeys::Monument,
                "filled_map.monument",
                9,
            ),
            "minecraft:on_trial_chambers_maps" => (
                "trial_chambers",
                StructureKeys::TrialChambers,
                "filled_map.trial_chambers",
                34,
            ),
            "minecraft:on_woodland_explorer_maps" => (
                "woodland_mansions",
                StructureKeys::Mansion,
                "filled_map.mansion",
                8,
            ),
            _ => return None,
        };

        let world = self.get_entity().world.load().clone();
        let target = find_nearest_structure_start(
            self.get_entity().block_pos.load(),
            StructureSet::get(structure_set)?,
            &[structure],
            100,
            &world.level.world_gen,
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
            .await
            .decorations
            .push(crate::world::map::MapDecoration {
                icon_type,
                x: 0,
                z: 0,
                direction: 8,
                display_name: None,
            });

        let mut stack = ItemStack::new(1, &Item::FILLED_MAP);
        stack.patch.push((
            DataComponent::MapId,
            Some(MapIdImpl { id: map_id }.to_dyn()),
        ));
        stack.patch.push((
            DataComponent::ItemName,
            Some(ItemNameImpl { name: name.into() }.to_dyn()),
        ));
        Some(stack)
    }

    pub async fn add_trades(&self, profession: VillagerProfession, level: i32) {
        use pumpkin_data::villager::VillagerTradeModifier;
        use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
        use rand::seq::IndexedRandom;
        use rand::{RngExt, SeedableRng, rngs::StdRng};
        use std::borrow::Cow;

        let villager_type = self.villager_data.lock().await.type_enum();
        let mut offers = self.offers.lock().await;

        if let Some(trade_set) = profession.trade_set(level) {
            let mut rng = StdRng::from_rng(&mut rand::rng());
            let mut remaining_trades = trade_set.trades.iter().collect::<Vec<_>>();
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

                match trade.modifier {
                    VillagerTradeModifier::None => {}
                    VillagerTradeModifier::EnchantRandomly => {
                        (base_cost_a, output, cost_b) = enchanted_book_offer_items(&mut rng);
                    }
                    VillagerTradeModifier::EnchantWithLevels { min, max } => {
                        let Some((enchanted, additional_cost)) =
                            enchant_trade_item(&mut rng, trade.gives.item, min, max)
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
                    VillagerTradeModifier::ExplorationMap { destination } => {
                        let Some(map) = self.create_explorer_map(destination).await else {
                            continue;
                        };
                        output = map;
                    }
                    VillagerTradeModifier::RandomDyes => apply_random_dye(&mut rng, &mut output),
                    VillagerTradeModifier::RandomPotion => {
                        let potion_name = pumpkin_data::tag::Potion::MINECRAFT_TRADEABLE
                            .0
                            .choose(&mut rng)
                            .expect("tradeable potion tag must not be empty");
                        apply_potion(&mut output, potion_name);
                    }
                    VillagerTradeModifier::SuspiciousStew => {
                        apply_random_stew_effect(&mut rng, &mut output);
                    }
                    VillagerTradeModifier::Potion(potion) => apply_potion(&mut output, potion),
                }
                offers.push(pumpkin_protocol::java::client::play::MerchantOffer {
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
    }

    pub async fn generate_trades(&self, profession: VillagerProfession, level: i32) {
        self.offers.lock().await.clear();
        self.add_trades(profession, level).await;
    }

    async fn update_special_prices(&self, player: &Player) {
        let player_uuid = player.get_entity().entity_uuid;
        let reputation = self
            .gossips
            .lock()
            .await
            .get(&player_uuid)
            .map_or(0, |gossips| {
                gossips
                    .iter()
                    .map(|(kind, value)| kind.weight() * value)
                    .sum()
            });
        let hero_amplifier = player
            .living_entity
            .get_effect(&StatusEffect::HERO_OF_THE_VILLAGE)
            .await
            .map(|effect| i32::from(effect.amplifier));

        let mut offers = self.offers.lock().await;
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

    async fn reset_special_prices(&self) {
        for offer in self.offers.lock().await.iter_mut() {
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
                .unwrap()
                .as_ref()
                .is_some_and(|(uuid, id)| *uuid == player_uuid && *id == sync_id)
            && entity
                .bounding_box
                .load()
                .squared_magnitude(player.eye_position())
                < range * range
    }

    async fn complete_trade(&self, offer_index: usize, world: &Arc<World>, player_uuid: Uuid) {
        let (xp_gain, reward_exp) = {
            let mut offers = self.offers.lock().await;
            let Some(offer) = offers.get_mut(offer_index) else {
                return;
            };
            offer.uses += 1;
            (offer.xp, offer.reward_exp)
        };

        let current_xp = self.xp.fetch_add(xp_gain, Ordering::Relaxed) + xp_gain;
        let mut reward_xp = {
            use rand::RngExt;
            rand::rng().random_range(3..=6)
        };

        let current_level = self.villager_data.lock().await.level.0;
        if current_level < 5 {
            let max_xp = match current_level {
                1 => 10,
                2 => 70,
                3 => 150,
                4 => 250,
                _ => 0,
            };
            if current_xp >= max_xp {
                self.merchant_update_timer.store(40, Ordering::Relaxed);
                self.increase_profession_level_on_update
                    .store(true, Ordering::Relaxed);
                reward_xp += 5;
            }
        }
        self.get_entity()
            .play_sound(pumpkin_data::sound::Sound::EntityVillagerYes);
        self.trade_sound_cooldown.store(20, Ordering::Relaxed);
        *self.last_traded_player.lock().await = Some(player_uuid);
        if reward_exp {
            let position = self.get_entity().pos.load().add_raw(0.0, 0.5, 0.0);
            ExperienceOrbEntity::spawn(world, position, reward_xp).await;
        }

        if let Some(player) = world.get_player_by_uuid(player_uuid) {
            trigger_trade_advancement(&player).await;
        }
    }

    async fn resend_offers_to_trading_player(&self) {
        let Some((player_uuid, sync_id)) = *self.trading_player.lock().unwrap() else {
            return;
        };
        let world = self.get_entity().world.load();
        let Some(player) = world.get_player_by_uuid(player_uuid) else {
            return;
        };
        let offers = self.offers.lock().await.clone();
        let current_level = self.villager_data.lock().await.level;

        let screen = player.current_screen_handler.lock().await.clone();
        let mut screen = screen.lock().await;
        if screen.sync_id() != sync_id {
            return;
        }
        if let Some(handler) = screen.as_any_mut().downcast_mut::<MerchantScreenHandler>() {
            handler.offers.clone_from(&offers);
            handler.update_result_slot().await;
        } else {
            return;
        }
        drop(screen);
        player
            .client
            .enqueue_packet(&CMerchantOffers::new(
                VarInt(i32::from(sync_id)),
                offers,
                current_level,
                VarInt(self.xp.load(Ordering::Relaxed)),
                true,
                true,
            ))
            .await;
    }

    async fn decay_gossips(&self, game_time: i64) {
        let last_decay = self.last_gossip_decay_time.load(Ordering::Relaxed);
        if last_decay == 0 {
            self.last_gossip_decay_time
                .store(game_time, Ordering::Relaxed);
            return;
        }
        if game_time < last_decay + 24_000 {
            return;
        }

        let mut gossips = self.gossips.lock().await;
        for values in gossips.values_mut() {
            values.retain(|kind, value| {
                *value -= kind.daily_decay();
                *value > 0
            });
        }
        gossips.retain(|_, values| !values.is_empty());
        self.last_gossip_decay_time
            .store(game_time, Ordering::Relaxed);
    }

    async fn work_at_job_site(&self, game_time: i64, day_time: i64, day: i64) {
        use rand::RngExt;

        if !(2_000..9_000).contains(&day_time)
            || game_time - self.last_worked_at_poi.load(Ordering::Relaxed) < 300
            || !rand::rng().random_bool(0.5)
        {
            return;
        }
        self.last_worked_at_poi.store(game_time, Ordering::Relaxed);

        let Some(job_site) = self.get_job_site() else {
            return;
        };
        if job_site
            .to_centered_f64()
            .squared_distance_to_vec(&self.get_entity().pos.load())
            >= 1.73f64.powi(2)
        {
            return;
        }

        let profession = self.villager_data.lock().await.profession_enum();
        if let Some(sound) = profession.work_sound() {
            self.get_entity().play_sound(sound);
        }

        let last_restock = self.last_restock_time.load(Ordering::Relaxed);
        let last_check_day = self.last_restock_check_day.swap(day, Ordering::Relaxed);
        if game_time > last_restock + 12_000 || (last_check_day > 0 && day > last_check_day) {
            let missed_restock_count = (2 - self.restocks_today.load(Ordering::Relaxed)).max(0);
            let mut offers = self.offers.lock().await;
            if missed_restock_count > 0 {
                for offer in offers.iter_mut() {
                    offer.reset_uses();
                }
            }
            for _ in 0..missed_restock_count {
                for offer in offers.iter_mut() {
                    offer.update_demand();
                }
            }
            drop(offers);
            self.last_restock_time.store(game_time, Ordering::Relaxed);
            self.restocks_today.store(0, Ordering::Relaxed);
            self.resend_offers_to_trading_player().await;
        }

        let restocks_today = self.restocks_today.load(Ordering::Relaxed);
        let allowed = restocks_today == 0
            || (restocks_today < 2
                && game_time > self.last_restock_time.load(Ordering::Relaxed) + 2_400);
        if !allowed {
            return;
        }

        let mut offers = self.offers.lock().await;
        if !offers
            .iter()
            .any(pumpkin_protocol::java::client::play::MerchantOffer::needs_restock)
        {
            return;
        }
        for offer in offers.iter_mut() {
            offer.update_demand();
            offer.reset_uses();
        }
        self.last_restock_time.store(game_time, Ordering::Relaxed);
        self.restocks_today.fetch_add(1, Ordering::Relaxed);
        drop(offers);
        self.resend_offers_to_trading_player().await;
    }

    #[expect(clippy::too_many_lines)]
    async fn update_job_site(&self, world: &crate::world::World) {
        let data = *self.villager_data.lock().await;
        let profession = data.profession_enum();
        let is_adult = self.get_entity().age.load(Ordering::Relaxed) >= 0;

        if profession == VillagerProfession::Nitwit || !is_adult {
            if let Some(site) = self.get_job_site() {
                world
                    .villager_poi
                    .lock()
                    .await
                    .release(site, self.get_entity().entity_uuid);
                *self.job_site.lock().unwrap() = None;
                self.job_site_pending.store(false, Ordering::Relaxed);
            }
            return;
        }

        if let Some(current_site) = self.get_job_site()
            && current_site
                .to_centered_f64()
                .squared_distance_to_vec(&self.get_entity().pos.load())
                < 16.0f64.powi(2)
        {
            let (block, _state) = world.get_block_and_state(&current_site);
            let expected = (profession != VillagerProfession::None).then_some(profession);
            let valid = world
                .villager_poi
                .lock()
                .await
                .claim(current_site, block, self.poi_owner(), expected)
                .is_some();

            if !valid {
                *self.job_site.lock().unwrap() = None;
                self.job_site_pending.store(false, Ordering::Relaxed);
                if self.xp.load(Ordering::Relaxed) == 0
                    && data.level.0 <= 1
                    && profession != VillagerProfession::None
                {
                    let r#type = self.villager_data.lock().await.type_enum();
                    self.set_villager_data(VillagerData::new(r#type, VillagerProfession::None, 1))
                        .await;
                }
            }
        }

        if self.get_job_site().is_none() {
            let profession = self.villager_data.lock().await.profession_enum();
            let expected = (profession != VillagerProfession::None).then_some(profession);
            let pos = self.get_entity().block_pos.load();
            let start = BlockPos::new(pos.0.x - 10, pos.0.y - 4, pos.0.z - 10);
            let end = BlockPos::new(pos.0.x + 10, pos.0.y + 4, pos.0.z + 10);
            let mut candidates = Vec::new();

            let indexed_sites = world
                .villager_poi
                .lock()
                .await
                .available_job_sites(pos, 48, expected);
            let saved_sites = world.portal_poi.lock().await.get_in_square(pos, 48, None);
            for position in indexed_sites.into_iter().chain(saved_sites) {
                let delta = position.0 - pos.0;
                if i64::from(delta.x).pow(2) + i64::from(delta.y).pow(2) + i64::from(delta.z).pow(2)
                    > 48i64.pow(2)
                    || candidates
                        .iter()
                        .any(|(_, candidate, _, _)| *candidate == position)
                {
                    continue;
                }
                let (block, _state) = world.get_block_and_state(&position);
                if let Some(site_profession) = profession_for_block(block)
                    && expected.is_none_or(|profession| profession == site_profession)
                {
                    let distance = position
                        .to_centered_f64()
                        .squared_distance_to_vec(&self.get_entity().pos.load());
                    candidates.push((distance, position, block, site_profession));
                }
            }

            for position in BlockPos::iterate(start, end) {
                let (block, _state) = world.get_block_and_state(&position);
                let Some(site_profession) = profession_for_block(block) else {
                    continue;
                };
                if expected.is_some_and(|profession| profession != site_profession)
                    || candidates
                        .iter()
                        .any(|(_, candidate, _, _)| *candidate == position)
                {
                    continue;
                }
                let distance = position
                    .to_centered_f64()
                    .squared_distance_to_vec(&self.get_entity().pos.load());
                candidates.push((distance, position, block, site_profession));
            }
            candidates.sort_by(|left, right| left.0.total_cmp(&right.0));

            let owner = self.poi_owner();
            let mut navigator = Navigator::default();
            let mut claimed = None;
            for (_, position, block, _) in candidates.into_iter().take(5) {
                if !navigator
                    .can_reach_within(
                        &self.mob_entity.living_entity,
                        position.to_centered_f64(),
                        1.73,
                    )
                    .await
                {
                    continue;
                }
                if world
                    .villager_poi
                    .lock()
                    .await
                    .claim(position, block, owner.clone(), expected)
                    .is_some()
                {
                    claimed = Some(position);
                    break;
                }
            }

            if let Some(site) = claimed {
                *self.job_site.lock().unwrap() = Some(site);
                self.job_site_pending.store(true, Ordering::Relaxed);
            }
        }

        if self.job_site_pending.load(Ordering::Relaxed)
            && let Some(site) = self.get_job_site()
            && site
                .to_centered_f64()
                .squared_distance_to_vec(&self.get_entity().pos.load())
                < 2.0f64.powi(2)
        {
            let (block, _state) = world.get_block_and_state(&site);
            if let Some(claimed_profession) = profession_for_block(block) {
                let profession = self.villager_data.lock().await.profession_enum();
                if profession != VillagerProfession::None && profession != claimed_profession {
                    return;
                }
                world.send_entity_status(
                    self.get_entity(),
                    pumpkin_data::entity::EntityStatus::VillagerHappy,
                );
                self.job_site_pending.store(false, Ordering::Relaxed);
                if profession == VillagerProfession::None {
                    let r#type = self.villager_data.lock().await.type_enum();
                    self.set_villager_data(VillagerData::new(r#type, claimed_profession, 1))
                        .await;
                }
            }
        }
    }

    pub fn set_unhappy(&self) {
        let entity = self.get_entity();
        self.unhappy_counter.store(40, Ordering::Relaxed);
        entity.send_meta_data(
            &[Metadata::new(
                TrackedData::UNHAPPY_COUNTER,
                MetaDataType::INT,
                VarInt(40),
            )],
            None,
        );
        entity.play_sound(pumpkin_data::sound::Sound::EntityVillagerNo);
    }

    pub async fn open_trading_screen(&self, player: &Arc<Player>) {
        use pumpkin_protocol::codec::var_int::VarInt;
        use pumpkin_protocol::java::client::play::CMerchantOffers;

        // Open the merchant screen and then send the current offers packet
        if let Some(sync_id) = player.open_handled_screen(self, None).await {
            let offers = self.offers.lock().await.clone();
            let villager_data = self.villager_data.lock().await;

            player
                .client
                .enqueue_packet(&CMerchantOffers::new(
                    VarInt(sync_id as i32),
                    offers,
                    villager_data.level,
                    VarInt(self.xp.load(Ordering::Relaxed)),
                    true,
                    true,
                ))
                .await;
        }
    }
}

impl ScreenHandlerFactory for VillagerEntity {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<pumpkin_inventory::player::player_inventory::PlayerInventory>,
        player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let self_weak = self.self_weak.lock().unwrap().clone().unwrap();
            let server_player = player.as_any().downcast_ref::<Player>();
            let player_uuid =
                server_player.map_or_else(uuid::Uuid::nil, |p| p.get_entity().entity_uuid);
            if let Some(player) = server_player {
                self.update_special_prices(player).await;
            }
            let offers = self.offers.lock().await;
            let world = self.get_entity().world.load().clone();

            let mut handler = MerchantScreenHandler::new(
                sync_id,
                player_inventory,
                self.merchant_inventory.clone(),
                offers.clone(),
            )
            .await;

            self.is_trading.store(true, Ordering::Relaxed);
            *self.trading_player.lock().unwrap() = Some((player_uuid, sync_id));
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
                let close_weak = close_weak.clone();
                Box::pin(async move {
                    if let Some(villager) = close_weak.upgrade() {
                        villager.is_trading.store(false, Ordering::Relaxed);
                        *villager.trading_player.lock().unwrap() = None;
                        villager.reset_special_prices().await;
                    }
                })
            }));

            handler.on_trade = Some(Box::new(move |offer_index| {
                let self_weak = self_weak.clone();
                let world = world.clone();
                Box::pin(async move {
                    if let Some(villager) = self_weak.upgrade() {
                        villager
                            .complete_trade(offer_index, &world, player_uuid)
                            .await;
                    }
                })
            }));

            Some(Arc::new(Mutex::new(handler)) as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        let profession = self
            .villager_data
            .try_lock()
            .map_or(VillagerProfession::None, |data| data.profession_enum());
        TextComponent::translate(profession.translation_key(), [])
    }
}

impl NBTStorage for VillagerEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.entity.write_nbt(nbt).await;
            let data = self.villager_data.lock().await;
            let mut villager_data_nbt = NbtCompound::new();
            villager_data_nbt.put_int("Type", data.r#type.0);
            villager_data_nbt.put_int("Profession", data.profession.0);
            villager_data_nbt.put_int("Level", data.level.0);
            nbt.put_compound("VillagerData", villager_data_nbt);

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

            let job_site_pos = *self.job_site.lock().unwrap();
            if let Some(pos) = job_site_pos {
                nbt.put_int("JobSiteX", pos.0.x);
                nbt.put_int("JobSiteY", pos.0.y);
                nbt.put_int("JobSiteZ", pos.0.z);
                nbt.put_bool(
                    "JobSitePending",
                    self.job_site_pending.load(Ordering::Relaxed),
                );
            }

            let home_pos = *self.home_pos.lock().unwrap();
            if let Some(pos) = home_pos {
                nbt.put_int("HomeX", pos.0.x);
                nbt.put_int("HomeY", pos.0.y);
                nbt.put_int("HomeZ", pos.0.z);
            }

            // Save Offers
            {
                let offers = self.offers.lock().await;
                let mut recipes = Vec::new();
                for offer in offers.iter() {
                    let mut recipe = NbtCompound::new();

                    let mut buy = NbtCompound::new();
                    offer.base_cost_a.0.write_item_stack(&mut buy);
                    recipe.put_compound("buy", buy);

                    if let Some(cost_b) = &offer.cost_b
                        && !cost_b.0.is_empty()
                    {
                        let mut buy_b = NbtCompound::new();
                        cost_b.0.write_item_stack(&mut buy_b);
                        recipe.put_compound("buyB", buy_b);
                    }

                    let mut sell_item = NbtCompound::new();
                    offer.output.0.write_item_stack(&mut sell_item);
                    recipe.put_compound("sell", sell_item);

                    recipe.put_int("uses", offer.uses);
                    recipe.put_int("maxUses", offer.max_uses);
                    recipe.put_bool("rewardExp", offer.reward_exp);
                    recipe.put_int("xp", offer.xp);
                    recipe.put_float("priceMultiplier", offer.price_multiplier);
                    recipe.put_int("specialPrice", offer.special_price);
                    recipe.put_int("demand", offer.demand);

                    recipes.push(pumpkin_nbt::tag::NbtTag::Compound(recipe));
                }
                let mut offers_compound = NbtCompound::new();
                offers_compound.put("Recipes", pumpkin_nbt::tag::NbtTag::List(recipes));
                nbt.put_compound("Offers", offers_compound);
            };

            // Inventory
            let inventory = self.inventory.lock().await;
            let mut inventory_list = Vec::new();
            for stack_mutex in inventory.iter() {
                let stack = stack_mutex.lock().await;
                if !stack.is_empty() {
                    let mut item_nbt = NbtCompound::new();
                    stack.write_item_stack(&mut item_nbt);
                    inventory_list.push(pumpkin_nbt::tag::NbtTag::Compound(item_nbt));
                }
            }
            nbt.put("Inventory", pumpkin_nbt::tag::NbtTag::List(inventory_list));

            // Gossips
            let gossips = self.gossips.lock().await;
            let mut gossip_list = Vec::new();
            for (uuid, types) in gossips.iter() {
                for (gtype, value) in types {
                    let mut gossip_nbt = NbtCompound::new();
                    let uuid_val = uuid.as_u128();
                    gossip_nbt.put(
                        "Target",
                        pumpkin_nbt::tag::NbtTag::IntArray(vec![
                            (uuid_val >> 96) as i32,
                            ((uuid_val >> 64) & 0xFFFF_FFFF) as i32,
                            ((uuid_val >> 32) & 0xFFFF_FFFF) as i32,
                            (uuid_val & 0xFFFF_FFFF) as i32,
                        ]),
                    );
                    gossip_nbt.put_string("Type", gtype.name().to_owned());
                    gossip_nbt.put_int("Value", *value);
                    gossip_list.push(pumpkin_nbt::tag::NbtTag::Compound(gossip_nbt));
                }
            }
            nbt.put("Gossips", pumpkin_nbt::tag::NbtTag::List(gossip_list));
        })
    }

    #[allow(clippy::too_many_lines)]
    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity
                .living_entity
                .entity
                .read_nbt_non_mut(nbt)
                .await;
            if let Some(villager_data_nbt) = nbt.get_compound("VillagerData") {
                let mut data = self.villager_data.lock().await;
                if let Some(t) = villager_data_nbt.get_int("Type") {
                    data.r#type = VarInt(t);
                }
                if let Some(p) = villager_data_nbt.get_int("Profession") {
                    data.profession = VarInt(p);
                }
                if let Some(l) = villager_data_nbt.get_int("Level") {
                    data.level = VarInt(l);
                }
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

            if let (Some(x), Some(y), Some(z)) = (
                nbt.get_int("JobSiteX"),
                nbt.get_int("JobSiteY"),
                nbt.get_int("JobSiteZ"),
            ) {
                *self.job_site.lock().unwrap() = Some(BlockPos::new(x, y, z));
                self.job_site_pending.store(
                    nbt.get_bool("JobSitePending").unwrap_or(false),
                    Ordering::Relaxed,
                );
            } else {
                *self.job_site.lock().unwrap() = None;
                self.job_site_pending.store(false, Ordering::Relaxed);
            }

            if let (Some(x), Some(y), Some(z)) = (
                nbt.get_int("HomeX").or_else(|| nbt.get_int("BedX")),
                nbt.get_int("HomeY").or_else(|| nbt.get_int("BedY")),
                nbt.get_int("HomeZ").or_else(|| nbt.get_int("BedZ")),
            ) {
                *self.home_pos.lock().unwrap() = Some(BlockPos::new(x, y, z));
            } else {
                *self.home_pos.lock().unwrap() = None;
            }

            if let Some(offers_compound) = nbt.get_compound("Offers")
                && let Some(recipes) = offers_compound.get_list("Recipes")
            {
                let mut offers = self.offers.lock().await;
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
                            let price_multiplier =
                                recipe.get_float("priceMultiplier").unwrap_or(0.05);
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
                let mut inventory = self.inventory.lock().await;
                inventory.clear();
                for tag in inventory_list {
                    if let Some(item_compound) = tag.extract_compound()
                        && let Some(stack) = ItemStack::read_item_stack(item_compound)
                    {
                        inventory.push(Arc::new(Mutex::new(stack)));
                    }
                }
            }

            // Gossips
            if let Some(gossip_list) = nbt.get_list("Gossips") {
                let mut gossips = self.gossips.lock().await;
                gossips.clear();
                for tag in gossip_list {
                    if let Some(gossip_nbt) = tag.extract_compound() {
                        let uuid = gossip_nbt.get_int_array("Target").map(|uuid_array| {
                            Uuid::from_u128(
                                (uuid_array[0] as u128) << 96
                                    | (uuid_array[1] as u128) << 64
                                    | (uuid_array[2] as u128) << 32
                                    | (uuid_array[3] as u128),
                            )
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
                            gossips.entry(uuid).or_default().insert(gossip_type, val);
                        }
                    }
                }
            }
        })
    }
}

impl Mob for VillagerEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_job_site(&self) -> Option<BlockPos> {
        *self.job_site.lock().unwrap()
    }

    fn is_job_site_pending(&self) -> crate::entity::EntityBaseFuture<'_, bool> {
        Box::pin(async move { self.job_site_pending.load(Ordering::Relaxed) })
    }

    fn release_pending_job_site(
        &self,
        position: BlockPos,
    ) -> crate::entity::EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            if self.get_job_site() != Some(position)
                || !self.job_site_pending.load(Ordering::Relaxed)
            {
                return;
            }
            self.get_entity()
                .world
                .load()
                .villager_poi
                .lock()
                .await
                .release(position, self.get_entity().entity_uuid);
            if self.get_job_site() == Some(position) {
                *self.job_site.lock().unwrap() = None;
                self.job_site_pending.store(false, Ordering::Relaxed);
            }
        })
    }

    fn get_trading_player(&self) -> Option<Arc<Player>> {
        let trading_player = *self.trading_player.lock().unwrap();
        let (player_uuid, _) = trading_player?;
        self.get_entity()
            .world
            .load()
            .get_player_by_uuid(player_uuid)
    }

    fn get_home(&self) -> Option<BlockPos> {
        *self.home_pos.lock().unwrap()
    }

    fn mob_init_data_tracker(&self) -> crate::entity::EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            entity.send_meta_data(
                &[Metadata::new(
                    VILLAGER_DATA_TRACKED_ID,
                    MetaDataType::VILLAGER_DATA,
                    *self.villager_data.lock().await,
                )],
                None,
            );
            if entity.age.load(Ordering::Relaxed) < 0 {
                entity.send_meta_data(
                    &[Metadata::new(
                        TrackedData::BABY_ID,
                        MetaDataType::BOOLEAN,
                        true,
                    )],
                    None,
                );
            }
        })
    }

    fn on_damage<'a>(
        &'a self,
        _damage_type: pumpkin_data::damage::DamageType,
        source: Option<&'a dyn EntityBase>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let Some(source) = source.filter(|source| {
                source.get_entity().entity_type == &pumpkin_data::entity::EntityType::PLAYER
            }) else {
                return;
            };
            let mut gossips = self.gossips.lock().await;
            let value = gossips
                .entry(source.get_entity().entity_uuid)
                .or_default()
                .entry(GossipType::MinorNegative)
                .or_default();
            *value = (*value + 25).min(GossipType::MinorNegative.max_value());
            drop(gossips);
            self.get_entity().world.load().send_entity_status(
                self.get_entity(),
                pumpkin_data::entity::EntityStatus::VillagerAngry,
            );
        })
    }

    #[expect(clippy::too_many_lines)]
    fn mob_tick<'a>(
        &'a self,
        _caller: &'a Arc<dyn EntityBase>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let world = self.get_entity().world.load();

            let unhappy_counter = self.unhappy_counter.load(Ordering::Relaxed);
            if unhappy_counter > 0 {
                let unhappy_counter = unhappy_counter - 1;
                self.unhappy_counter
                    .store(unhappy_counter, Ordering::Relaxed);
                self.get_entity().send_meta_data(
                    &[Metadata::new(
                        TrackedData::UNHAPPY_COUNTER,
                        MetaDataType::INT,
                        VarInt(unhappy_counter),
                    )],
                    None,
                );
            }
            self.trade_sound_cooldown
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |cooldown| {
                    (cooldown > 0).then_some(cooldown - 1)
                })
                .ok();

            let last_traded_player = self.last_traded_player.lock().await.take();
            if let Some(player_uuid) = last_traded_player {
                let mut gossips = self.gossips.lock().await;
                let value = gossips
                    .entry(player_uuid)
                    .or_default()
                    .entry(GossipType::Trading)
                    .or_default();
                *value = (*value + 2).min(GossipType::Trading.max_value());
                drop(gossips);
                world.send_entity_status(
                    self.get_entity(),
                    pumpkin_data::entity::EntityStatus::VillagerHappy,
                );
            }

            if !self.is_trading.load(Ordering::Relaxed)
                && self.merchant_update_timer.load(Ordering::Relaxed) > 0
                && self.merchant_update_timer.fetch_sub(1, Ordering::Relaxed) == 1
            {
                if self
                    .increase_profession_level_on_update
                    .swap(false, Ordering::Relaxed)
                {
                    let mut data = *self.villager_data.lock().await;
                    data.level.0 += 1;
                    self.set_villager_data(data).await;
                    self.add_trades(data.profession_enum(), data.level.0).await;
                }
                self.mob_entity
                    .living_entity
                    .add_effect(Effect {
                        effect_type: &StatusEffect::REGENERATION,
                        duration: 200,
                        amplifier: 0,
                        ambient: false,
                        show_particles: true,
                        show_icon: true,
                        blend: false,
                    })
                    .await;
            }

            let (game_time, day_time, day) = {
                let time = world.level_time.lock().await;
                (time.world_age, time.query_daytime(), time.query_day())
            };
            self.decay_gossips(game_time).await;
            self.work_at_job_site(game_time, day_time, day).await;

            let age = self.get_entity().age.load(Ordering::Relaxed);
            if age % 20 != 0 {
                return;
            }
            self.update_job_site(&world).await;

            // 1. Bed / Sleeping logic (for all villagers: babies, nitwits, adults)
            let is_sleeping = self.get_entity().pose.load() == EntityPose::Sleeping;

            // Check if current bed is still valid
            if let Some(current_home) = self.get_home_pos() {
                let (block, state) = world.get_block_and_state(&current_home);
                let valid = if block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_BEDS) {
                    let bed_props = BedProperties::from_state_id(state.id, block);
                    bed_props.part == BedPart::Head
                } else {
                    false
                };

                if !valid {
                    *self.home_pos.lock().unwrap() = None;
                    if is_sleeping {
                        // Wake up if bed was broken
                        self.get_entity().set_pose(EntityPose::Standing);
                        self.get_entity().send_meta_data(
                            &[Metadata::new(
                                TrackedData::SLEEPING_POS_ID,
                                MetaDataType::OPTIONAL_BLOCK_POS,
                                None::<BlockPos>,
                            )],
                            None,
                        );
                    }
                }
            }

            // If no bed, search for one
            if self.get_home_pos().is_none() {
                let pos = self.get_entity().block_pos.load();
                let start = BlockPos::new(pos.0.x - 16, pos.0.y - 4, pos.0.z - 16);
                let end = BlockPos::new(pos.0.x + 16, pos.0.y + 4, pos.0.z + 16);

                let aabb = BoundingBox::new(
                    Vector3::new(
                        pos.0.x as f64 - 32.0,
                        pos.0.y as f64 - 16.0,
                        pos.0.z as f64 - 32.0,
                    ),
                    Vector3::new(
                        pos.0.x as f64 + 32.0,
                        pos.0.y as f64 + 16.0,
                        pos.0.z as f64 + 32.0,
                    ),
                );
                let nearby_entities = world.get_all_at_box(&aabb);

                let mut claimed_homes = Vec::new();
                for entity in nearby_entities {
                    if entity.get_entity().entity_id != self.get_entity().entity_id
                        && entity.get_entity().entity_type
                            == &pumpkin_data::entity::EntityType::VILLAGER
                        && let Some(home) = entity.get_home_pos()
                    {
                        claimed_homes.push(home);
                    }
                }

                let mut best_home = None;
                let mut best_dist = f64::MAX;

                for p in BlockPos::iterate(start, end) {
                    let (block, state) = world.get_block_and_state(&p);
                    if block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_BEDS) {
                        let bed_props = BedProperties::from_state_id(state.id, block);
                        let bed_head_pos = if bed_props.part == BedPart::Head {
                            p
                        } else {
                            p.offset(bed_props.facing.to_offset())
                        };

                        if claimed_homes.contains(&bed_head_pos) {
                            continue;
                        }

                        let dist = bed_head_pos
                            .to_f64()
                            .squared_distance_to_vec(&self.get_entity().pos.load());
                        if dist < best_dist {
                            best_dist = dist;
                            best_home = Some(bed_head_pos);
                        }
                    }
                }

                if let Some(home) = best_home {
                    *self.home_pos.lock().unwrap() = Some(home);
                }
            }

            // Handle Sleeping/Waking up based on time
            let is_sleeping = self.get_entity().pose.load() == EntityPose::Sleeping;
            if let Some(home_pos) = self.get_home_pos() {
                let time = world.level_time.lock().await.time_of_day;
                let is_night = (12000..=23000).contains(&time);

                if is_night {
                    if !is_sleeping {
                        // Check distance to bed. If close enough, go to sleep
                        let dist = home_pos
                            .to_f64()
                            .squared_distance_to_vec(&self.get_entity().pos.load());
                        if dist <= 4.0 {
                            // Within 2 blocks (squared distance 4.0)
                            let (block, state) = world.get_block_and_state(&home_pos);
                            if block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_BEDS) {
                                let bed_props = BedProperties::from_state_id(state.id, block);
                                if !bed_props.occupied {
                                    // Make bed occupied
                                    BedBlock::set_occupied(
                                        true, &world, block, &home_pos, state.id,
                                    )
                                    .await;

                                    self.get_entity().set_pose(EntityPose::Sleeping);
                                    self.get_entity().send_meta_data(
                                        &[Metadata::new(
                                            TrackedData::SLEEPING_POS_ID,
                                            MetaDataType::OPTIONAL_BLOCK_POS,
                                            Some(home_pos),
                                        )],
                                        None,
                                    );
                                }
                            }
                        }
                    }
                } else if is_sleeping {
                    // It is day, wake up!
                    let (block, state) = world.get_block_and_state(&home_pos);
                    if block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_BEDS) {
                        let bed_props = BedProperties::from_state_id(state.id, block);
                        if bed_props.occupied {
                            BedBlock::set_occupied(false, &world, block, &home_pos, state.id).await;
                        }
                    }

                    self.get_entity().set_pose(EntityPose::Standing);
                    self.get_entity().send_meta_data(
                        &[Metadata::new(
                            TrackedData::SLEEPING_POS_ID,
                            MetaDataType::OPTIONAL_BLOCK_POS,
                            None::<BlockPos>,
                        )],
                        None,
                    );
                }
            }
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut pumpkin_data::item_stack::ItemStack,
    ) -> crate::entity::EntityBaseFuture<'a, bool> {
        let player = player.clone();
        Box::pin(async move {
            if item_stack.item == &Item::VILLAGER_SPAWN_EGG
                || self.mob_entity.living_entity.health.load() <= 0.0
                || self.is_trading.load(Ordering::Relaxed)
                || self.get_entity().pose.load() == EntityPose::Sleeping
            {
                return false;
            }
            if self.get_entity().age.load(Ordering::Relaxed) < 0 {
                self.set_unhappy();
                return true;
            }

            let mut offers = self.offers.lock().await;
            if offers.is_empty() {
                let data = self.villager_data.lock().await;
                if data.profession_enum() != VillagerProfession::None
                    && data.profession_enum() != VillagerProfession::Nitwit
                {
                    let prof = data.profession_enum();
                    let level = data.level.0;
                    drop(data);
                    drop(offers);
                    self.generate_trades(prof, level).await;
                    offers = self.offers.lock().await;
                } else {
                    drop(data);
                }
            }

            if offers.is_empty() {
                self.set_unhappy();
                return true;
            }
            drop(offers);

            player
                .increment_stat(
                    pumpkin_data::statistic::StatisticCategory::Custom,
                    pumpkin_data::statistic::CustomStatistic::TalkedToVillager as i32,
                    1,
                )
                .await;

            self.open_trading_screen(&player).await;

            true
        })
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::data_component_impl::{EnchantmentsImpl, StoredEnchantmentsImpl};
    use pumpkin_data::villager::VillagerTradeModifier;
    use pumpkin_util::version::JavaMinecraftVersion;

    use super::*;

    #[test]
    fn villager_data_metadata_uses_the_villager_tracker_slot() {
        let data = VillagerData::new(VillagerType::Plains, VillagerProfession::Librarian, 1);
        let metadata = Metadata::new(VILLAGER_DATA_TRACKED_ID, MetaDataType::VILLAGER_DATA, data);
        let mut bytes = Vec::new();

        metadata
            .write(&mut bytes, &JavaMinecraftVersion::V_26_2)
            .unwrap();

        assert_eq!(bytes, [19, 18, 2, 9, 1]);
    }

    #[test]
    fn unhappy_counter_metadata_uses_the_abstract_villager_tracker_slot() {
        let metadata = Metadata::new(TrackedData::UNHAPPY_COUNTER, MetaDataType::INT, VarInt(40));
        let mut bytes = Vec::new();

        metadata
            .write(&mut bytes, &JavaMinecraftVersion::V_26_2)
            .unwrap();

        assert_eq!(bytes, [18, 1, 40]);
    }

    #[test]
    fn enchanted_book_offer_has_vanilla_items_and_a_nonzero_price() {
        let (emeralds, enchanted_book, book) = enchanted_book_offer_items(&mut rand::rng());
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
    fn generated_trades_keep_dynamic_modifiers_and_secondary_costs() {
        let librarian = VillagerProfession::Librarian.trade_set(1).unwrap();
        let enchanted_book = librarian
            .trades
            .iter()
            .find(|trade| trade.modifier == VillagerTradeModifier::EnchantRandomly)
            .unwrap();
        assert!(enchanted_book.wants.item == &Item::EMERALD);
        assert!(enchanted_book.wants_b.unwrap().item == &Item::BOOK);
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
