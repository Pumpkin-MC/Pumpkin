use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use pumpkin_data::Block;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::potion::Effect;
use pumpkin_data::tag::Taggable;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_data::world::WorldEvent;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{MerchantOffer, Metadata};
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::entity::mob::zombie::ZombieEntityBase;
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::passive::villager::{
    GossipType, VillagerData, VillagerEntity, VillagerProfession, villager_type_from_biome_id,
};
use crate::entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture};

pub struct ZombieVillagerEntity {
    pub mob_entity: Arc<ZombieEntityBase>,
    pub villager_data: Mutex<VillagerData>,
    pub villager_data_finalized: AtomicBool,
    pub gossips: Mutex<HashMap<Uuid, HashMap<GossipType, i32>>>,
    pub offers: Mutex<Vec<MerchantOffer>>,
    pub villager_xp: AtomicI32,
    pub conversion_time: AtomicI32,
    pub conversion_starter: Mutex<Option<Uuid>>,
}

impl ZombieVillagerEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let villager_type = {
            let pos = entity.block_pos.load();
            let world = entity.world.load();
            villager_type_from_biome_id(world.get_biome(&pos).registry_id)
        };
        let mob_entity = ZombieEntityBase::new(entity);
        let zombie = Self {
            mob_entity,
            villager_data: Mutex::new(VillagerData::new(
                villager_type,
                VillagerProfession::None,
                1,
            )),
            villager_data_finalized: AtomicBool::new(false),
            gossips: Mutex::new(HashMap::new()),
            offers: Mutex::new(Vec::new()),
            villager_xp: AtomicI32::new(0),
            conversion_time: AtomicI32::new(-1),
            conversion_starter: Mutex::new(None),
        };
        Arc::new(zombie)
    }

    fn copy_conversion_state(source: &Entity, target: &Entity) {
        target.yaw.store(source.yaw.load());
        target.pitch.store(source.pitch.load());
        target.head_yaw.store(source.head_yaw.load());
        target.body_yaw.store(source.body_yaw.load());
        target.velocity.store(source.velocity.load());
        target
            .on_ground
            .store(source.on_ground.load(Ordering::Relaxed), Ordering::Relaxed);
        target
            .age
            .store(source.age.load(Ordering::Relaxed), Ordering::Relaxed);
        target
            .fire_ticks
            .store(source.fire_ticks.load(Ordering::Relaxed), Ordering::Relaxed);
        target.has_visual_fire.store(
            source.has_visual_fire.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
    }

    pub async fn from_villager(villager: &VillagerEntity) -> Arc<Self> {
        let source = villager.get_entity();
        let converted = Self::new(Entity::new(
            source.world.load().clone(),
            source.pos.load(),
            &EntityType::ZOMBIE_VILLAGER,
        ));
        let target = converted.get_entity();

        Self::copy_conversion_state(source, target);

        let villager_data = *villager.villager_data.lock().await;
        let gossips = villager.gossips.lock().await.clone();
        let offers = villager.offers.lock().await.clone();
        *converted.villager_data.lock().await = villager_data;
        *converted.gossips.lock().await = gossips;
        *converted.offers.lock().await = offers;
        converted
            .villager_xp
            .store(villager.xp.load(Ordering::Relaxed), Ordering::Relaxed);
        converted
            .villager_data_finalized
            .store(true, Ordering::Relaxed);

        converted
    }

    fn is_converting(&self) -> bool {
        self.conversion_time.load(Ordering::Relaxed) >= 0
    }

    fn sync_conversion_metadata(&self) {
        self.get_entity().send_meta_data(
            &[
                Metadata::new(
                    TrackedData::CONVERTING,
                    MetaDataType::BOOLEAN,
                    self.is_converting(),
                ),
                Metadata::new(
                    TrackedData::CONVERTING_ID,
                    MetaDataType::BOOLEAN,
                    self.is_converting(),
                ),
            ],
            None,
        );
    }

    async fn start_converting(&self, starter: Option<Uuid>, time: i32) {
        let time = time.max(1);
        self.conversion_time.store(time, Ordering::Relaxed);
        *self.conversion_starter.lock().await = starter;
        self.sync_conversion_metadata();

        let living = &self.mob_entity.mob_entity.living_entity;
        living.remove_effect(&StatusEffect::WEAKNESS).await;
        living
            .add_effect(Effect {
                effect_type: &StatusEffect::STRENGTH,
                duration: time,
                amplifier: 0,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: false,
            })
            .await;
        self.get_entity().world.load().send_entity_status(
            self.get_entity(),
            pumpkin_data::entity::EntityStatus::ZombieConverting,
        );
    }

    fn conversion_progress(&self) -> i32 {
        if rand::random::<f32>() >= 0.01 {
            return 1;
        }

        let world = self.get_entity().world.load();
        let center = self.get_entity().block_pos.load();
        let mut amount = 1;
        let mut special_blocks = 0;

        for x in center.0.x - 4..center.0.x + 4 {
            for y in center.0.y - 4..center.0.y + 4 {
                for z in center.0.z - 4..center.0.z + 4 {
                    if special_blocks >= 14 {
                        return amount;
                    }

                    let block = world.get_block(&BlockPos::new(x, y, z));
                    if block != &Block::IRON_BARS
                        && !block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_BEDS)
                    {
                        continue;
                    }

                    special_blocks += 1;
                    if rand::random::<f32>() < 0.3 {
                        amount += 1;
                    }
                }
            }
        }

        amount
    }

    async fn finish_conversion(&self) {
        let source = self.get_entity();
        let world = source.world.load().clone();
        let Some(zombie) = world.get_entity_by_id(source.entity_id) else {
            return;
        };
        if zombie.get_entity().entity_type != &EntityType::ZOMBIE_VILLAGER {
            return;
        }

        let custom_name = source.custom_name.load().as_ref().clone();
        let custom_name_visible = source.custom_name_visible.load(Ordering::Relaxed);
        let villager = VillagerEntity::new(Entity::new(
            world.clone(),
            source.pos.load(),
            &EntityType::VILLAGER,
        ));
        Self::copy_conversion_state(source, villager.get_entity());

        let villager_data = *self.villager_data.lock().await;
        let gossips = self.gossips.lock().await.clone();
        let offers = self.offers.lock().await.clone();
        let starter = *self.conversion_starter.lock().await;
        *villager.villager_data.lock().await = villager_data;
        *villager.gossips.lock().await = gossips;
        *villager.offers.lock().await = offers;
        villager
            .xp
            .store(self.villager_xp.load(Ordering::Relaxed), Ordering::Relaxed);

        let villager_base: Arc<dyn EntityBase> = villager.clone();
        let block_pos = source.block_pos.load();
        world.remove_entity(zombie.as_ref()).await;
        world.broadcast_entity_spawn(&villager_base);
        villager.mob_init_data_tracker().await;
        world.add_entity_silent(villager_base).await;

        let villager_entity = villager.get_entity();
        if let Some(custom_name) = custom_name {
            villager_entity.set_custom_name(custom_name);
        }
        if custom_name_visible {
            villager_entity.set_custom_name_visible(true);
        }
        villager
            .mob_entity
            .living_entity
            .add_effect(Effect {
                effect_type: &StatusEffect::NAUSEA,
                duration: 200,
                amplifier: 0,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: false,
            })
            .await;

        if let Some(starter) = starter
            && let Some(player) = world.get_player_by_uuid(starter)
        {
            player
                .trigger_advancement(
                    crate::entity::player::advancement::trigger::AdvancementTrigger::CuredZombieVillager,
                )
                .await;
        }
        world.sync_world_event(WorldEvent::SoundZombieConverted, block_pos, 0);
    }
}

