use crossbeam::atomic::AtomicCell;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockState;
use pumpkin_data::BlockStateId;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, tag, tag::Taggable, tracked_data::TrackedData};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::generation::structure::template::{BlockStateResolver, PaletteEntry};
use pumpkin_world::world::BlockFlags;
use std::sync::atomic::{AtomicBool, AtomicI32};
use std::sync::{Arc, atomic::Ordering};
use tokio::sync::Mutex;

use crate::block::blocks::falling::FallingBlock;
use crate::block::entities::{block_entity_from_nbt, has_block_block_entity};
use crate::{
    entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture, living::LivingEntity},
    server::Server,
    world::World,
};

const DEFAULT_FALL_DAMAGE_MAX: i32 = 40;

pub struct FallingEntity {
    entity: Entity,
    block_state_id: AtomicCell<BlockStateId>,
    time: AtomicI32,
    drop_item: AtomicBool,
    cancel_drop: AtomicBool,
    hurt_entities: AtomicBool,
    fall_damage_per_distance: AtomicCell<f32>,
    fall_damage_max: AtomicI32,
    block_data: Mutex<Option<NbtCompound>>,
    /// Distance fallen since the last time this entity was on the ground, used by the
    /// `causeFallDamage`-equivalent crushing/anvil-damage-tier logic on landing. Not persisted:
    /// vanilla's `FallingBlockEntity` does not save it either.
    fall_distance: AtomicCell<f64>,
}

impl FallingEntity {
    pub fn new(entity: Entity, block_state_id: BlockStateId) -> Self {
        let is_anvil = Block::from_state_id(block_state_id).has_tag(&tag::Block::MINECRAFT_ANVIL);
        // `AnvilBlock.falling`: `entity.setHurtsEntities(2.0F, 40)`, applied at spawn time since
        // it is unconditional for every anvil falling entity.
        let (fall_damage_per_distance, fall_damage_max) = if is_anvil {
            (2.0, 40)
        } else {
            (0.0, DEFAULT_FALL_DAMAGE_MAX)
        };
        Self {
            entity,
            block_state_id: AtomicCell::new(block_state_id),
            time: AtomicI32::new(0),
            drop_item: AtomicBool::new(true),
            cancel_drop: AtomicBool::new(false),
            hurt_entities: AtomicBool::new(is_anvil),
            fall_damage_per_distance: AtomicCell::new(fall_damage_per_distance),
            fall_damage_max: AtomicI32::new(fall_damage_max),
            block_data: Mutex::new(None),
            fall_distance: AtomicCell::new(0.0),
        }
    }

    /// Vanilla `FallingBlockEntity.setHurtsEntities`: called by block placement code (anvils)
    /// to opt this falling instance into crushing living entities on landing.
    pub fn set_hurts_entities(&self, damage_per_distance: f32, damage_max: i32) {
        self.hurt_entities.store(true, Ordering::Relaxed);
        self.fall_damage_per_distance.store(damage_per_distance);
        self.fall_damage_max.store(damage_max, Ordering::Relaxed);
    }

    /// Replaced the current Block and Spawns a new Falling one
    pub async fn replace_spawn(world: &Arc<World>, position: BlockPos, block_state: BlockStateId) {
        Self::replace_spawn_hurting(world, position, block_state, None).await;
    }

    /// Same as [`Self::replace_spawn`], but optionally opts the spawned instance into crushing
    /// living entities (`setHurtsEntities`) *before* it is handed to the world, so the tick loop
    /// can never observe the entity without its damage parameters set.
    pub async fn replace_spawn_hurting(
        world: &Arc<World>,
        position: BlockPos,
        block_state: BlockStateId,
        hurts_entities: Option<(f32, i32)>,
    ) {
        // Replace the original block, TODO: use fluid state
        world
            .set_block_state(
                &position,
                Block::AIR.default_state.id,
                BlockFlags::NOTIFY_ALL,
            )
            .await;

        let spawn_position = position.0.to_f64().add_raw(0.5, 0.0, 0.5);
        let entity = Entity::new(world.clone(), spawn_position, &EntityType::FALLING_BLOCK);
        entity
            .data
            .store(i32::from(block_state.as_u16()), Ordering::Relaxed);
        let entity = Arc::new(Self::new(entity, block_state));
        if let Some((damage_per_distance, damage_max)) = hurts_entities {
            entity.set_hurts_entities(damage_per_distance, damage_max);
        }
        world.spawn_entity(entity).await;
    }

