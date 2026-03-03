pub mod gossip;
pub mod inventory;
pub mod schedule;
pub mod trade_tables;

use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, Ordering},
};
use tokio::sync::Mutex;

use pumpkin_data::sound::Sound;
use pumpkin_data::{
    attributes::Attributes,
    entity::{EntityStatus, EntityType},
    tracked_data::TrackedData,
};
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
        breed::BreedGoal, claim_workstation::ClaimWorkstationGoal, escape_danger::EscapeDangerGoal,
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
    #[must_use]
    pub const fn from_i32(v: i32) -> Self {
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
    #[must_use]
    pub const fn workstation_block(&self) -> Option<&'static str> {
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
    #[must_use]
    pub const fn from_i32(v: i32) -> Self {
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

/// Wire format for villager data metadata (type 19 = `VILLAGER_DATA`)
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

/// Vanilla: `getMinAmbientSoundDelay()` returns 80 for most mobs
const MIN_AMBIENT_SOUND_DELAY: i32 = 80;

pub struct VillagerEntity {
    pub mob_entity: MobEntity,
    pub profession: AtomicI32,
    pub villager_type: AtomicI32,
    pub level: AtomicI32,
    pub experience: AtomicI32,
    pub trade_offers: Mutex<Vec<TradeOffer>>,
    pub trading_player_id: Arc<AtomicI32>,
    pub workstation_pos: Mutex<Option<BlockPos>>,
    pub bed_pos: Mutex<Option<BlockPos>>,
    pub gossips: Mutex<GossipContainer>,
    pub restock_count: AtomicI32,
    pub last_restock_tick: std::sync::atomic::AtomicI64,
    pub last_golem_spawn_tick: std::sync::atomic::AtomicI64,
    last_reset_day: std::sync::atomic::AtomicI64,
    ambient_sound_chance: AtomicI32,
    head_shake_ticks: AtomicI32,
    pub inventory: Mutex<inventory::VillagerInventory>,
    /// Receives trade completion indices from the merchant screen handler.
    trade_completion_rx: Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<usize>>>,
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
            trading_player_id: Arc::new(AtomicI32::new(-1)),
            workstation_pos: Mutex::new(None),
            bed_pos: Mutex::new(None),
            gossips: Mutex::new(GossipContainer::new()),
            restock_count: AtomicI32::new(0),
            last_restock_tick: std::sync::atomic::AtomicI64::new(0),
            last_golem_spawn_tick: std::sync::atomic::AtomicI64::new(0),
            last_reset_day: std::sync::atomic::AtomicI64::new(-1),
            ambient_sound_chance: AtomicI32::new(MIN_AMBIENT_SOUND_DELAY),
            head_shake_ticks: AtomicI32::new(0),
            inventory: Mutex::new(inventory::VillagerInventory::new()),
            trade_completion_rx: Mutex::new(None),
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
            goal_selector.add_goal(2, BreedGoal::new(0.5));
            goal_selector.add_goal(3, Box::new(WorkAtStationGoal::new()));
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

    #[must_use]
    pub fn get_profession(&self) -> VillagerProfession {
        VillagerProfession::from_i32(self.profession.load(Ordering::Acquire))
    }

    pub fn set_profession(&self, profession: VillagerProfession) {
        self.profession.store(profession as i32, Ordering::Release);
    }

    #[must_use]
    pub fn get_villager_type(&self) -> VillagerType {
        VillagerType::from_i32(self.villager_type.load(Ordering::Relaxed))
    }

    pub fn set_villager_type(&self, vtype: VillagerType) {
        self.villager_type.store(vtype as i32, Ordering::Relaxed);
    }

    #[must_use]
    pub fn get_level(&self) -> i32 {
        self.level.load(Ordering::Acquire)
    }

    pub fn set_level(&self, level: i32) {
        self.level.store(level.clamp(1, 5), Ordering::Release);
    }

    #[must_use]
    pub fn get_experience(&self) -> i32 {
        self.experience.load(Ordering::Acquire)
    }

    pub fn add_experience(&self, amount: i32) {
        self.experience.fetch_add(amount, Ordering::Release);
    }

    #[must_use]
    pub fn should_level_up(&self) -> bool {
        let level = self.get_level();
        if level >= 5 {
            return false;
        }
        let xp = self.get_experience();
        xp >= XP_THRESHOLDS[level as usize]
    }

    pub fn level_up(&self) -> bool {
        loop {
            let current = self.level.load(Ordering::Acquire);
            if current >= 5 {
                return false;
            }
            if self
                .level
                .compare_exchange(current, current + 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return true;
            }
        }
    }

    #[must_use]
    pub fn is_trading(&self) -> bool {
        self.trading_player_id.load(Ordering::Acquire) != -1
    }

    pub fn set_trading_player(&self, player_id: i32) {
        self.trading_player_id.store(player_id, Ordering::Release);
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
        if self.should_level_up() && self.level_up() {
            let new_level = self.get_level();
            self.populate_trades_for_level(new_level).await;
            self.sync_villager_data().await;
            // Happy particles on level up
            self.send_entity_status(EntityStatus::AddVillagerHappyParticles)
                .await;
        }
    }

    /// Send the merchant offers packet to a player with demand-based pricing.
    pub async fn send_merchant_offers(&self, player: &Player, window_id: u8) {
        // Clone trade data and get gossip adjustment, then drop both locks
        // before building packet and sending over the network
        let offers: Vec<TradeOffer> = self.trade_offers.lock().await.clone();
        let price_adj = self.get_price_adjustment(&player.gameprofile.id).await;
        let villager_level = self.level.load(Ordering::Relaxed);
        let villager_xp = self.experience.load(Ordering::Relaxed);

        let trades: Vec<MerchantTrade<'_>> = offers
            .iter()
            .map(|offer| {
                // Vanilla demand formula: demand_adj = floor(price_multiplier * max(0, demand) * base_price)
                let base_price = offer.input1.item_count as i32;
                let demand_adj =
                    (offer.price_multiplier * offer.demand.max(0) as f32 * base_price as f32)
                        .floor() as i32;
                let adjusted_special =
                    (offer.special_price + demand_adj + price_adj).max(1 - base_price);

                MerchantTrade {
                    input1: &offer.input1,
                    input2: offer.input2.as_ref().unwrap_or(ItemStack::EMPTY),
                    output: &offer.output,
                    uses: offer.uses,
                    max_uses: offer.max_uses,
                    xp_reward: offer.xp_reward,
                    special_price: adjusted_special,
                    price_multiplier: offer.price_multiplier,
                    demand: offer.demand,
                }
            })
            .collect();

        let packet = CMerchantOffers {
            window_id: VarInt(i32::from(window_id)),
            trades: &trades,
            villager_level: VarInt(villager_level),
            villager_xp: VarInt(villager_xp),
            is_regular_villager: true,
            can_restock: true,
        };

        player.client.enqueue_packet(&packet).await;
    }

    /// Called when a trade is completed at the given index.
    pub async fn on_trade_completed(&self, index: usize, player_uuid: Option<uuid::Uuid>) {
        let mut offers = self.trade_offers.lock().await;
        if let Some(offer) = offers.get_mut(index) {
            offer.uses += 1;
            offer.demand += 1;
            let xp = offer.xp_reward;
            drop(offers);
            self.add_experience(xp);
            self.try_level_up().await;

            // Trade sound
            self.mob_entity
                .living_entity
                .entity
                .play_sound(Sound::EntityVillagerYes)
                .await;

            // Add trading gossip for the player
            if let Some(uuid) = player_uuid {
                self.gossips.lock().await.add(GossipType::Trading, uuid, 2);
            }
        }
    }

    /// Restock all trades (called when villager works at workstation).
    pub async fn restock(&self) {
        let mut offers = self.trade_offers.lock().await;
        for offer in offers.iter_mut() {
            // Vanilla demand formula: demand = max(0, demand + uses - max_uses)
            offer.demand = (offer.demand + offer.uses - offer.max_uses).max(0);
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
            // Only lose profession if level 1 and no trades have ever been used
            let level = self.get_level();
            if level <= 1 {
                let offers = self.trade_offers.lock().await;
                let any_used = offers.iter().any(|o| o.uses > 0);
                drop(offers);
                if !any_used {
                    self.set_profession(VillagerProfession::None);
                    self.trade_offers.lock().await.clear();
                }
            }
        }
    }

    /// Tick ambient sound countdown; plays ambient sound when timer reaches zero.
    async fn tick_ambient_sound(&self) {
        let chance = self.ambient_sound_chance.fetch_sub(1, Ordering::Relaxed);
        if chance <= 0 {
            self.ambient_sound_chance
                .store(MIN_AMBIENT_SOUND_DELAY, Ordering::Relaxed);
            self.mob_entity
                .living_entity
                .entity
                .play_sound(Sound::EntityVillagerAmbient)
                .await;
        }
    }

    /// Tick head-shake animation counter.
    fn tick_head_shake(&self) {
        let ticks = self.head_shake_ticks.load(Ordering::Relaxed);
        if ticks > 0 {
            self.head_shake_ticks.fetch_sub(1, Ordering::Relaxed);
        }
    }

    /// Send entity status to all nearby players.
    async fn send_entity_status(&self, status: EntityStatus) {
        let entity = &self.mob_entity.living_entity.entity;
        entity.world.load().send_entity_status(entity, status).await;
    }

    /// Get the work sound for the current profession.
    #[must_use]
    pub fn get_work_sound(&self) -> Option<Sound> {
        match self.get_profession() {
            VillagerProfession::Armorer => Some(Sound::EntityVillagerWorkArmorer),
            VillagerProfession::Butcher => Some(Sound::EntityVillagerWorkButcher),
            VillagerProfession::Cartographer => Some(Sound::EntityVillagerWorkCartographer),
            VillagerProfession::Cleric => Some(Sound::EntityVillagerWorkCleric),
            VillagerProfession::Farmer => Some(Sound::EntityVillagerWorkFarmer),
            VillagerProfession::Fisherman => Some(Sound::EntityVillagerWorkFisherman),
            VillagerProfession::Fletcher => Some(Sound::EntityVillagerWorkFletcher),
            VillagerProfession::Leatherworker => Some(Sound::EntityVillagerWorkLeatherworker),
            VillagerProfession::Librarian => Some(Sound::EntityVillagerWorkLibrarian),
            VillagerProfession::Mason => Some(Sound::EntityVillagerWorkMason),
            VillagerProfession::Shepherd => Some(Sound::EntityVillagerWorkShepherd),
            VillagerProfession::Toolsmith => Some(Sound::EntityVillagerWorkToolsmith),
            VillagerProfession::Weaponsmith => Some(Sound::EntityVillagerWorkWeaponsmith),
            VillagerProfession::None | VillagerProfession::Nitwit => None,
        }
    }
}

/// Factory for creating merchant screen handlers (used by `Player::open_handled_screen`).
struct MerchantScreenHandlerFactory {
    trade_offers: Vec<MerchantTradeOffer>,
    villager_entity_id: i32,
    villager_trading_lock: Arc<AtomicI32>,
    trade_completion_tx: Option<tokio::sync::mpsc::UnboundedSender<usize>>,
}

impl ScreenHandlerFactory for MerchantScreenHandlerFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> std::pin::Pin<Box<dyn Future<Output = Option<SharedScreenHandler>> + Send + 'a>> {
        Box::pin(async move {
            let mut handler = MerchantScreenHandler::new(
                sync_id,
                player_inventory,
                self.trade_completion_tx.clone(),
            );
            handler.villager_entity_id = self.villager_entity_id;
            handler.set_trade_offers(self.trade_offers.clone()).await;
            handler.villager_trading_lock = Some(self.villager_trading_lock.clone());
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

            // Workstation position
            if let Some(ws_pos) = *self.workstation_pos.lock().await {
                let mut ws_nbt = pumpkin_nbt::compound::NbtCompound::new();
                ws_nbt.put_int("X", ws_pos.0.x);
                ws_nbt.put_int("Y", ws_pos.0.y);
                ws_nbt.put_int("Z", ws_pos.0.z);
                nbt.put_component("WorkstationPos", ws_nbt);
            }

            // Bed position
            if let Some(bed_pos) = *self.bed_pos.lock().await {
                let mut bed_nbt = pumpkin_nbt::compound::NbtCompound::new();
                bed_nbt.put_int("X", bed_pos.0.x);
                bed_nbt.put_int("Y", bed_pos.0.y);
                bed_nbt.put_int("Z", bed_pos.0.z);
                nbt.put_component("BedPos", bed_nbt);
            }

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

                    let mut output_nbt = pumpkin_nbt::compound::NbtCompound::new();
                    offer.output.write_item_stack(&mut output_nbt);
                    recipe.put_component("sell", output_nbt);

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

            // Inventory
            self.inventory.lock().await.write_nbt(nbt);
        })
    }

    #[allow(clippy::too_many_lines)]
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

            // Workstation position
            if let Some(ws) = nbt.get_compound("WorkstationPos") {
                if let (Some(x), Some(y), Some(z)) =
                    (ws.get_int("X"), ws.get_int("Y"), ws.get_int("Z"))
                {
                    *self.workstation_pos.lock().await = Some(BlockPos(Vector3::new(x, y, z)));
                }
            }

            // Bed position
            if let Some(bed) = nbt.get_compound("BedPos") {
                if let (Some(x), Some(y), Some(z)) =
                    (bed.get_int("X"), bed.get_int("Y"), bed.get_int("Z"))
                {
                    *self.bed_pos.lock().await = Some(BlockPos(Vector3::new(x, y, z)));
                }
            }

            // Trade offers
            if let Some(recipes) = nbt
                .get_compound("Offers")
                .and_then(|o| o.get_list("Recipes"))
            {
                let mut offers = self.trade_offers.lock().await;
                offers.clear();
                for recipe_tag in recipes.iter().take(20) {
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
                        let price_multiplier = recipe.get_float("priceMultiplier").unwrap_or(0.05);
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

            // Gossips
            if let Some(gossip_list) = nbt.get_list("Gossips") {
                let mut gossips = self.gossips.lock().await;
                gossips.entries.clear();
                for gossip_tag in gossip_list.iter().take(256) {
                    if let pumpkin_nbt::tag::NbtTag::Compound(g) = gossip_tag
                        && let (Some(type_name), Some(int_arr), Some(value)) = (
                            g.get_string("Type"),
                            g.get_int_array("Target"),
                            g.get_int("Value"),
                        )
                        && int_arr.len() == 4
                    {
                        let hi = ((int_arr[0] as u64) << 32) | (int_arr[1] as u32 as u64);
                        let lo = ((int_arr[2] as u64) << 32) | (int_arr[3] as u32 as u64);
                        let uuid = uuid::Uuid::from_u128(((hi as u128) << 64) | (lo as u128));
                        if let Some(gtype) = GossipType::from_name(type_name) {
                            gossips.entries.push(GossipEntry {
                                gossip_type: gtype,
                                target: uuid,
                                value: value.clamp(0, gtype.max_value()),
                            });
                        }
                    }
                }
            }

            // Inventory
            self.inventory.lock().await.read_nbt(nbt);
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

    #[allow(clippy::too_many_lines)]
    fn mob_tick<'a>(
        &'a self,
        _caller: &'a Arc<dyn EntityBase>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        Box::pin(async {
            let entity = &self.mob_entity.living_entity.entity;
            let world = entity.world.load_full();
            let pos = entity.pos.load();
            let block_pos = BlockPos(Vector3::new(
                pos.x.floor() as i32,
                pos.y.floor() as i32,
                pos.z.floor() as i32,
            ));

            // --- Ambient sound + head shake ---
            self.tick_ambient_sound().await;
            self.tick_head_shake();

            // --- Reset trading state if player disconnected or closed the screen ---
            if self.is_trading() {
                // Check if the trading player is still connected and has the merchant screen open
                let trading_id = self.trading_player_id.load(Ordering::Relaxed);
                let entities = world.entities.load();
                let mut player_still_trading = false;
                for other in entities.iter() {
                    if other.get_entity().entity_id == trading_id {
                        player_still_trading = true;
                        break;
                    }
                }
                if !player_still_trading {
                    // Use compare_exchange to avoid overwriting a new player's session
                    let _ = self.trading_player_id.compare_exchange(
                        trading_id,
                        -1,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    );
                    // Clean up the trade completion channel
                    *self.trade_completion_rx.lock().await = None;
                }
            }

            // --- Drain trade completion channel (process completed trades) ---
            let completed_trades = {
                let mut rx_guard = self.trade_completion_rx.lock().await;
                let mut trades = Vec::new();
                if let Some(ref mut rx) = *rx_guard {
                    while let Ok(trade_idx) = rx.try_recv() {
                        trades.push(trade_idx);
                    }
                }
                trades
            };
            if !completed_trades.is_empty() {
                let trading_id = self.trading_player_id.load(Ordering::Relaxed);
                let player_uuid = if trading_id != -1 {
                    let entities = world.entities.load();
                    entities
                        .iter()
                        .find(|e| e.get_entity().entity_id == trading_id)
                        .map(|e| e.get_entity().entity_uuid)
                } else {
                    None
                };
                for trade_idx in completed_trades {
                    self.on_trade_completed(trade_idx, player_uuid).await;
                }
            }

            // --- Cache time values once per tick (avoid repeated lock acquisitions) ---
            let (time_of_day, world_age) = {
                let time = world.level_time.lock().await;
                (time.time_of_day, time.world_age)
            };
            let activity = schedule::VillagerActivity::from_time(time_of_day);
            let age = entity.age.load(Ordering::Relaxed);

            // --- Distance check: close trading if player is too far ---
            if self.is_trading() {
                let trading_id = self.trading_player_id.load(Ordering::Relaxed);
                let entities = world.entities.load();
                if let Some(trader) = entities
                    .iter()
                    .find(|e| e.get_entity().entity_id == trading_id)
                {
                    let trader_pos = trader.get_entity().pos.load();
                    let dx = pos.x - trader_pos.x;
                    let dz = pos.z - trader_pos.z;
                    if dx * dx + dz * dz > 256.0 {
                        // > 16 blocks away, close trading
                        let _ = self.trading_player_id.compare_exchange(
                            trading_id,
                            -1,
                            Ordering::SeqCst,
                            Ordering::SeqCst,
                        );
                        *self.trade_completion_rx.lock().await = None;
                    }
                }
            }

            // --- Workstation validity check (every 200 ticks) ---
            if age % 200 == 0 && self.get_profession() != VillagerProfession::None {
                let ws_guard = self.workstation_pos.lock().await;
                if let Some(ws_pos) = *ws_guard {
                    drop(ws_guard);
                    let block = world.get_block(&ws_pos).await;
                    let short_name = block.name.strip_prefix("minecraft:").unwrap_or(block.name);
                    let still_valid = poi::block_to_poi_type(short_name)
                        .and_then(poi::poi_type_to_profession)
                        .is_some();
                    if !still_valid {
                        self.clear_workstation().await;
                        self.sync_villager_data().await;
                    }
                }
            }

            // --- Workstation claiming (unemployed villagers, every 20 ticks) ---
            if self.get_profession() == VillagerProfession::None && age % 20 == 0 {
                let mut poi_storage = world.portal_poi.lock().await;
                let nearby = poi_storage.get_in_square(block_pos, 48, None);
                drop(poi_storage);

                for candidate in nearby {
                    let block = world.get_block(&candidate).await;
                    let short_name = block.name.strip_prefix("minecraft:").unwrap_or(block.name);

                    if let Some(poi_type) = poi::block_to_poi_type(short_name)
                        && let Some(profession_id) = poi::poi_type_to_profession(poi_type)
                    {
                        let profession = VillagerProfession::from_i32(profession_id);
                        self.set_profession(profession);
                        *self.workstation_pos.lock().await = Some(candidate);
                        self.populate_all_trades().await;
                        self.sync_villager_data().await;
                        // Happy particles on workstation claim
                        self.send_entity_status(EntityStatus::AddVillagerHappyParticles)
                            .await;
                        break;
                    }
                }
            }

            // --- Restock: when working and near workstation ---
            if activity.is_working() {
                let ws_guard = self.workstation_pos.lock().await;
                let ws_pos_opt = *ws_guard;
                drop(ws_guard);
                if let Some(ws_pos) = ws_pos_opt {
                    let dx = pos.x - (ws_pos.0.x as f64 + 0.5);
                    let dz = pos.z - (ws_pos.0.z as f64 + 0.5);
                    let dist_sq = dx * dx + dz * dz;
                    // Within 2 blocks of workstation
                    if dist_sq <= 4.0 {
                        // Play work sound periodically
                        if age % 40 == 0
                            && let Some(work_sound) = self.get_work_sound()
                        {
                            entity.play_sound(work_sound).await;
                        }
                        // Restock if needed (with 2400 tick cooldown)
                        let last_restock = self.last_restock_tick.load(Ordering::Relaxed);
                        if self.restock_count.load(Ordering::Relaxed) < 2
                            && (world_age - last_restock) > 2400
                        {
                            self.restock().await;
                            self.last_restock_tick.store(world_age, Ordering::Relaxed);
                        }
                    }
                }
            }

            // Reset restock count and decay gossip once per day (using atomic day tracking)
            let current_day = time_of_day / 24000;
            let last_day = self.last_reset_day.load(Ordering::Acquire);
            if current_day > last_day
                && self
                    .last_reset_day
                    .compare_exchange(last_day, current_day, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
            {
                self.restock_count.store(0, Ordering::Relaxed);
                self.gossips.lock().await.decay();
            }

            // --- Breeding willingness check (every 100 ticks) ---
            if age % 100 == 0
                && age >= 0
                && !self.mob_entity.is_in_love()
                && self.mob_entity.is_breeding_ready()
            {
                let inv = self.inventory.lock().await;
                if inv.is_willing() {
                    drop(inv);
                    // Check for an unclaimed bed nearby
                    let mut poi_storage = world.portal_poi.lock().await;
                    let beds = poi_storage.get_in_square(
                        block_pos,
                        48,
                        Some(pumpkin_world::poi::POI_TYPE_HOME),
                    );
                    drop(poi_storage);
                    if !beds.is_empty() {
                        // Consume food and enter love mode
                        self.inventory.lock().await.consume_food_for_breeding();
                        self.mob_entity.set_love_ticks(600);
                        self.send_entity_status(EntityStatus::AddVillagerHeartParticles)
                            .await;
                    }
                }
            }

            // --- Iron Golem spawning check (staggered per villager, every 100 ticks) ---
            // Stagger by entity_id so not all villagers check on the same tick
            if world_age % 100 == (entity.entity_id as i64 % 100) {
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
                        // Check cooldown (separate from restock)
                        let last_spawn = self.last_golem_spawn_tick.load(Ordering::Relaxed);
                        if world_age - last_spawn > 6000 {
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
                                self.last_golem_spawn_tick
                                    .store(world_age, Ordering::Relaxed);
                                // Spawn iron golem nearby
                                let spawn_pos = Vector3::new(pos.x + 2.0, pos.y, pos.z + 2.0);
                                let golem = crate::entity::r#type::from_type(
                                    &pumpkin_data::entity::EntityType::IRON_GOLEM,
                                    spawn_pos,
                                    &world,
                                    uuid::Uuid::new_v4(),
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

            // Baby villagers can't trade
            if self
                .mob_entity
                .living_entity
                .entity
                .age
                .load(Ordering::Relaxed)
                < 0
            {
                self.head_shake_ticks.store(40, Ordering::Relaxed);
                self.mob_entity
                    .living_entity
                    .entity
                    .play_sound(Sound::EntityVillagerNo)
                    .await;
                self.send_entity_status(EntityStatus::AddVillagerAngryParticles)
                    .await;
                return false;
            }

            // Nitwits and unemployed villagers can't trade
            if profession == VillagerProfession::None || profession == VillagerProfession::Nitwit {
                self.head_shake_ticks.store(40, Ordering::Relaxed);
                self.mob_entity
                    .living_entity
                    .entity
                    .play_sound(Sound::EntityVillagerNo)
                    .await;
                self.send_entity_status(EntityStatus::AddVillagerAngryParticles)
                    .await;
                return false;
            }

            // Atomically claim the trading slot to prevent race conditions
            // If another player is already trading, this will fail
            if self
                .trading_player_id
                .compare_exchange(-1, player.entity_id(), Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
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

            // Create trade completion channel
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<usize>();
            *self.trade_completion_rx.lock().await = Some(rx);

            // Open the merchant screen
            let factory = MerchantScreenHandlerFactory {
                trade_offers: trade_snapshots,
                villager_entity_id: self.mob_entity.living_entity.entity.entity_id,
                villager_trading_lock: self.trading_player_id.clone(),
                trade_completion_tx: Some(tx),
            };
            if let Some(window_id) = player.open_handled_screen(&factory, None).await {
                self.send_merchant_offers(player, window_id).await;
                self.mob_entity
                    .living_entity
                    .entity
                    .play_sound(Sound::EntityVillagerTrade)
                    .await;
            } else {
                // Failed to open screen, reset trading state
                self.set_trading_player(-1);
                return false;
            }

            true
        })
    }

    fn on_damage<'a>(
        &'a self,
        _damage_type: pumpkin_data::damage::DamageType,
        source: Option<&'a dyn EntityBase>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity
                .living_entity
                .entity
                .play_sound(Sound::EntityVillagerHurt)
                .await;

            // Add major negative gossip if damaged by a player
            if let Some(attacker) = source {
                if attacker.get_entity().entity_type.id
                    == pumpkin_data::entity::EntityType::PLAYER.id
                {
                    self.gossips.lock().await.add(
                        GossipType::MajorNegative,
                        attacker.get_entity().entity_uuid,
                        25,
                    );
                }
            }
        })
    }
}

/// Convert `VillagerType` enum to string name (for NBT).
#[must_use]
const fn villager_type_name(vtype: VillagerType) -> &'static str {
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

/// Parse `VillagerType` from string name (for NBT).
#[must_use]
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

/// Convert `VillagerProfession` enum to string name (for NBT).
#[must_use]
const fn profession_name(prof: VillagerProfession) -> &'static str {
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

/// Parse `VillagerProfession` from string name (for NBT).
#[must_use]
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
