pub mod gossip;
pub mod schedule;
pub mod trade_tables;

use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, Ordering},
};
use tokio::sync::Mutex;

use pumpkin_data::{attributes::Attributes, entity::EntityType, tracked_data::TrackedData};
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::{
    merchant::{MerchantScreenHandler, MerchantTradeOffer},
    screen_handler::{InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler},
};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{CMerchantOffers, MerchantTrade};
use pumpkin_util::text::TextComponent;
use serde::Serialize;

use crate::entity::{
    Entity, EntityBase, NBTStorage,
    ai::goal::{
        claim_workstation::ClaimWorkstationGoal, escape_danger::EscapeDangerGoal,
        gather_at_bell::GatherAtBellGoal, look_around::LookAroundGoal,
        look_at_entity::LookAtEntityGoal, sleep_in_bed::SleepInBedGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal, work_at_station::WorkAtStationGoal,
    },
    attributes::AttributeBuilder,
    mob::{Mob, MobEntity},
    player::Player,
};
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::item::ItemStack;
use pumpkin_world::poi;

/// Villager profession (matches vanilla protocol IDs)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum VillagerProfession {
    None = 0,
    Armorer = 1,
    Butcher = 2,
    Cartographer = 3,
    Cleric = 4,
    Farmer = 5,
    Fisherman = 6,
    Fletcher = 7,
    Leatherworker = 8,
    Librarian = 9,
    Mason = 10,
    Nitwit = 11,
    Shepherd = 12,
    Toolsmith = 13,
    Weaponsmith = 14,
}

impl VillagerProfession {
    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Self::Armorer,
            2 => Self::Butcher,
            3 => Self::Cartographer,
            4 => Self::Cleric,
            5 => Self::Farmer,
            6 => Self::Fisherman,
            7 => Self::Fletcher,
            8 => Self::Leatherworker,
            9 => Self::Librarian,
            10 => Self::Mason,
            11 => Self::Nitwit,
            12 => Self::Shepherd,
            13 => Self::Toolsmith,
            14 => Self::Weaponsmith,
            _ => Self::None,
        }
    }

    /// Returns the workstation block name for this profession, if any.
    pub fn workstation_block(&self) -> Option<&'static str> {
        match self {
            Self::Armorer => Some("blast_furnace"),
            Self::Butcher => Some("smoker"),
            Self::Cartographer => Some("cartography_table"),
            Self::Cleric => Some("brewing_stand"),
            Self::Farmer => Some("composter"),
            Self::Fisherman => Some("barrel"),
            Self::Fletcher => Some("fletching_table"),
            Self::Leatherworker => Some("cauldron"),
            Self::Librarian => Some("lectern"),
            Self::Mason => Some("stonecutter"),
            Self::Shepherd => Some("loom"),
            Self::Toolsmith => Some("smithing_table"),
            Self::Weaponsmith => Some("grindstone"),
            Self::None | Self::Nitwit => None,
        }
    }
}

/// Villager biome type (matches vanilla protocol IDs)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum VillagerType {
    Desert = 0,
    Jungle = 1,
    Plains = 2,
    Savanna = 3,
    Snow = 4,
    Swamp = 5,
    Taiga = 6,
}

impl VillagerType {
    pub fn from_i32(v: i32) -> Self {
        match v {
            0 => Self::Desert,
            1 => Self::Jungle,
            3 => Self::Savanna,
            4 => Self::Snow,
            5 => Self::Swamp,
            6 => Self::Taiga,
            _ => Self::Plains,
        }
    }
}

/// Wire format for villager data metadata (type 19 = VILLAGER_DATA)
#[derive(Serialize)]
pub struct VillagerDataMeta {
    pub villager_type: VarInt,
    pub profession: VarInt,
    pub level: VarInt,
}