    /// The item form of the block this falling entity carries, matching vanilla's
    /// `spawnAtLocation(serverLevel, block)` (drops the block converted to an item).
    fn carried_item(&self) -> Option<&'static Item> {
        let block = Block::from_state_id(self.block_state_id.load());
        Item::from_id(block.item_id)
    }

    async fn drop_carried_item(&self, world: &Arc<World>, pos: &BlockPos) {
        if let Some(item) = self.carried_item() {
            world.drop_stack(pos, ItemStack::new(1, item)).await;
        }
    }

    fn entity_drops_enabled(world: &World) -> bool {
        world.level_info.load().game_rules.entity_drops
    }

    /// `FallingBlockEntity.causeFallDamage`: hurts living entities in the falling block's
    /// current bounding box, then (anvils only) rolls the chip/damage/destroy progression.
    /// Called once per landing tick; `fall_distance` is the accumulated fall distance already
    /// consumed (reset) by the caller.
    async fn crush_and_progress(&self, world: &Arc<World>, fall_distance: f64) {
        if !self.hurt_entities.load(Ordering::Relaxed) {
            return;
        }
        let fall_distance_int = crate::block::blocks::falling::fall_damage_distance(fall_distance);
        if fall_distance_int < 0 {
            return;
        }

        let damage = crate::block::blocks::falling::fall_damage_amount(
            fall_distance_int,
            self.fall_damage_per_distance.load(),
            self.fall_damage_max.load(Ordering::Relaxed),
        );

        let falling_block = Block::from_state_id(self.block_state_id.load());
        let is_anvil = falling_block.has_tag(&tag::Block::MINECRAFT_ANVIL);
        let damage_type = if is_anvil {
            DamageType::FALLING_ANVIL
        } else if falling_block == &Block::POINTED_DRIPSTONE {
            // A falling stalactite kills with the "fallingStalactite" death message
            // (`minecraft:falling_stalactite` damage type), not the generic falling-block one.
            DamageType::FALLING_STALACTITE
        } else {
            DamageType::FALLING_BLOCK
        };

        let bb = self.entity.bounding_box.load();
        let mut candidates = world.get_entities_at_box(&bb);
        for player in world.get_players_at_box(&bb) {
            candidates.push(player as Arc<dyn EntityBase>);
        }
        for candidate in candidates {
            if candidate.get_living_entity().is_none() || !candidate.get_entity().is_alive() {
                continue;
            }
            if candidate.is_spectator() {
                continue;
            }
            if let Some(player) = candidate.get_player()
                && player.gamemode.load() == GameMode::Creative
            {
                continue;
            }
            candidate
                .damage(candidate.as_ref(), damage, damage_type)
                .await;
        }

        if is_anvil && damage > 0.0 {
            let chance = crate::block::blocks::falling::anvil_damage_chance(fall_distance_int);
            if rand::random::<f32>() < chance {
                match crate::block::blocks::falling::anvil_damage_tier(falling_block) {
                    Some(next) => self.block_state_id.store(next.default_state.id),
                    None => self.cancel_drop.store(true, Ordering::Relaxed),
                }
            }
        }
    }

    /// `AnvilBlock.onLand`: `level.levelEvent(1031, pos, 0)`.
    fn play_land_sound(world: &Arc<World>, pos: &BlockPos, block: &'static Block) {
        if block.has_tag(&tag::Block::MINECRAFT_ANVIL) {
            world.sync_world_event(WorldEvent::SoundAnvilLand, *pos, 0);
        }
    }

    /// `AnvilBlock.onBrokenAfterFall`: `level.levelEvent(1029, pos, 0)`.
    fn play_broken_sound(world: &Arc<World>, pos: &BlockPos, block: &'static Block) {
        if block.has_tag(&tag::Block::MINECRAFT_ANVIL) {
            world.sync_world_event(WorldEvent::SoundAnvilBroken, *pos, 0);
        }
    }

    /// Resolves what happens once the falling block has settled onto solid ground: either it
    /// gets placed (successfully or replaced by a solidified variant) or it breaks and drops.
    #[allow(clippy::too_many_arguments)]
    async fn handle_landing(
        &self,
        server: &Server,
        world: &Arc<World>,
        pos: &BlockPos,
        falling_state_id: BlockStateId,
        falling_block: &'static Block,
        current_state: &'static BlockState,
        is_concrete: bool,
        is_stuck_in_water: bool,
    ) {
        let entity = &self.entity;
        if self.cancel_drop.load(Ordering::Relaxed) {
            Self::play_broken_sound(world, pos, falling_block);
            entity.remove().await;
            return;
        }

        let may_replace = current_state.replaceable();
        let below_pos = pos.down();
        let below_state = world.get_block_state(&below_pos);
        let below_block = Block::from_state_id(below_state.id);
        let would_continue_falling = FallingBlock::can_fall_through(below_state, below_block)
            && !(is_concrete && is_stuck_in_water);
        let placed_state = BlockState::from_id(falling_state_id);
        // A falling stalactite can never survive landing: it fell down a column of passable
        // blocks, so there is nothing above it to hang from, while a downward-pointing pointed
        // dripstone requires support above. `can_place_at` would nevertheless accept it, because
        // it is asked with `BlockDirection::Down` and would latch onto the floor below, placing a
        // stalactite standing upside down on the ground. Force the break/drop branch instead.
        let cannot_survive_landing =
            crate::block::blocks::dripstone::is_stalactite(falling_block, falling_state_id);
        let would_survive = !cannot_survive_landing
            && world.block_registry.can_place_at(
                Some(server),
                Some(world),
                &**world,
                None,
                falling_block,
                placed_state,
                pos,
                Some(BlockDirection::Down),
                None,
            )
            && !would_continue_falling;

        if may_replace && would_survive {
            let final_state_id = crate::block::blocks::falling::on_land_state(
                &**world,
                pos,
                falling_state_id,
                current_state,
            );
            world
                .set_block_state(pos, final_state_id, BlockFlags::NOTIFY_ALL)
                .await;
            Self::play_land_sound(world, pos, falling_block);
            entity.remove().await;

            let stored_block_data = self.block_data.lock().await.take();
            if let Some(block_data) = stored_block_data
                && has_block_block_entity(Block::from_state_id(final_state_id))
                && let Some(block_entity) = world.get_block_entity(pos)
            {
                let mut merged = NbtCompound::new();
                block_entity.write_internal(&mut merged).await;
                merged.child_tags.extend(block_data.child_tags);
                if let Some(new_entity) = block_entity_from_nbt(&merged) {
                    world.add_block_entity(new_entity);
                }
            }
        } else {
            Self::play_broken_sound(world, pos, falling_block);
            entity.remove().await;
            if self.drop_item.load(Ordering::Relaxed) && Self::entity_drops_enabled(world) {
                self.drop_carried_item(world, pos).await;
            }
        }
    }
}

