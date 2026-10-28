use std::sync::{Arc, atomic::Ordering};

use pumpkin_data::{
    Block, BlockDirection, BlockStateId,
    block_properties::{PotentSulfurLikeProperties, PotentSulfurState},
    fluid::Fluid,
    sound::{Sound, SoundCategory},
    tag::{self, Taggable},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;

use crate::{
    block::{
        BlockBehaviour, GetStateForNeighborUpdateArgs, OnPlaceArgs, OnSyncedBlockEventArgs,
        PlacedArgs, entities::potent_sulfur::PotentSulfurBlockEntity,
    },
    world::World,
};

#[pumpkin_block("minecraft:potent_sulfur")]
pub struct PotentSulfurBlock;

impl PotentSulfurBlock {
    #[must_use]
    pub const fn get_eruption_start_sound(state: PotentSulfurState) -> Option<Sound> {
        match state {
            PotentSulfurState::Continuous => Some(Sound::BlockPotentSulfurGeyserContinuousEruption),
            PotentSulfurState::Erupting => Some(Sound::BlockPotentSulfurGeyserEruption),
            PotentSulfurState::Dry | PotentSulfurState::Wet | PotentSulfurState::Dormant => None,
        }
    }

    pub fn notify_eruption_started(world: &World, pos: &BlockPos, state: PotentSulfurState) {
        if matches!(
            state,
            PotentSulfurState::Erupting | PotentSulfurState::Continuous
        ) {
            world.add_synced_block_event(*pos, 0, 0);
            if let Some(sound) = Self::get_eruption_start_sound(state) {
                world.play_sound(sound, SoundCategory::Blocks, &pos.to_f64());
            }
        }
    }

    #[must_use]
    pub fn is_water_source_block(world: &World, pos: &BlockPos) -> bool {
        let (fluid, fluid_state) = world.get_fluid_and_fluid_state(pos);
        fluid.matches_type(&Fluid::WATER) && fluid_state.is_source
    }

    #[must_use]
    pub fn valid_block_state(
        world: &World,
        pos: &BlockPos,
        state_id: BlockStateId,
    ) -> BlockStateId {
        let mut props = PotentSulfurLikeProperties::from_state_id(state_id);
        if !Self::is_water_source_block(world, &pos.up()) {
            props.potent_sulfur_state = PotentSulfurState::Dry;
            return props.to_state_id(&Block::POTENT_SULFUR);
        }

        let below_pos = pos.down();
        let below_block = world.get_block(&below_pos);
        let (_, below_fluid_state) = world.get_fluid_and_fluid_state(&below_pos);
        let is_source_if_fluid = below_fluid_state.is_empty || below_fluid_state.is_source;

        if below_block.has_tag(&tag::Block::MINECRAFT_CAUSES_CONTINUOUS_GEYSER_ERUPTIONS)
            && is_source_if_fluid
        {
            props.potent_sulfur_state = PotentSulfurState::Continuous;
        } else if below_block.has_tag(&tag::Block::MINECRAFT_CAUSES_PERIODIC_GEYSER_ERUPTIONS)
            && is_source_if_fluid
        {
            let current_state = props.potent_sulfur_state;
            let is_geyser = matches!(
                current_state,
                PotentSulfurState::Erupting | PotentSulfurState::Dormant
            );
            if !is_geyser
                && let Some(entity) = world.get_block_entity(pos)
                && let Some(sulfur) = entity.as_any().downcast_ref::<PotentSulfurBlockEntity>()
            {
                sulfur.waiting_countdown.store(-1, Ordering::Relaxed);
            }
            props.potent_sulfur_state = if current_state == PotentSulfurState::Erupting {
                PotentSulfurState::Erupting
            } else {
                PotentSulfurState::Dormant
            };
        } else {
            props.potent_sulfur_state = PotentSulfurState::Wet;
        }

        props.to_state_id(&Block::POTENT_SULFUR)
    }
}

impl BlockBehaviour for PotentSulfurBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        Self::valid_block_state(args.world, args.position, args.block.default_state.id)
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        if args.world.get_block_entity(args.position).is_none() {
            args.world
                .add_block_entity(Arc::new(PotentSulfurBlockEntity::new(*args.position)));
        }
        let props = PotentSulfurLikeProperties::from_state_id(args.state_id);
        Self::notify_eruption_started(args.world, args.position, props.potent_sulfur_state);
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if matches!(args.direction, BlockDirection::Up | BlockDirection::Down) {
            let new_state_id = Self::valid_block_state(args.world, args.position, args.state_id);
            if new_state_id != args.state_id {
                let new_props = PotentSulfurLikeProperties::from_state_id(new_state_id);
                Self::notify_eruption_started(
                    args.world,
                    args.position,
                    new_props.potent_sulfur_state,
                );
            }
            new_state_id
        } else {
            args.state_id
        }
    }

    fn on_synced_block_event(&self, _args: OnSyncedBlockEventArgs<'_>) -> bool {
        true
    }
}
