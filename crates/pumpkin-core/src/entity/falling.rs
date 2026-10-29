use crossbeam::atomic::AtomicCell;
use pumpkin_data::Block;
use pumpkin_data::BlockState;
use pumpkin_data::BlockStateId;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::client::CUpdateBlock;
use pumpkin_protocol::java::client::play::CBlockUpdate;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::resource_location::ToResourceLocation;
use pumpkin_world::generation::structure::template::{BlockStateResolver, PaletteEntry};
use pumpkin_world::world::BlockFlags;
use std::sync::atomic::{AtomicBool, AtomicI32};
use std::sync::{Arc, Mutex, atomic::Ordering};

use crate::{
    block::{
        blocks::falling::FallingBlock, entities::block_entity_from_nbt,
        registry::can_replace_with_other_block,
    },
    entity::{Entity, EntityBase, living::LivingEntity},
    plugin::api::events::entity::entity_change_block::EntityChangeBlockEvent,
    server::Server,
    world::World,
};

/// Vanilla `Entity.getAirDrag`.
const AIR_DRAG: f64 = 0.98;
/// Vanilla `FallingBlockEntity.tick`: drop after this many ticks while outside the build height.
const TIME_OUT_OF_WORLD: i32 = 100;
/// Vanilla `FallingBlockEntity.tick`: drop after this many ticks while still falling.
const TIME_MAX: i32 = 600;

fn state_to_nbt(state_id: BlockStateId) -> NbtCompound {
    let block = Block::from_state_id(state_id);
    let properties = block
        .properties(state_id)
        .map(|props| props.to_props())
        .unwrap_or_default();
    PaletteEntry::with_properties(block.to_resource_location(), properties).to_nbt_compound()
}

/// Vanilla falls back to sand when `BlockState` is missing or unknown.
fn state_from_nbt(nbt: &NbtCompound) -> BlockStateId {
    nbt.get_compound("BlockState")
        .and_then(|state| PaletteEntry::from_nbt_compound(state).ok())
        .and_then(|entry| BlockStateResolver::resolve_simple(&entry))
        .map_or(Block::SAND.default_state.id, |state| state.id)
}

pub struct FallingEntity {
    entity: Entity,
    block_state_id: AtomicCell<BlockStateId>,
    time: AtomicI32,
    /// Vanilla `FallingBlockEntity.dropItem`.
    drop_item: AtomicBool,
    /// Vanilla `FallingBlockEntity.cancelDrop`: land as nothing, neither block nor item.
    cancel_drop: AtomicBool,
    /// Vanilla `FallingBlockEntity.blockData`, NBT `TileEntityData`.
    block_data: Mutex<Option<NbtCompound>>,
}

impl FallingEntity {
    pub const fn new(mut entity: Entity, block_state_id: BlockStateId) -> Self {
        entity.blocks_building = true;
        Self {
            entity,
            block_state_id: AtomicCell::new(block_state_id),
            time: AtomicI32::new(0),
            drop_item: AtomicBool::new(true),
            cancel_drop: AtomicBool::new(false),
            block_data: Mutex::new(None),
        }
    }

    /// Replaced the current Block and Spawns a new Falling one (synchronous)
    pub fn replace_spawn(world: &Arc<World>, position: BlockPos, block_state: BlockStateId) {
        let block = Block::from_state_id(block_state);
        // Vanilla copies the state with `WATERLOGGED` cleared onto the entity.
        // `None` means the state has no water to clear.
        let falling_state = block
            .set_waterlogged(block_state, false)
            .unwrap_or(block_state);
        // Vanilla `FallingBlockEntity.fall`: leave `state.getFluidState().createLegacyBlock()`.
        let leftover = if block.is_waterlogged(block_state) {
            Block::WATER.default_state.id
        } else {
            Block::AIR.default_state.id
        };

        let spawn_pos = position.0.to_f64().add_raw(0.5, 0.0, 0.5);
        let entity = Entity::new(world.clone(), spawn_pos, &EntityType::FALLING_BLOCK);
        if world.cancelled_by_plugin(|| {
            EntityChangeBlockEvent::new(
                entity.entity_id,
                position,
                Block::from_state_id(leftover).to_resource_location(),
            )
        }) {
            return;
        }

        world.set_block_state(&position, leftover, BlockFlags::NOTIFY_ALL);
        entity
            .data
            .store(i32::from(falling_state.as_u16()), Ordering::Relaxed);
        world.spawn_entity_non_save(Arc::new(Self::new(entity, falling_state)));
    }