impl NBTStorage for FallingEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            let state_id = self.block_state_id.load();
            let block = Block::from_state_id(state_id);
            let mut block_state_compound = NbtCompound::new();
            block_state_compound.put_string("Name", format!("minecraft:{}", block.name));
            if let Some(properties) = block.properties(state_id) {
                let props = properties.to_props();
                if !props.is_empty() {
                    let mut properties_compound = NbtCompound::new();
                    for (key, value) in props {
                        properties_compound.put_string(key, value.to_string());
                    }
                    block_state_compound.put_compound("Properties", properties_compound);
                }
            }
            nbt.put_compound("BlockState", block_state_compound);

            nbt.put_int("Time", self.time.load(Ordering::Relaxed));
            nbt.put_bool("DropItem", self.drop_item.load(Ordering::Relaxed));
            nbt.put_bool("HurtEntities", self.hurt_entities.load(Ordering::Relaxed));
            nbt.put_float("FallHurtAmount", self.fall_damage_per_distance.load());
            nbt.put_int("FallHurtMax", self.fall_damage_max.load(Ordering::Relaxed));
            if let Some(block_data) = &*self.block_data.lock().await {
                nbt.put_compound("TileEntityData", block_data.clone());
            }
            nbt.put_bool("CancelDrop", self.cancel_drop.load(Ordering::Relaxed));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            if let Some(block_state_compound) = nbt.get_compound("BlockState")
                && let Some(name) = block_state_compound.get_string("Name")
            {
                let properties = block_state_compound.get_compound("Properties").map_or_else(
                    Vec::new,
                    |props_compound| {
                        props_compound
                            .child_tags
                            .iter()
                            .filter_map(|(key, value)| {
                                if let pumpkin_nbt::tag::NbtTag::String(v) = value {
                                    Some((key.to_string(), v.to_string()))
                                } else {
                                    None
                                }
                            })
                            .collect()
                    },
                );
                let entry = PaletteEntry::with_properties(name.to_string(), properties);
                if let Some(state) = BlockStateResolver::resolve_simple(&entry) {
                    self.block_state_id.store(state.id);
                    self.entity
                        .data
                        .store(i32::from(state.id.as_u16()), Ordering::Relaxed);
                }
            }

            self.time
                .store(nbt.get_int("Time").unwrap_or(0), Ordering::Relaxed);
            self.drop_item
                .store(nbt.get_bool("DropItem").unwrap_or(true), Ordering::Relaxed);
            let default_hurt_entities = Block::from_state_id(self.block_state_id.load())
                .has_tag(&tag::Block::MINECRAFT_ANVIL);
            self.hurt_entities.store(
                nbt.get_bool("HurtEntities")
                    .unwrap_or(default_hurt_entities),
                Ordering::Relaxed,
            );
            self.fall_damage_per_distance
                .store(nbt.get_float("FallHurtAmount").unwrap_or(0.0));
            self.fall_damage_max.store(
                nbt.get_int("FallHurtMax")
                    .unwrap_or(DEFAULT_FALL_DAMAGE_MAX),
                Ordering::Relaxed,
            );
            *self.block_data.lock().await = nbt.get_compound("TileEntityData").cloned();
            self.cancel_drop.store(
                nbt.get_bool("CancelDrop").unwrap_or(false),
                Ordering::Relaxed,
            );
        })
    }
}

