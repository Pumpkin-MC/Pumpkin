use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, AtomicI64, Ordering};
use std::sync::{Arc, Weak};
use uuid::Uuid;

use crate::block::blocks::bed::BedBlock;
use pumpkin_data::Block;
use pumpkin_data::block_properties::{
    BedPart, BlockProperties, WhiteBedLikeProperties as BedProperties,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::{EntityPose, EntityType};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tag::Taggable;
use pumpkin_data::tracked_data::TrackedData;
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
    ai::goal::{
        avoid_entity::AvoidEntityGoal, interact_with_door::InteractWithDoorGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        villager_schedule::VillagerScheduleGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};
use crate::world::World;

pub mod data;
pub mod gossip;
pub use data::{
    BREEDING_FOOD_THRESHOLD, GossipType, VillagerData, VillagerProfession, VillagerType,
    get_food_points, villager_type_at, villager_type_by_biome,
};
pub use gossip::GossipContainer;

pub struct VillagerEntity {
    pub mob_entity: MobEntity,
    pub villager_data: Mutex<VillagerData>,
    pub food_level: AtomicI32,
    pub xp: AtomicI32,
    pub last_restock_time: AtomicI64,
    pub restocks_today: AtomicI32,
    pub gossips: Mutex<GossipContainer>,
    pub last_gossip_decay_time: AtomicI64,
    /// Vanilla `LAST_SLEPT` brain memory (`Villager::golemSpawnConditionsMet`). World-age tick
    /// at which this villager last entered the sleeping pose; 0 == never slept.
    pub last_slept_time: AtomicI64,
    /// Vanilla `GOLEM_DETECTED_RECENTLY` brain memory (599-tick expiry), set both by a
    /// successful golem spawn and by nearby-golem detection. World-age tick after which the
    /// memory is considered expired; 0 == not set.
    pub golem_detected_until: AtomicI64,
    pub inventory: Arc<Mutex<Vec<Arc<Mutex<ItemStack>>>>>,
    pub merchant_inventory: Arc<SimpleInventory>,
    pub offers: Mutex<Vec<pumpkin_protocol::java::client::play::MerchantOffer>>,
    pub job_site: std::sync::Mutex<Option<BlockPos>>,
    pub home_pos: std::sync::Mutex<Option<BlockPos>>,
    pub self_weak: std::sync::Mutex<Option<Weak<Self>>>,
}

impl VillagerEntity {
    #[allow(clippy::too_many_lines)]
    pub fn new(entity: Entity) -> Arc<Self> {
        // Vanilla `Villager#finalizeSpawn` sets the type from `VillagerType.byBiome`
        // at the spawn position.
        let villager_type = data::villager_type_at(&entity);
        let mob_entity = MobEntity::new(entity);
        let villager_data = VillagerData::new(villager_type, VillagerProfession::None, 1);
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
            restocks_today: AtomicI32::new(0),
            gossips: Mutex::new(GossipContainer::new()),
            last_gossip_decay_time: AtomicI64::new(0),
            last_slept_time: AtomicI64::new(0),
            golem_detected_until: AtomicI64::new(0),
            inventory,
            merchant_inventory: Arc::new(SimpleInventory::new(3)),
            offers: Mutex::new(Vec::new()),
            job_site: std::sync::Mutex::new(None),
            home_pos: std::sync::Mutex::new(None),
            self_weak: std::sync::Mutex::new(None),
        };
        let mob_arc = Arc::new(villager);
        *mob_arc.self_weak.lock().unwrap() = Some(Arc::downgrade(&mob_arc));
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        // Vanilla `Villager` constructor: `this.getNavigation().setCanOpenDoors(true);`.
        mob_arc
            .mob_entity
            .navigator
            .lock()
            .unwrap()
            .set_can_open_doors(true);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            // Approximates vanilla's brain-based `InteractWithDoor` behavior
            // (`VillagerGoalPackages.java:37`, `Pair.of(0, InteractWithDoor.create())`) with the
            // goal-based port, same as `CopperGolemEntity` does for its own `InteractWithDoor`
            // core-activity entry.
            goal_selector.add_goal(0, Box::new(InteractWithDoorGoal::new(true)));
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

            // `VillagerGoalPackages.getWorkPackage`/`getRestPackage`, simplified: walk to
            // the claimed job site/bed on schedule. Priority 1 (below AvoidEntityGoal at
            // 1, above WanderAroundGoal at 2) so it preempts wandering during work/rest
            // hours and yields MOVE back to wandering during meet/idle hours.
            goal_selector.add_goal(1, Box::new(VillagerScheduleGoal::new(0.5)));

            // Basic movement and looking (Vanilla uses 0.5 speed)
            goal_selector.add_goal(2, Box::new(WanderAroundGoal::new(0.5)));
            goal_selector.add_goal(
                3,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(
                4,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::VILLAGER, 8.0),
            );
            goal_selector.add_goal(5, Box::new(RandomLookAroundGoal::default()));
        };

        // Send initial metadata
        mob_arc.get_entity().send_meta_data(
            &[Metadata::new(
                TrackedData::VILLAGER_DATA,
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
                TrackedData::VILLAGER_DATA,
                MetaDataType::VILLAGER_DATA,
                data,
            )],
            None,
        );

        if old_profession != data.profession {
            self.generate_trades(data.profession_enum(), data.level.0)
                .await;
            if let Some(sound) = data.profession_enum().work_sound() {
                self.get_entity().play_sound(sound);
            }
        }
    }

    pub async fn add_trades(&self, profession: VillagerProfession, level: i32) {
        use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
        use rand::seq::IndexedRandom;
        use std::borrow::Cow;

        let mut offers = self.offers.lock().await;

        if let Some(trade_set) = profession.trade_set(level) {
            let mut rng = rand::rng();
            let chosen_trades = trade_set.trades.sample(&mut rng, trade_set.amount as usize);

            for trade in chosen_trades {
                offers.push(pumpkin_protocol::java::client::play::MerchantOffer {
                    base_cost_a: ItemStackSerializer(Cow::Owned(ItemStack::new(
                        trade.wants.count as u8,
                        trade.wants.item,
                    ))),
                    output: ItemStackSerializer(Cow::Owned(ItemStack::new(
                        trade.gives.count as u8,
                        trade.gives.item,
                    ))),
                    cost_b: trade.wants_b.as_ref().map(|b| {
                        ItemStackSerializer(Cow::Owned(ItemStack::new(b.count as u8, b.item)))
                    }),
                    is_disabled: false,
                    uses: 0,
                    max_uses: trade.max_uses,
                    xp: trade.xp,
                    special_price: 0,
                    price_multiplier: trade.price_multiplier,
                    demand: 0,
                });
            }
        }
    }

    pub async fn generate_trades(&self, profession: VillagerProfession, level: i32) {
        self.offers.lock().await.clear();
        self.add_trades(profession, level).await;
    }

    pub fn set_unhappy(&self) {
        let entity = self.get_entity();
        entity
            .world
            .load()
            .send_entity_status(entity, pumpkin_data::entity::EntityStatus::VillagerAngry);
        entity.play_sound(pumpkin_data::sound::Sound::EntityVillagerNo);
    }

    pub async fn open_trading_screen(&self, player: &Arc<Player>) {
        use pumpkin_protocol::codec::var_int::VarInt;
        use pumpkin_protocol::java::client::play::CMerchantOffers;

        // Open the merchant screen and then send the current offers packet
        if let Some(sync_id) = player.open_handled_screen(self, None).await {
            let mut offers = self.offers.lock().await.clone();
            let reputation = self
                .gossips
                .lock()
                .await
                .get_reputation(player.get_entity().entity_uuid, |_| true);
            if reputation != 0 {
                for offer in &mut offers {
                    offer.special_price +=
                        gossip::reputation_price_discount(reputation, offer.price_multiplier);
                }
            }
            // `Villager::updateSpecialPrices` (`Villager.java:450-458`): Hero of the Village
            // stacks an additional discount on top of the reputation one above, proportional
            // to the offer's un-modified `cost_a` count and the effect's amplifier.
            if let Some(effect) = player
                .living_entity
                .get_effect(&pumpkin_data::effect::StatusEffect::HERO_OF_THE_VILLAGE)
                .await
            {
                let modifier = 0.3 + 0.0625 * f64::from(effect.amplifier);
                for offer in &mut offers {
                    let base_count = f64::from(offer.base_cost_a.0.item_count);
                    #[allow(clippy::cast_possible_truncation)]
                    let cost_reduction = (modifier * base_count).floor() as i32;
                    offer.special_price -= cost_reduction.max(1);
                }
            }
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

    /// Vanilla `LivingEntity::stopSleeping`, invoked by `BedBlock::kickVillagerOutOfBed`
    /// when a player uses an occupied bed: un-occupies the bed and stands the villager up.
    pub async fn stop_sleeping(&self, world: &Arc<World>) {
        let home_pos = *self.home_pos.lock().unwrap();
        if let Some(home_pos) = home_pos {
            let (block, state) = world.get_block_and_state(&home_pos);
            if block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_BEDS) {
                let bed_props = BedProperties::from_state_id(state.id, block);
                if bed_props.occupied {
                    BedBlock::set_occupied(false, world, block, &home_pos, state.id).await;
                }
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

    /// Vanilla `Villager::wantsToSpawnGolem` + `golemSpawnConditionsMet`
    /// (`Villager.java:850-852, 896-899`). Despite the "panicking neighbors" framing in
    /// vanilla's constant names, the actual gate checked in code is "slept within the last
    /// in-game day" (`LAST_SLEPT` recency), not a live panic flag -- Pumpkin has no brain/
    /// panic-activity system, so this is ported exactly as read from the cited lines rather
    /// than inventing a stricter panic-based gate.
    #[must_use]
    pub fn wants_to_spawn_golem(&self, world_age: i64) -> bool {
        golem_spawn_conditions_met(
            self.last_slept_time.load(Ordering::Relaxed),
            self.golem_detected_until.load(Ordering::Relaxed),
            world_age,
        )
    }

    /// Vanilla `Villager::spawnGolemIfNeeded` (`Villager.java:834-848`).
    ///
    /// **Trigger deviation from vanilla, documented per repo standing rule:** vanilla only
    /// reaches this method from `Villager::gossip`, itself only called by the panic-activity
    /// brain package when two panicking villagers meet at close range. Pumpkin has neither a
    /// brain/activity system nor any villager panic goal (confirmed absent from the goal
    /// directory), so there is no faithful trigger to call this from. As the pragmatic
    /// approximation documented in the design doc, this is called from `on_damage` (a
    /// villager being attacked is the closest existing analogue to "a villager in a crisis"
    /// this codebase can express without building a full panic-goal subsystem) rather than
    /// from a gossip-transfer event. The summoning *mechanics* below (AABB scan, agreement
    /// count, spawn, suppression) are ported faithfully from the cited vanilla lines.
    pub async fn spawn_golem_if_needed(
        &self,
        world: &Arc<World>,
        world_age: i64,
        villagers_needed: usize,
    ) {
        if !self.wants_to_spawn_golem(world_age) {
            return;
        }

        let pos = self.get_entity().pos.load();
        let aabb = BoundingBox::new(
            Vector3::new(pos.x - 10.0, pos.y - 10.0, pos.z - 10.0),
            Vector3::new(pos.x + 10.0, pos.y + 10.0, pos.z + 10.0),
        );

        // Vanilla's `.limit(5L)` -- a hard cap regardless of `villagers_needed` (`Villager.java:838`).
        let agreeing = world
            .get_all_at_box(&aabb)
            .into_iter()
            .filter(|e| {
                e.get_entity().entity_type == &EntityType::VILLAGER
                    && e.cast_any()
                        .downcast_ref::<Self>()
                        .is_some_and(|v| v.wants_to_spawn_golem(world_age))
            })
            .count()
            .min(5);
        if agreeing < villagers_needed {
            return;
        }

        // `SpawnUtil.trySpawnMob(IRON_GOLEM, ..., 10, 8, 6, ...)` (`Villager.java:840-842`):
        // vanilla searches a 10-horizontal/8-up/6-down radius for a valid position. The exact
        // placement-validity rules inside `SpawnUtil.trySpawnMob` were not read for this pass
        // (flagged in the design doc as needing follow-up); this uses the simplest possible
        // approximation available from existing Rust infrastructure -- `World::is_space_empty`
        // for a golem-sized bounding box plus a solid-ground check directly below -- searched
        // over a small horizontal ring at the villager's own height, rather than porting
        // `SpawnUtil`'s full search shape.
        let block_pos = self.get_entity().block_pos.load();
        let mut spawn_pos = None;
        'search: for dx in -3..=3i32 {
            for dz in -3..=3i32 {
                let candidate =
                    BlockPos::new(block_pos.0.x + dx, block_pos.0.y, block_pos.0.z + dz);
                let below = candidate.down();
                let (_, below_state) = world.get_block_and_state(&below);
                if !below_state.is_solid() {
                    continue;
                }
                let feet = candidate.to_f64();
                let check_box = BoundingBox::new(
                    Vector3::new(feet.x - 0.7, feet.y, feet.z - 0.7),
                    Vector3::new(feet.x + 0.7, feet.y + 2.7, feet.z + 0.7),
                );
                if world.is_space_empty(check_box) {
                    spawn_pos = Some(candidate);
                    break 'search;
                }
            }
        }

        let Some(spawn_pos) = spawn_pos else {
            return;
        };

        let entity = Entity::new(
            world.clone(),
            spawn_pos.to_centered_f64(),
            &EntityType::IRON_GOLEM,
        );
        let golem = crate::entity::passive::iron_golem::IronGolemEntity::new(entity);
        world.spawn_entity(golem).await;

        // `nearbyVillagers.forEach(GolemSensor::golemDetected)` (`Villager.java:844`) --
        // every villager in the *unfiltered* nearby list is suppressed, not just the ones
        // that agreed. Re-scan since `nearby_villagers` above was already filtered.
        for entity in world.get_all_at_box(&aabb) {
            if entity.get_entity().entity_type == &EntityType::VILLAGER
                && let Some(villager) = entity.cast_any().downcast_ref::<Self>()
            {
                villager
                    .golem_detected_until
                    .store(world_age + 599, Ordering::Relaxed);
            }
        }
    }

    /// Vanilla `Villager::restock` (`Villager.java:365-375`): recompute demand for every
    /// offer and reset its use counter. Does not resend the updated offers to a currently
    /// trading player -- vanilla's `resendOffersToTradingPlayer` (`Villager.java:377-385`)
    /// has no Rust equivalent since `VillagerEntity` doesn't track a persistent
    /// "currently trading player" handle; a player with the trade screen already open will
    /// see the new prices next time they reopen it. Documented deviation, not silently
    /// dropped.
    pub async fn restock(&self, world_age: i64) {
        let mut offers = self.offers.lock().await;
        for offer in offers.iter_mut() {
            offer.update_demand();
            offer.uses = 0;
        }
        drop(offers);
        self.last_restock_time.store(world_age, Ordering::Relaxed);
        self.restocks_today.fetch_add(1, Ordering::Relaxed);
    }

    /// Vanilla `Villager::shouldRestock`/`needsToRestock` (`Villager.java:387-419`),
    /// approximated: a villager may restock up to twice per in-game day, at least
    /// 12000 ticks (half a day) apart, whenever any offer has been used at least once.
    /// Vanilla's exact `isNewDay` check against a `Timelines.OVERWORLD_DAY` clock was not
    /// read for this pass (flagged in the design doc); this derives the day boundary from
    /// `world_age / 24000`, which is the same 24000-tick day length already used elsewhere
    /// in this file (gossip decay), and resets the daily restock counter on a day rollover.
    pub async fn maybe_restock(&self, world_age: i64) {
        let last_restock = self.last_restock_time.load(Ordering::Relaxed);
        if is_new_restock_day(last_restock, world_age) {
            self.restocks_today.store(0, Ordering::Relaxed);
        }

        if !restock_is_due(
            last_restock,
            self.restocks_today.load(Ordering::Relaxed),
            world_age,
        ) {
            return;
        }

        let needs_restock = {
            let offers = self.offers.lock().await;
            offers.iter().any(|o| o.uses > 0)
        };
        if needs_restock {
            self.restock(world_age).await;
        }
    }
}

/// Vanilla `Villager::golemSpawnConditionsMet` (`Villager.java:896-899`), extracted as a
/// pure function for unit testing.
#[must_use]
const fn golem_spawn_conditions_met(
    last_slept: i64,
    golem_detected_until: i64,
    world_age: i64,
) -> bool {
    if last_slept == 0 || world_age - last_slept >= 24000 {
        return false;
    }
    golem_detected_until == 0 || world_age >= golem_detected_until
}

/// Day-boundary check for the restock-counter reset, extracted as a pure function for unit
/// testing. See `maybe_restock`'s doc comment for the approximation this makes relative to
/// vanilla's `Timelines.OVERWORLD_DAY`-based `isNewDay`.
#[must_use]
const fn is_new_restock_day(last_restock: i64, world_age: i64) -> bool {
    last_restock != 0 && world_age / 24000 != last_restock / 24000
}

/// Vanilla `Villager::shouldRestock` (`Villager.java:387-419`) gate (minus the `needsRestock`
/// per-offer check, which needs the offers list and stays in `maybe_restock`), extracted as a
/// pure function for unit testing.
#[must_use]
const fn restock_is_due(last_restock: i64, restocks_today: i32, world_age: i64) -> bool {
    if restocks_today >= 2 {
        return false;
    }
    last_restock == 0 || world_age - last_restock >= 12000
}

impl ScreenHandlerFactory for VillagerEntity {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<pumpkin_inventory::player::player_inventory::PlayerInventory>,
        player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let offers = self.offers.lock().await;
            let self_weak = self.self_weak.lock().unwrap().clone().unwrap();
            let player_uuid = player
                .as_any()
                .downcast_ref::<crate::entity::player::Player>()
                .map_or_else(uuid::Uuid::nil, |p| p.get_entity().entity_uuid);
            let world = self.get_entity().world.load().clone();

            let mut handler = MerchantScreenHandler::new(
                sync_id,
                player_inventory,
                self.merchant_inventory.clone(),
                offers.clone(),
            )
            .await;

            handler.on_trade = Some(Box::new(move |offer_index| {
                if let Some(villager) = self_weak.upgrade() {
                    let world = world.clone();
                    tokio::spawn(async move {
                        if let Some(player) = world.get_player_by_uuid(player_uuid) {
                            let mut offers = villager.offers.lock().await;
                            if offer_index < offers.len() {
                                let offer = &mut offers[offer_index];
                                offer.uses += 1;
                                let reward_exp = !offer.is_disabled;

                                // `Villager::customServerAiStep` -> `onReputationEventFrom(TRADE, ...)`
                                // (Villager.java:257-258, 859-860): TRADING gossip, +2 (`REPUTATION_CHANGE_PER_TRADE`).
                                villager.gossips.lock().await.add(
                                    player_uuid,
                                    GossipType::Trading,
                                    2,
                                );

                                let xp_gain = offer.xp;
                                let current_xp =
                                    villager.xp.fetch_add(xp_gain, Ordering::Relaxed) + xp_gain;

                                let mut leveled_up = false;
                                let mut data = villager.villager_data.lock().await;
                                let current_level = data.level.0;
                                if current_level < 5 {
                                    let max_xp = match current_level {
                                        1 => 10,
                                        2 => 70,
                                        3 => 150,
                                        4 => 250,
                                        _ => 0,
                                    };
                                    if current_xp >= max_xp {
                                        data.level.0 += 1;
                                        let new_level = data.level.0;
                                        let prof = data.profession_enum();
                                        drop(data);
                                        leveled_up = true;

                                        // Level up! Add new trades for the new level
                                        villager.add_trades(prof, new_level).await;

                                        // Play sound & particles for level up!
                                        let entity = villager.get_entity();
                                        entity.world.load().send_entity_status(
                                            entity,
                                            pumpkin_data::entity::EntityStatus::VillagerHappy,
                                        );
                                        entity.play_sound(
                                            pumpkin_data::sound::Sound::EntityVillagerCelebrate,
                                        );
                                    } else {
                                        drop(data);
                                    }
                                } else {
                                    drop(data);
                                }

                                if reward_exp {
                                    let mut player_xp: u32 = 3 + rand::random_range(0..4u32);
                                    if leveled_up {
                                        player_xp += 5;
                                    }
                                    let entity = villager.get_entity();
                                    crate::entity::experience_orb::ExperienceOrbEntity::spawn(
                                        &entity.world.load(),
                                        entity.pos.load(),
                                        player_xp,
                                    )
                                    .await;
                                }

                                let current_level = villager.villager_data.lock().await.level;
                                player
                                    .client
                                    .enqueue_packet(&CMerchantOffers::new(
                                        VarInt(sync_id as i32),
                                        offers.clone(),
                                        current_level,
                                        VarInt(current_xp),
                                        true,
                                        true,
                                    ))
                                    .await;
                            }
                        }
                    });
                }
            }));

            Some(Arc::new(Mutex::new(handler)) as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        // TODO: Localized name based on profession
        TextComponent::translate_cross(
            pumpkin_data::translation::java::ENTITY_MINECRAFT_VILLAGER,
            pumpkin_data::translation::bedrock::ENTITY_VILLAGER_NAME,
            [],
        )
    }
}

impl NBTStorage for VillagerEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.write_nbt(nbt).await;
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

            let job_site_pos = *self.job_site.lock().unwrap();
            if let Some(pos) = job_site_pos {
                nbt.put_int("JobSiteX", pos.0.x);
                nbt.put_int("JobSiteY", pos.0.y);
                nbt.put_int("JobSiteZ", pos.0.z);
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
                    recipe.put_bool("rewardExp", !offer.is_disabled);
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
            for (uuid, types) in gossips.raw() {
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
                    gossip_nbt.put_int("Type", *gtype as i32);
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
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
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

            if let (Some(x), Some(y), Some(z)) = (
                nbt.get_int("JobSiteX"),
                nbt.get_int("JobSiteY"),
                nbt.get_int("JobSiteZ"),
            ) {
                *self.job_site.lock().unwrap() = Some(BlockPos::new(x, y, z));
            } else {
                *self.job_site.lock().unwrap() = None;
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

                        if let (Some(buy), Some(sell_item)) = (buy, sell_item) {
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
                                is_disabled: !reward_exp,
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
                let mut raw: HashMap<Uuid, HashMap<GossipType, i32>> = HashMap::new();
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
                        if let (Some(uuid), Some(gtype), Some(val)) = (
                            uuid,
                            gossip_nbt.get_int("Type"),
                            gossip_nbt.get_int("Value"),
                        ) {
                            let gossip_type = match gtype {
                                0 => GossipType::MajorNegative,
                                1 => GossipType::MinorNegative,
                                2 => GossipType::MajorPositive,
                                3 => GossipType::MinorPositive,
                                4 => GossipType::Trading,
                                _ => continue,
                            };
                            raw.entry(uuid).or_default().insert(gossip_type, val);
                        }
                    }
                }
                *self.gossips.lock().await = GossipContainer::from_raw(raw);
            }
        })
    }
}

