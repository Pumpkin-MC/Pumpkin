use crate::block::pumpkin_block::{
    GetStateForNeighborUpdateArgs, NormalUseArgs, OnNeighborUpdateArgs, OnPlaceArgs,
    UseWithItemArgs,
};
use crate::block::registry::BlockActionResult;
use async_trait::async_trait;
use pumpkin_data::block_properties::Axis;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{
    Block,
    block_properties::{
        BlockProperties, EnumVariants, Instrument, Integer0To24, NoteBlockLikeProperties,
    },
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;

use crate::{
    block::pumpkin_block::{OnSyncedBlockEventArgs, PumpkinBlock},
    world::World,
};

use super::redstone::block_receives_redstone_power;

#[pumpkin_block("minecraft:note_block")]
pub struct NoteBlock;

impl NoteBlock {
    pub async fn play_note(props: &NoteBlockLikeProperties, world: &World, pos: &BlockPos) {
        if !is_base_block(props.instrument) || world.get_block_state(&pos.up()).await.is_air() {
            world.add_synced_block_event(*pos, 0, 0).await;
        }
    }
    fn get_note_pitch(note: u16) -> f32 {
        2.0f64.powf((f64::from(note) - 12.0) / 12.0) as f32
    }

    async fn get_state_with_instrument(
        world: &World,
        pos: &BlockPos,
        state: BlockStateId,
        block: &Block,
    ) -> BlockStateId {
        let upper_instrument = world.get_block_state(&pos.up()).await.instrument;

        let mut note_props = NoteBlockLikeProperties::from_state_id(state, block);
        if !is_base_block(upper_instrument) {
            note_props.instrument = upper_instrument;
            return note_props.to_state_id(block);
        }
        let below_instrument = world.get_block_state(&pos.down()).await.instrument;
        let below_instrument = if is_base_block(below_instrument) {
            below_instrument
        } else {
            Instrument::Harp
        };
        note_props.instrument = below_instrument;
        note_props.to_state_id(block)
    }
}

#[async_trait]
impl PumpkinBlock for NoteBlock {
    async fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let block_state = args.world.get_block_state(args.location).await;
        let mut note_props = NoteBlockLikeProperties::from_state_id(block_state.id, args.block);
        let powered = block_receives_redstone_power(args.world, args.location).await;
        // check if powered state changed
        if note_props.powered != powered {
            if powered {
                Self::play_note(&note_props, args.world, args.location).await;
            }
            note_props.powered = powered;
            args.world
                .set_block_state(
                    args.location,
                    note_props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
        }
    }

    async fn normal_use(&self, args: NormalUseArgs<'_>) {
        let block_state = args.world.get_block_state(args.location).await;
        let mut note_props = NoteBlockLikeProperties::from_state_id(block_state.id, args.block);
        let next_index = note_props.note.to_index() + 1;
        // Increment and check if max
        note_props.note = if next_index >= Integer0To24::variant_count() {
            Integer0To24::from_index(0)
        } else {
            Integer0To24::from_index(next_index)
        };
        args.world
            .set_block_state(
                args.location,
                note_props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            )
            .await;
        Self::play_note(&note_props, args.world, args.location).await;
    }

    async fn use_with_item(&self, _args: UseWithItemArgs<'_>) -> BlockActionResult {
        // TODO
        BlockActionResult::Continue
    }

    async fn on_synced_block_event(&self, args: OnSyncedBlockEventArgs<'_>) -> bool {
        let block_state = args.world.get_block_state(args.location).await;
        let note_props = NoteBlockLikeProperties::from_state_id(block_state.id, args.block);
        let instrument = note_props.instrument;
        let pitch = if is_base_block(instrument) {
            // checks if can be pitched
            Self::get_note_pitch(note_props.note.to_index())
        } else {
            1.0 // default pitch
        };
        // check hasCustomSound
        args.world
            .play_sound_raw(
                convert_instrument_to_sound(instrument) as u16,
                SoundCategory::Records,
                &args.location.to_f64(),
                3.0,
                pitch,
            )
            .await;
        true
    }

    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        Self::get_state_with_instrument(
            args.world,
            args.location,
            Block::NOTE_BLOCK.default_state.id,
            args.block,
        )
        .await
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.direction.to_axis() == Axis::Y {
            return Self::get_state_with_instrument(
                args.world,
                args.location,
                args.state_id,
                args.block,
            )
            .await;
        }
        args.state_id
    }
}

fn convert_instrument_to_sound(instrument: Instrument) -> Sound {
    match instrument {
        Instrument::Harp => Sound::BlockNoteBlockHarp,
        Instrument::Basedrum => Sound::BlockNoteBlockBasedrum,
        Instrument::Snare => Sound::BlockNoteBlockSnare,
        Instrument::Hat => Sound::BlockNoteBlockHat,
        Instrument::Bass => Sound::BlockNoteBlockBass,
        Instrument::Flute => Sound::BlockNoteBlockFlute,
        Instrument::Bell => Sound::BlockNoteBlockBell,
        Instrument::Guitar => Sound::BlockNoteBlockGuitar,
        Instrument::Chime => Sound::BlockNoteBlockChime,
        Instrument::Xylophone => Sound::BlockNoteBlockXylophone,
        Instrument::IronXylophone => Sound::BlockNoteBlockIronXylophone,
        Instrument::CowBell => Sound::BlockNoteBlockCowBell,
        Instrument::Didgeridoo => Sound::BlockNoteBlockDidgeridoo,
        Instrument::Bit => Sound::BlockNoteBlockBit,
        Instrument::Banjo => Sound::BlockNoteBlockBanjo,
        Instrument::Pling => Sound::BlockNoteBlockPling,
        Instrument::Zombie => Sound::BlockNoteBlockImitateZombie,
        Instrument::Skeleton => Sound::BlockNoteBlockImitateSkeleton,
        Instrument::Creeper => Sound::BlockNoteBlockImitateCreeper,
        Instrument::Dragon => Sound::BlockNoteBlockImitateEnderDragon,
        Instrument::WitherSkeleton => Sound::BlockNoteBlockImitateWitherSkeleton,
        Instrument::Piglin => Sound::BlockNoteBlockImitatePiglin,
        Instrument::CustomHead => Sound::UiButtonClick,
    }
}

fn is_base_block(instrument: Instrument) -> bool {
    matches!(
        instrument,
        Instrument::Harp
            | Instrument::Basedrum
            | Instrument::Snare
            | Instrument::Hat
            | Instrument::Bass
            | Instrument::Flute
            | Instrument::Bell
            | Instrument::Guitar
            | Instrument::Chime
            | Instrument::Xylophone
            | Instrument::IronXylophone
            | Instrument::CowBell
            | Instrument::Didgeridoo
            | Instrument::Bit
            | Instrument::Banjo
    )
}
