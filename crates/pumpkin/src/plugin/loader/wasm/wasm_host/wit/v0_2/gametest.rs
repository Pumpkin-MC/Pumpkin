use crate::plugin::loader::wasm::wasm_host::state::PluginHostState;
use pumpkin_host_bindings::v0_2::pumpkin::plugin::gametest::{
    self, AsyncTestCallbackId, Block, BlockPermutation, BlockPredicateCallbackId, BlockTypeOrId,
    Dimension, DimensionLocation, DimensionTypeOrId, Direction, Entity, EntityPredicateCallbackId,
    FenceConnectivity, FluidType, GameMode, GameTestSequence, ItemStack, ItemTypeOrId,
    LookDuration, MoveToOptions, NavigationResult, Player, PlayerSkinData,
    RegistrationBuilder, SculkSpreader, SimulatedPlayer, Tags, Test, TestCallbackId, Vector2,
    Vector3, VoidCallbackId,
};
use wasmtime::component::Resource;

impl gametest::Host for PluginHostState {
    async fn get_player_skin(
        &mut self,
        _player: Resource<Player>,
    ) -> wasmtime::Result<PlayerSkinData> {
        Err(wasmtime::Error::msg(
            "gametest.get-player-skin not implemented",
        ))
    }

    async fn register(
        &mut self,
        _test_class_name: String,
        _test_name: String,
        _test_function: TestCallbackId,
    ) -> wasmtime::Result<Resource<RegistrationBuilder>> {
        Err(wasmtime::Error::msg("gametest.register not implemented"))
    }

    async fn register_async(
        &mut self,
        _test_class_name: String,
        _test_name: String,
        _test_function: AsyncTestCallbackId,
    ) -> wasmtime::Result<Resource<RegistrationBuilder>> {
        Err(wasmtime::Error::msg(
            "gametest.register-async not implemented",
        ))
    }

    async fn set_after_batch_callback(
        &mut self,
        _batch_name: String,
        _batch_callback: VoidCallbackId,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.set-after-batch-callback not implemented",
        ))
    }

    async fn set_before_batch_callback(
        &mut self,
        _batch_name: String,
        _batch_callback: VoidCallbackId,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.set-before-batch-callback not implemented",
        ))
    }

    async fn spawn_simulated_player(
        &mut self,
        _location: DimensionLocation,
        _name: String,
        _game_mode: GameMode,
    ) -> wasmtime::Result<Resource<SimulatedPlayer>> {
        Err(wasmtime::Error::msg(
            "gametest.spawn-simulated-player not implemented",
        ))
    }
}

impl gametest::HostBlock for PluginHostState {
    async fn get_permutation(
        &mut self,
        _res: Resource<Block>,
    ) -> wasmtime::Result<BlockPermutation> {
        Err(wasmtime::Error::msg(
            "gametest.block.get-permutation not implemented",
        ))
    }

    async fn drop(&mut self, _res: Resource<Block>) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg("gametest.block.drop not implemented"))
    }
}

impl gametest::HostSculkSpreader for PluginHostState {
    async fn get_max_charge(&mut self, _res: Resource<SculkSpreader>) -> wasmtime::Result<f64> {
        Err(wasmtime::Error::msg(
            "gametest.sculk-spreader.get-max-charge not implemented",
        ))
    }

    async fn add_cursors_with_offset(
        &mut self,
        _res: Resource<SculkSpreader>,
        _offset: Vector3,
        _charge: f64,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.sculk-spreader.add-cursors-with-offset not implemented",
        ))
    }

    async fn get_cursor_position(
        &mut self,
        _res: Resource<SculkSpreader>,
        _index: f64,
    ) -> wasmtime::Result<Vector3> {
        Err(wasmtime::Error::msg(
            "gametest.sculk-spreader.get-cursor-position not implemented",
        ))
    }

    async fn get_number_of_cursors(
        &mut self,
        _res: Resource<SculkSpreader>,
    ) -> wasmtime::Result<f64> {
        Err(wasmtime::Error::msg(
            "gametest.sculk-spreader.get-number-of-cursors not implemented",
        ))
    }

    async fn get_total_charge(&mut self, _res: Resource<SculkSpreader>) -> wasmtime::Result<f64> {
        Err(wasmtime::Error::msg(
            "gametest.sculk-spreader.get-total-charge not implemented",
        ))
    }

    async fn drop(&mut self, _res: Resource<SculkSpreader>) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.sculk-spreader.drop not implemented",
        ))
    }
}

