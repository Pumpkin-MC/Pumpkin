use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, AtomicI64, Ordering};
use std::sync::{Arc, Weak};
use uuid::Uuid;

use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_inventory::merchant::merchant_screen_handler::MerchantScreenHandler;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{CMerchantOffers, Metadata};
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::SimpleInventory;
use tokio::sync::Mutex;

use crate::entity::player::Player;
use crate::entity::{
    Entity, EntityBase, NBTStorage,
    ai::goal::{
        avoid_entity::AvoidEntityGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub mod data;
pub use data::{
    BREEDING_FOOD_THRESHOLD, GossipType, VillagerData, VillagerProfession, VillagerType,
    get_food_points,
};

pub struct VillagerEntity {
    pub mob_entity: MobEntity,
    pub villager_data: Mutex<VillagerData>,
    pub food_level: AtomicI32,
    pub xp: AtomicI32,
    pub last_restock_time: AtomicI64,
    pub restocks_today: AtomicI32,
    pub gossips: Mutex<HashMap<Uuid, HashMap<GossipType, i32>>>,
    pub inventory: Arc<Mutex<Vec<Arc<Mutex<ItemStack>>>>>,
    pub merchant_inventory: Arc<SimpleInventory>,
    pub offers: Mutex<Vec<pumpkin_protocol::java::client::play::MerchantOffer>>,
    pub self_arc: Mutex<Option<Weak<Self>>>,
}

impl VillagerEntity {
    #[expect(clippy::too_many_lines)]
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let villager_data = VillagerData {
            r#type: VillagerType::Plains,
            profession: VillagerProfession::None,
            level: 1,
        };
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
            gossips: Mutex::new(HashMap::new()),
            inventory,
            merchant_inventory: Arc::new(SimpleInventory::new(3)),
            offers: Mutex::new(Vec::new()),
            self_arc: Mutex::new(None),
        };
        let mob_arc = Arc::new(villager);
        *mob_arc.self_arc.lock().await = Some(Arc::downgrade(&mob_arc));
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;

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
        mob_arc
            .get_entity()
            .send_meta_data(&[Metadata::new(
                TrackedData::VILLAGER_DATA,
                MetaDataType::VILLAGER_DATA,
                villager_data,
            )])
            .await;

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
        let mut villager_data = self.villager_data.lock().await;
        let old_profession = villager_data.profession;
        *villager_data = data;
        self.get_entity()
            .send_meta_data(&[Metadata::new(
                TrackedData::VILLAGER_DATA,
                MetaDataType::VILLAGER_DATA,
                data,
            )])
            .await;

        if old_profession != data.profession {
            self.generate_trades(data.profession, data.level).await;
            if let Some(sound) = data.profession.work_sound() {
                self.get_entity().play_sound(sound).await;
            }
        }
    }

    pub async fn generate_trades(&self, profession: VillagerProfession, level: i32) {
        use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
        use rand::seq::IndexedRandom;
        use std::borrow::Cow;

        let mut offers = self.offers.lock().await;
        offers.clear();

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

    pub async fn set_unhappy(&self) {
        let entity = self.get_entity();
        entity
            .world
            .load()
            .send_entity_status(entity, pumpkin_data::entity::EntityStatus::VillagerAngry)
            .await;
        entity
            .play_sound(pumpkin_data::sound::Sound::EntityVillagerNo)
            .await;
    }

    pub async fn open_trading_screen(&self, player: &Arc<Player>) {
        let self_weak = self.self_arc.lock().await;
        if let Some(self_arc) = self_weak.as_ref().and_then(std::sync::Weak::upgrade) {
            player.open_handled_screen(&*self_arc, None).await;

            let offers = self.offers.lock().await;
            let villager_data = self.villager_data.lock().await;

            player
                .client
                .enqueue_packet(&CMerchantOffers::new(
                    player.screen_handler_sync_id.load(Ordering::Relaxed).into(),
                    offers.clone(),
                    VarInt(villager_data.level),
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
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let offers = self.offers.lock().await;
            let handler = MerchantScreenHandler::new(
                sync_id,
                player_inventory,
                self.merchant_inventory.clone(),
                offers.clone(),
            )
            .await;
            Some(Arc::new(Mutex::new(handler)) as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        // TODO: Localized name based on profession
        TextComponent::text("Villager")
    }
}

impl NBTStorage for VillagerEntity {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut pumpkin_nbt::pnbt::PNbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            let data = self.villager_data.lock().await;
            nbt.put_int(data.r#type as i32);
            nbt.put_int(data.profession as i32);
            nbt.put_int(data.level);

            nbt.put_int(self.food_level.load(Ordering::Relaxed));
            nbt.put_int(self.xp.load(Ordering::Relaxed));
            nbt.put_long(self.last_restock_time.load(Ordering::Relaxed));
            nbt.put_int(self.restocks_today.load(Ordering::Relaxed));

            // Inventory
            let inventory = self.inventory.lock().await;
            nbt.put_int(inventory.len() as i32);
            for stack_mutex in inventory.iter() {
                let stack = stack_mutex.lock().await;
                stack.write_item_stack_pnbt(nbt);
            }

            // Gossips (Simplified: just save counts per UUID and type)
            let gossips = self.gossips.lock().await;
            nbt.put_int(gossips.len() as i32);
            for (uuid, types) in gossips.iter() {
                nbt.put_uuid(uuid);
                nbt.put_int(types.len() as i32);
                for (gtype, value) in types {
                    nbt.put_int(*gtype as i32);
                    nbt.put_int(*value);
                }
            }
        })
    }

    fn read_nbt_non_mut<'a>(
        &'a self,
        nbt: &'a mut pumpkin_nbt::pnbt::PNbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            let mut data = self.villager_data.lock().await;
            if let Ok(t) = nbt.get_int() {
                data.r#type = VillagerType::try_from(t).unwrap_or(VillagerType::Plains);
            }
            if let Ok(p) = nbt.get_int() {
                data.profession =
                    VillagerProfession::try_from(p).unwrap_or(VillagerProfession::None);
            }
            if let Ok(l) = nbt.get_int() {
                data.level = l;
            }

            if let Ok(food) = nbt.get_int() {
                self.food_level.store(food, Ordering::Relaxed);
            }
            if let Ok(xp) = nbt.get_int() {
                self.xp.store(xp, Ordering::Relaxed);
            }
            if let Ok(restock) = nbt.get_long() {
                self.last_restock_time.store(restock, Ordering::Relaxed);
            }
            if let Ok(today) = nbt.get_int() {
                self.restocks_today.store(today, Ordering::Relaxed);
            }

            // Inventory
            if let Ok(inv_len) = nbt.get_int() {
                let mut inventory = self.inventory.lock().await;
                inventory.clear();
                for _ in 0..inv_len {
                    let stack =
                        ItemStack::read_item_stack_pnbt(nbt).unwrap_or(ItemStack::EMPTY.clone());
                    inventory.push(Arc::new(Mutex::new(stack)));
                }
            }

            // Gossips
            if let Ok(gossip_len) = nbt.get_int() {
                let mut gossips = self.gossips.lock().await;
                gossips.clear();
                for _ in 0..gossip_len {
                    if let Ok(uuid) = nbt.get_uuid()
                        && let Ok(types_len) = nbt.get_int()
                    {
                        let mut types = HashMap::new();
                        for _ in 0..types_len {
                            if let (Ok(gtype), Ok(val)) = (nbt.get_int(), nbt.get_int()) {
                                let gossip_type = match gtype {
                                    0 => GossipType::MajorNegative,
                                    1 => GossipType::MinorNegative,
                                    2 => GossipType::MajorPositive,
                                    3 => GossipType::MinorPositive,
                                    4 => GossipType::Trading,
                                    _ => continue,
                                };
                                types.insert(gossip_type, val);
                            }
                        }
                        gossips.insert(uuid, types);
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

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        _item_stack: &'a mut pumpkin_data::item_stack::ItemStack,
    ) -> crate::entity::EntityBaseFuture<'a, bool> {
        let player = player.clone();
        Box::pin(async move {
            if self.get_entity().age.load(Ordering::Relaxed) < 0 {
                self.set_unhappy().await;
                return true;
            }

            let offers = self.offers.lock().await;
            if offers.is_empty() {
                self.set_unhappy().await;
                return true;
            }
            drop(offers);

            self.open_trading_screen(&player).await;

            true
        })
    }
}
