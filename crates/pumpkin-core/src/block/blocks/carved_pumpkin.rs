use std::sync::Arc;

use pumpkin_data::{
    Block, BlockDirection, BlockId, BlockStateId, block_properties::WallTorchLikeProperties,
    entity::EntityType, world::WorldEvent,
};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::world::BlockFlags;

use crate::{
    block::{BlockBehaviour, BlockMetadata, OnPlaceArgs, PlacedArgs},
    entity::{
        Entity, EntityBase,
        passive::{iron_golem::IronGolemEntity, snow_golem::SnowGolemEntity},
    },
    world::World,
};

const SNOW_GOLEM_BASE: [&str; 3] = [" ", "#", "#"];
const SNOW_GOLEM_FULL: [&str; 3] = ["^", "#", "#"];
const IRON_GOLEM_BASE: [&str; 3] = ["~ ~", "###", "~#~"];
const IRON_GOLEM_FULL: [&str; 3] = ["~^~", "###", "~#~"];

pub struct GolemPattern {
    entity_type: &'static EntityType,
    rows: &'static [&'static str],
    origin: BlockPos,
    forwards: BlockDirection,
    up: BlockDirection,
}

impl GolemPattern {
    fn block_pos(&self, right: i32, down: i32) -> BlockPos {
        let up = self.up.to_offset();
        let right_vector = self.forwards.to_offset().cross(&up);
        self.origin.offset(up * -down + right_vector * right)
    }

    fn cells(&self) -> impl Iterator<Item = BlockPos> + '_ {
        let width = self.rows[0].len() as i32;
        (0..width).flat_map(move |x| (0..self.rows.len() as i32).map(move |y| self.block_pos(x, y)))
    }
}

fn cell_matches(cell: u8, world: &World, pos: &BlockPos, entity_type: &EntityType) -> bool {
    match cell {
        b'^' => {
            let block = world.get_block(pos);
            block == &Block::CARVED_PUMPKIN || block == &Block::JACK_O_LANTERN
        }
        b'#' if entity_type == &EntityType::SNOW_GOLEM => {
            world.get_block(pos) == &Block::SNOW_BLOCK
        }
        b'#' => world.get_block(pos) == &Block::IRON_BLOCK,
        b'~' => world.get_block_state(pos).is_air(),
        _ => true,
    }
}

fn find_pattern(
    world: &World,
    origin: &BlockPos,
    entity_type: &'static EntityType,
    rows: &'static [&'static str],
) -> Option<GolemPattern> {
    let dist = rows.len().max(rows[0].len()) as i32;
    for z in 0..dist {
        for y in 0..dist {
            for x in 0..dist {
                let test_pos = origin.offset(Vector3::new(x, y, z));
                for forwards in BlockDirection::all() {
                    for up in BlockDirection::all() {
                        if up == forwards || up == forwards.opposite() {
                            continue;
                        }
                        let pattern = GolemPattern {
                            entity_type,
                            rows,
                            origin: test_pos,
                            forwards,
                            up,
                        };
                        let matched = rows.iter().enumerate().all(|(down, row)| {
                            row.bytes().enumerate().all(|(right, cell)| {
                                cell_matches(
                                    cell,
                                    world,
                                    &pattern.block_pos(right as i32, down as i32),
                                    entity_type,
                                )
                            })
                        });
                        if matched {
                            return Some(pattern);
                        }
                    }
                }
            }
        }
    }
    None
}

#[must_use]
pub fn can_spawn_golem(world: &World, top_pos: &BlockPos) -> bool {
    find_pattern(world, top_pos, &EntityType::SNOW_GOLEM, &SNOW_GOLEM_BASE).is_some()
        || find_pattern(world, top_pos, &EntityType::IRON_GOLEM, &IRON_GOLEM_BASE).is_some()
}

fn try_spawn_golem(world: &Arc<World>, top_pos: &BlockPos) {
    if let Some(pattern) = find_pattern(world, top_pos, &EntityType::SNOW_GOLEM, &SNOW_GOLEM_FULL) {
        let spawn_pos = pattern.block_pos(0, 2);
        let golem = SnowGolemEntity::new(Entity::new(
            world.clone(),
            Vector3::default(),
            &EntityType::SNOW_GOLEM,
        ));
        spawn_golem_in_world(world, &pattern, &(golem as Arc<dyn EntityBase>), spawn_pos);
        return;
    }
    if let Some(pattern) = find_pattern(world, top_pos, &EntityType::IRON_GOLEM, &IRON_GOLEM_FULL) {
        let spawn_pos = pattern.block_pos(1, 2);
        let golem = IronGolemEntity::new(Entity::new(
            world.clone(),
            Vector3::default(),
            &EntityType::IRON_GOLEM,
        ));
        golem.set_player_created(true);
        spawn_golem_in_world(world, &pattern, &(golem as Arc<dyn EntityBase>), spawn_pos);
    }
}

fn spawn_golem_in_world(
    world: &Arc<World>,
    pattern: &GolemPattern,
    golem: &Arc<dyn EntityBase>,
    spawn_pos: BlockPos,
) {
    for pos in pattern.cells() {
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

    let entity = golem.get_entity();
    entity.set_pos(Vector3::new(
        f64::from(spawn_pos.0.x) + 0.5,
        f64::from(spawn_pos.0.y) + 0.05,
        f64::from(spawn_pos.0.z) + 0.5,
    ));
    world.spawn_entity(golem.clone());

    let entity_type = format!("minecraft:{}", pattern.entity_type.resource_name);
    for player in world.get_players_at_box(&entity.bounding_box.load().expand_all(5.0)) {
        player.trigger_advancement(
            crate::entity::player::advancement::trigger::AdvancementTrigger::SummonedEntity {
                entity_type: entity_type.clone(),
            },
        );
    }

    for pos in pattern.cells() {
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
        try_spawn_golem(args.world, args.position);
    }
}