impl gametest::HostRegistrationBuilder for PluginHostState {
    async fn batch(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _batch_name: String,
    ) -> wasmtime::Result<Resource<RegistrationBuilder>> {
        Err(wasmtime::Error::msg(
            "gametest.registration-builder.batch not implemented",
        ))
    }

    async fn max_attempts(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _attempt_count: f64,
    ) -> wasmtime::Result<Resource<RegistrationBuilder>> {
        Err(wasmtime::Error::msg(
            "gametest.registration-builder.max-attempts not implemented",
        ))
    }

    async fn max_ticks(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _tick_count: f64,
    ) -> wasmtime::Result<Resource<RegistrationBuilder>> {
        Err(wasmtime::Error::msg(
            "gametest.registration-builder.max-ticks not implemented",
        ))
    }

    async fn padding(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _padding_blocks: f64,
    ) -> wasmtime::Result<Resource<RegistrationBuilder>> {
        Err(wasmtime::Error::msg(
            "gametest.registration-builder.padding not implemented",
        ))
    }

    async fn required(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _is_required: bool,
    ) -> wasmtime::Result<Resource<RegistrationBuilder>> {
        Err(wasmtime::Error::msg(
            "gametest.registration-builder.required not implemented",
        ))
    }

    async fn required_successful_attempts(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _attempt_count: f64,
    ) -> wasmtime::Result<Resource<RegistrationBuilder>> {
        Err(wasmtime::Error::msg(
            "gametest.registration-builder.required-successful-attempts not implemented",
        ))
    }

    async fn rotate_test(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _rotate: bool,
    ) -> wasmtime::Result<Resource<RegistrationBuilder>> {
        Err(wasmtime::Error::msg(
            "gametest.registration-builder.rotate-test not implemented",
        ))
    }

    async fn setup_ticks(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _tick_count: f64,
    ) -> wasmtime::Result<Resource<RegistrationBuilder>> {
        Err(wasmtime::Error::msg(
            "gametest.registration-builder.setup-ticks not implemented",
        ))
    }

    async fn structure_location(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _structure_location: Vector3,
        _structure_dimension: Option<DimensionTypeOrId>,
    ) -> wasmtime::Result<Resource<RegistrationBuilder>> {
        Err(wasmtime::Error::msg(
            "gametest.registration-builder.structure-location not implemented",
        ))
    }

    async fn structure_name(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _structure_name: String,
    ) -> wasmtime::Result<Resource<RegistrationBuilder>> {
        Err(wasmtime::Error::msg(
            "gametest.registration-builder.structure-name not implemented",
        ))
    }

    async fn tag(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _tag: String,
    ) -> wasmtime::Result<Resource<RegistrationBuilder>> {
        Err(wasmtime::Error::msg(
            "gametest.registration-builder.tag not implemented",
        ))
    }

    async fn drop(&mut self, _res: Resource<RegistrationBuilder>) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.registration-builder.drop not implemented",
        ))
    }
}