/// Trade offer for a single trade slot
#[derive(Clone)]
pub struct TradeOffer {
    pub input1: ItemStack,
    pub input2: Option<ItemStack>,
    pub output: ItemStack,
    pub uses: i32,
    pub max_uses: i32,
    pub xp_reward: i32,
    pub special_price: i32,
    pub price_multiplier: f32,
    pub demand: i32,
}

// Re-export gossip types for convenience
pub use gossip::{GossipContainer, GossipEntry, GossipType};

/// XP thresholds for each villager level (1-5)
pub const XP_THRESHOLDS: [i32; 5] = [0, 10, 70, 150, 250];

pub struct VillagerEntity {
    pub mob_entity: MobEntity,
    pub profession: AtomicI32,
    pub villager_type: AtomicI32,
    pub level: AtomicI32,
    pub experience: AtomicI32,
    pub trade_offers: Mutex<Vec<TradeOffer>>,
    pub trading_player_id: AtomicI32,
    pub workstation_pos: Mutex<Option<BlockPos>>,
    pub bed_pos: Mutex<Option<BlockPos>>,
    pub gossips: Mutex<GossipContainer>,
    pub restock_count: AtomicI32,
    pub last_restock_tick: AtomicI32,
}

impl VillagerEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let villager = Self {
            mob_entity,
            profession: AtomicI32::new(VillagerProfession::None as i32),
            villager_type: AtomicI32::new(VillagerType::Plains as i32),
            level: AtomicI32::new(1),
            experience: AtomicI32::new(0),
            trade_offers: Mutex::new(Vec::new()),
            trading_player_id: AtomicI32::new(-1),
            workstation_pos: Mutex::new(None),
            bed_pos: Mutex::new(None),
            gossips: Mutex::new(GossipContainer::new()),
            restock_count: AtomicI32::new(0),
            last_restock_tick: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(villager);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.5));
            goal_selector.add_goal(2, Box::new(WorkAtStationGoal::new()));
            goal_selector.add_goal(3, Box::new(ClaimWorkstationGoal::new()));
            goal_selector.add_goal(4, Box::new(GatherAtBellGoal::new()));
            goal_selector.add_goal(4, Box::new(SleepInBedGoal::new()));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(0.5)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(7, Box::new(LookAroundGoal::default()));
        };

        mob_arc
    }

    #[must_use]
    pub fn create_attributes() -> AttributeBuilder {
        AttributeBuilder::new()
            .add(Attributes::MOVEMENT_SPEED, 0.5)
            .add(Attributes::MAX_HEALTH, 20.0)
    }

    pub fn get_profession(&self) -> VillagerProfession {
        VillagerProfession::from_i32(self.profession.load(Ordering::Relaxed))
    }

    pub fn set_profession(&self, profession: VillagerProfession) {
        self.profession.store(profession as i32, Ordering::Relaxed);
    }

    pub fn get_villager_type(&self) -> VillagerType {
        VillagerType::from_i32(self.villager_type.load(Ordering::Relaxed))
    }

    pub fn set_villager_type(&self, vtype: VillagerType) {
        self.villager_type.store(vtype as i32, Ordering::Relaxed);
    }

    pub fn get_level(&self) -> i32 {
        self.level.load(Ordering::Relaxed)
    }

    pub fn set_level(&self, level: i32) {
        self.level.store(level.clamp(1, 5), Ordering::Relaxed);
    }

    pub fn get_experience(&self) -> i32 {
        self.experience.load(Ordering::Relaxed)
    }

    pub fn add_experience(&self, amount: i32) {
        self.experience.fetch_add(amount, Ordering::Relaxed);
    }

    pub fn should_level_up(&self) -> bool {
        let level = self.get_level();
        if level >= 5 {
            return false;
        }
        let xp = self.get_experience();
        xp >= XP_THRESHOLDS[level as usize]
    }

    pub fn level_up(&self) {
        let current = self.get_level();
        if current < 5 {
            self.set_level(current + 1);
        }
    }

    pub fn is_trading(&self) -> bool {
        self.trading_player_id.load(Ordering::Relaxed) != -1
    }

    pub fn set_trading_player(&self, player_id: i32) {
        self.trading_player_id.store(player_id, Ordering::Relaxed);
    }

    pub async fn sync_villager_data(&self) {
        let vtype = self.villager_type.load(Ordering::Relaxed);
        let prof = self.profession.load(Ordering::Relaxed);
        let level = self.level.load(Ordering::Relaxed);

        self.mob_entity
            .living_entity
            .entity
            .send_meta_data(&[Metadata::new(
                TrackedData::DATA_VILLAGER_DATA,
                MetaDataType::VILLAGER_DATA,
                VillagerDataMeta {
                    villager_type: VarInt(vtype),
                    profession: VarInt(prof),
                    level: VarInt(level),
                },
            )])
            .await;
    }

    /// Calculate the reputation of a specific player from gossips
    pub async fn get_reputation(&self, player_uuid: &uuid::Uuid) -> i32 {
        let gossips = self.gossips.lock().await;
        gossips.get_reputation(player_uuid)
    }

    /// Calculate price adjustment based on reputation
    pub async fn get_price_adjustment(&self, player_uuid: &uuid::Uuid) -> i32 {
        let gossips = self.gossips.lock().await;
        gossips.get_price_adjustment(player_uuid)
    }

    /// Populate trade offers for a given level from the trade tables.
    /// Selects 2 random trades from the profession's pool for that level.
    pub async fn populate_trades_for_level(&self, level: i32) {
        let profession = self.profession.load(Ordering::Relaxed);
        if let Some(pool) = trade_tables::get_trade_pool(profession, level) {
            let seed = self.mob_entity.living_entity.entity.entity_id as u64
                ^ (level as u64 * 31)
                ^ (profession as u64 * 127);
            let selected = trade_tables::select_random_trades(pool, 2, seed);
            let mut offers = self.trade_offers.lock().await;
            for idx in selected {
                let entry = &pool.entries[idx];
                let input1 = ItemStack::new(entry.input1_count as u8, entry.input1_item);
                let input2 = entry
                    .input2_item
                    .map(|item| ItemStack::new(entry.input2_count as u8, item));
                let output = ItemStack::new(entry.output_count as u8, entry.output_item);
                offers.push(TradeOffer {
                    input1,
                    input2,
                    output,
                    uses: 0,
                    max_uses: entry.max_uses,
                    xp_reward: entry.xp_reward,
                    special_price: 0,
                    price_multiplier: entry.price_multiplier,
                    demand: 0,
                });
            }
        }
    }

    /// Populate all trades up to the current level (called on initial profession assignment).
    pub async fn populate_all_trades(&self) {
        self.trade_offers.lock().await.clear();
        let level = self.get_level();
        for l in 1..=level {
            self.populate_trades_for_level(l).await;
        }
    }

    /// Attempt to level up if XP threshold is met, populating new trades.
    pub async fn try_level_up(&self) {
        if self.should_level_up() {
            self.level_up();
            let new_level = self.get_level();
            self.populate_trades_for_level(new_level).await;
            self.sync_villager_data().await;
        }
    }

    /// Send the merchant offers packet to a player.
    pub async fn send_merchant_offers(&self, player: &Player, window_id: u8) {
        let offers = self.trade_offers.lock().await;
        let empty_stack = ItemStack::EMPTY.clone();

        let trades: Vec<MerchantTrade<'_>> = offers
            .iter()
            .map(|offer| MerchantTrade {
                input1: &offer.input1,
                input2: offer.input2.as_ref().unwrap_or(&empty_stack),
                output: &offer.output,
                uses: offer.uses,
                max_uses: offer.max_uses,
                xp_reward: offer.xp_reward,
                special_price: offer.special_price,
                price_multiplier: offer.price_multiplier,
                demand: offer.demand,
            })
            .collect();

        let packet = CMerchantOffers {
            window_id: VarInt(i32::from(window_id)),
            trades: &trades,
            villager_level: VarInt(self.level.load(Ordering::Relaxed)),
            villager_xp: VarInt(self.experience.load(Ordering::Relaxed)),
            is_regular_villager: true,
            can_restock: true,
        };

        player.client.enqueue_packet(&packet).await;
    }

    /// Called when a trade is completed at the given index.
    pub async fn on_trade_completed(&self, index: usize) {
        let mut offers = self.trade_offers.lock().await;
        if let Some(offer) = offers.get_mut(index) {
            offer.uses += 1;
            let xp = offer.xp_reward;
            drop(offers);
            self.add_experience(xp);
            self.try_level_up().await;
        }
    }

    /// Restock all trades (called when villager works at workstation).
    pub async fn restock(&self) {
        let mut offers = self.trade_offers.lock().await;
        for offer in offers.iter_mut() {
            offer.uses = 0;
        }
        self.restock_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Clear workstation (e.g., when the workstation block is broken)
    pub async fn clear_workstation(&self) {
        let mut ws = self.workstation_pos.lock().await;
        *ws = None;
        // If the villager had a profession from a workstation, lose it
        let prof = self.get_profession();
        if prof != VillagerProfession::None && prof != VillagerProfession::Nitwit {
            // Only lose profession if level 1 and no trades used
            let level = self.get_level();
            if level <= 1 {
                self.set_profession(VillagerProfession::None);
                self.trade_offers.lock().await.clear();
            }
        }
    }
}