impl NBTStorage for ZombieVillagerEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.write_nbt(nbt).await;

            let data = *self.villager_data.lock().await;
            let mut villager_data = NbtCompound::new();
            villager_data.put_int("Type", data.r#type.0);
            villager_data.put_int("Profession", data.profession.0);
            villager_data.put_int("Level", data.level.0);
            nbt.put_compound("VillagerData", villager_data);
            nbt.put_bool(
                "VillagerDataFinalized",
                self.villager_data_finalized.load(Ordering::Relaxed),
            );
            nbt.put_int("Xp", self.villager_xp.load(Ordering::Relaxed));

            let conversion_time = self.conversion_time.load(Ordering::Relaxed);
            nbt.put_int(
                "ConversionTime",
                if conversion_time >= 0 {
                    conversion_time
                } else {
                    -1
                },
            );
            let conversion_starter = *self.conversion_starter.lock().await;
            if let Some(starter) = conversion_starter {
                nbt.put_uuid("ConversionPlayer", starter);
            }

            let offers = self.offers.lock().await;
            let mut recipes = Vec::with_capacity(offers.len());
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

                let mut sell = NbtCompound::new();
                offer.output.0.write_item_stack(&mut sell);
                recipe.put_compound("sell", sell);
                recipe.put_int("uses", offer.uses);
                recipe.put_int("maxUses", offer.max_uses);
                recipe.put_bool("rewardExp", !offer.is_disabled);
                recipe.put_int("xp", offer.xp);
                recipe.put_float("priceMultiplier", offer.price_multiplier);
                recipe.put_int("specialPrice", offer.special_price);
                recipe.put_int("demand", offer.demand);
                recipes.push(NbtTag::Compound(recipe));
            }
            let mut offers_nbt = NbtCompound::new();
            offers_nbt.put("Recipes", NbtTag::List(recipes));
            nbt.put_compound("Offers", offers_nbt);

            let gossips = self.gossips.lock().await;
            let mut gossip_list = Vec::new();
            for (target, values) in gossips.iter() {
                for (kind, value) in values {
                    let mut gossip = NbtCompound::new();
                    gossip.put_uuid("Target", *target);
                    gossip.put_int("Type", *kind as i32);
                    gossip.put_int("Value", *value);
                    gossip_list.push(NbtTag::Compound(gossip));
                }
            }
            nbt.put("Gossips", NbtTag::List(gossip_list));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.read_nbt_non_mut(nbt).await;

            if let Some(villager_data) = nbt.get_compound("VillagerData") {
                let mut data = self.villager_data.lock().await;
                if let Some(r#type) = villager_data.get_int("Type") {
                    data.r#type = VarInt(r#type);
                }
                if let Some(profession) = villager_data.get_int("Profession") {
                    data.profession = VarInt(profession);
                }
                if let Some(level) = villager_data.get_int("Level") {
                    data.level = VarInt(level);
                }
            }
            self.villager_data_finalized.store(
                nbt.get_bool("VillagerDataFinalized")
                    .or_else(|| nbt.get_compound("VillagerData").map(|_| true))
                    .unwrap_or(false),
                Ordering::Relaxed,
            );
            self.villager_xp
                .store(nbt.get_int("Xp").unwrap_or(0), Ordering::Relaxed);
            self.conversion_time.store(
                nbt.get_int("ConversionTime").unwrap_or(-1),
                Ordering::Relaxed,
            );
            *self.conversion_starter.lock().await = nbt.get_uuid("ConversionPlayer");

            if let Some(offers_nbt) = nbt.get_compound("Offers")
                && let Some(recipes) = offers_nbt.get_list("Recipes")
            {
                let mut offers = self.offers.lock().await;
                offers.clear();
                for recipe in recipes {
                    let Some(recipe) = recipe.extract_compound() else {
                        continue;
                    };
                    let buy = recipe
                        .get_compound("buy")
                        .and_then(ItemStack::read_item_stack);
                    let buy_b = recipe
                        .get_compound("buyB")
                        .and_then(ItemStack::read_item_stack);
                    let sell = recipe
                        .get_compound("sell")
                        .and_then(ItemStack::read_item_stack);
                    if let (Some(buy), Some(sell)) = (buy, sell) {
                        offers.push(MerchantOffer {
                            base_cost_a: buy.into(),
                            output: sell.into(),
                            cost_b: buy_b.map(Into::into),
                            is_disabled: !recipe.get_bool("rewardExp").unwrap_or(true),
                            uses: recipe.get_int("uses").unwrap_or(0),
                            max_uses: recipe.get_int("maxUses").unwrap_or(12),
                            xp: recipe.get_int("xp").unwrap_or(2),
                            special_price: recipe.get_int("specialPrice").unwrap_or(0),
                            price_multiplier: recipe.get_float("priceMultiplier").unwrap_or(0.05),
                            demand: recipe.get_int("demand").unwrap_or(0),
                        });
                    }
                }
            }

            if let Some(gossip_list) = nbt.get_list("Gossips") {
                let mut gossips = self.gossips.lock().await;
                gossips.clear();
                for gossip in gossip_list {
                    let Some(gossip) = gossip.extract_compound() else {
                        continue;
                    };
                    let (Some(target), Some(kind), Some(value)) = (
                        gossip.get_uuid("Target"),
                        gossip.get_int("Type"),
                        gossip.get_int("Value"),
                    ) else {
                        continue;
                    };
                    let kind = match kind {
                        0 => GossipType::MajorNegative,
                        1 => GossipType::MinorNegative,
                        2 => GossipType::MajorPositive,
                        3 => GossipType::MinorPositive,
                        4 => GossipType::Trading,
                        _ => continue,
                    };
                    gossips.entry(target).or_default().insert(kind, value);
                }
            }
        })
    }
}

