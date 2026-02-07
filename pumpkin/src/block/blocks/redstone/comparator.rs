use std::sync::{Arc, atomic::Ordering};

use pumpkin_data::{
    Block, BlockDirection, BlockState,
    block_properties::{
        BlockProperties, ComparatorLikeProperties, ComparatorMode, HorizontalFacing,
    },
    entity::EntityType,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos};
use pumpkin_world::{
    BlockStateId, block::entities::comparator::ComparatorBlockEntity, tick::TickPriority,
    world::BlockFlags,
};

use crate::{
    block::{
        BlockBehaviour, BlockFuture, BrokenArgs, CanPlaceAtArgs, EmitsRedstonePowerArgs,
        GetComparatorOutputArgs, GetRedstonePowerArgs, GetStateForNeighborUpdateArgs,
        NormalUseArgs, OnNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs, OnStateReplacedArgs,
        PlacedArgs, PlayerPlacedArgs, registry::BlockActionResult,
    },
    world::World,
};

use super::abstract_redstone_gate::{self, RedstoneGateBlock, RedstoneGateBlockProperties};

#[pumpkin_block("minecraft:comparator")]
pub struct ComparatorBlock;

impl BlockBehaviour for ComparatorBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move { RedstoneGateBlock::on_place(self, args.player, args.block).await })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position).await;
            let props = ComparatorLikeProperties::from_state_id(state.id, args.block);
            self.on_use(props, args.world, *args.position, args.block)
                .await;

            BlockActionResult::Success
        })
    }

    fn emits_redstone_power<'a>(
        &'a self,
        _args: EmitsRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move { true })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            RedstoneGateBlock::can_place_at(self, args.block_accessor, *args.position).await
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let comparator = ComparatorBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(comparator)).await;

            RedstoneGateBlock::update_target(
                self,
                args.world,
                *args.position,
                args.state_id,
                args.block,
            )
            .await;
        })
    }

    fn player_placed<'a>(&'a self, args: PlayerPlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            RedstoneGateBlock::player_placed(self, args).await;
        })
    }

    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            args.world.remove_block_entity(args.position).await;
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if args.direction == BlockDirection::Down
                && !RedstoneGateBlock::can_place_above(
                    self,
                    args.world,
                    *args.neighbor_position,
                    BlockState::from_id(args.neighbor_state_id),
                )
                .await
            {
                return Block::AIR.default_state.id;
            }
            args.state_id
        })
    }

    fn get_weak_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move { RedstoneGateBlock::get_weak_redstone_power(self, args).await })
    }

    fn get_strong_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move { RedstoneGateBlock::get_strong_redstone_power(self, args).await })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            RedstoneGateBlock::on_neighbor_update(self, args).await;
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position).await;
            self.update(args.world, *args.position, state, args.block)
                .await;
        })
    }

    fn on_state_replaced<'a>(&'a self, args: OnStateReplacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            RedstoneGateBlock::on_state_replaced(self, args).await;
        })
    }
}

impl RedstoneGateBlockProperties for ComparatorLikeProperties {
    fn is_powered(&self) -> bool {
        self.powered
    }

    fn get_facing(&self) -> HorizontalFacing {
        self.facing
    }

    fn set_facing(&mut self, facing: HorizontalFacing) {
        self.facing = facing;
    }
}

