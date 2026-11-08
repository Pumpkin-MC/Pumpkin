use std::sync::Arc;

use pumpkin_data::{
    Block, BlockDirection, BlockId, BlockStateId, block_properties::WallTorchLikeProperties,
    entity::EntityType, world::WorldEvent,
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::{
    block::{BlockBehaviour, BlockMetadata, OnPlaceArgs, PlacedArgs, PlayerPlacedArgs},
    entity::{
        Entity, EntityBase,
        passive::{
            copper_golem::{CopperGolemEntity, WeatherState},
            iron_golem::IronGolemEntity,
            snow_golem::SnowGolemEntity,
        },
    },
    world::World,
};

pub struct GolemPattern {
    entity_type: &'static EntityType,
    blocks: Vec<BlockPos>,
    base: BlockPos,
}

// Mojang uses some BlockPattern magic, way too complex tbh
#[must_use]
pub fn find_golem_pattern(world: &Arc<World>, pos: &BlockPos) -> Option<GolemPattern> {
    let down_pos = pos.down();
    let upper = world.get_block(&down_pos);
    let lower = world.get_block(&down_pos.down());

    if upper == &Block::SNOW_BLOCK && lower == &Block::SNOW_BLOCK {
        return Some(GolemPattern {
            entity_type: &EntityType::SNOW_GOLEM,
            blocks: vec![*pos, down_pos, down_pos.down()],
            base: down_pos.down(),
        });
    }

    if upper != &Block::IRON_BLOCK || lower != &Block::IRON_BLOCK {
        return find_copper_golem_pattern(world, pos);
    }

    for dir in [BlockDirection::North, BlockDirection::West] {
        let arm1 = down_pos.offset(dir.to_offset());
        let arm2 = down_pos.offset(dir.opposite().to_offset());
        let above_arm1 = arm1.up();
        let above_arm2 = arm2.up();
        let below_arm1 = arm1.down();
        let below_arm2 = arm2.down();

        if world.get_block(&arm1) == &Block::IRON_BLOCK
            && world.get_block(&arm2) == &Block::IRON_BLOCK
            && world.get_block_state(&above_arm1).is_air()
            && world.get_block_state(&above_arm2).is_air()
            && world.get_block_state(&below_arm1).is_air()
            && world.get_block_state(&below_arm2).is_air()
        {
            return Some(GolemPattern {
                entity_type: &EntityType::IRON_GOLEM,
                blocks: vec![*pos, down_pos, down_pos.down(), arm1, arm2],
                base: down_pos.down(),
            });
        }
    }

    find_copper_golem_pattern(world, pos)
}

fn find_copper_golem_pattern(world: &World, origin: &BlockPos) -> Option<GolemPattern> {
    let requires_pumpkin = is_pumpkin(world.get_block(origin));
    // BlockPattern.find scans a 2x2x2 cube, X first, then forward/up orientations.
    for z in 0..=1 {
        for y in 0..=1 {
            for x in 0..=1 {
                let pumpkin = origin.offset(pumpkin_util::math::vector3::Vector3::new(x, y, z));
                if requires_pumpkin && !is_pumpkin(world.get_block(&pumpkin)) {
                    continue;
                }
                for direction in [
                    BlockDirection::South,
                    BlockDirection::North,
                    BlockDirection::East,
                    BlockDirection::West,
                    BlockDirection::Up,
                    BlockDirection::Down,
                ] {
                    let copper = pumpkin.offset(direction.to_offset());
                    let body = world.get_block(&copper);
                    if body.id.has_tag(pumpkin_data::tag::Block::MINECRAFT_COPPER) {
                        return Some(GolemPattern {
                            entity_type: &EntityType::COPPER_GOLEM,
                            blocks: vec![pumpkin, copper],
                            base: pumpkin,
                        });
                    }
                }
            }
        }
    }
    None
}

fn is_pumpkin(block: &Block) -> bool {
    block == &Block::CARVED_PUMPKIN || block == &Block::JACK_O_LANTERN
}

fn spawn_copper_golem(world: &Arc<World>, pattern: &GolemPattern) {
    let [pumpkin, copper] = pattern.blocks.as_slice() else {
        return;
    };
    let (pumpkin_block, pumpkin_state) = world.get_block_and_state(pumpkin);
    if !is_pumpkin(pumpkin_block) {
        return;
    }
    let (body, copper_state) = world.get_block_and_state_id(copper);
    let facing = WallTorchLikeProperties::from_state_id(pumpkin_state.id).facing;
    let position = pumpkin.to_f64().add_raw(0.0, 0.05, 0.0);
    let golem = CopperGolemEntity::new(Entity::new(
        world.clone(),
        position,
        &EntityType::COPPER_GOLEM,
    ));
    let weather_state = match body.id {
        BlockId::EXPOSED_COPPER | BlockId::WAXED_EXPOSED_COPPER => WeatherState::Exposed,
        BlockId::WEATHERED_COPPER | BlockId::WAXED_WEATHERED_COPPER => WeatherState::Weathered,
        BlockId::OXIDIZED_COPPER | BlockId::WAXED_OXIDIZED_COPPER => WeatherState::Oxidized,
        _ => WeatherState::Unaffected,
    };
    golem.set_weather_state(weather_state);
    if !world.spawn_entity(golem.clone()) {
        return;
    }
    for (position, state) in [(*pumpkin, pumpkin_state.id), (*copper, copper_state)] {
        world.set_block_state(
            &position,
            Block::AIR.default_state.id,
            BlockFlags::NOTIFY_LISTENERS,
        );
        world.sync_world_event(
            WorldEvent::ParticlesDestroyBlock,
            position,
            i32::from(state.as_u16()),
        );
    }
    for position in &pattern.blocks {
        world.update_neighbors_at(position, &Block::AIR, None);
    }
    let chest_state =
        super::chests::CopperChestBlock::get_from_copper_block(body, facing, world, copper);
    world.set_block_state(copper, chest_state, BlockFlags::NOTIFY_LISTENERS);
    golem.play_spawn_sound();
}

fn spawn_golem(world: &Arc<World>, pattern: &GolemPattern) {
    if pattern.entity_type == &EntityType::COPPER_GOLEM {
        spawn_copper_golem(world, pattern);
        return;
    }
    for &pos in &pattern.blocks {
        let state_id = world.get_block_state_id(&pos);
        world.set_block_state(
            &pos,
            Block::AIR.default_state.id,
            BlockFlags::NOTIFY_LISTENERS,
        );
        world.sync_world_event(
            WorldEvent::ParticlesDestroyBlock,
            pos,
            i32::from(state_id.as_u16()),
        );
    }

    let mut spawn_pos = pattern.base.to_centered_f64();
    spawn_pos.y = f64::from(pattern.base.0.y) + 0.05;
    let entity = Entity::new(world.clone(), spawn_pos, pattern.entity_type);
    let golem: Arc<dyn EntityBase> = if pattern.entity_type == &EntityType::SNOW_GOLEM {
        SnowGolemEntity::new(entity)
    } else {
        IronGolemEntity::new(entity)
    };
    world.spawn_entity(golem);

    for &pos in &pattern.blocks {
        world.update_neighbors(&pos, None);
    }
}

pub struct CarvedPumpkinBlock;

impl BlockMetadata for CarvedPumpkinBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::JACK_O_LANTERN, BlockId::CARVED_PUMPKIN].into()
    }
}

impl BlockBehaviour for CarvedPumpkinBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = WallTorchLikeProperties::default(args.block);
        props.facing = args
            .player
            .living_entity
            .entity
            .get_horizontal_facing()
            .opposite();
        props.to_state_id(args.block)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        if Block::from_state_id(args.old_state_id) == args.block {
            return;
        }
        if let Some(pattern) = find_golem_pattern(args.world, args.position) {
            spawn_golem(args.world, &pattern);
        }
    }

    fn player_placed(&self, args: PlayerPlacedArgs<'_>) {
        if let Some(pattern) = find_golem_pattern(args.world, args.position)
            && pattern.entity_type == &EntityType::IRON_GOLEM
        {
            args.player.trigger_advancement(
                crate::entity::player::advancement::trigger::AdvancementTrigger::SummonedEntity {
                    entity_type: "minecraft:iron_golem".to_string(),
                },
            );
        }
    }
}