fn block_to_profession(block: &Block) -> Option<VillagerProfession> {
    if block == &Block::COMPOSTER {
        Some(VillagerProfession::Farmer)
    } else if block == &Block::LECTERN {
        Some(VillagerProfession::Librarian)
    } else if block == &Block::BLAST_FURNACE {
        Some(VillagerProfession::Armorer)
    } else if block == &Block::SMOKER {
        Some(VillagerProfession::Butcher)
    } else if block == &Block::CARTOGRAPHY_TABLE {
        Some(VillagerProfession::Cartographer)
    } else if block == &Block::BREWING_STAND {
        Some(VillagerProfession::Cleric)
    } else if block == &Block::BARREL {
        Some(VillagerProfession::Fisherman)
    } else if block == &Block::FLETCHING_TABLE {
        Some(VillagerProfession::Fletcher)
    } else if block == &Block::CAULDRON
        || block == &Block::WATER_CAULDRON
        || block == &Block::LAVA_CAULDRON
        || block == &Block::POWDER_SNOW_CAULDRON
    {
        Some(VillagerProfession::Leatherworker)
    } else if block == &Block::STONECUTTER {
        Some(VillagerProfession::Mason)
    } else if block == &Block::LOOM {
        Some(VillagerProfession::Shepherd)
    } else if block == &Block::SMITHING_TABLE {
        Some(VillagerProfession::Toolsmith)
    } else if block == &Block::GRINDSTONE {
        Some(VillagerProfession::Weaponsmith)
    } else {
        None
    }
}