impl Mob for ZombieVillagerEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity.mob_entity
    }

    fn is_mob_baby(&self) -> bool {
        self.mob_entity.is_mob_baby()
    }

    fn requires_custom_persistence(&self) -> bool {
        self.is_converting() || self.villager_xp.load(Ordering::Relaxed) > 0
    }

    fn supports_break_door_goal(&self) -> bool {
        true
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.mob_entity.mob_init_data_tracker().await;
            let villager_data = *self.villager_data.lock().await;
            self.sync_conversion_metadata();
            self.get_entity().send_meta_data(
                &[Metadata::new(
                    TrackedData::VILLAGER_DATA,
                    MetaDataType::VILLAGER_DATA,
                    villager_data,
                )],
                None,
            );
            self.get_entity().send_meta_data(
                &[Metadata::new(
                    TrackedData::VILLAGER_DATA_FINALIZED,
                    MetaDataType::BOOLEAN,
                    self.villager_data_finalized.load(Ordering::Relaxed),
                )],
                None,
            );
        })
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let conversion_time = self.conversion_time.load(Ordering::Relaxed);
            if conversion_time < 0 || !self.mob_entity.mob_entity.living_entity.is_alive() {
                return;
            }

            let remaining = (conversion_time - self.conversion_progress()).max(0);
            let state = if remaining == 0 { -2 } else { remaining };
            if self
                .conversion_time
                .compare_exchange(conversion_time, state, Ordering::Relaxed, Ordering::Relaxed)
                .is_err()
            {
                return;
            }

            if remaining == 0 {
                self.finish_conversion().await;
            }
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<crate::entity::player::Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if item_stack.item != &Item::GOLDEN_APPLE {
                return self.mob_entity.mob_interact(player, item_stack).await;
            }

            let living = &self.mob_entity.mob_entity.living_entity;
            if self.is_converting() || !living.has_effect(&StatusEffect::WEAKNESS).await {
                return true;
            }

            if player.gamemode.load() != GameMode::Creative {
                item_stack.decrement(1);
            }
            // Keep the (non-Send) thread rng out of the await point.
            let conversion_time = rand::rng().random_range(3600..=6000);
            self.start_converting(Some(player.get_entity().entity_uuid), conversion_time)
                .await;
            true
        })
    }
}