    /// Vanilla merges `TileEntityData` over the landed block's block entity.
    fn apply_block_data(&self, world: &Arc<World>, pos: BlockPos) {
        let block_data = self
            .block_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(data) = block_data.as_ref() else {
            return;
        };
        let Some(block_entity) = world.get_block_entity(&pos) else {
            return;
        };
        let mut merged = NbtCompound::new();
        block_entity.write_internal(&mut merged);
        for (key, value) in &data.child_tags {
            // Position and id stay those of the landed block entity.
            if !matches!(key.as_ref(), "x" | "y" | "z" | "id") {
                merged.child_tags.insert(key.clone(), value.clone());
            }
        }
        if let Some(reloaded) = block_entity_from_nbt(&merged) {
            world.add_block_entity(reloaded);
        }
    }

    fn drop_as_item(&self) {
        let world = self.entity.world.load();
        if self.drop_item.load(Ordering::Relaxed)
            && world.level_info.load().game_rules.entity_drops
            && let Some(item) =
                Item::from_id(Block::from_state_id(self.block_state_id.load()).item_id)
        {
            self.entity.spawn_at_location(ItemStack::new(1, item));
        }
    }
}

impl EntityBase for FallingEntity {
    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        let entity = &self.entity;
        if self.block_state_id.load() == BlockStateId::AIR {
            entity.remove();
            return;
        }
        // `Time` comes from NBT; wraps like vanilla's `time++` instead of overflowing.
        let time = self.time.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
        let mut velo = entity.velocity.load();
        velo.y -= self.get_gravity();

        entity.velocity.store(velo);

        // TODO: vanilla treats concrete powder in water as still falling (`isStuckInWater`, plus a source-fluid clip above speed 1).
        entity.move_entity(caller, velo);
        entity.tick_block_collisions(caller);
        if entity.on_ground.load(Ordering::Relaxed) {
            entity
                .velocity
                .store(entity.velocity.load().multiply(0.7, -0.5, 0.7));
            let world = entity.world.load();
            let landing_pos = self.entity.block_pos.load();
            let (current_block, current_state) = world.get_block_and_state(&landing_pos);
            // Vanilla waits until a piston has finished moving a block through the landing cell.
            if current_block != &Block::MOVING_PISTON {
                let mut state_id = self.block_state_id.load();
                let block = Block::from_state_id(state_id);
                let (below_block, below_state) = world.get_block_and_state(&landing_pos.down());
                let may_replace = can_replace_with_other_block(current_block, current_state);
                let would_continue_falling =
                    FallingBlock::can_fall_through(below_state, below_block);
                let would_survive = world.block_registry.can_place_at(
                    None,
                    Some(&**world),
                    &**world,
                    None,
                    block,
                    state_id.to_state(),
                    &landing_pos,
                    None,
                    None,
                ) && !would_continue_falling;
                if self.cancel_drop.load(Ordering::Relaxed) {
                    // Vanilla `cancelDrop`: the entity just disappears.
                } else if may_replace && would_survive {
                    if let Some(concrete) = FallingBlock::concrete_of(block)
                        && FallingBlock::should_solidify(&**world, &landing_pos)
                    {
                        state_id = concrete.default_state.id;
                    }
                    let place_block = Block::from_state_id(state_id);
                    if place_block.is_waterloggable()
                        && world
                            .get_fluid(&landing_pos)
                            .has_tag(&tag::Fluid::MINECRAFT_WATER)
                        && let Some(waterlogged) = place_block.set_waterlogged(state_id, true)
                    {
                        state_id = waterlogged;
                    }
                    if world.cancelled_by_plugin(|| {
                        EntityChangeBlockEvent::new(
                            entity.entity_id,
                            landing_pos,
                            Block::from_state_id(state_id).to_resource_location(),
                        )
                    }) {
                        // Vanilla drops the block when `setBlock` refuses.
                        self.drop_as_item();
                    } else {
                        world.set_block_state(&landing_pos, state_id, BlockFlags::NOTIFY_ALL);
                        self.apply_block_data(&world, landing_pos);
                        // block updates to watchers before the despawn, else a invisible block gap until the tick flush.
                        let placed = world.get_block_state_id(&landing_pos);
                        world.send_to_tracking_players_editioned(
                            entity,
                            &CBlockUpdate::new(landing_pos, i32::from(placed.as_u16()).into()),
                            &CUpdateBlock::new(landing_pos, BlockState::to_be_network_id(placed)),
                        );
                    }
                } else {
                    self.drop_as_item();
                }
                self.entity.remove();
            }
        } else {
            let pos = entity.block_pos.load();
            let world = entity.world.load();
            let out_of_world = pos.0.y <= world.get_bottom_y() || pos.0.y > world.get_top_y();
            if (time > TIME_OUT_OF_WORLD && out_of_world) || time > TIME_MAX {
                self.drop_as_item();
                self.entity.remove();
            }
        }

