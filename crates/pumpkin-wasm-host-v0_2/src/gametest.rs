use crate::pumpkin::plugin::common::{BlockPos, Position};
use crate::pumpkin::plugin::gametest::{
    self, AsyncTestCallbackId, BlockPermutation, BlockPredicateCallbackId, Dimension,
    DimensionLocation, DimensionTypeOrId, Direction, Entity, EntityPredicateCallbackId,
    FenceConnectivity, FluidType, GameMode, GameTestSequence, ItemStack, ItemTypeOrId,
    LookDuration, MoveToOptions, NavigationResult, Player, RegistrationBuilder, SculkSpreader,
    SimulatedPlayer, Test, TestCallbackId, Vector2, Vector3, VoidCallbackId,
};
use pumpkin_wasm_host_common::state::PluginHostState;
use wasmtime::component::Resource;

impl gametest::Host for PluginHostState {
    fn register(
        &mut self,
        _test_class_name: String,
        _test_name: String,
        _test_function: TestCallbackId,
    ) -> wasmtime::Result<Result<Resource<RegistrationBuilder>, String>> {
        Ok(Err("gametest.register not implemented".to_string()))
    }

    fn register_async(
        &mut self,
        _test_class_name: String,
        _test_name: String,
        _test_function: AsyncTestCallbackId,
    ) -> wasmtime::Result<Result<Resource<RegistrationBuilder>, String>> {
        Ok(Err("gametest.register-async not implemented".to_string()))
    }

    fn set_after_batch_callback(
        &mut self,
        _batch_name: String,
        _batch_callback: VoidCallbackId,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.set-after-batch-callback not implemented".to_string()
        ))
    }

    fn set_before_batch_callback(
        &mut self,
        _batch_name: String,
        _batch_callback: VoidCallbackId,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.set-before-batch-callback not implemented".to_string()
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn spawn_simulated_player(
        &mut self,
        _location: DimensionLocation,
        _name: String,
        _game_mode: GameMode,
    ) -> wasmtime::Result<Result<Resource<SimulatedPlayer>, String>> {
        Ok(Err(
            "gametest.spawn-simulated-player not implemented".to_string()
        ))
    }
}

impl gametest::HostSculkSpreader for PluginHostState {
    fn get_max_charge(&mut self, _res: Resource<SculkSpreader>) -> wasmtime::Result<u32> {
        Err(wasmtime::Error::msg(
            "gametest.sculk-spreader.get-max-charge not implemented",
        ))
    }

    fn add_cursors_with_offset(
        &mut self,
        _res: Resource<SculkSpreader>,
        _offset: Vector3,
        _charge: u32,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.sculk-spreader.add-cursors-with-offset not implemented".to_string(),
        ))
    }

    fn get_cursor_position(
        &mut self,
        _res: Resource<SculkSpreader>,
        _index: u32,
    ) -> wasmtime::Result<Result<BlockPos, String>> {
        Ok(Err(
            "gametest.sculk-spreader.get-cursor-position not implemented".to_string(),
        ))
    }

    fn get_number_of_cursors(&mut self, _res: Resource<SculkSpreader>) -> wasmtime::Result<u32> {
        Err(wasmtime::Error::msg(
            "gametest.sculk-spreader.get-number-of-cursors not implemented",
        ))
    }

    fn get_total_charge(&mut self, _res: Resource<SculkSpreader>) -> wasmtime::Result<u32> {
        Err(wasmtime::Error::msg(
            "gametest.sculk-spreader.get-total-charge not implemented",
        ))
    }

    fn drop(&mut self, _res: Resource<SculkSpreader>) -> wasmtime::Result<()> {
        Ok(())
    }
}