impl gametest::HostGameTestSequence for PluginHostState {
    async fn then_execute(
        &mut self,
        _res: Resource<GameTestSequence>,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<Resource<GameTestSequence>> {
        Err(wasmtime::Error::msg(
            "gametest.game-test-sequence.then-execute not implemented",
        ))
    }

    async fn then_execute_after(
        &mut self,
        _res: Resource<GameTestSequence>,
        _delay_ticks: f64,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<Resource<GameTestSequence>> {
        Err(wasmtime::Error::msg(
            "gametest.game-test-sequence.then-execute-after not implemented",
        ))
    }

    async fn then_execute_for(
        &mut self,
        _res: Resource<GameTestSequence>,
        _tick_count: f64,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<Resource<GameTestSequence>> {
        Err(wasmtime::Error::msg(
            "gametest.game-test-sequence.then-execute-for not implemented",
        ))
    }

    async fn then_fail(
        &mut self,
        _res: Resource<GameTestSequence>,
        _error_message: String,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.game-test-sequence.then-fail not implemented",
        ))
    }

    async fn then_idle(
        &mut self,
        _res: Resource<GameTestSequence>,
        _delay_ticks: f64,
    ) -> wasmtime::Result<Resource<GameTestSequence>> {
        Err(wasmtime::Error::msg(
            "gametest.game-test-sequence.then-idle not implemented",
        ))
    }

    async fn then_succeed(&mut self, _res: Resource<GameTestSequence>) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.game-test-sequence.then-succeed not implemented",
        ))
    }

    async fn then_wait(
        &mut self,
        _res: Resource<GameTestSequence>,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<Resource<GameTestSequence>> {
        Err(wasmtime::Error::msg(
            "gametest.game-test-sequence.then-wait not implemented",
        ))
    }

    async fn then_wait_after(
        &mut self,
        _res: Resource<GameTestSequence>,
        _delay_ticks: f64,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<Resource<GameTestSequence>> {
        Err(wasmtime::Error::msg(
            "gametest.game-test-sequence.then-wait-after not implemented",
        ))
    }

    async fn drop(&mut self, _res: Resource<GameTestSequence>) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.game-test-sequence.drop not implemented",
        ))
    }
}

impl gametest::HostTags for PluginHostState {
    async fn suite_all(&mut self) -> wasmtime::Result<String> {
        Err(wasmtime::Error::msg(
            "gametest.tags.suite-all not implemented",
        ))
    }

    async fn suite_debug(&mut self) -> wasmtime::Result<String> {
        Err(wasmtime::Error::msg(
            "gametest.tags.suite-debug not implemented",
        ))
    }

    async fn suite_default(&mut self) -> wasmtime::Result<String> {
        Err(wasmtime::Error::msg(
            "gametest.tags.suite-default not implemented",
        ))
    }

    async fn suite_disabled(&mut self) -> wasmtime::Result<String> {
        Err(wasmtime::Error::msg(
            "gametest.tags.suite-disabled not implemented",
        ))
    }

    async fn suite_next_update(&mut self) -> wasmtime::Result<String> {
        Err(wasmtime::Error::msg(
            "gametest.tags.suite-next-update not implemented",
        ))
    }

    async fn drop(&mut self, _res: Resource<Tags>) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg("gametest.tags.drop not implemented"))
    }
}