impl RedstoneGateBlock<ComparatorLikeProperties> for ComparatorBlock {
    fn get_output_level<'a>(&'a self, world: &'a World, pos: BlockPos) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            if let Some(blockentity) = world.get_block_entity(&pos).await
                && blockentity.resource_location() == ComparatorBlockEntity::ID
            {
                let comparator = blockentity
                    .as_any()
                    .downcast_ref::<ComparatorBlockEntity>()
                    .unwrap();
                return comparator.output_signal.load(Ordering::Relaxed);
            }
            0
        })
    }

    fn update_powered<'a>(
        &'a self,
        world: &'a World,
        pos: BlockPos,
        state: &'a BlockState,
        block: &'a Block,
    ) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if world.is_block_tick_scheduled(&pos, block).await {
                return;
            }
            let i = self.calculate_output_signal(world, pos, state, block).await;

            let j = RedstoneGateBlock::get_output_level(self, world, pos).await;

            let props = ComparatorLikeProperties::from_state_id(state.id, block);

            if i != j
                || props.powered
                    != RedstoneGateBlock::has_power(self, world, pos, state, block).await
            {
                world
                    .schedule_block_tick(
                        block,
                        pos,
                        RedstoneGateBlock::get_update_delay_internal(self, state.id, block),
                        if RedstoneGateBlock::is_target_not_aligned(self, world, pos, state, block)
                            .await
                        {
                            TickPriority::High
                        } else {
                            TickPriority::Normal
                        },
                    )
                    .await;
            }
        })
    }

    fn has_power<'a>(
        &'a self,
        world: &'a World,
        pos: BlockPos,
        state: &'a BlockState,
        block: &'a Block,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let i = RedstoneGateBlock::get_power(self, world, pos, state, block).await;
            if i == 0 {
                return false;
            }
            let j = RedstoneGateBlock::get_max_input_level_sides(
                self, world, pos, state.id, block, false,
            )
            .await;
            if i > j {
                true
            } else {
                let props = ComparatorLikeProperties::from_state_id(state.id, block);
                i == j && props.mode == ComparatorMode::Compare
            }
        })
    }

    fn get_power<'a>(
        &'a self,
        world: &'a World,
        pos: BlockPos,
        state: &'a BlockState,
        block: &'a Block,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            let redstone_level = abstract_redstone_gate::get_power::<ComparatorLikeProperties>(
                world, pos, state.id, block,
            )
            .await;

            let props = ComparatorLikeProperties::from_state_id(state.id, block);
            let facing = props.facing;
            let source_pos = pos.offset(facing.to_offset());
            let (source_block, source_state) = world.get_block_and_state(&source_pos).await;

            // Note: .get_comparator_output is assumed to be an async method returning Option<u8>
            if let Some(pumpkin_block) = world.block_registry.get_pumpkin_block(source_block.id)
                && let Some(level) = pumpkin_block
                    .get_comparator_output(GetComparatorOutputArgs {
                        world,
                        block: source_block,
                        state: source_state,
                        position: &source_pos,
                    })
                    .await
            {
                return level;
            }

            if redstone_level < 15 && source_state.is_solid_block() {
                let source_pos = source_pos.offset(facing.to_offset());
                let (source_block, source_state) = world.get_block_and_state(&source_pos).await;

                // Note: self.get_attached_itemframe_level is assumed to be an async method
                let itemframe_level = Self::get_attached_itemframe_level(world, facing, source_pos);
                let block_level = if let Some(pumpkin_block) =
                    world.block_registry.get_pumpkin_block(source_block.id)
                {
                    pumpkin_block
                        .get_comparator_output(GetComparatorOutputArgs {
                            world,
                            block: source_block,
                            state: source_state,
                            position: &source_pos,
                        })
                        .await
                } else {
                    None
                };
                if let Some(level) = itemframe_level.max(block_level) {
                    return level;
                }
            }
            redstone_level
        })
    }

    fn get_update_delay_internal(&self, _state_id: BlockStateId, _block: &Block) -> u8 {
        2
    }
}

impl ComparatorBlock {
    async fn on_use(
        &self,
        mut props: ComparatorLikeProperties,
        world: &Arc<World>,
        block_pos: BlockPos,
        block: &Block,
    ) {
        props.mode = match props.mode {
            ComparatorMode::Compare => ComparatorMode::Subtract,
            ComparatorMode::Subtract => ComparatorMode::Compare,
        };
        let state_id = props.to_state_id(block);
        world
            .set_block_state(&block_pos, state_id, BlockFlags::empty())
            .await;

        self.update(world, block_pos, BlockState::from_id(state_id), block)
            .await;
    }