fn profession_matches_block(profession: VillagerProfession, block: &Block) -> bool {
    match profession {
        VillagerProfession::Farmer => block == &Block::COMPOSTER,
        VillagerProfession::Librarian => block == &Block::LECTERN,
        VillagerProfession::Armorer => block == &Block::BLAST_FURNACE,
        VillagerProfession::Butcher => block == &Block::SMOKER,
        VillagerProfession::Cartographer => block == &Block::CARTOGRAPHY_TABLE,
        VillagerProfession::Cleric => block == &Block::BREWING_STAND,
        VillagerProfession::Fisherman => block == &Block::BARREL,
        VillagerProfession::Fletcher => block == &Block::FLETCHING_TABLE,
        VillagerProfession::Leatherworker => {
            block == &Block::CAULDRON
                || block == &Block::WATER_CAULDRON
                || block == &Block::LAVA_CAULDRON
                || block == &Block::POWDER_SNOW_CAULDRON
        }
        VillagerProfession::Mason => block == &Block::STONECUTTER,
        VillagerProfession::Shepherd => block == &Block::LOOM,
        VillagerProfession::Toolsmith => block == &Block::SMITHING_TABLE,
        VillagerProfession::Weaponsmith => block == &Block::GRINDSTONE,
        _ => false,
    }
}

