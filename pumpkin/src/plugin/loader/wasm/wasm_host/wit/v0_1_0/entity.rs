use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
    wit::v0_1_0::pumpkin::{
        self,
        plugin::{
            common::{BlockPosition, DamageType, Position},
            entity::{
                BlockEntity, CommandBlockEntity, Entity, EntityBase, ItemEntity, ItemStack,
                LivingEntity, ProjectileHit,
            },
            player::Player,
            server::Server,
            text::TextComponent,
            world::World,
        },
    },
};

impl pumpkin::plugin::entity::Host for PluginHostState {}

impl pumpkin::plugin::entity::HostBlockEntity for PluginHostState {
    async fn resource_location(&mut self, _block_entity: Resource<BlockEntity>) -> String {
        todo!()
    }

    async fn get_position(&mut self, _block_entity: Resource<BlockEntity>) -> BlockPosition {
        todo!()
    }

    async fn get_id(&mut self, _block_entity: Resource<BlockEntity>) -> u32 {
        todo!()
    }

    async fn is_dirty(&mut self, _block_entity: Resource<BlockEntity>) -> bool {
        todo!()
    }

    async fn clear_dirty(&mut self, _block_entity: Resource<BlockEntity>) {
        todo!()
    }

    async fn drop(&mut self, _rep: Resource<BlockEntity>) -> wasmtime::Result<()> {
        todo!()
    }
}

impl pumpkin::plugin::entity::HostCommandBlockEntity for PluginHostState {
    async fn get_block_entity(
        &mut self,
        _command_block_entity: Resource<CommandBlockEntity>,
    ) -> Resource<BlockEntity> {
        todo!()
    }

    async fn last_output(&mut self, _command_block_entity: Resource<CommandBlockEntity>) -> String {
        todo!()
    }

    async fn track_output(&mut self, _command_block_entity: Resource<CommandBlockEntity>) -> bool {
        todo!()
    }

    async fn success_count(&mut self, _command_block_entity: Resource<CommandBlockEntity>) -> u32 {
        todo!()
    }

    async fn command(&mut self, _command_block_entity: Resource<CommandBlockEntity>) -> String {
        todo!()
    }

    async fn auto(&mut self, _command_block_entity: Resource<CommandBlockEntity>) -> bool {
        todo!()
    }

    async fn condition_met(&mut self, _command_block_entity: Resource<CommandBlockEntity>) -> bool {
        todo!()
    }

    async fn powered(&mut self, _command_block_entity: Resource<CommandBlockEntity>) -> bool {
        todo!()
    }

    async fn drop(&mut self, _rep: Resource<CommandBlockEntity>) -> wasmtime::Result<()> {
        todo!()
    }
}

impl pumpkin::plugin::entity::HostEntityBase for PluginHostState {
    async fn tick(
        &mut self,
        _entity_base: Resource<EntityBase>,
        _caller: Resource<EntityBase>,
        _server: Resource<Server>,
    ) {
        todo!()
    }

    async fn init_data_tracker(&mut self, _entity_base: Resource<EntityBase>) {
        todo!()
    }

    async fn teleport(
        &mut self,
        _entity_base: Resource<EntityBase>,
        _position: Position,
        _yaw: Option<i32>,
        _pitch: Option<i32>,
        _world: Resource<World>,
    ) {
        todo!()
    }

    async fn is_pushed_by_fluids(&mut self, _entity_base: Resource<EntityBase>) -> bool {
        todo!()
    }

    async fn is_immune_to_explosion(&mut self, _entity_base: Resource<EntityBase>) -> bool {
        todo!()
    }

    async fn get_gravity(&mut self, _entity_base: Resource<EntityBase>) -> f64 {
        todo!()
    }

    async fn tick_in_void(&mut self, _entity_base: Resource<EntityBase>) {
        todo!()
    }

    async fn damage(
        &mut self,
        _entity_base: Resource<EntityBase>,
        _caller: Resource<EntityBase>,
        _amount: i32,
        _damage_type: Resource<DamageType>,
    ) {
        todo!()
    }

