use crate::block::blocks::fire::FireBlockBase;
use crate::block::blocks::fire::fire::FireBlock;
use crate::entity::player::Player;
use crate::world::World;
use crate::world::portal::nether::NetherPortal;
use pumpkin_data::block_properties::HorizontalAxis;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, BlockStateId, tag};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

pub struct Ignition;

impl Ignition {
    pub async fn ignite_block<F, Fut>(
        ignite_logic: F,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        block: &Block,
    ) -> bool
    where
        F: FnOnce(Arc<World>, BlockPos, BlockStateId) -> Fut,
        Fut: Future<Output = ()>,
    {
        let world = player.world();
        let pos = location.offset(face.to_offset());

        if world.get_fluid(&location).name != Fluid::EMPTY.name {
            return false;
        }

        let state_id = world.get_block_state_id(&location);

        if let Some(new_state_id) = can_be_lit(block, state_id) {
            ignite_logic(world.clone(), location, new_state_id).await;
            return true;
        }

        // Light a Nether portal without placing fire first (vanilla onPlace creates
        // portal immediately; placing fire then replacing it causes a client flicker).
        if try_light_nether_portal(&world, &pos, face).await {
            return true;
        }

        let fire_block = FireBlockBase::get_fire_type(&world, &pos);
        let state_id = FireBlock.get_state_for_position(&world, &fire_block, &pos);
        if FireBlockBase::can_place_at(&world, &pos) {
            ignite_logic(world.clone(), pos, state_id).await;
            return true;
        }

        false
    }
}

async fn try_light_nether_portal(
    world: &Arc<World>,
    pos: &BlockPos,
    face: BlockDirection,
) -> bool {
    let dimension = &world.dimension;
    if dimension != &Dimension::OVERWORLD && dimension != &Dimension::THE_NETHER {
        return false;
    }

    let first_axis = if face.is_horizontal() {
        face.rotate_counter_clockwise()
            .to_horizontal_axis()
            .unwrap_or(HorizontalAxis::X)
    } else {
        HorizontalAxis::X
    };

    if let Some(portal) = NetherPortal::get_new_portal(world, pos, first_axis) {
        portal.create(world).await;
        return true;
    }
    false
}

fn can_be_lit(block: &Block, state_id: BlockStateId) -> Option<BlockStateId> {
    // Vanilla only lights the clicked block itself for campfires, candles and candle cakes.
    // See `CampfireBlock::canLight`, `CandleBlock::canLight` and `CandleCakeBlock::canLight`.
    // Everything else that merely carries a `lit` property (furnaces, redstone lamps, copper
    // bulbs, ...) must fall through to placing a fire block instead.
    if !block.has_tag(&tag::Block::MINECRAFT_CAMPFIRES)
        && !block.has_tag(&tag::Block::MINECRAFT_CANDLES)
        && !block.has_tag(&tag::Block::MINECRAFT_CANDLE_CAKES)
    {
        return None;
    }

    let mut props = {
        let props = &block.properties(state_id)?;
        props.to_props()
    };

    let (_, value) = props.iter_mut().find(|(k, _)| *k == "lit")?;
    *value = "true";

    let new_state_id = block.from_properties(&props).to_state_id(block);

    (new_state_id != state_id).then_some(new_state_id)
}
