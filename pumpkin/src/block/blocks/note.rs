use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockFuture, GetStateForNeighborUpdateArgs, NormalUseArgs, OnNeighborUpdateArgs, OnPlaceArgs,
    UseWithItemArgs,
};
use pumpkin_data::block_properties::{Axis, NoteblockInstrument};
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, EnumVariants, Integer0To24, NoteBlockLikeProperties},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;

use crate::{
    block::{BlockBehaviour, OnSyncedBlockEventArgs},
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
        ((f32::from(note) - 12.0) / 12.0).exp2()
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
            NoteblockInstrument::Harp
        };
        note_props.instrument = below_instrument;
        note_props.to_state_id(block)
    }
}

impl BlockBehaviour for NoteBlock {
    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let block_state = args.world.get_block_state(args.position).await;
            let mut note_props = NoteBlockLikeProperties::from_state_id(block_state.id, args.block);
            let powered = block_receives_redstone_power(args.world, args.position).await;
            // check if powered state changed
            if note_props.powered != powered {
                if powered {
                    Self::play_note(&note_props, args.world, args.position).await;
                }
                note_props.powered = powered;
                args.world
                    .set_block_state(
                        args.position,
                        note_props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let block_state = args.world.get_block_state(args.position).await;
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
                    args.position,
                    note_props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
            Self::play_note(&note_props, args.world, args.position).await;

            BlockActionResult::Success
        })
    }

    fn use_with_item<'a>(
        &'a self,
        _args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            // TODO
            BlockActionResult::PassToDefaultBlockAction
        })
    }

    fn on_synced_block_event<'a>(
        &'a self,
        args: OnSyncedBlockEventArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let block_state = args.world.get_block_state(args.position).await;
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
                    &args.position.to_f64(),
                    3.0,
                    pitch,
                )
                .await;
            true
        })
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            Self::get_state_with_instrument(
                args.world,
                args.position,
                Block::NOTE_BLOCK.default_state.id,
                args.block,
            )
            .await
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if args.direction.to_axis() == Axis::Y {
                return Self::get_state_with_instrument(
                    args.world,
                    args.position,
                    args.state_id,
                    args.block,
                )
                .await;
            }
            args.state_id
        })
    }
}

const fn convert_instrument_to_sound(instrument: NoteblockInstrument) -> Sound {
    match instrument {
        NoteblockInstrument::Harp => Sound::BlockNoteBlockHarp,
        NoteblockInstrument::Basedrum => Sound::BlockNoteBlockBasedrum,
        NoteblockInstrument::Snare => Sound::BlockNoteBlockSnare,
        NoteblockInstrument::Hat => Sound::BlockNoteBlockHat,
        NoteblockInstrument::Bass => Sound::BlockNoteBlockBass,
        NoteblockInstrument::Flute => Sound::BlockNoteBlockFlute,
        NoteblockInstrument::Bell => Sound::BlockNoteBlockBell,
        NoteblockInstrument::Guitar => Sound::BlockNoteBlockGuitar,
        NoteblockInstrument::Chime => Sound::BlockNoteBlockChime,
        NoteblockInstrument::Xylophone => Sound::BlockNoteBlockXylophone,
        NoteblockInstrument::IronXylophone => Sound::BlockNoteBlockIronXylophone,
        NoteblockInstrument::CowBell => Sound::BlockNoteBlockCowBell,
        NoteblockInstrument::Didgeridoo => Sound::BlockNoteBlockDidgeridoo,
        NoteblockInstrument::Bit => Sound::BlockNoteBlockBit,
        NoteblockInstrument::Banjo => Sound::BlockNoteBlockBanjo,
        NoteblockInstrument::Pling => Sound::BlockNoteBlockPling,
        NoteblockInstrument::Zombie => Sound::BlockNoteBlockImitateZombie,
        NoteblockInstrument::Skeleton => Sound::BlockNoteBlockImitateSkeleton,
        NoteblockInstrument::Creeper => Sound::BlockNoteBlockImitateCreeper,
        NoteblockInstrument::Dragon => Sound::BlockNoteBlockImitateEnderDragon,
        NoteblockInstrument::WitherSkeleton => Sound::BlockNoteBlockImitateWitherSkeleton,
        NoteblockInstrument::Piglin => Sound::BlockNoteBlockImitatePiglin,
        NoteblockInstrument::CustomHead => Sound::UiButtonClick,
        NoteblockInstrument::Trumpet => Sound::BlockNoteBlockTrumpet,
        NoteblockInstrument::TrumpetExposed => Sound::BlockNoteBlockTrumpetExposed,
        NoteblockInstrument::TrumpetOxidized => Sound::BlockNoteBlockTrumpetOxidized,
        NoteblockInstrument::TrumpetWeathered => Sound::BlockNoteBlockTrumpetWeathered,
    }
}

const fn is_base_block(instrument: NoteblockInstrument) -> bool {
    matches!(
        instrument,
        NoteblockInstrument::Harp
            | NoteblockInstrument::Basedrum
            | NoteblockInstrument::Snare
            | NoteblockInstrument::Hat
            | NoteblockInstrument::Bass
            | NoteblockInstrument::Flute
            | NoteblockInstrument::Bell
            | NoteblockInstrument::Guitar
            | NoteblockInstrument::Chime
            | NoteblockInstrument::Xylophone
            | NoteblockInstrument::IronXylophone
            | NoteblockInstrument::CowBell
            | NoteblockInstrument::Didgeridoo
            | NoteblockInstrument::Bit
            | NoteblockInstrument::Banjo
    )
}