impl gametest::HostSimulatedPlayer for PluginHostState {
    async fn as_player(
        &mut self,
        _res: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<Resource<Player>> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.as-player not implemented",
        ))
    }

    async fn get_head_rotation(
        &mut self,
        _res: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<Vector2> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.get-head-rotation not implemented",
        ))
    }

    async fn get_is_sprinting(
        &mut self,
        _res: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.get-is-sprinting not implemented",
        ))
    }

    async fn set_is_sprinting(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _value: bool,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.set-is-sprinting not implemented",
        ))
    }

    async fn attack(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.attack not implemented",
        ))
    }

    async fn attack_entity(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _entity: Resource<Entity>,
    ) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.attack-entity not implemented",
        ))
    }

    async fn break_block(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _block_location: Vector3,
        _direction: Option<Direction>,
    ) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.break-block not implemented",
        ))
    }

    async fn chat(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _message: String,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.chat not implemented",
        ))
    }

    async fn disconnect(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.disconnect not implemented",
        ))
    }

    async fn drop_selected_item(
        &mut self,
        _res: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.drop-selected-item not implemented",
        ))
    }

    async fn fly(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.fly not implemented",
        ))
    }

    async fn give_item(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _item_stack: Resource<ItemStack>,
        _select_slot: Option<bool>,
    ) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.give-item not implemented",
        ))
    }

    async fn glide(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.glide not implemented",
        ))
    }

    async fn interact(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.interact not implemented",
        ))
    }

    async fn interact_with_block(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _block_location: Vector3,
        _direction: Option<Direction>,
    ) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.interact-with-block not implemented",
        ))
    }

    async fn interact_with_entity(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _entity: Resource<Entity>,
    ) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.interact-with-entity not implemented",
        ))
    }

    async fn jump(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.jump not implemented",
        ))
    }

    async fn look_at_block(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _block_location: Vector3,
        _duration: Option<LookDuration>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.look-at-block not implemented",
        ))
    }

    async fn look_at_entity(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _entity: Resource<Entity>,
        _duration: Option<LookDuration>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.look-at-entity not implemented",
        ))
    }

    async fn look_at_location(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _location: Vector3,
        _duration: Option<LookDuration>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.look-at-location not implemented",
        ))
    }

    async fn move_(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _west_east: f64,
        _north_south: f64,
        _speed: Option<f64>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.move not implemented",
        ))
    }

    async fn move_relative(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _left_right: f64,
        _backward_forward: f64,
        _speed: Option<f64>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.move-relative not implemented",
        ))
    }

    async fn move_to_block(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _block_location: Vector3,
        _options: Option<MoveToOptions>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.move-to-block not implemented",
        ))
    }

    async fn move_to_location(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _location: Vector3,
        _options: Option<MoveToOptions>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.move-to-location not implemented",
        ))
    }

    async fn navigate_to_block(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _block_location: Vector3,
        _speed: Option<f64>,
    ) -> wasmtime::Result<NavigationResult> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.navigate-to-block not implemented",
        ))
    }

    async fn navigate_to_entity(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _entity: Resource<Entity>,
        _speed: Option<f64>,
    ) -> wasmtime::Result<NavigationResult> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.navigate-to-entity not implemented",
        ))
    }

    async fn navigate_to_location(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _location: Vector3,
        _speed: Option<f64>,
    ) -> wasmtime::Result<NavigationResult> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.navigate-to-location not implemented",
        ))
    }

    async fn navigate_to_locations(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _locations: Vec<Vector3>,
        _speed: Option<f64>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.navigate-to-locations not implemented",
        ))
    }

    async fn respawn(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.respawn not implemented",
        ))
    }

    async fn rotate_body(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _angle_in_degrees: f64,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.rotate-body not implemented",
        ))
    }

    async fn set_body_rotation(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _angle_in_degrees: f64,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.set-body-rotation not implemented",
        ))
    }

    async fn set_item(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _item_stack: Resource<ItemStack>,
        _slot: f64,
        _select_slot: Option<bool>,
    ) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.set-item not implemented",
        ))
    }

    async fn set_skin(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _options: PlayerSkinData,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.set-skin not implemented",
        ))
    }

    async fn start_build(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _slot: Option<f64>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.start-build not implemented",
        ))
    }

    async fn stop_breaking_block(
        &mut self,
        _res: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.stop-breaking-block not implemented",
        ))
    }

    async fn stop_build(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.stop-build not implemented",
        ))
    }

    async fn stop_flying(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.stop-flying not implemented",
        ))
    }

    async fn stop_gliding(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.stop-gliding not implemented",
        ))
    }

    async fn stop_interacting(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.stop-interacting not implemented",
        ))
    }

    async fn stop_moving(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.stop-moving not implemented",
        ))
    }

    async fn stop_swimming(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.stop-swimming not implemented",
        ))
    }

    async fn stop_using_item(
        &mut self,
        _res: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<Option<Resource<ItemStack>>> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.stop-using-item not implemented",
        ))
    }

    async fn swim(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.swim not implemented",
        ))
    }

    async fn use_item(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _item_stack: Resource<ItemStack>,
    ) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.use-item not implemented",
        ))
    }

    async fn use_item_in_slot(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _slot: f64,
    ) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.use-item-in-slot not implemented",
        ))
    }

    async fn use_item_in_slot_on_block(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _slot: f64,
        _block_location: Vector3,
        _direction: Option<Direction>,
        _face_location: Option<Vector3>,
    ) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.use-item-in-slot-on-block not implemented",
        ))
    }

    async fn use_item_on_block(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _item_stack: Resource<ItemStack>,
        _block_location: Vector3,
        _direction: Option<Direction>,
        _face_location: Option<Vector3>,
    ) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.use-item-on-block not implemented",
        ))
    }

    async fn drop(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.drop not implemented",
        ))
    }
}