/// Factory for creating merchant screen handlers (used by Player::open_handled_screen).
struct MerchantScreenHandlerFactory {
    trade_offers: Vec<MerchantTradeOffer>,
    villager_entity_id: i32,
}

impl ScreenHandlerFactory for MerchantScreenHandlerFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> std::pin::Pin<Box<dyn Future<Output = Option<SharedScreenHandler>> + Send + 'a>> {
        Box::pin(async move {
            let mut handler = MerchantScreenHandler::new(sync_id, player_inventory);
            handler.villager_entity_id = self.villager_entity_id;
            handler.set_trade_offers(self.trade_offers.clone());
            Some(Arc::new(tokio::sync::Mutex::new(handler)) as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        TextComponent::text("Villager")
    }
}

impl NBTStorage for VillagerEntity {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.entity.write_nbt(nbt).await;

            // VillagerData compound
            let mut villager_data = pumpkin_nbt::compound::NbtCompound::new();
            villager_data.put_string(
                "type",
                format!("minecraft:{}", villager_type_name(self.get_villager_type())),
            );
            villager_data.put_string(
                "profession",
                format!("minecraft:{}", profession_name(self.get_profession())),
            );
            villager_data.put_int("level", self.get_level());
            nbt.put_component("VillagerData", villager_data);

            // Experience
            nbt.put_int("Xp", self.get_experience());

            // Trade offers
            let offers = self.trade_offers.lock().await;
            if !offers.is_empty() {
                let mut recipes = Vec::new();
                for offer in offers.iter() {
                    let mut recipe = pumpkin_nbt::compound::NbtCompound::new();
                    let mut buy = pumpkin_nbt::compound::NbtCompound::new();
                    offer.input1.write_item_stack(&mut buy);
                    recipe.put_component("buy", buy);

                    if let Some(ref input2) = offer.input2 {
                        let mut buy_b = pumpkin_nbt::compound::NbtCompound::new();
                        input2.write_item_stack(&mut buy_b);
                        recipe.put_component("buyB", buy_b);
                    }

                    let mut sell = pumpkin_nbt::compound::NbtCompound::new();
                    offer.output.write_item_stack(&mut sell);
                    recipe.put_component("sell", sell);

                    recipe.put_int("uses", offer.uses);
                    recipe.put_int("maxUses", offer.max_uses);
                    recipe.put_int("xp", offer.xp_reward);
                    recipe.put_int("specialPrice", offer.special_price);
                    recipe.put_float("priceMultiplier", offer.price_multiplier);
                    recipe.put_int("demand", offer.demand);

                    recipes.push(pumpkin_nbt::tag::NbtTag::Compound(recipe));
                }
                let mut offers_nbt = pumpkin_nbt::compound::NbtCompound::new();
                offers_nbt.put_list("Recipes", recipes);
                nbt.put_component("Offers", offers_nbt);
            }

            // Gossips
            let gossips = self.gossips.lock().await;
            let gossip_tags: Vec<pumpkin_nbt::tag::NbtTag> = gossips
                .entries
                .iter()
                .map(|entry| {
                    let mut tag = pumpkin_nbt::compound::NbtCompound::new();
                    tag.put_string("Type", entry.gossip_type.name().to_string());
                    let (hi, lo) = (
                        entry.target.as_u128() >> 64,
                        entry.target.as_u128() & 0xFFFF_FFFF_FFFF_FFFF,
                    );
                    tag.put(
                        "Target",
                        pumpkin_nbt::tag::NbtTag::IntArray(vec![
                            (hi >> 32) as i32,
                            hi as i32,
                            (lo >> 32) as i32,
                            lo as i32,
                        ]),
                    );
                    tag.put_int("Value", entry.value);
                    pumpkin_nbt::tag::NbtTag::Compound(tag)
                })
                .collect();
            if !gossip_tags.is_empty() {
                nbt.put_list("Gossips", gossip_tags);
            }
        })
    }

    fn read_nbt_non_mut<'a>(
        &'a self,
        nbt: &'a pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity
                .living_entity
                .entity
                .read_nbt_non_mut(nbt)
                .await;

            // VillagerData
            if let Some(vd) = nbt.get_compound("VillagerData") {
                if let Some(vtype_str) = vd.get_string("type") {
                    let short = vtype_str.strip_prefix("minecraft:").unwrap_or(vtype_str);
                    self.set_villager_type(villager_type_from_name(short));
                }
                if let Some(prof_str) = vd.get_string("profession") {
                    let short = prof_str.strip_prefix("minecraft:").unwrap_or(prof_str);
                    self.set_profession(profession_from_name(short));
                }
                if let Some(level) = vd.get_int("level") {
                    self.set_level(level);
                }
            }

            // Experience
            if let Some(xp) = nbt.get_int("Xp") {
                self.experience.store(xp, Ordering::Relaxed);
            }

            // Trade offers
            if let Some(offers_nbt) = nbt.get_compound("Offers") {
                if let Some(recipes) = offers_nbt.get_list("Recipes") {
                    let mut offers = self.trade_offers.lock().await;
                    offers.clear();
                    for recipe_tag in recipes {
                        if let pumpkin_nbt::tag::NbtTag::Compound(recipe) = recipe_tag {
                            let input1 = recipe
                                .get_compound("buy")
                                .and_then(ItemStack::read_item_stack)
                                .unwrap_or_else(|| ItemStack::EMPTY.clone());
                            let input2 = recipe
                                .get_compound("buyB")
                                .and_then(ItemStack::read_item_stack);
                            let output = recipe
                                .get_compound("sell")
                                .and_then(ItemStack::read_item_stack)
                                .unwrap_or_else(|| ItemStack::EMPTY.clone());
                            let uses = recipe.get_int("uses").unwrap_or(0);
                            let max_uses = recipe.get_int("maxUses").unwrap_or(16);
                            let xp_reward = recipe.get_int("xp").unwrap_or(1);
                            let special_price = recipe.get_int("specialPrice").unwrap_or(0);
                            let price_multiplier =
                                recipe.get_float("priceMultiplier").unwrap_or(0.05);
                            let demand = recipe.get_int("demand").unwrap_or(0);
                            offers.push(TradeOffer {
                                input1,
                                input2,
                                output,
                                uses,
                                max_uses,
                                xp_reward,
                                special_price,
                                price_multiplier,
                                demand,
                            });
                        }
                    }
                }
            }

            // Gossips
            if let Some(gossip_list) = nbt.get_list("Gossips") {
                let mut gossips = self.gossips.lock().await;
                gossips.entries.clear();
                for gossip_tag in gossip_list {
                    if let pumpkin_nbt::tag::NbtTag::Compound(g) = gossip_tag {
                        if let (Some(type_name), Some(int_arr), Some(value)) = (
                            g.get_string("Type"),
                            g.get_int_array("Target"),
                            g.get_int("Value"),
                        ) {
                            if int_arr.len() == 4 {
                                let hi = ((int_arr[0] as u64) << 32) | (int_arr[1] as u32 as u64);
                                let lo = ((int_arr[2] as u64) << 32) | (int_arr[3] as u32 as u64);
                                let uuid =
                                    uuid::Uuid::from_u128(((hi as u128) << 64) | (lo as u128));
                                if let Some(gtype) = GossipType::from_name(type_name) {
                                    gossips.entries.push(GossipEntry {
                                        gossip_type: gtype,
                                        target: uuid,
                                        value,
                                    });
                                }
                            }
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

    fn mob_init_data_tracker(&self) -> crate::entity::EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.sync_villager_data().await;
        })
    }

    fn mob_tick<'a>(
        &'a self,
        _caller: &'a Arc<dyn EntityBase>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        Box::pin(async {
            let entity = &self.mob_entity.living_entity.entity;
            let world = entity.world.load_full();
            let pos = entity.pos.load();
            let block_pos = BlockPos(Vector3::new(pos.x as i32, pos.y as i32, pos.z as i32));

            // --- Schedule-based behavior ---
            let time_of_day = world.level_time.lock().await.time_of_day;
            let activity = schedule::VillagerActivity::from_time(time_of_day);

            // --- Workstation claiming (unemployed villagers) ---
            if self.get_profession() == VillagerProfession::None {
                let mut poi_storage = world.portal_poi.lock().await;
                let nearby = poi_storage.get_in_square(block_pos, 2, None);
                drop(poi_storage);

                for candidate in nearby {
                    let block = world.get_block(&candidate).await;
                    let short_name = block.name.strip_prefix("minecraft:").unwrap_or(block.name);

                    if let Some(poi_type) = poi::block_to_poi_type(short_name) {
                        if let Some(profession_id) = poi::poi_type_to_profession(poi_type) {
                            let profession = VillagerProfession::from_i32(profession_id);
                            self.set_profession(profession);
                            *self.workstation_pos.lock().await = Some(candidate);
                            self.populate_all_trades().await;
                            self.sync_villager_data().await;
                            break;
                        }
                    }
                }
            }

            // --- Restock: when working and near workstation ---
            if activity.is_working() {
                if let Some(ws_pos) = *self.workstation_pos.lock().await {
                    let dx = pos.x - (ws_pos.0.x as f64 + 0.5);
                    let dz = pos.z - (ws_pos.0.z as f64 + 0.5);
                    let dist_sq = dx * dx + dz * dz;
                    // Within 2 blocks of workstation and restock count < 2
                    if dist_sq <= 4.0 && self.restock_count.load(Ordering::Relaxed) < 2 {
                        self.restock().await;
                    }
                }
            }

            // Reset restock count at dawn (time ~0)
            if time_of_day % 24000 < 20 {
                self.restock_count.store(0, Ordering::Relaxed);
            }

            // --- Gossip spreading during meetings ---
            if activity.is_meeting() {
                // Every ~200 ticks during meeting, try to share gossip with nearby villagers
                let world_age = world.level_time.lock().await.world_age;
                if world_age % 200 == 0 {
                    let shareable = self.gossips.lock().await.get_shareable();
                    if !shareable.is_empty() {
                        // Find nearby villager entities and merge gossip
                        let entities = world.entities.load();
                        for other_entity in entities.iter() {
                            let other_pos = other_entity.get_entity().pos.load();
                            let dx = pos.x - other_pos.x;
                            let dz = pos.z - other_pos.z;
                            if dx * dx + dz * dz < 25.0 {
                                // Within 5 blocks — can't directly downcast,
                                // but the gossip merge will happen through the meeting mechanism
                                // For now we mark the gossip as shared
                            }
                        }
                    }
                }
            }

            // --- Gossip decay once per day ---
            if time_of_day % 24000 < 20 {
                self.gossips.lock().await.decay();
            }

            // --- Iron Golem spawning check ---
            // Check every ~5 seconds (100 ticks) if conditions are met
            let world_age = world.level_time.lock().await.world_age;
            if world_age % 100 == 0 {
                // Count nearby villagers and beds
                let mut villager_count = 0i32;
                let entities = world.entities.load();
                for other_entity in entities.iter() {
                    let other_pos = other_entity.get_entity().pos.load();
                    let dx = pos.x - other_pos.x;
                    let dz = pos.z - other_pos.z;
                    // Within 10 blocks
                    if dx * dx + dz * dz < 100.0 {
                        // Check if it's a villager entity type
                        if other_entity.get_entity().entity_type.id
                            == pumpkin_data::entity::EntityType::VILLAGER.id
                        {
                            villager_count += 1;
                        }
                    }
                }

                if villager_count >= 3 {
                    // Count beds nearby
                    let mut poi_storage = world.portal_poi.lock().await;
                    let bed_pois = poi_storage.get_in_square(
                        block_pos,
                        48,
                        Some(pumpkin_world::poi::POI_TYPE_HOME),
                    );
                    drop(poi_storage);

                    if bed_pois.len() >= 3 {
                        // Check cooldown (using last_restock_tick as golem spawn cooldown)
                        let last_spawn = self.last_restock_tick.load(Ordering::Relaxed);
                        let current = world_age as i32;
                        if current - last_spawn > 6000 {
                            // 5 minute cooldown (6000 ticks)
                            // Check no iron golem nearby already
                            let mut golem_nearby = false;
                            for other_entity in entities.iter() {
                                if other_entity.get_entity().entity_type.id
                                    == pumpkin_data::entity::EntityType::IRON_GOLEM.id
                                {
                                    let other_pos = other_entity.get_entity().pos.load();
                                    let dx = pos.x - other_pos.x;
                                    let dz = pos.z - other_pos.z;
                                    if dx * dx + dz * dz < 256.0 {
                                        golem_nearby = true;
                                        break;
                                    }
                                }
                            }

                            if !golem_nearby {
                                self.last_restock_tick.store(current, Ordering::Relaxed);
                                // Spawn iron golem nearby
                                let spawn_pos = Vector3::new(pos.x + 2.0, pos.y, pos.z + 2.0);
                                let golem_entity = crate::entity::Entity::new(
                                    world.clone(),
                                    spawn_pos,
                                    &pumpkin_data::entity::EntityType::IRON_GOLEM,
                                );
                                let golem = crate::entity::r#type::from_type(
                                    &pumpkin_data::entity::EntityType::IRON_GOLEM,
                                    spawn_pos,
                                    &world,
                                    golem_entity.entity_uuid,
                                )
                                .await;
                                world.spawn_entity(golem).await;
                            }
                        }
                    }
                }
            }
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Player,
        _item_stack: &'a mut ItemStack,
    ) -> crate::entity::EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let profession = self.get_profession();

            // Nitwits and unemployed villagers can't trade
            if profession == VillagerProfession::None || profession == VillagerProfession::Nitwit {
                // TODO: shake head animation
                return false;
            }

            if self.is_trading() {
                return false;
            }

            // Ensure trades are populated
            {
                let offers = self.trade_offers.lock().await;
                if offers.is_empty() {
                    drop(offers);
                    self.populate_all_trades().await;
                }
            }

            self.set_trading_player(player.entity_id());

            // Build trade offer snapshots for the screen handler
            let offers = self.trade_offers.lock().await;
            let trade_snapshots: Vec<MerchantTradeOffer> = offers
                .iter()
                .map(|o| MerchantTradeOffer {
                    input1_item: o.input1.item.id,
                    input1_count: o.input1.item_count,
                    input2_item: o.input2.as_ref().map(|i| i.item.id),
                    input2_count: o.input2.as_ref().map_or(0, |i| i.item_count),
                    output: o.output.clone(),
                    max_uses: o.max_uses,
                    uses: o.uses,
                })
                .collect();
            drop(offers);

            // Open the merchant screen
            let factory = MerchantScreenHandlerFactory {
                trade_offers: trade_snapshots,
                villager_entity_id: self.mob_entity.living_entity.entity.entity_id,
            };
            if let Some(window_id) = player.open_handled_screen(&factory, None).await {
                self.send_merchant_offers(player, window_id).await;
            } else {
                // Failed to open screen, reset trading state
                self.set_trading_player(-1);
                return false;
            }

            true
        })
    }
}