impl EntityBase for FallingEntity {
    fn is_pickable(&self) -> bool {
        self.entity.is_alive()
    }

    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.entity;
            entity.tick(caller, server).await;
            self.time.fetch_add(1, Ordering::Relaxed);

            let original_velo = entity.velocity.load();
            let mut velo = original_velo;
            velo.y -= self.get_gravity();

            entity.velocity.store(velo);

            let y_before = entity.pos.load().y;
            entity.move_entity(caller, velo).await;
            entity.tick_block_collisions(caller, server).await;
            let fell = y_before - entity.pos.load().y;
            if fell > 0.0 {
                self.fall_distance.store(self.fall_distance.load() + fell);
            } else {
                self.fall_distance.store(0.0);
            }

            let world = entity.world.load();
            let pos = entity.block_pos.load();
            let on_ground = entity.on_ground.load(Ordering::Relaxed);

            if on_ground {
                let fall_distance = self.fall_distance.swap(0.0);
                self.crush_and_progress(&world, fall_distance).await;
            }

            let falling_state_id = self.block_state_id.load();
            let falling_block = Block::from_state_id(falling_state_id);
            let is_concrete =
                crate::block::blocks::falling::concrete_for_powder(falling_block).is_some();
            let pos_state = world.get_block_state(&pos);
            // Approximates vanilla's `isStuckInWater`; the two-step clip-based fast-fall check
            // (`ClipContext` scan for concrete moving faster than 1 block/tick) is not ported.
            let is_stuck_in_water = is_concrete && pos_state.is_waterlogged();

            if !on_ground && !is_stuck_in_water {
                let time = self.time.load(Ordering::Relaxed);
                let outside_world = pos.0.y <= world.get_bottom_y() || pos.0.y > world.get_top_y();
                if (time > 100 && outside_world) || time > 600 {
                    if self.drop_item.load(Ordering::Relaxed) && Self::entity_drops_enabled(&world)
                    {
                        self.drop_carried_item(&world, &pos).await;
                    }
                    entity.remove().await;
                    return;
                }
            } else {
                entity.velocity.store(velo.multiply(0.7, -0.5, 0.7));
                let current_state = world.get_block_state(&pos);
                if Block::from_state_id(current_state.id) != &Block::MOVING_PISTON {
                    self.handle_landing(
                        server,
                        &world,
                        &pos,
                        falling_state_id,
                        falling_block,
                        current_state,
                        is_concrete,
                        is_stuck_in_water,
                    )
                    .await;
                    return;
                }
            }

            entity.velocity.store(velo.multiply(0.98, 0.98, 0.98));

            if entity.velocity_dirty.swap(false, Ordering::SeqCst) {
                entity.send_pos_rot();
                entity.send_velocity();
            }
        })
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::START_POS,
                    MetaDataType::BLOCK_POS,
                    self.entity.block_pos.load(),
                )],
                None,
            );
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    /// Vanilla `hurtServer`: falling blocks are never damaged directly (`isAttackable() ==
    /// false`), and this correctly returns false unconditionally. The crushing-a-living-entity
    /// behavior on landing is a *separate* vanilla method, `causeFallDamage`, driven by the
    /// falling block's own fall-distance tracking, not by this take-damage entry point.
    /// Pumpkin's fall-distance/fall-damage dispatch (`LivingEntity::handle_fall_damage`) is
    /// currently `LivingEntity`-only with no generic per-`Entity` hook this non-living entity
    /// could plug into, so the anvil/`hurt_entities` crush-on-landing behavior (including the
    /// anvil damage-stage roll) is not implemented. `hurt_entities`/`fall_damage_per_distance`/
    /// `fall_damage_max` are still tracked and persisted so a follow-up has the state to build
    /// on; this is a known, reported gap, not something this override should attempt to guess.
    fn damage<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move { false })
    }

    fn get_gravity(&self) -> f64 {
        0.04
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
