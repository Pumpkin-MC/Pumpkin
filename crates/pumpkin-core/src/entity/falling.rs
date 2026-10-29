use pumpkin_data::Block;
use pumpkin_data::BlockState;
use pumpkin_data::BlockStateId;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_protocol::bedrock::client::CUpdateBlock;
use pumpkin_protocol::java::client::play::CBlockUpdate;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use std::sync::atomic::AtomicI32;
use std::sync::{Arc, atomic::Ordering};

use crate::{
    block::{blocks::falling::FallingBlock, registry::can_replace_with_other_block},
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

pub struct FallingEntity {
    entity: Entity,
    block_state_id: BlockStateId,
    time: AtomicI32,
}

impl FallingEntity {
    pub const fn new(mut entity: Entity, block_state_id: BlockStateId) -> Self {
        entity.blocks_building = true;
        Self {
            entity,
            block_state_id,
            time: AtomicI32::new(0),
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
            Fluid::WATER.states[Fluid::WATER.default_state_index as usize].block_state_id
        } else {
            Block::AIR.default_state.id
        };

        let spawn_pos = position.0.to_f64().add_raw(0.5, 0.0, 0.5);
        let entity = Entity::new(world.clone(), spawn_pos, &EntityType::FALLING_BLOCK);
        if world.cancelled_by_plugin(|| {
            EntityChangeBlockEvent::new(
                entity.entity_id,
                position,
                format!("minecraft:{}", Block::from_state_id(leftover).name),
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

    fn drop_as_item(&self) {
        let world = self.entity.world.load();
        if world.level_info.load().game_rules.entity_drops
            && let Some(item) = Item::from_id(Block::from_state_id(self.block_state_id).item_id)
        {
            self.entity.spawn_at_location(ItemStack::new(1, item));
        }
    }
}

impl EntityBase for FallingEntity {
    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        let entity = &self.entity;
        let time = self.time.fetch_add(1, Ordering::Relaxed) + 1;
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
                let mut state_id = self.block_state_id;
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
                if may_replace && would_survive {
                    if block.has_tag(&tag::Block::MINECRAFT_CONCRETE_POWDERS)
                        && FallingBlock::should_solidify(&**world, &landing_pos)
                        && let Some(name) = block.name.strip_suffix("_powder")
                        && let Some(concrete) = Block::from_name(name)
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
                            format!("minecraft:{}", Block::from_state_id(state_id).name),
                        )
                    }) {
                        // Vanilla drops the block when `setBlock` refuses.
                        self.drop_as_item();
                    } else {
                        world.set_block_state(&landing_pos, state_id, BlockFlags::NOTIFY_ALL);
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
        let runtime_id = BlockState::to_be_network_id(self.block_state_id);
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