impl gametest::HostTest for PluginHostState {
    async fn assert(
        &mut self,
        _res: Resource<Test>,
        _condition: bool,
        _message: String,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg("gametest.test.assert not implemented"))
    }

    async fn assert_block_present(
        &mut self,
        _res: Resource<Test>,
        _block_type: BlockTypeOrId,
        _block_location: Vector3,
        _is_present: Option<bool>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.assert-block-present not implemented",
        ))
    }

    async fn assert_block_state(
        &mut self,
        _res: Resource<Test>,
        _block_location: Vector3,
        _callback: BlockPredicateCallbackId,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.assert-block-state not implemented",
        ))
    }

    async fn assert_can_reach_location(
        &mut self,
        _res: Resource<Test>,
        _mob: Resource<Entity>,
        _block_location: Vector3,
        _can_reach: Option<bool>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.assert-can-reach-location not implemented",
        ))
    }

    async fn assert_container_contains(
        &mut self,
        _res: Resource<Test>,
        _item_stack: Resource<ItemStack>,
        _block_location: Vector3,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.assert-container-contains not implemented",
        ))
    }

    async fn assert_container_empty(
        &mut self,
        _res: Resource<Test>,
        _block_location: Vector3,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.assert-container-empty not implemented",
        ))
    }

    async fn assert_entity_has_armor(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _armor_slot: f64,
        _armor_name: String,
        _armor_data: f64,
        _block_location: Vector3,
        _has_armor: Option<bool>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.assert-entity-has-armor not implemented",
        ))
    }

    async fn assert_entity_has_component(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _component_identifier: String,
        _block_location: Vector3,
        _has_component: Option<bool>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.assert-entity-has-component not implemented",
        ))
    }

    async fn assert_entity_instance_present(
        &mut self,
        _res: Resource<Test>,
        _entity: Resource<Entity>,
        _block_location: Vector3,
        _is_present: Option<bool>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.assert-entity-instance-present not implemented",
        ))
    }

    async fn assert_entity_instance_present_in_area(
        &mut self,
        _res: Resource<Test>,
        _entity: Resource<Entity>,
        _is_present: Option<bool>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.assert-entity-instance-present-in-area not implemented",
        ))
    }

    async fn assert_entity_present(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _block_location: Vector3,
        _search_distance: Option<f64>,
        _is_present: Option<bool>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.assert-entity-present not implemented",
        ))
    }

    async fn assert_entity_present_in_area(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _is_present: Option<bool>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.assert-entity-present-in-area not implemented",
        ))
    }

    async fn assert_entity_state(
        &mut self,
        _res: Resource<Test>,
        _block_location: Vector3,
        _entity_type_identifier: String,
        _callback: EntityPredicateCallbackId,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.assert-entity-state not implemented",
        ))
    }

    async fn assert_entity_touching(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _location: Vector3,
        _is_touching: Option<bool>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.assert-entity-touching not implemented",
        ))
    }

    async fn assert_is_waterlogged(
        &mut self,
        _res: Resource<Test>,
        _block_location: Vector3,
        _is_waterlogged: Option<bool>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.assert-is-waterlogged not implemented",
        ))
    }

    async fn assert_item_entity_count_is(
        &mut self,
        _res: Resource<Test>,
        _item_type: ItemTypeOrId,
        _block_location: Vector3,
        _search_distance: f64,
        _count: f64,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.assert-item-entity-count-is not implemented",
        ))
    }

    async fn assert_item_entity_present(
        &mut self,
        _res: Resource<Test>,
        _item_type: ItemTypeOrId,
        _block_location: Vector3,
        _search_distance: Option<f64>,
        _is_present: Option<bool>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.assert-item-entity-present not implemented",
        ))
    }

    async fn assert_redstone_power(
        &mut self,
        _res: Resource<Test>,
        _block_location: Vector3,
        _power: f64,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.assert-redstone-power not implemented",
        ))
    }

    async fn destroy_block(
        &mut self,
        _res: Resource<Test>,
        _block_location: Vector3,
        _drop_resources: Option<bool>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.destroy-block not implemented",
        ))
    }

    async fn fail(&mut self, _res: Resource<Test>, _error_message: String) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg("gametest.test.fail not implemented"))
    }

    async fn fail_if(
        &mut self,
        _res: Resource<Test>,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.fail-if not implemented",
        ))
    }

    async fn get_block(
        &mut self,
        _res: Resource<Test>,
        _block_location: Vector3,
    ) -> wasmtime::Result<Resource<Block>> {
        Err(wasmtime::Error::msg(
            "gametest.test.get-block not implemented",
        ))
    }

    async fn get_dimension(
        &mut self,
        _res: Resource<Test>,
    ) -> wasmtime::Result<Resource<Dimension>> {
        Err(wasmtime::Error::msg(
            "gametest.test.get-dimension not implemented",
        ))
    }

    async fn get_fence_connectivity(
        &mut self,
        _res: Resource<Test>,
        _block_location: Vector3,
    ) -> wasmtime::Result<FenceConnectivity> {
        Err(wasmtime::Error::msg(
            "gametest.test.get-fence-connectivity not implemented",
        ))
    }

    async fn get_sculk_spreader(
        &mut self,
        _res: Resource<Test>,
        _block_location: Vector3,
    ) -> wasmtime::Result<Option<Resource<SculkSpreader>>> {
        Err(wasmtime::Error::msg(
            "gametest.test.get-sculk-spreader not implemented",
        ))
    }

    async fn get_test_direction(&mut self, _res: Resource<Test>) -> wasmtime::Result<Direction> {
        Err(wasmtime::Error::msg(
            "gametest.test.get-test-direction not implemented",
        ))
    }

    async fn idle(
        &mut self,
        _res: Resource<Test>,
        _tick_delay: f64,
    ) -> wasmtime::Result<wasmtime::component::FutureReader<()>> {
        Err(wasmtime::Error::msg("gametest.test.idle not implemented"))
    }

    async fn is_cleaning_up(&mut self, _res: Resource<Test>) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.test.is-cleaning-up not implemented",
        ))
    }

    async fn is_completed(&mut self, _res: Resource<Test>) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.test.is-completed not implemented",
        ))
    }

    async fn kill_all_entities(&mut self, _res: Resource<Test>) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.kill-all-entities not implemented",
        ))
    }

    async fn on_player_jump(
        &mut self,
        _res: Resource<Test>,
        _mob: Resource<Entity>,
        _jump_amount: f64,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.on-player-jump not implemented",
        ))
    }

    async fn press_button(
        &mut self,
        _res: Resource<Test>,
        _block_location: Vector3,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.press-button not implemented",
        ))
    }

    async fn print(&mut self, _res: Resource<Test>, _text: String) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg("gametest.test.print not implemented"))
    }

    async fn pull_lever(
        &mut self,
        _res: Resource<Test>,
        _block_location: Vector3,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.pull-lever not implemented",
        ))
    }

    async fn pulse_redstone(
        &mut self,
        _res: Resource<Test>,
        _block_location: Vector3,
        _duration: f64,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.pulse-redstone not implemented",
        ))
    }

    async fn relative_block_location(
        &mut self,
        _res: Resource<Test>,
        _world_block_location: Vector3,
    ) -> wasmtime::Result<Vector3> {
        Err(wasmtime::Error::msg(
            "gametest.test.relative-block-location not implemented",
        ))
    }

    async fn relative_location(
        &mut self,
        _res: Resource<Test>,
        _world_location: Vector3,
    ) -> wasmtime::Result<Vector3> {
        Err(wasmtime::Error::msg(
            "gametest.test.relative-location not implemented",
        ))
    }

    async fn remove_simulated_player(
        &mut self,
        _res: Resource<Test>,
        _simulated_player: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.remove-simulated-player not implemented",
        ))
    }

    async fn rotate_direction(
        &mut self,
        _res: Resource<Test>,
        _direction: Direction,
    ) -> wasmtime::Result<Direction> {
        Err(wasmtime::Error::msg(
            "gametest.test.rotate-direction not implemented",
        ))
    }

    async fn rotate_vector(
        &mut self,
        _res: Resource<Test>,
        _vector: Vector3,
    ) -> wasmtime::Result<Vector3> {
        Err(wasmtime::Error::msg(
            "gametest.test.rotate-vector not implemented",
        ))
    }

    async fn run_after_delay(
        &mut self,
        _res: Resource<Test>,
        _delay_ticks: f64,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.run-after-delay not implemented",
        ))
    }

    async fn run_at_tick_time(
        &mut self,
        _res: Resource<Test>,
        _tick: f64,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.run-at-tick-time not implemented",
        ))
    }

    async fn run_on_finish(
        &mut self,
        _res: Resource<Test>,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.run-on-finish not implemented",
        ))
    }

    async fn set_block_permutation(
        &mut self,
        _res: Resource<Test>,
        _block_data: BlockPermutation,
        _block_location: Vector3,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.set-block-permutation not implemented",
        ))
    }

    async fn set_block_type(
        &mut self,
        _res: Resource<Test>,
        _block_type: BlockTypeOrId,
        _block_location: Vector3,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.set-block-type not implemented",
        ))
    }

    async fn set_fluid_container(
        &mut self,
        _res: Resource<Test>,
        _location: Vector3,
        _type: FluidType,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.set-fluid-container not implemented",
        ))
    }

    async fn set_tnt_fuse(
        &mut self,
        _res: Resource<Test>,
        _entity: Resource<Entity>,
        _fuse_length: f64,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.set-tnt-fuse not implemented",
        ))
    }

    async fn spawn(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _block_location: Vector3,
    ) -> wasmtime::Result<Resource<Entity>> {
        Err(wasmtime::Error::msg("gametest.test.spawn not implemented"))
    }

    async fn spawn_at_location(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _location: Vector3,
    ) -> wasmtime::Result<Resource<Entity>> {
        Err(wasmtime::Error::msg(
            "gametest.test.spawn-at-location not implemented",
        ))
    }

    async fn spawn_item(
        &mut self,
        _res: Resource<Test>,
        _item_stack: Resource<ItemStack>,
        _location: Vector3,
    ) -> wasmtime::Result<Resource<Entity>> {
        Err(wasmtime::Error::msg(
            "gametest.test.spawn-item not implemented",
        ))
    }

    async fn spawn_simulated_player(
        &mut self,
        _res: Resource<Test>,
        _block_location: Vector3,
        _name: Option<String>,
        _game_mode: Option<GameMode>,
    ) -> wasmtime::Result<Resource<SimulatedPlayer>> {
        Err(wasmtime::Error::msg(
            "gametest.test.spawn-simulated-player not implemented",
        ))
    }

    async fn spawn_without_behaviors(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _block_location: Vector3,
    ) -> wasmtime::Result<Resource<Entity>> {
        Err(wasmtime::Error::msg(
            "gametest.test.spawn-without-behaviors not implemented",
        ))
    }

    async fn spawn_without_behaviors_at_location(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _location: Vector3,
    ) -> wasmtime::Result<Resource<Entity>> {
        Err(wasmtime::Error::msg(
            "gametest.test.spawn-without-behaviors-at-location not implemented",
        ))
    }

    async fn spread_from_face_toward_direction(
        &mut self,
        _res: Resource<Test>,
        _block_location: Vector3,
        _from_face: Direction,
        _direction: Direction,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.spread-from-face-toward-direction not implemented",
        ))
    }

    async fn start_sequence(
        &mut self,
        _res: Resource<Test>,
    ) -> wasmtime::Result<Resource<GameTestSequence>> {
        Err(wasmtime::Error::msg(
            "gametest.test.start-sequence not implemented",
        ))
    }

    async fn succeed(&mut self, _res: Resource<Test>) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.succeed not implemented",
        ))
    }

    async fn succeed_if(
        &mut self,
        _res: Resource<Test>,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.succeed-if not implemented",
        ))
    }

    async fn succeed_on_tick(&mut self, _res: Resource<Test>, _tick: f64) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.succeed-on-tick not implemented",
        ))
    }

    async fn succeed_on_tick_when(
        &mut self,
        _res: Resource<Test>,
        _tick: f64,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.succeed-on-tick-when not implemented",
        ))
    }

    async fn succeed_when(
        &mut self,
        _res: Resource<Test>,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.succeed-when not implemented",
        ))
    }

    async fn succeed_when_block_present(
        &mut self,
        _res: Resource<Test>,
        _block_type: BlockTypeOrId,
        _block_location: Vector3,
        _is_present: Option<bool>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.succeed-when-block-present not implemented",
        ))
    }

    async fn succeed_when_entity_has_component(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _component_identifier: String,
        _block_location: Vector3,
        _has_component: bool,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.succeed-when-entity-has-component not implemented",
        ))
    }

    async fn succeed_when_entity_present(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _block_location: Vector3,
        _is_present: Option<bool>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.succeed-when-entity-present not implemented",
        ))
    }

    async fn trigger_internal_block_event(
        &mut self,
        _res: Resource<Test>,
        _block_location: Vector3,
        _event: String,
        _event_parameters: Option<Vec<f64>>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.trigger-internal-block-event not implemented",
        ))
    }

    async fn until(
        &mut self,
        _res: Resource<Test>,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<wasmtime::component::FutureReader<()>> {
        Err(wasmtime::Error::msg("gametest.test.until not implemented"))
    }

    async fn walk_to(
        &mut self,
        _res: Resource<Test>,
        _mob: Resource<Entity>,
        _block_location: Vector3,
        _speed_modifier: Option<f64>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.walk-to not implemented",
        ))
    }

    async fn walk_to_location(
        &mut self,
        _res: Resource<Test>,
        _mob: Resource<Entity>,
        _location: Vector3,
        _speed_modifier: Option<f64>,
    ) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg(
            "gametest.test.walk-to-location not implemented",
        ))
    }

    async fn world_block_location(
        &mut self,
        _res: Resource<Test>,
        _relative_block_location: Vector3,
    ) -> wasmtime::Result<Vector3> {
        Err(wasmtime::Error::msg(
            "gametest.test.world-block-location not implemented",
        ))
    }

    async fn world_location(
        &mut self,
        _res: Resource<Test>,
        _relative_location: Vector3,
    ) -> wasmtime::Result<Vector3> {
        Err(wasmtime::Error::msg(
            "gametest.test.world-location not implemented",
        ))
    }

    async fn drop(&mut self, _res: Resource<Test>) -> wasmtime::Result<()> {
        Err(wasmtime::Error::msg("gametest.test.drop not implemented"))
    }
}