impl Mob for VillagerEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_job_site(&self) -> Option<BlockPos> {
        *self.job_site.lock().unwrap()
    }

    fn get_home(&self) -> Option<BlockPos> {
        *self.home_pos.lock().unwrap()
    }

    /// `Villager::setLastHurtByMob` -> `onReputationEventFrom(VILLAGER_HURT, ...)`
    /// (Villager.java:585-593, 861-862): the hurt villager itself records
    /// `MINOR_NEGATIVE` gossip against its attacker.
    fn on_damage<'a>(
        &'a self,
        _damage_type: DamageType,
        _source: Option<&'a dyn EntityBase>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let attacker_id = self
                .mob_entity
                .living_entity
                .last_attacker_id
                .load(Ordering::Relaxed);
            if attacker_id == 0 {
                return;
            }
            let world = self.get_entity().world.load();
            if let Some(attacker) = world.get_entity_by_id(attacker_id) {
                self.gossips.lock().await.add(
                    attacker.get_entity().entity_uuid,
                    GossipType::MinorNegative,
                    25,
                );
            }

            // Golem summoning trigger deviation: see `spawn_golem_if_needed`'s doc comment.
            // Vanilla only reaches `spawnGolemIfNeeded` via panicking-villager gossip
            // exchange, which Pumpkin has no infrastructure for; being attacked is used here
            // as the closest existing "villager in a crisis" event.
            let world_age = world.get_world_age().await;
            self.spawn_golem_if_needed(&world, world_age, 5).await;
        })
    }

    /// `Villager::tellWitnessesThatIWasMurdered` -> `onReputationEventFrom(VILLAGER_KILLED, ...)`
    /// (Villager.java:615-624, 863-864): every witnessing villager (vanilla uses the brain's
    /// `NEAREST_VISIBLE_LIVING_ENTITIES` memory; approximated here with a 16-block box, the
    /// default `FOLLOW_RANGE` vanilla's sensor inflates by -- `Mob.java:167` -- since Pumpkin
    /// has no brain/sensor system) records `MAJOR_NEGATIVE` gossip against the murderer.
    fn on_mob_death<'a>(
        &'a self,
        cause: Option<&'a dyn EntityBase>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let Some(murderer) = cause else {
                return;
            };
            let murderer_uuid = murderer.get_entity().entity_uuid;
            let world = self.get_entity().world.load();
            let pos = self.get_entity().pos.load();
            let aabb = BoundingBox::new(
                Vector3::new(pos.x - 16.0, pos.y - 16.0, pos.z - 16.0),
                Vector3::new(pos.x + 16.0, pos.y + 16.0, pos.z + 16.0),
            );
            for entity in world.get_all_at_box(&aabb) {
                if entity.get_entity().entity_id == self.get_entity().entity_id
                    || entity.get_entity().entity_type != &EntityType::VILLAGER
                {
                    continue;
                }
                if let Some(villager) = entity.cast_any().downcast_ref::<Self>() {
                    villager
                        .gossips
                        .lock()
                        .await
                        .add(murderer_uuid, GossipType::MajorNegative, 25);
                }
            }
        })
    }

    #[expect(clippy::too_many_lines)]
    fn mob_tick<'a>(
        &'a self,
        _caller: &'a Arc<dyn EntityBase>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let age = self.get_entity().age.load(Ordering::Relaxed);
            if age % 20 != 0 {
                return;
            }

            let world = self.get_entity().world.load();

            // `Villager::maybeDecayGossip` (Villager.java:824-832): decays gossip once
            // per in-game day (24000 ticks), tracked from a baseline set on first tick.
            let world_age = world.get_world_age().await;
            let last_decay = self.last_gossip_decay_time.load(Ordering::Relaxed);
            if last_decay == 0 {
                self.last_gossip_decay_time
                    .store(world_age, Ordering::Relaxed);
            } else if world_age >= last_decay + 24000 {
                self.gossips.lock().await.decay();
                self.last_gossip_decay_time
                    .store(world_age, Ordering::Relaxed);
            }

            // `GolemSensor` equivalent (`GolemSensor.java`): approximated with a 16-block box
            // (vanilla scans the brain's `NEAREST_LIVING_ENTITIES` memory, itself populated
            // from a follow-range-sized box -- Pumpkin has no such memory, so this piggybacks
            // on the existing 20-tick cadence instead of a distinct sensor abstraction).
            {
                let pos = self.get_entity().pos.load();
                let aabb = BoundingBox::new(
                    Vector3::new(pos.x - 16.0, pos.y - 16.0, pos.z - 16.0),
                    Vector3::new(pos.x + 16.0, pos.y + 16.0, pos.z + 16.0),
                );
                if world
                    .get_all_at_box(&aabb)
                    .iter()
                    .any(|e| e.get_entity().entity_type == &EntityType::IRON_GOLEM)
                {
                    self.golem_detected_until
                        .store(world_age + 599, Ordering::Relaxed);
                }
            }

            self.maybe_restock(world_age).await;

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
                    // Vanilla `ValidateNearbyPoi`/`Villager.releasePoi`:
                    // release the claimed bed's ticket once it's no longer a
                    // valid (head-part) bed, e.g. it was broken.
                    world.release_poi(current_home).await;
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

            // If no bed, atomically claim the closest unclaimed one -
            // vanilla `AcquirePoi` (`SCAN_RANGE = 48`), via
            // `World::acquire_poi` (`PoiManager.take`,
            // `Occupancy.HAS_SPACE`). Because acquisition decrements the
            // POI's `free_tickets`, no other villager can claim the same
            // bed - unlike the old ad-hoc scan, this doesn't need to ask
            // every nearby villager what it has already claimed.
            if self.get_home_pos().is_none() {
                let pos = self.get_entity().block_pos.load();
                if let Some(home) = world
                    .acquire_poi(crate::world::village_poi::POI_TYPE_HOME, pos, 48)
                    .await
                {
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
                                    // Vanilla `LAST_SLEPT` brain memory, set whenever the
                                    // sleep-behavior brain task puts the villager to sleep
                                    // (referenced by `golemSpawnConditionsMet`,
                                    // `Villager.java:896-899`; the task that sets it wasn't
                                    // itself read for this pass).
                                    self.last_slept_time.store(world_age, Ordering::Relaxed);
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

            // 2. Job / Profession logic (skip for Nitwits and babies)
            let data = self.villager_data.lock().await;
            let is_adult = self.get_entity().age.load(Ordering::Relaxed) >= 0;
            let xp = self.xp.load(Ordering::Relaxed);
            let profession = data.profession_enum();
            drop(data);

            if profession == VillagerProfession::Nitwit || !is_adult {
                return;
            }

            if let Some(current_site) = self.get_job_site() {
                let (block, _state) = world.get_block_and_state(&current_site);
                let valid = if profession == VillagerProfession::None {
                    block_to_profession(block).is_some()
                } else {
                    profession_matches_block(profession, block)
                };

                if !valid {
                    *self.job_site.lock().unwrap() = None;
                    if xp == 0 && profession != VillagerProfession::None {
                        let r#type = self.villager_data.lock().await.type_enum();
                        self.set_villager_data(VillagerData::new(
                            r#type,
                            VillagerProfession::None,
                            1,
                        ))
                        .await;
                        self.offers.lock().await.clear();
                    }
                }
            }

            if self.get_job_site().is_none() {
                let pos = self.get_entity().block_pos.load();
                let start = BlockPos::new(pos.0.x - 10, pos.0.y - 4, pos.0.z - 10);
                let end = BlockPos::new(pos.0.x + 10, pos.0.y + 4, pos.0.z + 10);

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

                let mut claimed_sites = Vec::new();
                for entity in nearby_entities {
                    if entity.get_entity().entity_id != self.get_entity().entity_id
                        && entity.get_entity().entity_type
                            == &pumpkin_data::entity::EntityType::VILLAGER
                        && let Some(site) = entity.get_job_site_pos()
                    {
                        claimed_sites.push(site);
                    }
                }

                let mut best_site = None;
                let mut best_dist = f64::MAX;
                let mut best_profession = VillagerProfession::None;

                for p in BlockPos::iterate(start, end) {
                    if claimed_sites.contains(&p) {
                        continue;
                    }

                    let (block, _state) = world.get_block_and_state(&p);
                    if let Some(prof) = block_to_profession(block) {
                        if profession != VillagerProfession::None && prof != profession {
                            continue;
                        }

                        let dist = p
                            .to_f64()
                            .squared_distance_to_vec(&self.get_entity().pos.load());
                        if dist < best_dist {
                            best_dist = dist;
                            best_site = Some(p);
                            best_profession = prof;
                        }
                    }
                }

                if let Some(site) = best_site {
                    *self.job_site.lock().unwrap() = Some(site);
                    if profession == VillagerProfession::None {
                        let r#type = self.villager_data.lock().await.type_enum();
                        self.set_villager_data(VillagerData::new(r#type, best_profession, 1))
                            .await;
                    }
                }
            } else {
                let current_prof = self.villager_data.lock().await.profession_enum();
                if current_prof == VillagerProfession::None
                    && let Some(site) = self.get_job_site()
                {
                    let (block, _state) = world.get_block_and_state(&site);
                    if let Some(prof) = block_to_profession(block) {
                        let r#type = self.villager_data.lock().await.type_enum();
                        self.set_villager_data(VillagerData::new(r#type, prof, 1))
                            .await;
                    }
                }
            }
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        _item_stack: &'a mut pumpkin_data::item_stack::ItemStack,
    ) -> crate::entity::EntityBaseFuture<'a, bool> {
        let player = player.clone();
        Box::pin(async move {
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
mod villager_tick_logic_tests {
    use super::{golem_spawn_conditions_met, is_new_restock_day, restock_is_due};

    #[test]
    fn golem_conditions_require_recent_sleep() {
        assert!(!golem_spawn_conditions_met(0, 0, 100));
        assert!(golem_spawn_conditions_met(100, 0, 200));
        assert!(!golem_spawn_conditions_met(100, 0, 100 + 24000));
    }

    #[test]
    fn golem_conditions_respect_suppression_window() {
        assert!(!golem_spawn_conditions_met(100, 1000, 500));
        assert!(golem_spawn_conditions_met(100, 1000, 1000));
        assert!(golem_spawn_conditions_met(100, 1000, 1500));
    }

    #[test]
    fn restock_day_rollover_detected_by_24000_tick_boundary() {
        assert!(!is_new_restock_day(0, 100));
        assert!(!is_new_restock_day(100, 24000 - 1));
        assert!(is_new_restock_day(100, 24000));
    }

    #[test]
    fn restock_due_gates_on_count_and_cooldown() {
        assert!(restock_is_due(0, 0, 0));
        assert!(!restock_is_due(100, 0, 100 + 11999));
        assert!(restock_is_due(100, 0, 100 + 12000));
        assert!(!restock_is_due(100, 2, 100 + 24000));
    }
}