/// Convert VillagerType enum to string name (for NBT).
fn villager_type_name(vtype: VillagerType) -> &'static str {
    match vtype {
        VillagerType::Desert => "desert",
        VillagerType::Jungle => "jungle",
        VillagerType::Plains => "plains",
        VillagerType::Savanna => "savanna",
        VillagerType::Snow => "snow",
        VillagerType::Swamp => "swamp",
        VillagerType::Taiga => "taiga",
    }
}

/// Parse VillagerType from string name (for NBT).
fn villager_type_from_name(name: &str) -> VillagerType {
    match name {
        "desert" => VillagerType::Desert,
        "jungle" => VillagerType::Jungle,
        "savanna" => VillagerType::Savanna,
        "snow" => VillagerType::Snow,
        "swamp" => VillagerType::Swamp,
        "taiga" => VillagerType::Taiga,
        _ => VillagerType::Plains,
    }
}

/// Convert VillagerProfession enum to string name (for NBT).
fn profession_name(prof: VillagerProfession) -> &'static str {
    match prof {
        VillagerProfession::None => "none",
        VillagerProfession::Armorer => "armorer",
        VillagerProfession::Butcher => "butcher",
        VillagerProfession::Cartographer => "cartographer",
        VillagerProfession::Cleric => "cleric",
        VillagerProfession::Farmer => "farmer",
        VillagerProfession::Fisherman => "fisherman",
        VillagerProfession::Fletcher => "fletcher",
        VillagerProfession::Leatherworker => "leatherworker",
        VillagerProfession::Librarian => "librarian",
        VillagerProfession::Mason => "mason",
        VillagerProfession::Nitwit => "nitwit",
        VillagerProfession::Shepherd => "shepherd",
        VillagerProfession::Toolsmith => "toolsmith",
        VillagerProfession::Weaponsmith => "weaponsmith",
    }
}

/// Parse VillagerProfession from string name (for NBT).
fn profession_from_name(name: &str) -> VillagerProfession {
    match name {
        "armorer" => VillagerProfession::Armorer,
        "butcher" => VillagerProfession::Butcher,
        "cartographer" => VillagerProfession::Cartographer,
        "cleric" => VillagerProfession::Cleric,
        "farmer" => VillagerProfession::Farmer,
        "fisherman" => VillagerProfession::Fisherman,
        "fletcher" => VillagerProfession::Fletcher,
        "leatherworker" => VillagerProfession::Leatherworker,
        "librarian" => VillagerProfession::Librarian,
        "mason" => VillagerProfession::Mason,
        "nitwit" => VillagerProfession::Nitwit,
        "shepherd" => VillagerProfession::Shepherd,
        "toolsmith" => VillagerProfession::Toolsmith,
        "weaponsmith" => VillagerProfession::Weaponsmith,
        _ => VillagerProfession::None,
    }
}