        entity.velocity.store(
            entity
                .velocity
                .load()
                .multiply(AIR_DRAG, AIR_DRAG, AIR_DRAG),
        );
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_compound("BlockState", state_to_nbt(self.block_state_id.load()));
        nbt.put_int("Time", self.time.load(Ordering::Relaxed));
        nbt.put_bool("DropItem", self.drop_item.load(Ordering::Relaxed));
        nbt.put_bool("CancelDrop", self.cancel_drop.load(Ordering::Relaxed));
        if let Some(data) = self
            .block_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            nbt.put_compound("TileEntityData", data.clone());
        }
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        let state_id = state_from_nbt(nbt);
        self.block_state_id.store(state_id);
        self.entity
            .data
            .store(i32::from(state_id.as_u16()), Ordering::Relaxed);
        self.time
            .store(nbt.get_numeric_int("Time").unwrap_or(0), Ordering::Relaxed);
        self.drop_item
            .store(nbt.get_bool("DropItem").unwrap_or(true), Ordering::Relaxed);
        self.cancel_drop.store(
            nbt.get_bool("CancelDrop").unwrap_or(false),
            Ordering::Relaxed,
        );
        *self
            .block_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            nbt.get_compound("TileEntityData").cloned();
    }

    fn init_data_tracker(&self) {
        self.entity.set_synced_data(
            pumpkin_data::tracked_data::falling_block::START_POS,
            self.entity.block_pos.load(),
        );
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }
    fn damage(&self, _caller: &dyn EntityBase, _amount: f32, _damage_type: DamageType) -> bool {
        false
    }

    fn get_gravity(&self) -> f64 {
        0.04
    }

    fn entity_bedrock_spawn_metadata(
        &self,
    ) -> Option<pumpkin_protocol::bedrock::client::set_actor_data::SyncedActorDataList> {
        let mut meta =
            pumpkin_protocol::bedrock::client::set_actor_data::SyncedActorDataList::new();
        let runtime_id = BlockState::to_be_network_id(self.block_state_id.load());
        meta.set(
            pumpkin_protocol::bedrock::client::set_actor_data::entity_data_key::VARIANT,
            pumpkin_protocol::bedrock::client::set_actor_data::MetadataValue::Int(
                runtime_id as i32,
            ),
        );
        Some(meta)
    }

    fn bedrock_y_offset(&self) -> f64 {
        0.49
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{state_from_nbt, state_to_nbt};
    use pumpkin_data::Block;
    use pumpkin_data::block_properties::Facing;
    use pumpkin_nbt::compound::NbtCompound;

    fn wrapped(state: pumpkin_data::BlockStateId) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        nbt.put_compound("BlockState", state_to_nbt(state));
        nbt
    }

    #[test]
    fn plain_state_round_trips() {
        let state = Block::GRAVEL.default_state.id;
        assert_eq!(state_from_nbt(&wrapped(state)), state);
    }

    #[test]
    fn state_with_properties_round_trips() {
        let state = Block::REPEATER
            .from_properties(&[pumpkin_data::Property::Facing(Facing::South)])
            .to_state_id(&Block::REPEATER);
        assert_eq!(state_from_nbt(&wrapped(state)), state);
    }

    #[test]
    fn missing_or_unknown_state_is_sand() {
        assert_eq!(
            state_from_nbt(&NbtCompound::new()),
            Block::SAND.default_state.id
        );

        let mut unknown = NbtCompound::new();
        let mut entry = NbtCompound::new();
        entry.put_string("id", "minecraft:not_a_block".to_string());
        unknown.put_compound("BlockState", entry);
        assert_eq!(state_from_nbt(&unknown), Block::SAND.default_state.id);
    }
}
