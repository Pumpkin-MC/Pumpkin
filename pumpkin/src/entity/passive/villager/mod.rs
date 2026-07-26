use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, AtomicI64, Ordering};
use std::sync::{Arc, Weak};
use uuid::Uuid;

use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::inventory::SimpleInventory;
use tokio::sync::Mutex;

use crate::entity::player::Player;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        avoid_entity::AvoidEntityGoal, door_interact::OpenDoorGoal,
        escape_danger::EscapeDangerGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, villager_schedule::VillagerScheduleGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub mod data;
mod job;
mod nbt;
mod trading;
pub use data::{
    BREEDING_FOOD_THRESHOLD, GossipType, VillagerData, VillagerProfession, VillagerType,
    get_food_points, villager_type_from_biome_id,
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
    pub job_site: std::sync::Mutex<Option<BlockPos>>,
    pub home_pos: std::sync::Mutex<Option<BlockPos>>,
    pub self_weak: std::sync::Mutex<Option<Weak<Self>>>,
}

impl VillagerEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        // Match clothing to spawn biome (snowy → snow, desert → desert, …).
        let vtype = {
            let pos = mob_entity.living_entity.entity.block_pos.load();
            let world = mob_entity.living_entity.entity.world.load();
            let biome = world.get_biome(&pos);
            villager_type_from_biome_id(biome.registry_id)
        };
        let villager_data = VillagerData::new(vtype, VillagerProfession::None, 1);
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

        {
            // Vanilla Villager constructor: getNavigation().setCanOpenDoors(true).
            let mut navigator = mob_arc.mob_entity.navigator.lock().unwrap();
            navigator.set_can_open_doors(true);
        }

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            // Panic when damaged (vanilla PanicGoal / brain flee stand-in).
            goal_selector.add_goal(0, EscapeDangerGoal::new(0.5));
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

            // Vanilla villagers open doors and close them behind themselves
            // (brain InteractWithDoor; OpenDoorGoal is the goal-based stand-in).
            goal_selector.add_goal(2, Box::new(OpenDoorGoal::new(true)));
            // Brain schedule stand-in: walk home at dusk, to the job site by day.
            goal_selector.add_goal(2, Box::new(VillagerScheduleGoal::new(0.5)));

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
#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::item::Item;

    // Compile-time assertions that the public paths and signatures survived the
    // module split (re-exported through `crate::entity::passive::villager`).
    const _: fn(&str) -> VillagerType =
        crate::entity::passive::villager::villager_type_from_biome_id;
    const _: fn(&Item) -> i32 = crate::entity::passive::villager::get_food_points;
    const _: i32 = crate::entity::passive::villager::BREEDING_FOOD_THRESHOLD;

    #[test]
    fn villager_clothing_follows_spawn_biome() {
        assert_eq!(
            villager_type_from_biome_id("minecraft:desert"),
            VillagerType::Desert
        );
        assert_eq!(
            villager_type_from_biome_id("minecraft:jungle"),
            VillagerType::Jungle
        );
        assert_eq!(
            villager_type_from_biome_id("minecraft:snowy_plains"),
            VillagerType::Snow
        );
        assert_eq!(
            villager_type_from_biome_id("minecraft:plains"),
            VillagerType::Plains
        );
        // Unprefixed ids resolve the same way.
        assert_eq!(villager_type_from_biome_id("taiga"), VillagerType::Taiga);
    }

    #[test]
    fn food_points_match_vanilla_values() {
        assert_eq!(get_food_points(&Item::BREAD), 4);
        assert_eq!(get_food_points(&Item::CARROT), 1);
        assert_eq!(get_food_points(&Item::POTATO), 1);
        assert_eq!(get_food_points(&Item::BEETROOT), 1);
        assert_eq!(get_food_points(&Item::STONE), 0);
    }

    #[test]
    fn villager_data_round_trips_type_profession_and_level() {
        let data = VillagerData::new(VillagerType::Desert, VillagerProfession::Farmer, 2);
        assert_eq!(data.type_enum(), VillagerType::Desert);
        assert_eq!(data.profession_enum(), VillagerProfession::Farmer);
        assert_eq!(data.level.0, 2);
    }
}