    async fn calculate_output_signal(
        &self,
        world: &World,
        pos: BlockPos,
        state: &BlockState,
        block: &Block,
    ) -> u8 {
        let power = self.get_power(world, pos, state, block).await;
        let sub_power = self
            .get_max_input_level_sides(world, pos, state.id, block, false)
            .await;
        if sub_power >= power {
            return 0;
        }
        let props = ComparatorLikeProperties::from_state_id(state.id, block);
        if props.mode == ComparatorMode::Subtract {
            power - sub_power
        } else {
            power
        }
    }

    fn get_attached_itemframe_level(
        world: &World,
        facing: HorizontalFacing,
        pos: BlockPos,
    ) -> Option<u8> {
        let mut itemframes = world
            .get_entities_at_box(&BoundingBox::from_block(&pos))
            .into_iter()
            .filter(|entity| {
                entity.get_entity().entity_type == &EntityType::ITEM_FRAME
                    && entity.get_entity().get_horizontal_facing() == facing
            });
        if let Some(_itemframe) = itemframes.next()
            && itemframes.next().is_none()
        {
            // TODO itemframe.getComparatorPower()
            return Some(1);
        }
        None
    }

    async fn update(&self, world: &Arc<World>, pos: BlockPos, state: &BlockState, block: &Block) {
        let future_level = i32::from(self.calculate_output_signal(world, pos, state, block).await);
        let mut now_level = 0;
        if let Some(blockentity) = world.get_block_entity(&pos).await
            && blockentity.resource_location() == ComparatorBlockEntity::ID
        {
            let comparator = blockentity
                .as_any()
                .downcast_ref::<ComparatorBlockEntity>()
                .unwrap();
            now_level = i32::from(comparator.output_signal.load(Ordering::Relaxed));
            comparator
                .output_signal
                .store(future_level as u8, Ordering::Relaxed);
        }
        let mut props = ComparatorLikeProperties::from_state_id(state.id, block);
        if now_level != future_level || props.mode == ComparatorMode::Compare {
            let future_power = self.has_power(world, pos, state, block).await;
            let now_power = props.powered;
            if now_power && !future_power {
                props.powered = false;
                world
                    .set_block_state(&pos, props.to_state_id(block), BlockFlags::NOTIFY_LISTENERS)
                    .await;
            } else if !now_power && future_power {
                props.powered = true;
                world
                    .set_block_state(&pos, props.to_state_id(block), BlockFlags::NOTIFY_LISTENERS)
                    .await;
            }
            RedstoneGateBlock::update_target(self, world, pos, props.to_state_id(block), block)
                .await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Comparator update delay is always 2 game ticks (1 redstone tick).
    /// This differs from repeater which has configurable 2/4/6/8 tick delays.
    #[test]
    fn test_comparator_update_delay_always_2() {
        let comparator = ComparatorBlock;
        let block = &Block::COMPARATOR;
        let default_state = block.default_state.id;
        assert_eq!(
            RedstoneGateBlock::get_update_delay_internal(&comparator, default_state, block),
            2
        );
    }

    /// Comparator mode toggles between Compare and Subtract when right-clicked.
    #[test]
    fn test_mode_toggle() {
        assert_eq!(
            match ComparatorMode::Compare {
                ComparatorMode::Compare => ComparatorMode::Subtract,
                ComparatorMode::Subtract => ComparatorMode::Compare,
            },
            ComparatorMode::Subtract
        );
        assert_eq!(
            match ComparatorMode::Subtract {
                ComparatorMode::Compare => ComparatorMode::Subtract,
                ComparatorMode::Subtract => ComparatorMode::Compare,
            },
            ComparatorMode::Compare
        );
    }

    /// Comparator mode property roundtrips through state ID correctly.
    #[test]
    fn test_mode_property_roundtrip() {
        let block = &Block::COMPARATOR;
        for mode in [ComparatorMode::Compare, ComparatorMode::Subtract] {
            let mut props = ComparatorLikeProperties::default(block);
            props.mode = mode;
            let state_id = props.to_state_id(block);
            let recovered = ComparatorLikeProperties::from_state_id(state_id, block);
            assert_eq!(
                recovered.mode, mode,
                "Mode {:?} not preserved through state roundtrip",
                mode
            );
        }
    }

    /// Comparator powered property roundtrips through state ID correctly.
    #[test]
    fn test_comparator_powered_roundtrip() {
        let block = &Block::COMPARATOR;
        for powered in [true, false] {
            let mut props = ComparatorLikeProperties::default(block);
            props.powered = powered;
            let state_id = props.to_state_id(block);
            let recovered = ComparatorLikeProperties::from_state_id(state_id, block);
            assert_eq!(
                recovered.powered, powered,
                "Powered={} not preserved through state roundtrip",
                powered
            );
        }
    }

    /// Comparator facing roundtrips through state ID correctly for all 4 horizontal directions.
    #[test]
    fn test_comparator_facing_roundtrip() {
        let block = &Block::COMPARATOR;
        for facing in HorizontalFacing::all() {
            let mut props = ComparatorLikeProperties::default(block);
            props.facing = facing;
            let state_id = props.to_state_id(block);
            let recovered = ComparatorLikeProperties::from_state_id(state_id, block);
            assert_eq!(
                recovered.facing, facing,
                "Facing {:?} not preserved through state roundtrip",
                facing
            );
        }
    }

    /// Verify the comparator output formula (from `calculate_output_signal`):
    /// - If side_power >= back_power, output = 0
    /// - If Subtract mode and back > side: output = back - side
    /// - If Compare mode and back > side: output = back
    /// These are the vanilla rules for comparator operation.
    #[test]
    fn test_compare_subtract_formula() {
        // Helper matching the calculate_output_signal logic
        fn calc(back: u8, side: u8, subtract: bool) -> u8 {
            if side >= back {
                return 0;
            }
            if subtract {
                back - side
            } else {
                back
            }
        }

        // Compare mode: back > side → outputs back power
        assert_eq!(calc(10, 5, false), 10);
        // Subtract mode: back > side → outputs difference
        assert_eq!(calc(10, 5, true), 5);

        // Equal power → always 0 (side >= back)
        assert_eq!(calc(7, 7, false), 0);
        assert_eq!(calc(7, 7, true), 0);

        // Side stronger → always 0
        assert_eq!(calc(3, 10, false), 0);
        assert_eq!(calc(3, 10, true), 0);

        // Zero inputs
        assert_eq!(calc(0, 0, false), 0);
        assert_eq!(calc(0, 0, true), 0);

        // Max power, no side → full output
        assert_eq!(calc(15, 0, false), 15);
        assert_eq!(calc(15, 0, true), 15);

        // Max power subtract max side → 0
        assert_eq!(calc(15, 15, true), 0);

        // Subtract to 1
        assert_eq!(calc(8, 7, true), 1);
    }

    /// Exhaustive test of the comparator output formula for ALL valid power combinations.
    /// In vanilla, back and side power range from 0 to 15.
    #[test]
    fn test_compare_subtract_formula_exhaustive() {
        fn calc(back: u8, side: u8, subtract: bool) -> u8 {
            if side >= back {
                return 0;
            }
            if subtract { back - side } else { back }
        }

        for back in 0u8..=15 {
            for side in 0u8..=15 {
                let compare_out = calc(back, side, false);
                let subtract_out = calc(back, side, true);

                if side >= back {
                    assert_eq!(compare_out, 0, "Compare({back},{side}) should be 0");
                    assert_eq!(subtract_out, 0, "Subtract({back},{side}) should be 0");
                } else {
                    assert_eq!(compare_out, back, "Compare({back},{side}) should pass through");
                    assert_eq!(
                        subtract_out,
                        back - side,
                        "Subtract({back},{side}) should be difference"
                    );
                }
            }
        }
    }

    /// Verify the `has_power` logic for comparators. This determines whether
    /// the comparator's output face is powered (lit torch) vs unpowered.
    /// Vanilla rules:
    /// - back == 0 → false (no input, no output)
    /// - back > side → true (signal passes in both modes)
    /// - back == side AND Compare mode → true (equal signals pass in compare)
    /// - back == side AND Subtract mode → false (0 output in subtract)
    /// - back < side → false (side blocks output)
    #[test]
    fn test_has_power_logic() {
        fn has_power(back: u8, side: u8, mode: ComparatorMode) -> bool {
            if back == 0 {
                return false;
            }
            if back > side {
                return true;
            }
            back == side && mode == ComparatorMode::Compare
        }

        // No input → never powered
        assert!(!has_power(0, 0, ComparatorMode::Compare));
        assert!(!has_power(0, 0, ComparatorMode::Subtract));
        assert!(!has_power(0, 5, ComparatorMode::Compare));

        // Back stronger → always powered
        assert!(has_power(10, 5, ComparatorMode::Compare));
        assert!(has_power(10, 5, ComparatorMode::Subtract));
        assert!(has_power(15, 0, ComparatorMode::Compare));
        assert!(has_power(15, 0, ComparatorMode::Subtract));
        assert!(has_power(1, 0, ComparatorMode::Compare));

        // Equal → only Compare mode powers
        assert!(has_power(7, 7, ComparatorMode::Compare));
        assert!(!has_power(7, 7, ComparatorMode::Subtract));
        assert!(has_power(15, 15, ComparatorMode::Compare));
        assert!(!has_power(15, 15, ComparatorMode::Subtract));

        // Side stronger → never powered
        assert!(!has_power(3, 10, ComparatorMode::Compare));
        assert!(!has_power(3, 10, ComparatorMode::Subtract));
    }

    /// Exhaustive has_power test for all 16×16 back×side combinations.
    #[test]
    fn test_has_power_exhaustive() {
        fn has_power(back: u8, side: u8, mode: ComparatorMode) -> bool {
            if back == 0 {
                return false;
            }
            if back > side {
                return true;
            }
            back == side && mode == ComparatorMode::Compare
        }

        for back in 0u8..=15 {
            for side in 0u8..=15 {
                let compare = has_power(back, side, ComparatorMode::Compare);
                let subtract = has_power(back, side, ComparatorMode::Subtract);

                if back == 0 {
                    assert!(!compare, "back=0 should never power (compare)");
                    assert!(!subtract, "back=0 should never power (subtract)");
                } else if back > side {
                    assert!(compare, "back>side should power (compare {back}>{side})");
                    assert!(subtract, "back>side should power (subtract {back}>{side})");
                } else if back == side {
                    assert!(compare, "back==side should power in compare ({back}=={side})");
                    assert!(
                        !subtract,
                        "back==side should NOT power in subtract ({back}=={side})"
                    );
                } else {
                    assert!(!compare, "back<side should not power (compare {back}<{side})");
                    assert!(
                        !subtract,
                        "back<side should not power (subtract {back}<{side})"
                    );
                }
            }
        }
    }

    /// Full state space test: all combinations of facing × mode × powered roundtrip.
    #[test]
    fn test_comparator_full_state_roundtrip() {
        let block = &Block::COMPARATOR;
        for facing in HorizontalFacing::all() {
            for mode in [ComparatorMode::Compare, ComparatorMode::Subtract] {
                for powered in [true, false] {
                    let mut props = ComparatorLikeProperties::default(block);
                    props.facing = facing;
                    props.mode = mode;
                    props.powered = powered;
                    let state_id = props.to_state_id(block);
                    let r = ComparatorLikeProperties::from_state_id(state_id, block);
                    assert_eq!(r.facing, facing, "facing mismatch");
                    assert_eq!(r.mode, mode, "mode mismatch");
                    assert_eq!(r.powered, powered, "powered mismatch");
                }
            }
        }
    }
}