impl gametest::HostRegistrationBuilder for PluginHostState {
    fn batch(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _batch_name: String,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.registration-builder.batch not implemented".to_string()
        ))
    }

    fn max_attempts(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _attempt_count: u32,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.registration-builder.max-attempts not implemented".to_string(),
        ))
    }

    fn max_ticks(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _tick_count: u32,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.registration-builder.max-ticks not implemented".to_string(),
        ))
    }

    fn padding(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _padding_blocks: u32,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.registration-builder.padding not implemented".to_string()
        ))
    }

    fn required(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _is_required: bool,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.registration-builder.required not implemented".to_string(),
        ))
    }

    fn required_successful_attempts(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _attempt_count: u32,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.registration-builder.required-successful-attempts not implemented"
                .to_string(),
        ))
    }

    fn rotate_test(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _rotate: bool,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.registration-builder.rotate-test not implemented".to_string(),
        ))
    }

    fn setup_ticks(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _tick_count: u32,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.registration-builder.setup-ticks not implemented".to_string(),
        ))
    }

    fn structure_location(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _structure_location: BlockPos,
        _structure_dimension: Option<DimensionTypeOrId>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.registration-builder.structure-location not implemented".to_string(),
        ))
    }

    fn structure_name(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _structure_name: String,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.registration-builder.structure-name not implemented".to_string(),
        ))
    }

    fn tag(
        &mut self,
        _res: Resource<RegistrationBuilder>,
        _tag: String,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.registration-builder.tag not implemented".to_string()
        ))
    }

    fn drop(&mut self, _res: Resource<RegistrationBuilder>) -> wasmtime::Result<()> {
        Ok(())
    }
}

impl gametest::HostGameTestSequence for PluginHostState {
    fn then_execute(
        &mut self,
        _res: Resource<GameTestSequence>,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.game-test-sequence.then-execute not implemented".to_string(),
        ))
    }

    fn then_execute_after(
        &mut self,
        _res: Resource<GameTestSequence>,
        _delay_ticks: u32,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.game-test-sequence.then-execute-after not implemented".to_string(),
        ))
    }

    fn then_execute_for(
        &mut self,
        _res: Resource<GameTestSequence>,
        _tick_count: u32,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.game-test-sequence.then-execute-for not implemented".to_string(),
        ))
    }

    fn then_fail(
        &mut self,
        _res: Resource<GameTestSequence>,
        _error_message: String,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.game-test-sequence.then-fail not implemented".to_string()
        ))
    }

    fn then_idle(
        &mut self,
        _res: Resource<GameTestSequence>,
        _delay_ticks: u32,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.game-test-sequence.then-idle not implemented".to_string()
        ))
    }

    fn then_succeed(
        &mut self,
        _res: Resource<GameTestSequence>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.game-test-sequence.then-succeed not implemented".to_string(),
        ))
    }

    fn then_wait(
        &mut self,
        _res: Resource<GameTestSequence>,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.game-test-sequence.then-wait not implemented".to_string()
        ))
    }

    fn then_wait_after(
        &mut self,
        _res: Resource<GameTestSequence>,
        _delay_ticks: u32,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.game-test-sequence.then-wait-after not implemented".to_string(),
        ))
    }

    fn drop(&mut self, _res: Resource<GameTestSequence>) -> wasmtime::Result<()> {
        Ok(())
    }
}