    async fn is_spectator(&mut self, _entity_base: Resource<EntityBase>) -> bool {
        todo!()
    }

    async fn is_collidable(&mut self, _entity_base: Resource<EntityBase>) -> bool {
        todo!()
    }

    async fn can_hit(&mut self, _entity_base: Resource<EntityBase>) -> bool {
        todo!()
    }

    async fn is_flutterer(&mut self, _entity_base: Resource<EntityBase>) -> bool {
        todo!()
    }

    async fn get_y_velocity_drag(&mut self, _entity_base: Resource<EntityBase>) -> Option<f64> {
        todo!()
    }

    async fn damage_with_context(
        &mut self,
        _entity_base: Resource<EntityBase>,
        _caller: Resource<EntityBase>,
        _amount: i32,
        _damage_type: Resource<DamageType>,
        _position: Option<Position>,
        _source: Resource<EntityBase>,
        _cause: Resource<EntityBase>,
    ) -> bool {
        todo!()
    }

    async fn interact(
        &mut self,
        _entity_base: Resource<EntityBase>,
        _player: Resource<Player>,
        _item_stack: Resource<ItemStack>,
    ) -> bool {
        todo!()
    }

    async fn on_player_collision(
        &mut self,
        _entity_base: Resource<EntityBase>,
        _player: Resource<Player>,
    ) {
        todo!()
    }

    async fn on_hit(&mut self, _entity_base: Resource<EntityBase>, _hit: ProjectileHit) {
        todo!()
    }

    async fn set_paddle_state(
        &mut self,
        _entity_base: Resource<EntityBase>,
        _left: bool,
        _right: bool,
    ) {
        todo!()
    }

    async fn is_in_love(&mut self, _entity_base: Resource<EntityBase>) -> bool {
        todo!()
    }

    async fn is_breeding_ready(&mut self, _entity_base: Resource<EntityBase>) -> bool {
        todo!()
    }

    async fn reset_love(&mut self, _entity_base: Resource<EntityBase>) {
        todo!()
    }

    async fn set_breeding_cooldown(&mut self, _entity_base: Resource<EntityBase>, _ticks: i32) {
        todo!()
    }

    async fn is_panicking(&mut self, _entity_base: Resource<EntityBase>) -> bool {
        todo!()
    }

    async fn get_entity(&mut self, _entity_base: Resource<EntityBase>) -> Resource<Entity> {
        todo!()
    }

    async fn get_living_entity(
        &mut self,
        _entity_base: Resource<EntityBase>,
    ) -> Option<Resource<LivingEntity>> {
        todo!()
    }

    async fn get_item_entity(
        &mut self,
        _entity_base: Resource<EntityBase>,
    ) -> Option<Resource<ItemEntity>> {
        todo!()
    }

    async fn get_player(&mut self, _entity_base: Resource<EntityBase>) -> Option<Resource<Player>> {
        todo!()
    }

    async fn get_name(&mut self, _entity_base: Resource<EntityBase>) -> Resource<TextComponent> {
        todo!()
    }

    async fn get_display_name(
        &mut self,
        _entity_base: Resource<EntityBase>,
    ) -> Resource<TextComponent> {
        todo!()
    }

    async fn kill(&mut self, _entity_base: Resource<EntityBase>, _caller: Resource<EntityBase>) {
        todo!()
    }

    async fn drop(&mut self, _rep: Resource<EntityBase>) -> wasmtime::Result<()> {
        todo!()
    }
}

impl pumpkin::plugin::entity::HostEntity for PluginHostState {
    async fn drop(&mut self, _rep: Resource<Entity>) -> wasmtime::Result<()> {
        todo!()
    }
}

impl pumpkin::plugin::entity::HostItemEntity for PluginHostState {
    async fn drop(&mut self, _rep: Resource<ItemEntity>) -> wasmtime::Result<()> {
        todo!()
    }
}

impl pumpkin::plugin::entity::HostLivingEntity for PluginHostState {
    async fn drop(&mut self, _rep: Resource<LivingEntity>) -> wasmtime::Result<()> {
        todo!()
    }
}