impl gametest::HostSimulatedPlayer for PluginHostState {
    fn as_player(
        &mut self,
        _res: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<Result<Resource<Player>, String>> {
        Ok(Err(
            "gametest.simulated-player.as-player not implemented".to_string()
        ))
    }

    fn get_head_rotation(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<Vector2> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.get-head-rotation not implemented",
        ))
    }

    fn get_is_sprinting(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.simulated-player.get-is-sprinting not implemented",
        ))
    }

    fn set_is_sprinting(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _value: bool,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.set-is-sprinting not implemented".to_string(),
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn attack(
        &mut self,
        _res: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<Result<bool, String>> {
        Ok(Err(
            "gametest.simulated-player.attack not implemented".to_string()
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn attack_entity(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _entity: Resource<Entity>,
    ) -> wasmtime::Result<Result<bool, String>> {
        Ok(Err(
            "gametest.simulated-player.attack-entity not implemented".to_string(),
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn break_block(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _block_location: BlockPos,
        _direction: Option<Direction>,
    ) -> wasmtime::Result<Result<bool, String>> {
        Ok(Err(
            "gametest.simulated-player.break-block not implemented".to_string()
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn chat(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _message: String,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.chat not implemented".to_string()
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn disconnect(
        &mut self,
        _res: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.disconnect not implemented".to_string()
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn drop_selected_item(
        &mut self,
        _res: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<Result<bool, String>> {
        Ok(Err(
            "gametest.simulated-player.drop-selected-item not implemented".to_string(),
        ))
    }

    fn fly(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.fly not implemented".to_string()
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn give_item(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _item_stack: Resource<ItemStack>,
        _select_slot: Option<bool>,
    ) -> wasmtime::Result<Result<bool, String>> {
        Ok(Err(
            "gametest.simulated-player.give-item not implemented".to_string()
        ))
    }

    fn glide(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<Result<bool, String>> {
        Ok(Err(
            "gametest.simulated-player.glide not implemented".to_string()
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn interact(
        &mut self,
        _res: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<Result<bool, String>> {
        Ok(Err(
            "gametest.simulated-player.interact not implemented".to_string()
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn interact_with_block(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _block_location: BlockPos,
        _direction: Option<Direction>,
    ) -> wasmtime::Result<Result<bool, String>> {
        Ok(Err(
            "gametest.simulated-player.interact-with-block not implemented".to_string(),
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn interact_with_entity(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _entity: Resource<Entity>,
    ) -> wasmtime::Result<Result<bool, String>> {
        Ok(Err(
            "gametest.simulated-player.interact-with-entity not implemented".to_string(),
        ))
    }

    fn jump(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<Result<bool, String>> {
        Ok(Err(
            "gametest.simulated-player.jump not implemented".to_string()
        ))
    }

    fn look_at_block(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _block_location: BlockPos,
        _duration: Option<LookDuration>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.look-at-block not implemented".to_string(),
        ))
    }

    fn look_at_entity(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _entity: Resource<Entity>,
        _duration: Option<LookDuration>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.look-at-entity not implemented".to_string(),
        ))
    }

    fn look_at_location(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _location: Position,
        _duration: Option<LookDuration>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.look-at-location not implemented".to_string(),
        ))
    }

    fn move_(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _west_east: f64,
        _north_south: f64,
        _speed: Option<f64>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.move not implemented".to_string()
        ))
    }

    fn move_relative(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _left_right: f64,
        _backward_forward: f64,
        _speed: Option<f64>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.move-relative not implemented".to_string(),
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn move_to_block(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _block_location: BlockPos,
        _options: Option<MoveToOptions>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.move-to-block not implemented".to_string(),
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn move_to_location(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _location: Position,
        _options: Option<MoveToOptions>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.move-to-location not implemented".to_string(),
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn navigate_to_block(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _block_location: BlockPos,
        _speed: Option<f64>,
    ) -> wasmtime::Result<Result<NavigationResult, String>> {
        Ok(Err(
            "gametest.simulated-player.navigate-to-block not implemented".to_string(),
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn navigate_to_entity(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _entity: Resource<Entity>,
        _speed: Option<f64>,
    ) -> wasmtime::Result<Result<NavigationResult, String>> {
        Ok(Err(
            "gametest.simulated-player.navigate-to-entity not implemented".to_string(),
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn navigate_to_location(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _location: Position,
        _speed: Option<f64>,
    ) -> wasmtime::Result<Result<NavigationResult, String>> {
        Ok(Err(
            "gametest.simulated-player.navigate-to-location not implemented".to_string(),
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn navigate_to_locations(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _locations: Vec<Position>,
        _speed: Option<f64>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.navigate-to-locations not implemented".to_string(),
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn respawn(
        &mut self,
        _res: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<Result<bool, String>> {
        Ok(Err(
            "gametest.simulated-player.respawn not implemented".to_string()
        ))
    }

    fn rotate_body(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _angle_in_degrees: f64,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.rotate-body not implemented".to_string()
        ))
    }

    fn set_body_rotation(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _angle_in_degrees: f64,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.set-body-rotation not implemented".to_string(),
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn set_item(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _item_stack: Resource<ItemStack>,
        _slot: u8,
        _select_slot: Option<bool>,
    ) -> wasmtime::Result<Result<bool, String>> {
        Ok(Err(
            "gametest.simulated-player.set-item not implemented".to_string()
        ))
    }

    fn start_build(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _slot: Option<u8>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.start-build not implemented".to_string()
        ))
    }

    fn stop_breaking_block(
        &mut self,
        _res: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.stop-breaking-block not implemented".to_string(),
        ))
    }

    fn stop_build(
        &mut self,
        _res: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.stop-build not implemented".to_string()
        ))
    }

    fn stop_flying(
        &mut self,
        _res: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.stop-flying not implemented".to_string()
        ))
    }

    fn stop_gliding(
        &mut self,
        _res: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.stop-gliding not implemented".to_string(),
        ))
    }

    fn stop_interacting(
        &mut self,
        _res: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.stop-interacting not implemented".to_string(),
        ))
    }

    fn stop_moving(
        &mut self,
        _res: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.stop-moving not implemented".to_string()
        ))
    }

    fn stop_swimming(
        &mut self,
        _res: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.stop-swimming not implemented".to_string(),
        ))
    }

    fn stop_using_item(
        &mut self,
        _res: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<Result<Option<Resource<ItemStack>>, String>> {
        Ok(Err(
            "gametest.simulated-player.stop-using-item not implemented".to_string(),
        ))
    }

    fn swim(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.simulated-player.swim not implemented".to_string()
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn use_item(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _item_stack: Resource<ItemStack>,
    ) -> wasmtime::Result<Result<bool, String>> {
        Ok(Err(
            "gametest.simulated-player.use-item not implemented".to_string()
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn use_item_in_slot(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _slot: u8,
    ) -> wasmtime::Result<Result<bool, String>> {
        Ok(Err(
            "gametest.simulated-player.use-item-in-slot not implemented".to_string(),
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn use_item_in_slot_on_block(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _slot: u8,
        _block_location: BlockPos,
        _direction: Option<Direction>,
        _face_location: Option<Position>,
    ) -> wasmtime::Result<Result<bool, String>> {
        Ok(Err(
            "gametest.simulated-player.use-item-in-slot-on-block not implemented".to_string(),
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn use_item_on_block(
        &mut self,
        _res: Resource<SimulatedPlayer>,
        _item_stack: Resource<ItemStack>,
        _block_location: BlockPos,
        _direction: Option<Direction>,
        _face_location: Option<Position>,
    ) -> wasmtime::Result<Result<bool, String>> {
        Ok(Err(
            "gametest.simulated-player.use-item-on-block not implemented".to_string(),
        ))
    }

    fn drop(&mut self, _res: Resource<SimulatedPlayer>) -> wasmtime::Result<()> {
        Ok(())
    }
}

impl gametest::HostTest for PluginHostState {
    fn assert(
        &mut self,
        _res: Resource<Test>,
        _condition: bool,
        _message: String,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err("gametest.test.assert not implemented".to_string()))
    }

    fn assert_block_present(
        &mut self,
        _res: Resource<Test>,
        _block_type_id: String,
        _block_location: BlockPos,
        _is_present: Option<bool>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.assert-block-present not implemented".to_string()
        ))
    }

    fn assert_block_state(
        &mut self,
        _res: Resource<Test>,
        _block_location: BlockPos,
        _callback: BlockPredicateCallbackId,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.assert-block-state not implemented".to_string()
        ))
    }

    fn assert_can_reach_location(
        &mut self,
        _res: Resource<Test>,
        _mob: Resource<Entity>,
        _block_location: BlockPos,
        _can_reach: Option<bool>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.assert-can-reach-location not implemented".to_string(),
        ))
    }

    fn assert_container_contains(
        &mut self,
        _res: Resource<Test>,
        _item_stack: Resource<ItemStack>,
        _block_location: BlockPos,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.assert-container-contains not implemented".to_string(),
        ))
    }

    fn assert_container_empty(
        &mut self,
        _res: Resource<Test>,
        _block_location: BlockPos,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.assert-container-empty not implemented".to_string()
        ))
    }

    fn assert_entity_has_armor(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _armor_slot: u8,
        _armor_name: String,
        _armor_data: u32,
        _block_location: BlockPos,
        _has_armor: Option<bool>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.assert-entity-has-armor not implemented".to_string()
        ))
    }

    fn assert_entity_has_component(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _component_identifier: String,
        _block_location: BlockPos,
        _has_component: Option<bool>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.assert-entity-has-component not implemented".to_string(),
        ))
    }

    fn assert_entity_instance_present(
        &mut self,
        _res: Resource<Test>,
        _entity: Resource<Entity>,
        _block_location: BlockPos,
        _is_present: Option<bool>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.assert-entity-instance-present not implemented".to_string(),
        ))
    }

    fn assert_entity_instance_present_in_area(
        &mut self,
        _res: Resource<Test>,
        _entity: Resource<Entity>,
        _is_present: Option<bool>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.assert-entity-instance-present-in-area not implemented".to_string(),
        ))
    }

    fn assert_entity_present(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _block_location: BlockPos,
        _search_distance: Option<f64>,
        _is_present: Option<bool>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.assert-entity-present not implemented".to_string()
        ))
    }

    fn assert_entity_present_in_area(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _is_present: Option<bool>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.assert-entity-present-in-area not implemented".to_string(),
        ))
    }

    fn assert_entity_state(
        &mut self,
        _res: Resource<Test>,
        _block_location: BlockPos,
        _entity_type_identifier: String,
        _callback: EntityPredicateCallbackId,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.assert-entity-state not implemented".to_string()
        ))
    }

    fn assert_entity_touching(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _location: Position,
        _is_touching: Option<bool>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.assert-entity-touching not implemented".to_string()
        ))
    }

    fn assert_is_waterlogged(
        &mut self,
        _res: Resource<Test>,
        _block_location: BlockPos,
        _is_waterlogged: Option<bool>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.assert-is-waterlogged not implemented".to_string()
        ))
    }

    fn assert_item_entity_count_is(
        &mut self,
        _res: Resource<Test>,
        _item_type: ItemTypeOrId,
        _block_location: BlockPos,
        _search_distance: f64,
        _count: u32,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.assert-item-entity-count-is not implemented".to_string(),
        ))
    }

    fn assert_item_entity_present(
        &mut self,
        _res: Resource<Test>,
        _item_type: ItemTypeOrId,
        _block_location: BlockPos,
        _search_distance: Option<f64>,
        _is_present: Option<bool>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.assert-item-entity-present not implemented".to_string(),
        ))
    }

    fn assert_redstone_power(
        &mut self,
        _res: Resource<Test>,
        _block_location: BlockPos,
        _power: u32,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.assert-redstone-power not implemented".to_string()
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn destroy_block(
        &mut self,
        _res: Resource<Test>,
        _block_location: BlockPos,
        _drop_resources: Option<bool>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.destroy-block not implemented".to_string()
        ))
    }

    fn fail(
        &mut self,
        _res: Resource<Test>,
        _error_message: String,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err("gametest.test.fail not implemented".to_string()))
    }

    fn fail_if(
        &mut self,
        _res: Resource<Test>,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err("gametest.test.fail-if not implemented".to_string()))
    }

    fn get_dimension(&mut self, _res: Resource<Test>) -> wasmtime::Result<Resource<Dimension>> {
        Err(wasmtime::Error::msg(
            "gametest.test.get-dimension not implemented",
        ))
    }

    fn get_fence_connectivity(
        &mut self,
        _res: Resource<Test>,
        _block_location: BlockPos,
    ) -> wasmtime::Result<Result<FenceConnectivity, String>> {
        Ok(Err(
            "gametest.test.get-fence-connectivity not implemented".to_string()
        ))
    }

    fn get_sculk_spreader(
        &mut self,
        _res: Resource<Test>,
        _block_location: BlockPos,
    ) -> wasmtime::Result<Result<Option<Resource<SculkSpreader>>, String>> {
        Ok(Err(
            "gametest.test.get-sculk-spreader not implemented".to_string()
        ))
    }

    fn get_test_direction(&mut self, _res: Resource<Test>) -> wasmtime::Result<Direction> {
        Err(wasmtime::Error::msg(
            "gametest.test.get-test-direction not implemented",
        ))
    }

    fn idle(
        &mut self,
        _res: Resource<Test>,
        _tick_delay: u32,
    ) -> wasmtime::Result<Result<wasmtime::component::FutureReader<()>, String>> {
        Ok(Err("gametest.test.idle not implemented".to_string()))
    }

    fn is_cleaning_up(&mut self, _res: Resource<Test>) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.test.is-cleaning-up not implemented",
        ))
    }

    fn is_completed(&mut self, _res: Resource<Test>) -> wasmtime::Result<bool> {
        Err(wasmtime::Error::msg(
            "gametest.test.is-completed not implemented",
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn kill_all_entities(&mut self, _res: Resource<Test>) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.kill-all-entities not implemented".to_string()
        ))
    }

    fn on_player_jump(
        &mut self,
        _res: Resource<Test>,
        _mob: Resource<Entity>,
        _jump_amount: f64,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.on-player-jump not implemented".to_string()
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn press_button(
        &mut self,
        _res: Resource<Test>,
        _block_location: BlockPos,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err("gametest.test.press-button not implemented".to_string()))
    }

    fn print(
        &mut self,
        _res: Resource<Test>,
        _text: String,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err("gametest.test.print not implemented".to_string()))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn pull_lever(
        &mut self,
        _res: Resource<Test>,
        _block_location: BlockPos,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err("gametest.test.pull-lever not implemented".to_string()))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn pulse_redstone(
        &mut self,
        _res: Resource<Test>,
        _block_location: BlockPos,
        _duration: u32,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.pulse-redstone not implemented".to_string()
        ))
    }

    fn relative_block_location(
        &mut self,
        _res: Resource<Test>,
        _world_block_location: BlockPos,
    ) -> wasmtime::Result<BlockPos> {
        Err(wasmtime::Error::msg(
            "gametest.test.relative-block-location not implemented",
        ))
    }

    fn relative_location(
        &mut self,
        _res: Resource<Test>,
        _world_location: Position,
    ) -> wasmtime::Result<Position> {
        Err(wasmtime::Error::msg(
            "gametest.test.relative-location not implemented",
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn remove_simulated_player(
        &mut self,
        _res: Resource<Test>,
        _simulated_player: Resource<SimulatedPlayer>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.remove-simulated-player not implemented".to_string()
        ))
    }

    fn rotate_direction(
        &mut self,
        _res: Resource<Test>,
        _direction: Direction,
    ) -> wasmtime::Result<Direction> {
        Err(wasmtime::Error::msg(
            "gametest.test.rotate-direction not implemented",
        ))
    }

    fn rotate_vector(
        &mut self,
        _res: Resource<Test>,
        _vector: Vector3,
    ) -> wasmtime::Result<Vector3> {
        Err(wasmtime::Error::msg(
            "gametest.test.rotate-vector not implemented",
        ))
    }

    fn run_after_delay(
        &mut self,
        _res: Resource<Test>,
        _delay_ticks: u32,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.run-after-delay not implemented".to_string()
        ))
    }

    fn run_at_tick_time(
        &mut self,
        _res: Resource<Test>,
        _tick: u32,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.run-at-tick-time not implemented".to_string()
        ))
    }

    fn run_on_finish(
        &mut self,
        _res: Resource<Test>,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.run-on-finish not implemented".to_string()
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn set_block_permutation(
        &mut self,
        _res: Resource<Test>,
        _block_data: BlockPermutation,
        _block_location: BlockPos,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.set-block-permutation not implemented".to_string()
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn set_block_type(
        &mut self,
        _res: Resource<Test>,
        _block_type_id: String,
        _block_location: BlockPos,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.set-block-type not implemented".to_string()
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn set_fluid_container(
        &mut self,
        _res: Resource<Test>,
        _location: BlockPos,
        _type: FluidType,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.set-fluid-container not implemented".to_string()
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn set_tnt_fuse(
        &mut self,
        _res: Resource<Test>,
        _entity: Resource<Entity>,
        _fuse_length: u32,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err("gametest.test.set-tnt-fuse not implemented".to_string()))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn spawn(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _block_location: BlockPos,
    ) -> wasmtime::Result<Result<Resource<Entity>, String>> {
        Ok(Err("gametest.test.spawn not implemented".to_string()))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn spawn_at_location(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _location: Position,
    ) -> wasmtime::Result<Result<Resource<Entity>, String>> {
        Ok(Err(
            "gametest.test.spawn-at-location not implemented".to_string()
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn spawn_item(
        &mut self,
        _res: Resource<Test>,
        _item_stack: Resource<ItemStack>,
        _location: Position,
    ) -> wasmtime::Result<Result<Resource<Entity>, String>> {
        Ok(Err("gametest.test.spawn-item not implemented".to_string()))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn spawn_simulated_player(
        &mut self,
        _res: Resource<Test>,
        _block_location: BlockPos,
        _name: Option<String>,
        _game_mode: Option<GameMode>,
    ) -> wasmtime::Result<Result<Resource<SimulatedPlayer>, String>> {
        Ok(Err(
            "gametest.test.spawn-simulated-player not implemented".to_string()
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn spawn_without_behaviors(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _block_location: BlockPos,
    ) -> wasmtime::Result<Result<Resource<Entity>, String>> {
        Ok(Err(
            "gametest.test.spawn-without-behaviors not implemented".to_string()
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn spawn_without_behaviors_at_location(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _location: Position,
    ) -> wasmtime::Result<Result<Resource<Entity>, String>> {
        Ok(Err(
            "gametest.test.spawn-without-behaviors-at-location not implemented".to_string(),
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn spread_from_face_toward_direction(
        &mut self,
        _res: Resource<Test>,
        _block_location: BlockPos,
        _from_face: Direction,
        _direction: Direction,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.spread-from-face-toward-direction not implemented".to_string(),
        ))
    }

    fn start_sequence(
        &mut self,
        _res: Resource<Test>,
    ) -> wasmtime::Result<Result<Resource<GameTestSequence>, String>> {
        Ok(Err(
            "gametest.test.start-sequence not implemented".to_string()
        ))
    }

    fn succeed(&mut self, _res: Resource<Test>) -> wasmtime::Result<Result<(), String>> {
        Ok(Err("gametest.test.succeed not implemented".to_string()))
    }

    fn succeed_if(
        &mut self,
        _res: Resource<Test>,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err("gametest.test.succeed-if not implemented".to_string()))
    }

    fn succeed_on_tick(
        &mut self,
        _res: Resource<Test>,
        _tick: u32,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.succeed-on-tick not implemented".to_string()
        ))
    }

    fn succeed_on_tick_when(
        &mut self,
        _res: Resource<Test>,
        _tick: u32,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.succeed-on-tick-when not implemented".to_string()
        ))
    }

    fn succeed_when(
        &mut self,
        _res: Resource<Test>,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err("gametest.test.succeed-when not implemented".to_string()))
    }

    fn succeed_when_block_present(
        &mut self,
        _res: Resource<Test>,
        _block_type_id: String,
        _block_location: BlockPos,
        _is_present: Option<bool>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.succeed-when-block-present not implemented".to_string(),
        ))
    }

    fn succeed_when_entity_has_component(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _component_identifier: String,
        _block_location: BlockPos,
        _has_component: bool,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.succeed-when-entity-has-component not implemented".to_string(),
        ))
    }

    fn succeed_when_entity_present(
        &mut self,
        _res: Resource<Test>,
        _entity_type_identifier: String,
        _block_location: BlockPos,
        _is_present: Option<bool>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.succeed-when-entity-present not implemented".to_string(),
        ))
    }

    // TODO: make this async (list it in bindings.rs) once implemented
    fn trigger_internal_block_event(
        &mut self,
        _res: Resource<Test>,
        _block_location: BlockPos,
        _event: String,
        _event_parameters: Option<Vec<f64>>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.trigger-internal-block-event not implemented".to_string(),
        ))
    }

    fn until(
        &mut self,
        _res: Resource<Test>,
        _callback: VoidCallbackId,
    ) -> wasmtime::Result<Result<wasmtime::component::FutureReader<()>, String>> {
        Ok(Err("gametest.test.until not implemented".to_string()))
    }

    fn walk_to(
        &mut self,
        _res: Resource<Test>,
        _mob: Resource<Entity>,
        _block_location: BlockPos,
        _speed_modifier: Option<f64>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err("gametest.test.walk-to not implemented".to_string()))
    }

    fn walk_to_location(
        &mut self,
        _res: Resource<Test>,
        _mob: Resource<Entity>,
        _location: Position,
        _speed_modifier: Option<f64>,
    ) -> wasmtime::Result<Result<(), String>> {
        Ok(Err(
            "gametest.test.walk-to-location not implemented".to_string()
        ))
    }

    fn world_block_location(
        &mut self,
        _res: Resource<Test>,
        _relative_block_location: BlockPos,
    ) -> wasmtime::Result<BlockPos> {
        Err(wasmtime::Error::msg(
            "gametest.test.world-block-location not implemented",
        ))
    }

    fn world_location(
        &mut self,
        _res: Resource<Test>,
        _relative_location: Position,
    ) -> wasmtime::Result<Position> {
        Err(wasmtime::Error::msg(
            "gametest.test.world-location not implemented",
        ))
    }

    fn drop(&mut self, _res: Resource<Test>) -> wasmtime::Result<()> {
        Ok(())
    }
}
